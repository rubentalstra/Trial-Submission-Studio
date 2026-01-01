//! Observation decoding utilities.
//!
//! This module provides functions for decoding observation (row) data
//! from XPT binary format into typed values.

use crate::float::{ibm_to_ieee, is_missing};
use crate::types::{MissingValue, NumericValue, Observation, XptColumn, XptType, XptValue};

/// Decode a character value from bytes.
///
/// # Arguments
/// * `bytes` - Raw bytes to decode
/// * `trim` - Whether to trim trailing whitespace
///
/// # Returns
/// Decoded string value.
#[must_use]
pub fn decode_char(bytes: &[u8], trim: bool) -> String {
    let text = String::from_utf8_lossy(bytes);
    if trim {
        text.trim_end().to_string()
    } else {
        text.to_string()
    }
}

/// Decode a numeric value from IBM float bytes.
///
/// Handles all 28 SAS missing value codes (., ._, .A-.Z).
///
/// # Arguments
/// * `bytes` - Raw bytes (up to 8, IBM float format)
///
/// # Returns
/// Decoded numeric value or missing indicator.
#[must_use]
pub fn decode_numeric(bytes: &[u8]) -> NumericValue {
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

/// Parse a single row of observation data into an Observation.
///
/// # Arguments
/// * `row_bytes` - Raw bytes for one observation
/// * `columns` - Column definitions
/// * `trim_strings` - Whether to trim trailing spaces from character values
///
/// # Returns
/// Parsed observation with typed values.
#[must_use]
pub fn parse_observation(row_bytes: &[u8], columns: &[XptColumn], trim_strings: bool) -> Observation {
    let mut values = Vec::with_capacity(columns.len());
    let mut pos = 0usize;

    for column in columns {
        let len = column.length as usize;
        let slice = &row_bytes[pos..pos + len];

        let value = match column.data_type {
            XptType::Char => {
                let s = decode_char(slice, trim_strings);
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

    Observation::from_vec(values)
}

/// Calculate the observation length (bytes per row) from columns.
///
/// # Arguments
/// * `columns` - Column definitions
///
/// # Returns
/// Total bytes per observation, or None if overflow.
#[must_use]
pub fn observation_length(columns: &[XptColumn]) -> Option<usize> {
    let mut total = 0usize;
    for column in columns {
        total = total.checked_add(column.length as usize)?;
    }
    Some(total)
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
        assert_eq!(observation_length(&columns), Some(28));
    }

    #[test]
    fn test_parse_observation() {
        let columns = vec![
            XptColumn::character("NAME", 8),
            XptColumn::numeric("AGE"),
        ];

        // "JOHN    " + IBM 1.0
        let mut row_bytes = Vec::new();
        row_bytes.extend_from_slice(b"JOHN    ");
        row_bytes.extend_from_slice(&[0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let obs = parse_observation(&row_bytes, &columns, true);
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].as_str(), Some("JOHN"));
        assert!((obs[1].as_f64().unwrap() - 1.0).abs() < 1e-10);
    }
}
