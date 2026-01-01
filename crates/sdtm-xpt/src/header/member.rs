//! Member header record handling.
//!
//! Each dataset (member) in an XPT file has its own set of header records.
//!
//! # Structure
//!
//! 1. Member header: `HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!...`
//! 2. DSCRPTR header: `HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!...`
//! 3. Member data (80 bytes): Dataset name, version, etc.
//! 4. Member second (80 bytes): Modified datetime, label, type
//! 5. NAMESTR header: `HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!...`
//! 6. NAMESTR records: Variable definitions
//! 7. OBS header: `HEADER RECORD*******OBS     HEADER RECORD!!!!!!!...`
//! 8. Observation data

use crate::error::{Result, XptError};
use crate::types::{XptDataset, XptVersion, XptWriterOptions};

use super::common::{
    DSCRPTR_HEADER_V5, DSCRPTR_HEADER_V8, MEMBER_HEADER_V5, MEMBER_HEADER_V8, NAMESTR_HEADER_V5,
    NAMESTR_HEADER_V8, OBS_HEADER_V5, OBS_HEADER_V8, RECORD_LEN, align_to_record as align_offset,
    build_header_record, read_string, write_string,
};

// Re-export align_to_record for backward compatibility
pub use super::common::align_to_record;

/// Detect the member header version from record prefix.
#[must_use]
pub fn detect_member_version(record: &[u8]) -> Option<XptVersion> {
    if record.len() < RECORD_LEN {
        return None;
    }
    if record.starts_with(MEMBER_HEADER_V5.as_bytes()) {
        Some(XptVersion::V5)
    } else if record.starts_with(MEMBER_HEADER_V8.as_bytes()) {
        Some(XptVersion::V8)
    } else {
        None
    }
}

/// Validate a member header record (auto-detect version).
pub fn validate_member_header(record: &[u8]) -> Result<XptVersion> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("member header too short"));
    }
    detect_member_version(record).ok_or_else(|| XptError::missing_header("MEMBER or MEMBV8 HEADER"))
}

/// Validate a member header for a specific version.
pub fn validate_member_header_version(record: &[u8], version: XptVersion) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("member header too short"));
    }
    let prefix = match version {
        XptVersion::V5 => MEMBER_HEADER_V5,
        XptVersion::V8 => MEMBER_HEADER_V8,
    };
    if !record.starts_with(prefix.as_bytes()) {
        return Err(XptError::missing_header(match version {
            XptVersion::V5 => "MEMBER HEADER",
            XptVersion::V8 => "MEMBV8 HEADER",
        }));
    }
    Ok(())
}

/// Validate a DSCRPTR header record (auto-detect version).
pub fn validate_dscrptr_header(record: &[u8]) -> Result<XptVersion> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("dscrptr header too short"));
    }
    if record.starts_with(DSCRPTR_HEADER_V5.as_bytes()) {
        Ok(XptVersion::V5)
    } else if record.starts_with(DSCRPTR_HEADER_V8.as_bytes()) {
        Ok(XptVersion::V8)
    } else {
        Err(XptError::missing_header("DSCRPTR or DSCPTV8 HEADER"))
    }
}

/// Validate a DSCRPTR header for a specific version.
pub fn validate_dscrptr_header_version(record: &[u8], version: XptVersion) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("dscrptr header too short"));
    }
    let prefix = match version {
        XptVersion::V5 => DSCRPTR_HEADER_V5,
        XptVersion::V8 => DSCRPTR_HEADER_V8,
    };
    if !record.starts_with(prefix.as_bytes()) {
        return Err(XptError::missing_header(match version {
            XptVersion::V5 => "DSCRPTR HEADER",
            XptVersion::V8 => "DSCPTV8 HEADER",
        }));
    }
    Ok(())
}

/// Validate a NAMESTR header record (auto-detect version).
pub fn validate_namestr_header(record: &[u8]) -> Result<XptVersion> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("namestr header too short"));
    }
    if record.starts_with(NAMESTR_HEADER_V5.as_bytes()) {
        Ok(XptVersion::V5)
    } else if record.starts_with(NAMESTR_HEADER_V8.as_bytes()) {
        Ok(XptVersion::V8)
    } else {
        Err(XptError::missing_header("NAMESTR or NAMSTV8 HEADER"))
    }
}

/// Validate a NAMESTR header for a specific version.
pub fn validate_namestr_header_version(record: &[u8], version: XptVersion) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("namestr header too short"));
    }
    let prefix = match version {
        XptVersion::V5 => NAMESTR_HEADER_V5,
        XptVersion::V8 => NAMESTR_HEADER_V8,
    };
    if !record.starts_with(prefix.as_bytes()) {
        return Err(XptError::missing_header(match version {
            XptVersion::V5 => "NAMESTR HEADER",
            XptVersion::V8 => "NAMSTV8 HEADER",
        }));
    }
    Ok(())
}

/// Validate an OBS header record (auto-detect version).
pub fn validate_obs_header(record: &[u8]) -> Result<XptVersion> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("obs header too short"));
    }
    if record.starts_with(OBS_HEADER_V5.as_bytes()) {
        Ok(XptVersion::V5)
    } else if record.starts_with(OBS_HEADER_V8.as_bytes()) {
        Ok(XptVersion::V8)
    } else {
        Err(XptError::missing_header("OBS or OBSV8 HEADER"))
    }
}

/// Validate an OBS header for a specific version.
pub fn validate_obs_header_version(record: &[u8], version: XptVersion) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("obs header too short"));
    }
    let prefix = match version {
        XptVersion::V5 => OBS_HEADER_V5,
        XptVersion::V8 => OBS_HEADER_V8,
    };
    if !record.starts_with(prefix.as_bytes()) {
        return Err(XptError::missing_header(match version {
            XptVersion::V5 => "OBS HEADER",
            XptVersion::V8 => "OBSV8 HEADER",
        }));
    }
    Ok(())
}

/// Parse NAMESTR length from member header record.
///
/// The NAMESTR length is at offset 74-77 (4 ASCII digits).
/// Returns 140 (standard) or 136 (VAX/VMS).
pub fn parse_namestr_len(record: &[u8]) -> Result<usize> {
    if record.len() < 78 {
        return Err(XptError::invalid_format("member header too short"));
    }
    let text = read_string(record, 74, 4);
    text.trim()
        .parse::<usize>()
        .map_err(|_| XptError::NumericParse {
            field: "NAMESTR length".to_string(),
        })
}

/// Parse variable count from NAMESTR header record.
///
/// The variable count is at offset 54-57 (4 ASCII digits).
pub fn parse_variable_count(record: &[u8]) -> Result<usize> {
    if record.len() < 58 {
        return Err(XptError::invalid_format("namestr header too short"));
    }
    let text = read_string(record, 54, 4);
    text.trim()
        .parse::<usize>()
        .map_err(|_| XptError::NumericParse {
            field: "variable count".to_string(),
        })
}

/// Parse dataset name from member data record.
///
/// Dataset name is at offset 8-15 (8 characters).
/// Parse dataset name from member data record.
///
/// For V8 format, checks for extended name at offset 40-63 first.
/// Falls back to short name at offset 8-15.
pub fn parse_dataset_name(record: &[u8]) -> Result<String> {
    if record.len() < 16 {
        return Err(XptError::invalid_format("member data too short"));
    }

    // V8: Check for extended name at offset 40-63 (24 chars)
    if record.len() >= 64 {
        let extended_name = read_string(record, 40, 24);
        if !extended_name.is_empty() {
            return Ok(extended_name);
        }
    }

    // Fall back to short name at offset 8-15
    let name = read_string(record, 8, 8);
    if name.is_empty() {
        return Err(XptError::invalid_format("empty dataset name"));
    }
    Ok(name)
}

/// Parse dataset label from member second record.
///
/// Dataset label is at offset 32-71 (40 characters).
pub fn parse_dataset_label(record: &[u8]) -> Option<String> {
    if record.len() < 72 {
        return None;
    }
    let label = read_string(record, 32, 40);
    if label.is_empty() { None } else { Some(label) }
}

/// Parse dataset type from member second record.
///
/// Dataset type is at offset 72-79 (8 characters).
pub fn parse_dataset_type(record: &[u8]) -> Option<String> {
    if record.len() < 80 {
        return None;
    }
    let dtype = read_string(record, 72, 8);
    if dtype.is_empty() { None } else { Some(dtype) }
}

/// Build member header record with NAMESTR length for specified version.
#[must_use]
pub fn build_member_header(version: XptVersion, namestr_len: usize) -> [u8; RECORD_LEN] {
    let prefix = match version {
        XptVersion::V5 => MEMBER_HEADER_V5,
        XptVersion::V8 => MEMBER_HEADER_V8,
    };
    let mut record = build_header_record(prefix);

    // Observation header size at offset 64-67: "0160"
    write_string(&mut record, 64, "0160", 4);

    // NAMESTR length at offset 74-77: "0140" or "0136"
    let len_str = format!("{:04}", namestr_len);
    write_string(&mut record, 74, &len_str, 4);

    record
}

/// Build DSCRPTR header record for specified version.
#[must_use]
pub fn build_dscrptr_header(version: XptVersion) -> [u8; RECORD_LEN] {
    let prefix = match version {
        XptVersion::V5 => DSCRPTR_HEADER_V5,
        XptVersion::V8 => DSCRPTR_HEADER_V8,
    };
    build_header_record(prefix)
}

/// Build NAMESTR header record for specified version.
///
/// # Arguments
/// * `version` - XPT version (V5 or V8)
/// * `var_count` - Number of variables in the dataset
#[must_use]
pub fn build_namestr_header(version: XptVersion, var_count: usize) -> [u8; RECORD_LEN] {
    let prefix = match version {
        XptVersion::V5 => NAMESTR_HEADER_V5,
        XptVersion::V8 => NAMESTR_HEADER_V8,
    };
    let mut record = build_header_record(prefix);

    // Variable count at offset 54-57 (4 chars, right-justified)
    let count_str = format!("{:>4}", var_count);
    write_string(&mut record, 54, &count_str, 4);

    record
}

/// Build OBS header record for specified version.
#[must_use]
pub fn build_obs_header(version: XptVersion) -> [u8; RECORD_LEN] {
    let prefix = match version {
        XptVersion::V5 => OBS_HEADER_V5,
        XptVersion::V8 => OBS_HEADER_V8,
    };
    build_header_record(prefix)
}

/// Build member data record.
#[must_use]
pub fn build_member_data(dataset: &XptDataset, options: &XptWriterOptions) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // sas_symbol: "SAS     "
    write_string(&mut record, 0, "SAS", 8);

    // dsname: dataset name (short, 8 chars)
    // For V8 with long names, the full name goes at offset 40
    let short_name = if dataset.name.len() > 8 {
        &dataset.name[..8]
    } else {
        &dataset.name
    };
    write_string(&mut record, 8, short_name, 8);

    // sasdata: "SASDATA "
    write_string(&mut record, 16, "SASDATA", 8);

    // sasver: SAS version
    write_string(&mut record, 24, &options.sas_version, 8);

    // sas_os: Operating system
    write_string(&mut record, 32, &options.os_name, 8);

    // V8: Write long dataset name at offset 40-63 (24 chars max)
    // Combined with short name at offset 8, this gives us the full 32-char limit
    if options.version.supports_long_names() && dataset.name.len() > 8 {
        write_string(&mut record, 40, &dataset.name, 24);
    }
    // Otherwise: blanks (already set)

    // created: datetime
    write_string(&mut record, 64, &options.format_created(), 16);

    record
}

/// Build member second record.
#[must_use]
pub fn build_member_second(dataset: &XptDataset, options: &XptWriterOptions) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // modified: datetime
    write_string(&mut record, 0, &options.format_modified(), 16);

    // blanks: 16 spaces (already set)

    // dslabel: dataset label (40 chars)
    let label = dataset.effective_label();
    write_string(&mut record, 32, label, 40);

    // dstype: dataset type (8 chars)
    let dtype = dataset.dataset_type.as_deref().unwrap_or("");
    write_string(&mut record, 72, dtype, 8);

    record
}

/// Calculate total NAMESTR block size including padding.
#[must_use]
pub fn namestr_block_size(var_count: usize, namestr_len: usize) -> usize {
    let total = var_count * namestr_len;
    align_offset(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::NAMESTR_LEN;

    #[test]
    fn test_validate_headers_v5() {
        let version = XptVersion::V5;
        assert_eq!(
            validate_member_header(&build_member_header(version, NAMESTR_LEN)).unwrap(),
            version
        );
        assert_eq!(
            validate_dscrptr_header(&build_dscrptr_header(version)).unwrap(),
            version
        );
        assert_eq!(
            validate_namestr_header(&build_namestr_header(version, 5)).unwrap(),
            version
        );
        assert_eq!(
            validate_obs_header(&build_obs_header(version)).unwrap(),
            version
        );

        let invalid = [b'X'; RECORD_LEN];
        assert!(validate_member_header(&invalid).is_err());
    }

    #[test]
    fn test_validate_headers_v8() {
        let version = XptVersion::V8;
        assert_eq!(
            validate_member_header(&build_member_header(version, NAMESTR_LEN)).unwrap(),
            version
        );
        assert_eq!(
            validate_dscrptr_header(&build_dscrptr_header(version)).unwrap(),
            version
        );
        assert_eq!(
            validate_namestr_header(&build_namestr_header(version, 5)).unwrap(),
            version
        );
        assert_eq!(
            validate_obs_header(&build_obs_header(version)).unwrap(),
            version
        );
    }

    #[test]
    fn test_validate_header_version_mismatch() {
        // V5 header should fail V8 validation
        let v5_member = build_member_header(XptVersion::V5, NAMESTR_LEN);
        assert!(validate_member_header_version(&v5_member, XptVersion::V8).is_err());

        // V8 header should fail V5 validation
        let v8_member = build_member_header(XptVersion::V8, NAMESTR_LEN);
        assert!(validate_member_header_version(&v8_member, XptVersion::V5).is_err());
    }

    #[test]
    fn test_parse_namestr_len() {
        let header = build_member_header(XptVersion::V5, 140);
        assert_eq!(parse_namestr_len(&header).unwrap(), 140);

        let header = build_member_header(XptVersion::V8, 136);
        assert_eq!(parse_namestr_len(&header).unwrap(), 136);
    }

    #[test]
    fn test_parse_variable_count() {
        let header = build_namestr_header(XptVersion::V5, 25);
        assert_eq!(parse_variable_count(&header).unwrap(), 25);

        let header = build_namestr_header(XptVersion::V8, 50);
        assert_eq!(parse_variable_count(&header).unwrap(), 50);
    }

    #[test]
    fn test_build_and_parse_member_data() {
        let dataset = XptDataset::new("DM")
            .with_label("Demographics")
            .with_type("DATA");

        let options = XptWriterOptions::default();
        let record = build_member_data(&dataset, &options);

        let name = parse_dataset_name(&record).unwrap();
        assert_eq!(name, "DM");
    }

    #[test]
    fn test_build_and_parse_member_second() {
        let dataset = XptDataset::new("AE").with_label("Adverse Events");

        let options = XptWriterOptions::default();
        let record = build_member_second(&dataset, &options);

        let label = parse_dataset_label(&record);
        assert_eq!(label, Some("Adverse Events".to_string()));
    }

    #[test]
    fn test_align_to_record() {
        assert_eq!(align_to_record(0), 0);
        assert_eq!(align_to_record(80), 80);
        assert_eq!(align_to_record(81), 160);
        assert_eq!(align_to_record(160), 160);
        assert_eq!(align_to_record(140), 160);
        assert_eq!(align_to_record(280), 320);
    }

    #[test]
    fn test_namestr_block_size() {
        // 1 variable × 140 bytes = 140, aligned to 160
        assert_eq!(namestr_block_size(1, 140), 160);

        // 2 variables × 140 bytes = 280, aligned to 320
        assert_eq!(namestr_block_size(2, 140), 320);

        // 10 variables × 140 bytes = 1400, aligned to 1440
        assert_eq!(namestr_block_size(10, 140), 1440);
    }
}
