//! Library header record handling.
//!
//! The library header is the first set of records in an XPT file.
//!
//! # Structure
//!
//! 1. Fixed header: `HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!...`
//! 2. Real header (80 bytes): SAS version, OS, created datetime
//! 3. Second header (80 bytes): Modified datetime

use crate::error::{Result, XptError};
use crate::types::XptWriterOptions;

use super::datetime::parse_xpt_datetime;

/// Record length in bytes.
pub const RECORD_LEN: usize = 80;

/// Library header prefix.
pub const LIBRARY_HEADER_PREFIX: &str = "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!";

/// Library header information parsed from real headers.
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    /// SAS version string.
    pub sas_version: String,
    /// Operating system name.
    pub os_name: String,
    /// Created datetime string (ddMMMyy:hh:mm:ss).
    pub created: String,
    /// Modified datetime string.
    pub modified: String,
}

impl Default for LibraryInfo {
    fn default() -> Self {
        Self {
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: "01JAN70:00:00:00".to_string(),
            modified: "01JAN70:00:00:00".to_string(),
        }
    }
}

impl From<&XptWriterOptions> for LibraryInfo {
    fn from(opts: &XptWriterOptions) -> Self {
        Self {
            sas_version: opts.sas_version.clone(),
            os_name: opts.os_name.clone(),
            created: opts.format_created(),
            modified: opts.format_modified(),
        }
    }
}

/// Validate that a record starts with the library header prefix.
///
/// # Arguments
/// * `record` - 80-byte record
///
/// # Returns
/// Ok(()) if valid, error otherwise.
pub fn validate_library_header(record: &[u8]) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("record too short"));
    }
    if !record.starts_with(LIBRARY_HEADER_PREFIX.as_bytes()) {
        return Err(XptError::missing_header("LIBRARY HEADER"));
    }
    Ok(())
}

/// Parse the library real header (first 80 bytes after fixed header).
///
/// # Structure
///
/// | Offset | Length | Field       | Description              |
/// |--------|--------|-------------|--------------------------|
/// | 0-7    | 8      | sas_symbol1 | "SAS     "               |
/// | 8-15   | 8      | sas_symbol2 | "SAS     "               |
/// | 16-23  | 8      | saslib      | "SASLIB  "               |
/// | 24-31  | 8      | sasver      | SAS version              |
/// | 32-39  | 8      | sas_os      | Operating system         |
/// | 40-63  | 24     | blanks      | Spaces                   |
/// | 64-79  | 16     | created     | Created datetime         |
pub fn parse_real_header(record: &[u8]) -> Result<LibraryInfo> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("real header too short"));
    }

    let sas_version = read_string(record, 24, 8);
    let os_name = read_string(record, 32, 8);
    let created = read_string(record, 64, 16);

    Ok(LibraryInfo {
        sas_version,
        os_name,
        created,
        modified: String::new(), // Will be filled from second header
    })
}

/// Parse the second header (modified datetime).
///
/// The modified datetime is at offset 0-15 of this record.
pub fn parse_second_header(record: &[u8]) -> String {
    if record.len() < 16 {
        return String::new();
    }
    read_string(record, 0, 16)
}

/// Build the fixed library header record.
#[must_use]
pub fn build_library_header() -> [u8; RECORD_LEN] {
    build_fixed_header(LIBRARY_HEADER_PREFIX)
}

/// Build the real header record with library info.
#[must_use]
pub fn build_real_header(info: &LibraryInfo) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // sas_symbol1: "SAS     "
    write_string(&mut record, 0, "SAS", 8);

    // sas_symbol2: "SAS     "
    write_string(&mut record, 8, "SAS", 8);

    // saslib: "SASLIB  "
    write_string(&mut record, 16, "SASLIB", 8);

    // sasver: SAS version
    write_string(&mut record, 24, &info.sas_version, 8);

    // sas_os: Operating system
    write_string(&mut record, 32, &info.os_name, 8);

    // blanks: 24 spaces (already set by initialization)

    // created: datetime
    write_string(&mut record, 64, &info.created, 16);

    record
}

/// Build the second header record (modified datetime).
#[must_use]
pub fn build_second_header(modified: &str) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];
    write_string(&mut record, 0, modified, 16);
    record
}

/// Build a fixed header record with the given prefix.
fn build_fixed_header(prefix: &str) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // Copy prefix (48 bytes)
    let prefix_bytes = prefix.as_bytes();
    let copy_len = prefix_bytes.len().min(48);
    record[..copy_len].copy_from_slice(&prefix_bytes[..copy_len]);

    // Fill with '0' characters from offset 48 to 78
    for i in 48..78 {
        record[i] = b'0';
    }

    // Last 2 bytes are spaces (already set)

    record
}

/// Read a string from a byte slice, trimming trailing spaces.
fn read_string(data: &[u8], offset: usize, len: usize) -> String {
    data.get(offset..offset + len)
        .map(|slice| String::from_utf8_lossy(slice).trim_end().to_string())
        .unwrap_or_default()
}

/// Write a string to a buffer, space-padded.
fn write_string(buf: &mut [u8], offset: usize, value: &str, len: usize) {
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(len);
    buf[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
    // Remaining bytes are already spaces from initialization
}

/// Extract created datetime from library info.
#[must_use]
pub fn get_created_datetime(info: &LibraryInfo) -> Option<chrono::NaiveDateTime> {
    parse_xpt_datetime(&info.created)
}

/// Extract modified datetime from library info.
#[must_use]
pub fn get_modified_datetime(info: &LibraryInfo) -> Option<chrono::NaiveDateTime> {
    parse_xpt_datetime(&info.modified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_library_header() {
        let header = build_library_header();
        assert!(validate_library_header(&header).is_ok());

        let invalid = [b'X'; RECORD_LEN];
        assert!(validate_library_header(&invalid).is_err());
    }

    #[test]
    fn test_build_and_parse_real_header() {
        let info = LibraryInfo {
            sas_version: "9.4".to_string(),
            os_name: "LINUX".to_string(),
            created: "15MAR24:14:30:45".to_string(),
            modified: "15MAR24:15:00:00".to_string(),
        };

        let record = build_real_header(&info);
        let parsed = parse_real_header(&record).unwrap();

        assert_eq!(parsed.sas_version, "9.4");
        assert_eq!(parsed.os_name, "LINUX");
        assert_eq!(parsed.created, "15MAR24:14:30:45");
    }

    #[test]
    fn test_build_second_header() {
        let modified = "15MAR24:15:00:00";
        let record = build_second_header(modified);
        let parsed = parse_second_header(&record);
        assert_eq!(parsed, modified);
    }

    #[test]
    fn test_fixed_header_structure() {
        let header = build_library_header();

        // Check prefix
        assert!(header.starts_with(LIBRARY_HEADER_PREFIX.as_bytes()));

        // Check zeros section
        for i in 48..78 {
            assert_eq!(header[i], b'0', "byte {i} should be '0'");
        }

        // Check trailing spaces
        assert_eq!(header[78], b' ');
        assert_eq!(header[79], b' ');
    }

    #[test]
    fn test_library_info_from_options() {
        use chrono::NaiveDate;

        let dt = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap();

        let opts = XptWriterOptions::new()
            .with_sas_version("9.3")
            .with_os_name("WIN")
            .with_created(dt)
            .with_modified(dt);

        let info: LibraryInfo = (&opts).into();

        assert_eq!(info.sas_version, "9.3");
        assert_eq!(info.os_name, "WIN");
        assert!(info.created.contains("MAR"));
    }
}
