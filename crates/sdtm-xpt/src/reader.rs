//! XPT file reader.
//!
//! Provides functionality to read SAS Transport (XPT) files.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, XptError};
use crate::float::{ibm_to_ieee, is_missing};
use crate::header::{
    LabelSectionType, RECORD_LEN, align_to_record, is_label_header, parse_dataset_label,
    parse_dataset_name, parse_dataset_type, parse_labelv8_data, parse_labelv9_data,
    parse_namestr_len, parse_namestr_records, parse_variable_count, validate_dscrptr_header,
    validate_library_header, validate_member_header, validate_namestr_header, validate_obs_header,
};
use crate::types::{
    MissingValue, NumericValue, XptColumn, XptDataset, XptReaderOptions, XptType, XptValue,
};

/// XPT file reader.
///
/// Reads SAS Transport V5 or V8 format files with auto-detection.
pub struct XptReader<R: Read> {
    reader: BufReader<R>,
    options: XptReaderOptions,
}

impl<R: Read> XptReader<R> {
    /// Create a new XPT reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            options: XptReaderOptions::default(),
        }
    }

    /// Create a new XPT reader with options.
    pub fn with_options(reader: R, options: XptReaderOptions) -> Self {
        Self {
            reader: BufReader::new(reader),
            options,
        }
    }

    /// Read the entire XPT file into memory and parse it.
    ///
    /// # Returns
    /// The first dataset in the file.
    pub fn read_dataset(mut self) -> Result<XptDataset> {
        let data = self.read_all_bytes()?;
        parse_xpt_data(&data, &self.options)
    }

    /// Read all bytes from the reader.
    fn read_all_bytes(&mut self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        self.reader.read_to_end(&mut data)?;
        Ok(data)
    }
}

impl XptReader<File> {
    /// Open an XPT file for reading.
    ///
    /// # Arguments
    /// * `path` - Path to the XPT file
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                XptError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                XptError::Io(e)
            }
        })?;
        Ok(Self::new(file))
    }

    /// Open an XPT file with options.
    pub fn open_with_options(path: &Path, options: XptReaderOptions) -> Result<Self> {
        let file = File::open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                XptError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                XptError::Io(e)
            }
        })?;
        Ok(Self::with_options(file, options))
    }
}

/// Read an XPT file from a path.
///
/// This is a convenience function that opens and reads the file.
///
/// # Arguments
/// * `path` - Path to the XPT file
///
/// # Returns
/// The parsed dataset.
pub fn read_xpt(path: &Path) -> Result<XptDataset> {
    XptReader::open(path)?.read_dataset()
}

/// Read an XPT file with options.
pub fn read_xpt_with_options(path: &Path, options: XptReaderOptions) -> Result<XptDataset> {
    XptReader::open_with_options(path, options)?.read_dataset()
}

/// Parse XPT data from bytes.
fn parse_xpt_data(data: &[u8], options: &XptReaderOptions) -> Result<XptDataset> {
    // Minimum file size check
    if data.len() < RECORD_LEN * 8 {
        return Err(XptError::invalid_format("file too small"));
    }

    // Check record alignment
    if !data.len().is_multiple_of(RECORD_LEN) {
        return Err(XptError::invalid_format(
            "file length is not a multiple of 80",
        ));
    }

    let mut offset = 0usize;

    // Library header - auto-detect version from header prefix
    let library_header = read_record(data, offset)?;
    let version = validate_library_header(library_header)?;
    offset += RECORD_LEN;

    // Skip library real header and modified header
    offset += RECORD_LEN * 2;

    // Member header - validate against detected version
    let member_header = read_record(data, offset)?;
    let _member_version = validate_member_header(member_header)?;
    let namestr_len = parse_namestr_len(member_header)?;
    offset += RECORD_LEN;

    // DSCRPTR header - validate against detected version
    let dscrptr_header = read_record(data, offset)?;
    let _dscrptr_version = validate_dscrptr_header(dscrptr_header)?;
    offset += RECORD_LEN;

    // Member data
    let member_data = read_record(data, offset)?;
    let dataset_name = parse_dataset_name(member_data, version)?;
    offset += RECORD_LEN;

    // Member second
    let member_second = read_record(data, offset)?;
    let dataset_label = parse_dataset_label(member_second);
    let dataset_type = parse_dataset_type(member_second);
    offset += RECORD_LEN;

    // NAMESTR header - validate against detected version
    let namestr_header = read_record(data, offset)?;
    let _namestr_version = validate_namestr_header(namestr_header)?;
    let var_count = parse_variable_count(namestr_header, version)?;
    offset += RECORD_LEN;

    // NAMESTR records
    let namestr_total = var_count
        .checked_mul(namestr_len)
        .ok_or(XptError::ObservationOverflow)?;
    let namestr_data = read_block(data, offset, namestr_total)?;
    offset += namestr_total;
    offset = align_to_record(offset);

    // Parse NAMESTR records into columns (using detected version)
    let mut columns = parse_namestr_records(namestr_data, var_count, namestr_len, version)?;

    // V8: Check for optional LABELV8/V9 section before OBS header
    let next_record = read_record(data, offset)?;
    if let Some(label_type) = is_label_header(next_record) {
        offset += RECORD_LEN;

        // Find the OBS header to determine label section length
        let mut label_end = offset;
        while label_end + RECORD_LEN <= data.len() {
            let check_record = read_record(data, label_end)?;
            if validate_obs_header(check_record).is_ok() {
                break;
            }
            label_end += RECORD_LEN;
        }

        // Parse label data if present
        if label_end > offset {
            let label_data = &data[offset..label_end];
            match label_type {
                LabelSectionType::V8 => {
                    let _ = parse_labelv8_data(label_data, &mut columns);
                }
                LabelSectionType::V9 => {
                    let _ = parse_labelv9_data(label_data, &mut columns);
                }
                LabelSectionType::None => {}
            }
            offset = label_end;
        }
    }

    // OBS header - validate against detected version
    let obs_header = read_record(data, offset)?;
    let _obs_version = validate_obs_header(obs_header)?;
    offset += RECORD_LEN;

    // Calculate observation length
    let obs_len = observation_length(&columns)?;

    // Parse observations
    let rows = parse_observations(data, offset, obs_len, &columns, options)?;

    Ok(XptDataset {
        name: dataset_name,
        label: dataset_label,
        dataset_type,
        columns,
        rows,
    })
}

/// Read a single 80-byte record.
fn read_record(data: &[u8], offset: usize) -> Result<&[u8]> {
    data.get(offset..offset + RECORD_LEN)
        .ok_or(XptError::RecordOutOfBounds { offset })
}

/// Read a block of bytes.
fn read_block(data: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    data.get(offset..offset + len)
        .ok_or(XptError::RecordOutOfBounds { offset })
}

/// Calculate observation length from columns.
fn observation_length(columns: &[XptColumn]) -> Result<usize> {
    let mut total = 0usize;
    for column in columns {
        total = total
            .checked_add(column.length as usize)
            .ok_or(XptError::ObservationOverflow)?;
    }
    Ok(total)
}

/// Parse observation data into rows.
fn parse_observations(
    data: &[u8],
    offset: usize,
    obs_len: usize,
    columns: &[XptColumn],
    options: &XptReaderOptions,
) -> Result<Vec<Vec<XptValue>>> {
    if obs_len == 0 {
        return Ok(Vec::new());
    }

    if offset > data.len() {
        return Err(XptError::RecordOutOfBounds { offset });
    }

    let data_len = data.len().saturating_sub(offset);
    let mut rows_total = data_len / obs_len;
    let remainder = data_len % obs_len;

    // Check for non-space trailing bytes
    if remainder != 0 {
        let start = offset + rows_total * obs_len;
        let rem_bytes = &data[start..offset + data_len];
        if rem_bytes.iter().any(|&b| b != b' ') {
            return Err(XptError::TrailingBytes);
        }
    }

    // Trim trailing all-space rows
    while rows_total > 0 {
        let start = offset + (rows_total - 1) * obs_len;
        let row_bytes = &data[start..start + obs_len];
        if row_bytes.iter().all(|&b| b == b' ') {
            rows_total -= 1;
        } else {
            break;
        }
    }

    // Parse each row
    let mut output = Vec::with_capacity(rows_total);
    for row_idx in 0..rows_total {
        let start = offset + row_idx * obs_len;
        let row_bytes = &data[start..start + obs_len];
        let row = parse_row(row_bytes, columns, options);
        output.push(row);
    }

    Ok(output)
}

/// Parse a single row of observation data.
fn parse_row(row_bytes: &[u8], columns: &[XptColumn], options: &XptReaderOptions) -> Vec<XptValue> {
    let mut values = Vec::with_capacity(columns.len());
    let mut pos = 0usize;

    for column in columns {
        let len = column.length as usize;
        let slice = &row_bytes[pos..pos + len];

        let value = match column.data_type {
            XptType::Char => {
                let s = decode_char(slice, options.trim_strings);
                XptValue::Char(s)
            }
            XptType::Num => {
                let num = decode_numeric(slice);
                XptValue::Num(num)
            }
        };

        values.push(value);
        pos += len;
    }

    values
}

/// Decode a character value.
fn decode_char(bytes: &[u8], trim: bool) -> String {
    let text = String::from_utf8_lossy(bytes);
    if trim {
        text.trim_end().to_string()
    } else {
        text.to_string()
    }
}

/// Decode a numeric value.
fn decode_numeric(bytes: &[u8]) -> NumericValue {
    if bytes.is_empty() {
        return NumericValue::Missing(MissingValue::Standard);
    }

    // Check for missing value
    if let Some(missing) = is_missing(bytes) {
        return NumericValue::Missing(missing);
    }

    // Expand to 8 bytes if needed
    let mut buf = [0u8; 8];
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);

    // Convert IBM to IEEE
    let value = ibm_to_ieee(buf);
    NumericValue::Value(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_char() {
        assert_eq!(decode_char(b"hello   ", true), "hello");
        assert_eq!(decode_char(b"hello   ", false), "hello   ");
        assert_eq!(decode_char(b"", true), "");
    }

    #[test]
    fn test_decode_numeric_missing() {
        let missing_standard = [0x2e, 0, 0, 0, 0, 0, 0, 0];
        let result = decode_numeric(&missing_standard);
        assert!(result.is_missing());
        assert_eq!(result.missing_type(), Some(MissingValue::Standard));

        let missing_a = [0x41, 0, 0, 0, 0, 0, 0, 0];
        let result = decode_numeric(&missing_a);
        assert!(result.is_missing());
        assert_eq!(result.missing_type(), Some(MissingValue::Special('A')));
    }

    #[test]
    fn test_decode_numeric_value() {
        // IBM representation of 1.0
        let one = [0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = decode_numeric(&one);
        assert!(result.is_present());
        let value = result.value().unwrap();
        assert!((value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_observation_length() {
        let columns = vec![
            XptColumn::numeric("A"),       // 8 bytes
            XptColumn::character("B", 20), // 20 bytes
        ];
        assert_eq!(observation_length(&columns).unwrap(), 28);
    }
}
