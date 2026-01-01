//! XPT file validation module.
//!
//! This module provides comprehensive validation for SAS Transport (XPT) files,
//! checking conformance to the XPT specification including file structure,
//! header records, NAMESTR records, and observation data.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use sdtm_xpt::validate::{XptValidator, ValidationSeverity};
//!
//! let path = Path::new("dataset.xpt");
//! let result = XptValidator::validate_file(path).unwrap();
//!
//! println!("Valid: {}", result.is_valid());
//! println!("Errors: {}", result.error_count());
//! println!("Warnings: {}", result.warning_count());
//!
//! for issue in result.issues() {
//!     println!("{}: {} at {}", issue.severity, issue.message, issue.location);
//! }
//! ```

use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::{Result, XptError};
use crate::float::is_missing;
use crate::header::{
    NAMESTR_LEN, NAMESTR_LEN_VAX, RECORD_LEN, align_to_record, is_label_header, parse_dataset_name,
    parse_namestr, parse_namestr_len, parse_variable_count, validate_dscrptr_header,
    validate_library_header, validate_member_header, validate_namestr_header, validate_obs_header,
};
use crate::types::XptVersion;

/// Validation issue severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationSeverity {
    /// Informational message, not an error.
    Info,
    /// Warning - file is valid but has potential issues.
    Warning,
    /// Error - file violates XPT specification.
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Location in the XPT file where an issue was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationLocation {
    /// Byte offset in the file.
    pub offset: Option<usize>,
    /// Record number (if applicable).
    pub record: Option<usize>,
    /// Section of the file.
    pub section: String,
    /// Variable name (if applicable).
    pub variable: Option<String>,
    /// Row number (if applicable).
    pub row: Option<usize>,
}

impl ValidationLocation {
    /// Create a new location with section only.
    pub fn section(section: impl Into<String>) -> Self {
        Self {
            offset: None,
            record: None,
            section: section.into(),
            variable: None,
            row: None,
        }
    }

    /// Create a new location with offset.
    pub fn at_offset(offset: usize, section: impl Into<String>) -> Self {
        Self {
            offset: Some(offset),
            record: Some(offset / RECORD_LEN),
            section: section.into(),
            variable: None,
            row: None,
        }
    }

    /// Create a new location for a variable.
    pub fn variable(name: impl Into<String>, index: usize) -> Self {
        Self {
            offset: None,
            record: None,
            section: format!("NAMESTR[{}]", index),
            variable: Some(name.into()),
            row: None,
        }
    }

    /// Create a new location for observation data.
    pub fn observation(row: usize, variable: Option<String>) -> Self {
        Self {
            offset: None,
            record: None,
            section: "OBS".to_string(),
            variable,
            row: Some(row),
        }
    }
}

impl fmt::Display for ValidationLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = vec![self.section.clone()];
        if let Some(offset) = self.offset {
            parts.push(format!("offset {offset}"));
        }
        if let Some(ref var) = self.variable {
            parts.push(format!("variable {var}"));
        }
        if let Some(row) = self.row {
            parts.push(format!("row {row}"));
        }
        write!(f, "{}", parts.join(", "))
    }
}

/// A single validation issue found in the XPT file.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Severity of the issue.
    pub severity: ValidationSeverity,
    /// Issue code for programmatic handling.
    pub code: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Location in the file.
    pub location: ValidationLocation,
}

impl ValidationIssue {
    /// Create a new validation issue.
    pub fn new(
        severity: ValidationSeverity,
        code: &'static str,
        message: impl Into<String>,
        location: ValidationLocation,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            location,
        }
    }

    /// Create an error issue.
    pub fn error(
        code: &'static str,
        message: impl Into<String>,
        location: ValidationLocation,
    ) -> Self {
        Self::new(ValidationSeverity::Error, code, message, location)
    }

    /// Create a warning issue.
    pub fn warning(
        code: &'static str,
        message: impl Into<String>,
        location: ValidationLocation,
    ) -> Self {
        Self::new(ValidationSeverity::Warning, code, message, location)
    }

    /// Create an info issue.
    pub fn info(
        code: &'static str,
        message: impl Into<String>,
        location: ValidationLocation,
    ) -> Self {
        Self::new(ValidationSeverity::Info, code, message, location)
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} (at {})",
            self.code, self.severity, self.message, self.location
        )
    }
}

/// Result of XPT file validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// All validation issues found.
    issues: Vec<ValidationIssue>,
    /// Detected XPT version.
    pub version: Option<XptVersion>,
    /// Dataset name (if successfully parsed).
    pub dataset_name: Option<String>,
    /// Number of variables detected.
    pub variable_count: Option<usize>,
    /// Number of observations detected.
    pub observation_count: Option<usize>,
    /// File size in bytes.
    pub file_size: usize,
    /// Observation length in bytes.
    pub observation_length: Option<usize>,
}

impl ValidationResult {
    /// Create a new empty validation result.
    fn new(file_size: usize) -> Self {
        Self {
            issues: Vec::new(),
            version: None,
            dataset_name: None,
            variable_count: None,
            observation_count: None,
            file_size,
            observation_length: None,
        }
    }

    /// Add a validation issue.
    fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Check if the file is valid (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    /// Get all validation issues.
    #[must_use]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Get issues of a specific severity.
    #[must_use]
    pub fn issues_by_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }

    /// Get errors only.
    #[must_use]
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues_by_severity(ValidationSeverity::Error)
    }

    /// Get warnings only.
    #[must_use]
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues_by_severity(ValidationSeverity::Warning)
    }

    /// Get info messages only.
    #[must_use]
    pub fn infos(&self) -> Vec<&ValidationIssue> {
        self.issues_by_severity(ValidationSeverity::Info)
    }

    /// Count errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .count()
    }

    /// Count warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count()
    }

    /// Count info messages.
    #[must_use]
    pub fn info_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Info)
            .count()
    }

    /// Generate a summary report.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== XPT Validation Report ===".to_string());
        lines.push(format!("File size: {} bytes", self.file_size));

        if let Some(version) = self.version {
            lines.push(format!("Version: {version}"));
        }

        if let Some(ref name) = self.dataset_name {
            lines.push(format!("Dataset: {name}"));
        }

        if let Some(vars) = self.variable_count {
            lines.push(format!("Variables: {vars}"));
        }

        if let Some(obs) = self.observation_count {
            lines.push(format!("Observations: {obs}"));
        }

        if let Some(len) = self.observation_length {
            lines.push(format!("Observation length: {len} bytes"));
        }

        lines.push(String::new());
        lines.push(format!(
            "Status: {} ({} errors, {} warnings, {} info)",
            if self.is_valid() { "VALID" } else { "INVALID" },
            self.error_count(),
            self.warning_count(),
            self.info_count()
        ));

        if !self.issues.is_empty() {
            lines.push(String::new());
            lines.push("Issues:".to_string());
            for issue in &self.issues {
                lines.push(format!("  {issue}"));
            }
        }

        lines.join("\n")
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// XPT file validator.
///
/// Provides comprehensive validation of XPT files according to the
/// SAS Transport specification.
pub struct XptValidator;

impl XptValidator {
    /// Validate an XPT file from a path.
    ///
    /// # Arguments
    /// * `path` - Path to the XPT file
    ///
    /// # Returns
    /// Validation result with all issues found.
    pub fn validate_file(path: &Path) -> Result<ValidationResult> {
        let mut file = File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                XptError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                XptError::Io(e)
            }
        })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(Self::validate_bytes(&data))
    }

    /// Validate XPT data from bytes.
    ///
    /// # Arguments
    /// * `data` - Raw XPT file bytes
    ///
    /// # Returns
    /// Validation result with all issues found.
    #[must_use]
    pub fn validate_bytes(data: &[u8]) -> ValidationResult {
        let mut result = ValidationResult::new(data.len());

        // Phase 1: Basic file structure validation
        Self::validate_file_structure(data, &mut result);

        if result.error_count() > 0 {
            // Cannot continue if basic structure is invalid
            return result;
        }

        // Phase 2: Header validation
        let offset = Self::validate_headers(data, &mut result);

        if result.error_count() > 0 || offset.is_none() {
            return result;
        }

        let obs_offset = offset.unwrap();

        // Phase 3: NAMESTR validation
        let columns_info = Self::validate_namestrs(data, &mut result);

        // Phase 4: Observation data validation
        if let Some((obs_len, var_count)) = columns_info {
            Self::validate_observations(data, obs_offset, obs_len, var_count, &mut result);
        }

        result
    }

    /// Validate basic file structure.
    fn validate_file_structure(data: &[u8], result: &mut ValidationResult) {
        // Minimum size check (at least headers)
        if data.len() < RECORD_LEN * 8 {
            result.add_issue(ValidationIssue::error(
                "XPT001",
                format!(
                    "File too small: {} bytes (minimum {} bytes required)",
                    data.len(),
                    RECORD_LEN * 8
                ),
                ValidationLocation::section("FILE"),
            ));
            return;
        }

        // Record alignment check
        if data.len() % RECORD_LEN != 0 {
            result.add_issue(ValidationIssue::error(
                "XPT002",
                format!(
                    "File length {} is not a multiple of record length ({})",
                    data.len(),
                    RECORD_LEN
                ),
                ValidationLocation::section("FILE"),
            ));
        }

        // Total record count
        let record_count = data.len() / RECORD_LEN;
        result.add_issue(ValidationIssue::info(
            "XPT000",
            format!(
                "File contains {} records ({} bytes)",
                record_count,
                data.len()
            ),
            ValidationLocation::section("FILE"),
        ));
    }

    /// Validate header records.
    /// Returns the offset to observation data if successful.
    fn validate_headers(data: &[u8], result: &mut ValidationResult) -> Option<usize> {
        let mut offset = 0usize;

        // Library header (record 0)
        let library_header = &data[offset..offset + RECORD_LEN];
        match validate_library_header(library_header) {
            Ok(version) => {
                result.version = Some(version);
                result.add_issue(ValidationIssue::info(
                    "XPT010",
                    format!("Detected XPT version: {version}"),
                    ValidationLocation::at_offset(offset, "LIBRARY_HEADER"),
                ));
            }
            Err(e) => {
                result.add_issue(ValidationIssue::error(
                    "XPT011",
                    format!("Invalid library header: {e}"),
                    ValidationLocation::at_offset(offset, "LIBRARY_HEADER"),
                ));
                return None;
            }
        }
        offset += RECORD_LEN;

        // Real header and modified header (records 1-2) - just validate they exist
        let real_header = &data[offset..offset + RECORD_LEN];
        Self::validate_real_header(real_header, offset, result);
        offset += RECORD_LEN;

        let modified_header = &data[offset..offset + RECORD_LEN];
        Self::validate_modified_header(modified_header, offset, result);
        offset += RECORD_LEN;

        // Member header (record 3)
        let member_header = &data[offset..offset + RECORD_LEN];
        let namestr_len = match validate_member_header(member_header) {
            Ok(_version) => {
                match parse_namestr_len(member_header) {
                    Ok(len) => {
                        // Validate NAMESTR length
                        if len != NAMESTR_LEN && len != NAMESTR_LEN_VAX {
                            result.add_issue(ValidationIssue::warning(
                                "XPT021",
                                format!(
                                    "Unusual NAMESTR length: {} (expected {} or {})",
                                    len, NAMESTR_LEN, NAMESTR_LEN_VAX
                                ),
                                ValidationLocation::at_offset(offset, "MEMBER_HEADER"),
                            ));
                        }
                        len
                    }
                    Err(e) => {
                        result.add_issue(ValidationIssue::error(
                            "XPT022",
                            format!("Failed to parse NAMESTR length: {e}"),
                            ValidationLocation::at_offset(offset, "MEMBER_HEADER"),
                        ));
                        return None;
                    }
                }
            }
            Err(e) => {
                result.add_issue(ValidationIssue::error(
                    "XPT020",
                    format!("Invalid member header: {e}"),
                    ValidationLocation::at_offset(offset, "MEMBER_HEADER"),
                ));
                return None;
            }
        };
        offset += RECORD_LEN;

        // DSCRPTR header (record 4)
        let dscrptr_header = &data[offset..offset + RECORD_LEN];
        if let Err(e) = validate_dscrptr_header(dscrptr_header) {
            result.add_issue(ValidationIssue::error(
                "XPT030",
                format!("Invalid DSCRPTR header: {e}"),
                ValidationLocation::at_offset(offset, "DSCRPTR_HEADER"),
            ));
            return None;
        }
        offset += RECORD_LEN;

        // Member data (record 5) - contains dataset name
        // Use the existing parse_dataset_name from header/member.rs
        let member_data = &data[offset..offset + RECORD_LEN];
        let version = result.version.unwrap_or(XptVersion::V5);
        result.dataset_name = parse_dataset_name(member_data, version).ok();
        if let Some(ref name) = result.dataset_name {
            result.add_issue(ValidationIssue::info(
                "XPT040",
                format!("Dataset name: {name}"),
                ValidationLocation::at_offset(offset, "MEMBER_DATA"),
            ));
        }
        offset += RECORD_LEN;

        // Member second (record 6) - contains label and type
        let _member_second = &data[offset..offset + RECORD_LEN];
        offset += RECORD_LEN;

        // NAMESTR header (record 7)
        let namestr_header = &data[offset..offset + RECORD_LEN];
        let var_count = match validate_namestr_header(namestr_header) {
            Ok(_) => match parse_variable_count(namestr_header, version) {
                Ok(count) => {
                    result.variable_count = Some(count);
                    result.add_issue(ValidationIssue::info(
                        "XPT050",
                        format!("Variable count: {count}"),
                        ValidationLocation::at_offset(offset, "NAMESTR_HEADER"),
                    ));
                    count
                }
                Err(e) => {
                    result.add_issue(ValidationIssue::error(
                        "XPT051",
                        format!("Failed to parse variable count: {e}"),
                        ValidationLocation::at_offset(offset, "NAMESTR_HEADER"),
                    ));
                    return None;
                }
            },
            Err(e) => {
                result.add_issue(ValidationIssue::error(
                    "XPT052",
                    format!("Invalid NAMESTR header: {e}"),
                    ValidationLocation::at_offset(offset, "NAMESTR_HEADER"),
                ));
                return None;
            }
        };
        offset += RECORD_LEN;

        // Skip NAMESTR records
        let namestr_total = var_count * namestr_len;
        offset += namestr_total;
        offset = align_to_record(offset);

        // Check for optional LABELV8/V9 section
        if offset + RECORD_LEN <= data.len() {
            let next_record = &data[offset..offset + RECORD_LEN];
            if is_label_header(next_record).is_some() {
                result.add_issue(ValidationIssue::info(
                    "XPT060",
                    "Extended label section (LABELV8/V9) detected",
                    ValidationLocation::at_offset(offset, "LABEL_HEADER"),
                ));
                offset += RECORD_LEN;

                // Find OBS header
                while offset + RECORD_LEN <= data.len() {
                    let check_record = &data[offset..offset + RECORD_LEN];
                    if validate_obs_header(check_record).is_ok() {
                        break;
                    }
                    offset += RECORD_LEN;
                }
            }
        }

        // OBS header
        if offset + RECORD_LEN > data.len() {
            result.add_issue(ValidationIssue::error(
                "XPT070",
                "Missing OBS header (file truncated)",
                ValidationLocation::section("OBS_HEADER"),
            ));
            return None;
        }

        let obs_header = &data[offset..offset + RECORD_LEN];
        if let Err(e) = validate_obs_header(obs_header) {
            result.add_issue(ValidationIssue::error(
                "XPT071",
                format!("Invalid OBS header: {e}"),
                ValidationLocation::at_offset(offset, "OBS_HEADER"),
            ));
            return None;
        }
        offset += RECORD_LEN;

        Some(offset)
    }

    /// Validate the real header record (record 1).
    fn validate_real_header(data: &[u8], offset: usize, result: &mut ValidationResult) {
        // Extract SAS version info
        let sas_version = std::str::from_utf8(&data[24..32]).unwrap_or("").trim();
        let os_version = std::str::from_utf8(&data[32..40]).unwrap_or("").trim();

        if !sas_version.is_empty() {
            result.add_issue(ValidationIssue::info(
                "XPT012",
                format!("SAS version: {sas_version}, OS: {os_version}"),
                ValidationLocation::at_offset(offset, "REAL_HEADER"),
            ));
        }

        // Validate creation datetime format
        let datetime_str = std::str::from_utf8(&data[40..56]).unwrap_or("").trim();
        if !datetime_str.is_empty() {
            Self::validate_datetime(datetime_str, offset, "creation", result);
        }
    }

    /// Validate the modified header record (record 2).
    fn validate_modified_header(data: &[u8], offset: usize, result: &mut ValidationResult) {
        // Validate modification datetime format
        let datetime_str = std::str::from_utf8(&data[0..16]).unwrap_or("").trim();
        if !datetime_str.is_empty() {
            Self::validate_datetime(datetime_str, offset, "modification", result);
        }
    }

    /// Validate datetime format (DDmmmYY:HH:MM:SS).
    fn validate_datetime(
        datetime_str: &str,
        offset: usize,
        kind: &str,
        result: &mut ValidationResult,
    ) {
        // Expected format: DDmmmYY:HH:MM:SS (16 chars)
        // e.g., "21AUG20:09:14:29"
        if datetime_str.len() < 16 {
            result.add_issue(ValidationIssue::warning(
                "XPT013",
                format!("Short {kind} datetime: '{datetime_str}'"),
                ValidationLocation::at_offset(offset, "DATETIME"),
            ));
            return;
        }

        // Validate month abbreviation
        let valid_months = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        if datetime_str.len() >= 5 {
            let month = &datetime_str[2..5];
            if !valid_months.contains(&month) {
                result.add_issue(ValidationIssue::warning(
                    "XPT014",
                    format!("Invalid month in {kind} datetime: '{month}'"),
                    ValidationLocation::at_offset(offset, "DATETIME"),
                ));
            }
        }
    }

    /// Validate NAMESTR records using the existing parse_namestr function.
    /// Returns (observation_length, variable_count) if successful.
    fn validate_namestrs(data: &[u8], result: &mut ValidationResult) -> Option<(usize, usize)> {
        let version = result.version.unwrap_or(XptVersion::V5);
        let var_count = result.variable_count?;

        // Calculate NAMESTR section start
        let offset = RECORD_LEN * 8; // After 8 header records

        // Get NAMESTR length from member header
        let member_header = &data[RECORD_LEN * 3..RECORD_LEN * 4];
        let namestr_len = parse_namestr_len(member_header).ok()?;

        let mut obs_len = 0usize;
        let mut seen_names: HashSet<String> = HashSet::new();

        for i in 0..var_count {
            let namestr_offset = offset + i * namestr_len;
            if namestr_offset + namestr_len > data.len() {
                result.add_issue(ValidationIssue::error(
                    "XPT080",
                    format!("NAMESTR record {} extends beyond file", i),
                    ValidationLocation::at_offset(namestr_offset, format!("NAMESTR[{i}]")),
                ));
                return None;
            }

            let namestr_data = &data[namestr_offset..namestr_offset + namestr_len];

            // Use the existing parse_namestr function from header/namestr.rs
            let column = match parse_namestr(namestr_data, namestr_len, i, version) {
                Ok(col) => col,
                Err(e) => {
                    result.add_issue(ValidationIssue::error(
                        "XPT081",
                        format!("Failed to parse NAMESTR[{}]: {}", i, e),
                        ValidationLocation::at_offset(namestr_offset, format!("NAMESTR[{i}]")),
                    ));
                    return None;
                }
            };

            // Additional validations not covered by parse_namestr

            // Validate numeric length (warning only)
            if column.data_type.is_numeric() && column.length > 8 {
                result.add_issue(ValidationIssue::warning(
                    "XPT083",
                    format!(
                        "Numeric variable '{}' length {} exceeds standard (max 8)",
                        column.name, column.length
                    ),
                    ValidationLocation::variable(&column.name, i),
                ));
            }

            // Check for duplicate names
            let upper_name = column.name.to_uppercase();
            if seen_names.contains(&upper_name) {
                result.add_issue(ValidationIssue::error(
                    "XPT085",
                    format!("Duplicate variable name: {}", column.name),
                    ValidationLocation::variable(&column.name, i),
                ));
            } else {
                seen_names.insert(upper_name);
            }

            // Validate variable name format
            Self::validate_variable_name(&column.name, i, version, result);

            // Build variable type string for logging
            let var_type = if column.data_type.is_numeric() {
                "Num"
            } else {
                "Char"
            };

            // Build format string for logging
            let fmt_str = if let Some(ref fmt) = column.format {
                format!(
                    " format={}{}.{}",
                    fmt, column.format_length, column.format_decimals
                )
            } else {
                String::new()
            };

            // Build label string for logging
            let label_str = if let Some(ref label) = column.label {
                format!(" label='{}'", label)
            } else {
                String::new()
            };

            // Log variable info
            result.add_issue(ValidationIssue::info(
                "XPT091",
                format!(
                    "Variable: {} ({}, len={}){}{}",
                    column.name, var_type, column.length, label_str, fmt_str
                ),
                ValidationLocation::section("NAMESTR"),
            ));

            obs_len += column.length as usize;
        }

        // Add info about total observation length
        result.add_issue(ValidationIssue::info(
            "XPT090",
            format!("Total observation length: {} bytes", obs_len),
            ValidationLocation::section("NAMESTR"),
        ));

        result.observation_length = Some(obs_len);
        Some((obs_len, var_count))
    }

    /// Validate a variable name.
    fn validate_variable_name(
        name: &str,
        index: usize,
        version: XptVersion,
        result: &mut ValidationResult,
    ) {
        let limit = version.name_limit();

        // Check length
        if name.len() > limit {
            result.add_issue(ValidationIssue::error(
                "XPT087",
                format!(
                    "Variable name '{}' exceeds {} limit of {} characters",
                    name, version, limit
                ),
                ValidationLocation::variable(name, index),
            ));
        }

        // Check first character (must be letter or underscore)
        if let Some(first) = name.chars().next() {
            if !first.is_ascii_alphabetic() && first != '_' {
                result.add_issue(ValidationIssue::error(
                    "XPT088",
                    format!(
                        "Variable name '{}' must start with letter or underscore",
                        name
                    ),
                    ValidationLocation::variable(name, index),
                ));
            }
        }

        // Check all characters (letters, digits, underscores only)
        for ch in name.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                result.add_issue(ValidationIssue::warning(
                    "XPT089",
                    format!(
                        "Variable name '{}' contains invalid character: '{}'",
                        name, ch
                    ),
                    ValidationLocation::variable(name, index),
                ));
                break;
            }
        }
    }

    /// Validate observation data.
    fn validate_observations(
        data: &[u8],
        obs_offset: usize,
        obs_len: usize,
        var_count: usize,
        result: &mut ValidationResult,
    ) {
        if obs_len == 0 {
            result.observation_count = Some(0);
            return;
        }

        let data_len = data.len().saturating_sub(obs_offset);
        let mut obs_count = data_len / obs_len;
        let remainder = data_len % obs_len;

        // Check for non-padding trailing bytes
        if remainder != 0 {
            let start = obs_offset + obs_count * obs_len;
            let trailing = &data[start..];
            if trailing.iter().any(|&b| b != b' ') {
                result.add_issue(ValidationIssue::error(
                    "XPT100",
                    format!(
                        "Non-padding trailing bytes: {} bytes after {} complete observations",
                        remainder, obs_count
                    ),
                    ValidationLocation::at_offset(start, "OBS_TRAILING"),
                ));
            } else {
                result.add_issue(ValidationIssue::info(
                    "XPT101",
                    format!("Trailing padding: {} bytes", remainder),
                    ValidationLocation::section("OBS_TRAILING"),
                ));
            }
        }

        // Trim trailing all-space rows
        let mut actual_obs = obs_count;
        while actual_obs > 0 {
            let start = obs_offset + (actual_obs - 1) * obs_len;
            let row_bytes = &data[start..start + obs_len];
            if row_bytes.iter().all(|&b| b == b' ') {
                actual_obs -= 1;
            } else {
                break;
            }
        }

        if actual_obs < obs_count {
            result.add_issue(ValidationIssue::info(
                "XPT102",
                format!("Trailing empty rows removed: {}", obs_count - actual_obs),
                ValidationLocation::section("OBS"),
            ));
            obs_count = actual_obs;
        }

        result.observation_count = Some(obs_count);

        result.add_issue(ValidationIssue::info(
            "XPT103",
            format!("Observation count: {}", obs_count),
            ValidationLocation::section("OBS"),
        ));

        // Validate a sample of observations
        Self::validate_observation_sample(data, obs_offset, obs_len, var_count, obs_count, result);
    }

    /// Validate a sample of observations for data integrity.
    fn validate_observation_sample(
        data: &[u8],
        obs_offset: usize,
        obs_len: usize,
        var_count: usize,
        obs_count: usize,
        result: &mut ValidationResult,
    ) {
        if obs_count == 0 {
            return;
        }

        let version = result.version.unwrap_or(XptVersion::V5);

        // Get variable info from NAMESTR using the existing parse_namestr function
        let member_header = &data[RECORD_LEN * 3..RECORD_LEN * 4];
        let namestr_len = match parse_namestr_len(member_header) {
            Ok(len) => len,
            Err(_) => return,
        };

        let namestr_offset = RECORD_LEN * 8;
        let mut var_info: Vec<(String, bool, usize)> = Vec::new(); // (name, is_numeric, length)

        for i in 0..var_count {
            let ns_offset = namestr_offset + i * namestr_len;
            if ns_offset + namestr_len > data.len() {
                break;
            }
            let namestr_data = &data[ns_offset..ns_offset + namestr_len];

            // Use the existing parse_namestr function
            if let Ok(column) = parse_namestr(namestr_data, namestr_len, i, version) {
                var_info.push((
                    column.name,
                    column.data_type.is_numeric(),
                    column.length as usize,
                ));
            }
        }

        // Sample some rows (first, last, and some in the middle)
        let sample_rows: Vec<usize> = if obs_count <= 10 {
            (0..obs_count).collect()
        } else {
            let mut samples = vec![0, 1, 2];
            samples.push(obs_count / 2);
            samples.push(obs_count - 3);
            samples.push(obs_count - 2);
            samples.push(obs_count - 1);
            samples
        };

        let mut missing_counts: Vec<usize> = vec![0; var_count];
        let mut numeric_errors = 0;

        for row_idx in 0..obs_count {
            let row_start = obs_offset + row_idx * obs_len;
            if row_start + obs_len > data.len() {
                break;
            }

            let row_data = &data[row_start..row_start + obs_len];
            let mut var_pos = 0usize;

            for (var_idx, (name, is_numeric, length)) in var_info.iter().enumerate() {
                let var_data = &row_data[var_pos..var_pos + length];

                if *is_numeric {
                    // Numeric - check for valid IBM float or missing value
                    if is_missing(var_data).is_some() {
                        missing_counts[var_idx] += 1;
                    } else {
                        // Basic validation: first byte should be valid IBM exponent
                        // IBM float: first byte is sign (1 bit) + exponent (7 bits)
                        // For non-zero values, exponent should typically be 0x40-0x4F or similar
                        let first_byte = var_data[0];
                        if first_byte != 0 {
                            let exp = first_byte & 0x7F;
                            // Valid exponent range for reasonable values
                            if exp > 0x50 && first_byte != 0x2E {
                                // Only report if this is a sampled row
                                if sample_rows.contains(&row_idx) && numeric_errors < 5 {
                                    result.add_issue(ValidationIssue::warning(
                                        "XPT110",
                                        format!(
                                            "Unusual numeric value in {} (row {}): exponent 0x{:02X}",
                                            name, row_idx, exp
                                        ),
                                        ValidationLocation::observation(row_idx, Some(name.clone())),
                                    ));
                                    numeric_errors += 1;
                                }
                            }
                        }
                    }
                }

                var_pos += length;
            }
        }

        // Report missing value statistics
        for (var_idx, (name, is_numeric, _)) in var_info.iter().enumerate() {
            if *is_numeric && missing_counts[var_idx] > 0 {
                let pct = (missing_counts[var_idx] as f64 / obs_count as f64) * 100.0;
                if pct > 50.0 {
                    result.add_issue(ValidationIssue::warning(
                        "XPT111",
                        format!(
                            "High missing rate in {}: {}/{} ({:.1}%)",
                            name, missing_counts[var_idx], obs_count, pct
                        ),
                        ValidationLocation::section("OBS"),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_data_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
    }

    #[test]
    fn test_validate_lb_xpt() {
        let path = test_data_path().join("lb.xpt");
        if path.exists() {
            let result = XptValidator::validate_file(&path).expect("Failed to validate");

            println!("{}", result.summary());

            // Should be valid
            assert!(result.is_valid(), "lb.xpt should be valid");

            // Should have detected version
            assert!(result.version.is_some());

            // Should have dataset name
            assert_eq!(result.dataset_name.as_deref(), Some("LB"));

            // Should have 23 variables
            assert_eq!(result.variable_count, Some(23));

            // Should have observations
            assert!(result.observation_count.unwrap_or(0) > 0);
        }
    }

    #[test]
    fn test_validation_severity() {
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_empty_data() {
        let result = XptValidator::validate_bytes(&[]);
        assert!(!result.is_valid());
        assert!(result.error_count() > 0);
    }

    #[test]
    fn test_small_data() {
        let result = XptValidator::validate_bytes(&[0u8; 100]);
        assert!(!result.is_valid());
        assert!(result.errors().iter().any(|e| e.code == "XPT001"));
    }

    #[test]
    fn test_unaligned_data() {
        let result = XptValidator::validate_bytes(&[0u8; 641]); // Not multiple of 80
        assert!(!result.is_valid());
        assert!(result.errors().iter().any(|e| e.code == "XPT002"));
    }
}
