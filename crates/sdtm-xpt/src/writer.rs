//! XPT file writer.
//!
//! Provides functionality to write SAS Transport (XPT) files.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::error::{Result, XptError};
use crate::float::{encode_missing, ieee_to_ibm, truncate_ibm};
use crate::header::{
    LibraryInfo, RECORD_LEN, build_dscrptr_header, build_library_header, build_member_data,
    build_member_header, build_member_second, build_namestr, build_namestr_header,
    build_obs_header, build_real_header, build_second_header,
};
use crate::types::{NumericValue, XptColumn, XptDataset, XptType, XptValue, XptWriterOptions};

/// XPT file writer.
///
/// Writes SAS Transport V5 format files.
pub struct XptWriter<W: Write> {
    writer: BufWriter<W>,
    options: XptWriterOptions,
}

impl<W: Write> XptWriter<W> {
    /// Create a new XPT writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            options: XptWriterOptions::default(),
        }
    }

    /// Create a new XPT writer with options.
    pub fn with_options(writer: W, options: XptWriterOptions) -> Self {
        Self {
            writer: BufWriter::new(writer),
            options,
        }
    }

    /// Write a dataset to the XPT file.
    pub fn write_dataset(mut self, dataset: &XptDataset) -> Result<()> {
        validate_dataset(dataset)?;

        let info: LibraryInfo = (&self.options).into();

        // Library headers
        self.writer.write_all(&build_library_header())?;
        self.writer.write_all(&build_real_header(&info))?;
        self.writer
            .write_all(&build_second_header(&info.modified))?;

        // Member headers
        self.writer
            .write_all(&build_member_header(self.options.namestr_length))?;
        self.writer.write_all(&build_dscrptr_header())?;
        self.writer
            .write_all(&build_member_data(dataset, &self.options))?;
        self.writer
            .write_all(&build_member_second(dataset, &self.options))?;

        // NAMESTR header and records
        self.writer
            .write_all(&build_namestr_header(dataset.columns.len()))?;
        self.write_namestr_records(&dataset.columns)?;

        // OBS header and data
        self.writer.write_all(&build_obs_header())?;
        self.write_observations(dataset)?;

        self.writer.flush()?;
        Ok(())
    }

    /// Write NAMESTR records for all columns.
    fn write_namestr_records(&mut self, columns: &[XptColumn]) -> Result<()> {
        let mut record_writer = RecordWriter::new(&mut self.writer);
        let mut position = 0u32;

        for (idx, column) in columns.iter().enumerate() {
            let namestr = build_namestr(column, (idx + 1) as u16, position);
            record_writer.write_bytes(&namestr)?;
            position = position.saturating_add(column.length as u32);
        }

        record_writer.finish()?;
        Ok(())
    }

    /// Write observation data.
    fn write_observations(&mut self, dataset: &XptDataset) -> Result<()> {
        let obs_len = dataset.observation_length();
        let mut record_writer = RecordWriter::new(&mut self.writer);

        for row in dataset.rows.iter() {
            if row.len() != dataset.columns.len() {
                return Err(XptError::RowLengthMismatch {
                    expected: dataset.columns.len(),
                    actual: row.len(),
                });
            }

            let mut obs = vec![b' '; obs_len];
            let mut pos = 0usize;

            for (value, column) in row.iter().zip(dataset.columns.iter()) {
                let bytes = encode_value(value, column, &self.options);
                let end = pos + bytes.len();
                obs[pos..end].copy_from_slice(&bytes);
                pos += column.length as usize;
            }

            record_writer.write_bytes(&obs)?;
        }

        record_writer.finish()?;
        Ok(())
    }
}

impl XptWriter<File> {
    /// Create an XPT file for writing.
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self::new(file))
    }

    /// Create an XPT file with options.
    pub fn create_with_options(path: &Path, options: XptWriterOptions) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self::with_options(file, options))
    }
}

/// Write a dataset to an XPT file.
///
/// This is a convenience function that creates the file and writes the dataset.
///
/// # Arguments
/// * `path` - Path to the output XPT file
/// * `dataset` - The dataset to write
///
/// # Returns
/// Ok(()) on success.
pub fn write_xpt(path: &Path, dataset: &XptDataset) -> Result<()> {
    XptWriter::create(path)?.write_dataset(dataset)
}

/// Write a dataset to an XPT file with options.
pub fn write_xpt_with_options(
    path: &Path,
    dataset: &XptDataset,
    options: &XptWriterOptions,
) -> Result<()> {
    XptWriter::create_with_options(path, options.clone())?.write_dataset(dataset)
}

/// Validate a dataset before writing.
fn validate_dataset(dataset: &XptDataset) -> Result<()> {
    // Validate dataset name
    let name = normalize_name(&dataset.name);
    if name.is_empty() || name.len() > 8 {
        return Err(XptError::invalid_dataset_name(&dataset.name));
    }

    // Check for duplicate column names
    let mut seen = BTreeSet::new();
    for column in &dataset.columns {
        let col_name = normalize_name(&column.name);

        if col_name.is_empty() || col_name.len() > 8 {
            return Err(XptError::invalid_variable_name(&column.name));
        }

        if !seen.insert(col_name.clone()) {
            return Err(XptError::duplicate_variable(&column.name));
        }

        if column.length == 0 {
            return Err(XptError::zero_length(&column.name));
        }
    }

    // Validate row lengths
    for row in dataset.rows.iter() {
        if row.len() != dataset.columns.len() {
            return Err(XptError::RowLengthMismatch {
                expected: dataset.columns.len(),
                actual: row.len(),
            });
        }
    }

    Ok(())
}

/// Normalize a name (trim, uppercase).
fn normalize_name(name: &str) -> String {
    name.trim().to_uppercase()
}

/// Encode a value for writing.
fn encode_value(value: &XptValue, column: &XptColumn, options: &XptWriterOptions) -> Vec<u8> {
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

/// Encode a character value.
fn encode_char(value: &str, length: u16) -> Vec<u8> {
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

/// Encode a numeric value.
fn encode_numeric(value: &NumericValue, length: u16, options: &XptWriterOptions) -> Vec<u8> {
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

/// Helper for writing 80-byte records with overflow handling.
struct RecordWriter<'a, W: Write> {
    writer: &'a mut W,
    record: [u8; RECORD_LEN],
    pos: usize,
}

impl<'a, W: Write> RecordWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            record: [b' '; RECORD_LEN],
            pos: 0,
        }
    }

    fn write_bytes(&mut self, mut bytes: &[u8]) -> Result<()> {
        while !bytes.is_empty() {
            let remaining = RECORD_LEN - self.pos;
            let take = remaining.min(bytes.len());

            self.record[self.pos..self.pos + take].copy_from_slice(&bytes[..take]);
            self.pos += take;
            bytes = &bytes[take..];

            if self.pos == RECORD_LEN {
                self.writer.write_all(&self.record)?;
                self.record = [b' '; RECORD_LEN];
                self.pos = 0;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.pos > 0 {
            // Pad remaining bytes with spaces
            for idx in self.pos..RECORD_LEN {
                self.record[idx] = b' ';
            }
            self.writer.write_all(&self.record)?;
            self.pos = 0;
        }
        Ok(())
    }
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
    fn test_validate_dataset_valid() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 20)],
        );
        assert!(validate_dataset(&dataset).is_ok());
    }

    #[test]
    fn test_validate_dataset_empty_name() {
        let dataset = XptDataset::new("");
        assert!(validate_dataset(&dataset).is_err());
    }

    #[test]
    fn test_validate_dataset_long_name() {
        let dataset = XptDataset::new("VERYLONGNAME");
        assert!(validate_dataset(&dataset).is_err());
    }

    #[test]
    fn test_validate_dataset_duplicate_columns() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::numeric("AGE")],
        );
        assert!(validate_dataset(&dataset).is_err());
    }

    #[test]
    fn test_validate_dataset_zero_length() {
        let mut col = XptColumn::numeric("X");
        col.length = 0;
        let dataset = XptDataset::with_columns("TEST", vec![col]);
        assert!(validate_dataset(&dataset).is_err());
    }

    #[test]
    fn test_record_writer() {
        let mut output = Vec::new();
        {
            let mut writer = RecordWriter::new(&mut output);
            writer.write_bytes(&[b'A'; 50]).unwrap();
            writer.write_bytes(&[b'B'; 50]).unwrap();
            writer.finish().unwrap();
        }

        // Should have 2 records (100 bytes of data, 80 bytes per record)
        assert_eq!(output.len(), 160);
        assert_eq!(&output[0..50], &[b'A'; 50]);
        assert_eq!(&output[50..80], &[b'B'; 30]);
        assert_eq!(&output[80..100], &[b'B'; 20]);
    }
}
