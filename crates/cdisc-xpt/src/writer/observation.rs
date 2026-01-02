//! Observation encoding utilities.
//!
//! This module provides functions for encoding observation (row) data
//! into XPT binary format.

use crate::float::{encode_missing, ieee_to_ibm, truncate_ibm};
use crate::types::{NumericValue, Observation, XptColumn, XptType, XptValue, XptWriterOptions};

/// Encode a value for writing to XPT format.
///
/// Handles type coercion if the value type doesn't match the column type.
#[must_use]
pub fn encode_value(value: &XptValue, column: &XptColumn, options: &XptWriterOptions) -> Vec<u8> {
    match (value, column.data_type) {
        (XptValue::Char(s), XptType::Char) => encode_char(s, column.length),
        (XptValue::Num(n), XptType::Num) => encode_numeric(n, column.length, options),
        (XptValue::Char(s), XptType::Num) => {
            // Try to parse string as number
            let num = s.trim().parse::<f64>().ok().map(NumericValue::Value);
            let num = num.unwrap_or(NumericValue::Missing(options.default_missing));
            encode_numeric(&num, column.length, options)
        }
        (XptValue::Num(n), XptType::Char) => {
            // Convert number to string
            let s = n.to_string();
            encode_char(&s, column.length)
        }
    }
}

/// Encode a character value to XPT format.
///
/// The value is truncated if too long, or space-padded if too short.
/// Non-ASCII characters are replaced with '?'.
#[must_use]
pub fn encode_char(value: &str, length: u16) -> Vec<u8> {
    let len = length as usize;
    let mut out = Vec::with_capacity(len);

    for ch in value.chars().take(len) {
        if ch.is_ascii() {
            out.push(ch as u8);
        } else {
            out.push(b'?');
        }
    }

    // Pad with spaces
    while out.len() < len {
        out.push(b' ');
    }

    out
}

/// Encode a numeric value to IBM float format.
///
/// Non-finite values (NaN, infinity) are converted to missing.
#[must_use]
pub fn encode_numeric(value: &NumericValue, length: u16, options: &XptWriterOptions) -> Vec<u8> {
    let bytes = match value {
        NumericValue::Missing(m) => encode_missing(*m),
        NumericValue::Value(v) => {
            if !v.is_finite() {
                // Non-finite values become missing
                encode_missing(options.default_missing)
            } else {
                ieee_to_ibm(*v)
            }
        }
    };

    truncate_ibm(bytes, length as usize)
}

/// Encode an observation (row) to bytes.
///
/// # Arguments
/// * `observation` - The observation to encode
/// * `columns` - Column definitions
/// * `options` - Writer options
///
/// # Returns
/// Encoded bytes for the observation row.
#[must_use]
pub fn encode_observation(
    observation: &Observation,
    columns: &[XptColumn],
    options: &XptWriterOptions,
) -> Vec<u8> {
    let obs_len: usize = columns.iter().map(|c| c.length as usize).sum();
    let mut obs = vec![b' '; obs_len];
    let mut pos = 0usize;

    for (value, column) in observation.iter().zip(columns.iter()) {
        let bytes = encode_value(value, column, options);
        let end = pos + bytes.len();
        obs[pos..end].copy_from_slice(&bytes);
        pos += column.length as usize;
    }

    obs
}

/// Calculate the observation length (bytes per row) from columns.
#[must_use]
pub fn observation_length(columns: &[XptColumn]) -> usize {
    columns.iter().map(|c| c.length as usize).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MissingValue;

    #[test]
    fn test_encode_char() {
        let encoded = encode_char("hello", 10);
        assert_eq!(encoded, b"hello     ");
        assert_eq!(encoded.len(), 10);

        let encoded = encode_char("verylongstring", 5);
        assert_eq!(encoded, b"veryl");
        assert_eq!(encoded.len(), 5);
    }

    #[test]
    fn test_encode_numeric_value() {
        let options = XptWriterOptions::default();
        let num = NumericValue::Value(1.0);
        let encoded = encode_numeric(&num, 8, &options);
        assert_eq!(encoded.len(), 8);
        assert_eq!(encoded[0], 0x41); // IBM 1.0 starts with 0x41
    }

    #[test]
    fn test_encode_numeric_missing() {
        let options = XptWriterOptions::default();
        let num = NumericValue::Missing(MissingValue::Standard);
        let encoded = encode_numeric(&num, 8, &options);
        assert_eq!(encoded[0], 0x2e);
        assert!(encoded[1..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_encode_observation() {
        let columns = vec![XptColumn::character("NAME", 8), XptColumn::numeric("AGE")];
        let obs = Observation::new(vec![XptValue::character("JOHN"), XptValue::numeric(30.0)]);
        let options = XptWriterOptions::default();

        let encoded = encode_observation(&obs, &columns, &options);
        assert_eq!(encoded.len(), 16); // 8 + 8
        assert_eq!(&encoded[0..8], b"JOHN    ");
    }

    #[test]
    fn test_observation_length() {
        let columns = vec![
            XptColumn::numeric("A"),       // 8 bytes
            XptColumn::character("B", 20), // 20 bytes
        ];
        assert_eq!(observation_length(&columns), 28);
    }
}
