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
use crate::types::{XptDataset, XptWriterOptions};

use super::library::RECORD_LEN;

/// Member header prefix.
pub const MEMBER_HEADER_PREFIX: &str = "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!";

/// DSCRPTR header prefix.
pub const DSCRPTR_HEADER_PREFIX: &str = "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!";

/// NAMESTR header prefix.
pub const NAMESTR_HEADER_PREFIX: &str = "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!";

/// OBS header prefix.
pub const OBS_HEADER_PREFIX: &str = "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!";

/// Validate a member header record.
pub fn validate_member_header(record: &[u8]) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("member header too short"));
    }
    if !record.starts_with(MEMBER_HEADER_PREFIX.as_bytes()) {
        return Err(XptError::missing_header("MEMBER HEADER"));
    }
    Ok(())
}

/// Validate a DSCRPTR header record.
pub fn validate_dscrptr_header(record: &[u8]) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("dscrptr header too short"));
    }
    if !record.starts_with(DSCRPTR_HEADER_PREFIX.as_bytes()) {
        return Err(XptError::missing_header("DSCRPTR HEADER"));
    }
    Ok(())
}

/// Validate a NAMESTR header record.
pub fn validate_namestr_header(record: &[u8]) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("namestr header too short"));
    }
    if !record.starts_with(NAMESTR_HEADER_PREFIX.as_bytes()) {
        return Err(XptError::missing_header("NAMESTR HEADER"));
    }
    Ok(())
}

/// Validate an OBS header record.
pub fn validate_obs_header(record: &[u8]) -> Result<()> {
    if record.len() < RECORD_LEN {
        return Err(XptError::invalid_format("obs header too short"));
    }
    if !record.starts_with(OBS_HEADER_PREFIX.as_bytes()) {
        return Err(XptError::missing_header("OBS HEADER"));
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
pub fn parse_dataset_name(record: &[u8]) -> Result<String> {
    if record.len() < 16 {
        return Err(XptError::invalid_format("member data too short"));
    }
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

/// Build member header record with NAMESTR length.
#[must_use]
pub fn build_member_header(namestr_len: usize) -> [u8; RECORD_LEN] {
    let mut record = build_fixed_header(MEMBER_HEADER_PREFIX);

    // Observation header size at offset 64-67: "0160"
    write_string(&mut record, 64, "0160", 4);

    // NAMESTR length at offset 74-77: "0140" or "0136"
    let len_str = format!("{:04}", namestr_len);
    write_string(&mut record, 74, &len_str, 4);

    record
}

/// Build DSCRPTR header record.
#[must_use]
pub fn build_dscrptr_header() -> [u8; RECORD_LEN] {
    build_fixed_header(DSCRPTR_HEADER_PREFIX)
}

/// Build member data record.
#[must_use]
pub fn build_member_data(dataset: &XptDataset, options: &XptWriterOptions) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // sas_symbol: "SAS     "
    write_string(&mut record, 0, "SAS", 8);

    // dsname: dataset name
    write_string(&mut record, 8, &dataset.name, 8);

    // sasdata: "SASDATA "
    write_string(&mut record, 16, "SASDATA", 8);

    // sasver: SAS version
    write_string(&mut record, 24, &options.sas_version, 8);

    // sas_os: Operating system
    write_string(&mut record, 32, &options.os_name, 8);

    // blanks: 24 spaces (already set)

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

/// Build NAMESTR header record with variable count.
#[must_use]
pub fn build_namestr_header(var_count: usize) -> [u8; RECORD_LEN] {
    let mut record = build_fixed_header(NAMESTR_HEADER_PREFIX);

    // Variable count at offset 54-57
    let count_str = format!("{:04}", var_count);
    write_string(&mut record, 54, &count_str, 4);

    record
}

/// Build OBS header record.
#[must_use]
pub fn build_obs_header() -> [u8; RECORD_LEN] {
    build_fixed_header(OBS_HEADER_PREFIX)
}

/// Build a fixed header record.
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

    record
}

/// Read a string from bytes, trimming trailing spaces.
fn read_string(data: &[u8], offset: usize, len: usize) -> String {
    data.get(offset..offset + len)
        .map(|slice| String::from_utf8_lossy(slice).trim_end().to_string())
        .unwrap_or_default()
}

/// Write a string to buffer, space-padded.
fn write_string(buf: &mut [u8], offset: usize, value: &str, len: usize) {
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(len);
    buf[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
}

/// Calculate total NAMESTR block size including padding.
#[must_use]
pub fn namestr_block_size(var_count: usize, namestr_len: usize) -> usize {
    let total = var_count * namestr_len;
    align_to_record(total)
}

/// Align a size to the next record boundary (80 bytes).
#[must_use]
pub fn align_to_record(size: usize) -> usize {
    if size % RECORD_LEN == 0 {
        size
    } else {
        size + (RECORD_LEN - (size % RECORD_LEN))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::NAMESTR_LEN;

    #[test]
    fn test_validate_headers() {
        assert!(validate_member_header(&build_member_header(NAMESTR_LEN)).is_ok());
        assert!(validate_dscrptr_header(&build_dscrptr_header()).is_ok());
        assert!(validate_namestr_header(&build_namestr_header(5)).is_ok());
        assert!(validate_obs_header(&build_obs_header()).is_ok());

        let invalid = [b'X'; RECORD_LEN];
        assert!(validate_member_header(&invalid).is_err());
    }

    #[test]
    fn test_parse_namestr_len() {
        let header = build_member_header(140);
        assert_eq!(parse_namestr_len(&header).unwrap(), 140);

        let header = build_member_header(136);
        assert_eq!(parse_namestr_len(&header).unwrap(), 136);
    }

    #[test]
    fn test_parse_variable_count() {
        let header = build_namestr_header(25);
        assert_eq!(parse_variable_count(&header).unwrap(), 25);
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
