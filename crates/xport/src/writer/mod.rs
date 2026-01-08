//! XPT file writer.
//!
//! This module provides functionality to write SAS Transport (XPT) files
//! with both buffered and streaming approaches.
//!
//! # Usage
//!
//! ## Simple Writing
//!
//! ```no_run
//! use std::path::Path;
//! use xport::{XptDataset, XptColumn, XptValue, write_xpt};
//!
//! let mut dataset = XptDataset::with_columns("DM", vec![
//!     XptColumn::character("USUBJID", 20),
//!     XptColumn::numeric("AGE"),
//! ]);
//! dataset.add_row(vec![
//!     XptValue::character("STUDY-001"),
//!     XptValue::numeric(35.0),
//! ]);
//!
//! write_xpt(Path::new("dm.xpt"), &dataset).unwrap();
//! ```
//!
//! ## With Validation
//!
//! ```no_run
//! use std::path::Path;
//! use xport::{XptDataset, XptColumn, XptWriterBuilder};
//!
//! let dataset = XptDataset::with_columns("DM", vec![
//!     XptColumn::character("USUBJID", 20),
//! ]);
//!
//! let result = XptWriterBuilder::new()
//!     .fda_compliant()
//!     .validate(&dataset);
//!
//! if result.is_valid() {
//!     result.write_to_file(Path::new("dm.xpt"), &dataset).unwrap();
//! }
//! ```
//!
//! ## Streaming Writing (constant memory)
//!
//! ```no_run
//! use std::path::Path;
//! use xport::{XptColumn, Observation, XptValue};
//! use xport::writer::{DatasetInfo, StreamingWriter};
//!
//! let info = DatasetInfo::new("DM", vec![
//!     XptColumn::character("USUBJID", 20),
//!     XptColumn::numeric("AGE"),
//! ]);
//!
//! let mut writer = StreamingWriter::create(Path::new("dm.xpt"), info).unwrap();
//!
//! for i in 0..1000 {
//!     let obs = Observation::new(vec![
//!         XptValue::character(format!("SUBJ-{:03}", i)),
//!         XptValue::numeric(30.0 + (i % 50) as f64),
//!     ]);
//!     writer.write_observation(&obs).unwrap();
//! }
//!
//! writer.finish().unwrap();
//! ```

pub mod builder;
pub mod observation;
pub mod streaming;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::XptVersion;
use crate::error::{Result, XptError};
use crate::header::{
    LabelSectionType, LibraryInfo, RECORD_LEN, build_dscrptr_header, build_labelv8_data,
    build_labelv8_header, build_labelv9_data, build_labelv9_header, build_library_header,
    build_member_data, build_member_header, build_member_second, build_namestr,
    build_namestr_header, build_obs_header, build_real_header, build_second_header,
    determine_label_section,
};
use crate::types::{XptColumn, XptDataset, XptWriterOptions};

pub use builder::{ValidatedWriter, XptWriterBuilder, validate_dataset};
pub use observation::{encode_char, encode_numeric, encode_observation, encode_value};
pub use streaming::{DatasetInfo, StreamingWriter};

/// XPT file writer.
///
/// Writes SAS Transport V5 or V8 format files.
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
        let version = self.options.version;
        builder::validate_dataset_quick(dataset, version)?;

        let info: LibraryInfo = (&self.options).into();

        // Library headers
        self.writer.write_all(&build_library_header(version))?;
        self.writer.write_all(&build_real_header(&info))?;
        self.writer
            .write_all(&build_second_header(&info.modified))?;

        // Member headers
        self.writer
            .write_all(&build_member_header(version, self.options.namestr_length))?;
        self.writer.write_all(&build_dscrptr_header(version))?;
        self.writer
            .write_all(&build_member_data(dataset, &self.options))?;
        self.writer
            .write_all(&build_member_second(dataset, &self.options))?;

        // NAMESTR header and records
        self.writer
            .write_all(&build_namestr_header(version, dataset.columns.len()))?;
        self.write_namestr_records(&dataset.columns, version)?;

        // V8: Write LABELV8/V9 section if needed
        if version.supports_label_section() {
            self.write_label_section(&dataset.columns)?;
        }

        // OBS header and data
        self.writer.write_all(&build_obs_header(version))?;
        self.write_observations(dataset)?;

        self.writer.flush()?;
        Ok(())
    }

    /// Write NAMESTR records for all columns.
    fn write_namestr_records(&mut self, columns: &[XptColumn], version: XptVersion) -> Result<()> {
        let mut record_writer = RecordWriter::new(&mut self.writer);
        let mut position = 0u32;

        for (idx, column) in columns.iter().enumerate() {
            let namestr = build_namestr(column, (idx + 1) as u16, position, version);
            record_writer.write_bytes(&namestr)?;
            position = position.saturating_add(column.length as u32);
        }

        record_writer.finish()?;
        Ok(())
    }

    /// Write LABELV8 or LABELV9 section if needed (V8 format only).
    fn write_label_section(&mut self, columns: &[XptColumn]) -> Result<()> {
        match determine_label_section(columns) {
            LabelSectionType::None => Ok(()),
            LabelSectionType::V8 => {
                self.writer.write_all(&build_labelv8_header())?;
                let data = build_labelv8_data(columns);
                if !data.is_empty() {
                    let mut record_writer = RecordWriter::new(&mut self.writer);
                    record_writer.write_bytes(&data)?;
                    record_writer.finish()?;
                }
                Ok(())
            }
            LabelSectionType::V9 => {
                self.writer.write_all(&build_labelv9_header())?;
                let data = build_labelv9_data(columns);
                if !data.is_empty() {
                    let mut record_writer = RecordWriter::new(&mut self.writer);
                    record_writer.write_bytes(&data)?;
                    record_writer.finish()?;
                }
                Ok(())
            }
        }
    }

    /// Write observation data.
    fn write_observations(&mut self, dataset: &XptDataset) -> Result<()> {
        let obs_len = dataset.observation_length();
        let mut record_writer = RecordWriter::new(&mut self.writer);

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            if row.len() != dataset.columns.len() {
                return Err(XptError::row_length_mismatch(
                    row_idx,
                    dataset.columns.len(),
                    row.len(),
                ));
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

/// Helper for writing 80-byte records with overflow handling.
pub(crate) struct RecordWriter<'a, W: Write> {
    writer: &'a mut W,
    record: [u8; RECORD_LEN],
    pos: usize,
}

impl<'a, W: Write> RecordWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            record: [b' '; RECORD_LEN],
            pos: 0,
        }
    }

    pub fn write_bytes(&mut self, mut bytes: &[u8]) -> Result<()> {
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

    pub fn finish(&mut self) -> Result<()> {
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
    use crate::types::{MissingValue, NumericValue};

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
    fn test_validate_dataset_valid_v5() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 20)],
        );
        assert!(validate_dataset(&dataset, XptVersion::V5).is_ok());
    }

    #[test]
    fn test_validate_dataset_valid_v8() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 20)],
        );
        assert!(validate_dataset(&dataset, XptVersion::V8).is_ok());
    }

    #[test]
    fn test_validate_dataset_empty_name() {
        let dataset = XptDataset::new("");
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
        assert!(validate_dataset(&dataset, XptVersion::V8).is_err());
    }

    #[test]
    fn test_validate_dataset_long_name_v5() {
        // V5 limit is 8 characters
        let dataset = XptDataset::new("VERYLONGNAME");
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
    }

    #[test]
    fn test_validate_dataset_long_name_v8() {
        // V8 limit is 32 characters
        let dataset = XptDataset::new("VERYLONGNAME"); // 12 chars, OK for V8
        assert!(validate_dataset(&dataset, XptVersion::V8).is_ok());

        // Create a name that exceeds V8 limit (33 chars)
        let dataset = XptDataset::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567");
        assert!(validate_dataset(&dataset, XptVersion::V8).is_err());
    }

    #[test]
    fn test_validate_dataset_duplicate_columns() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::numeric("AGE")],
        );
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
    }

    #[test]
    fn test_validate_dataset_zero_length() {
        let mut col = XptColumn::numeric("X");
        col.length = 0;
        let dataset = XptDataset::with_columns("TEST", vec![col]);
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
    }

    #[test]
    fn test_validate_dataset_long_variable_name() {
        // V5 limit is 8 characters
        let dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("VERYLONGVARNAME")]);
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
        assert!(validate_dataset(&dataset, XptVersion::V8).is_ok()); // V8 allows up to 32
    }

    #[test]
    fn test_validate_dataset_long_label() {
        // V5 limit is 40 characters, V8 is 256
        let long_label = "A".repeat(50);
        let mut col = XptColumn::numeric("AGE");
        col.label = Some(long_label);
        let dataset = XptDataset::with_columns("TEST", vec![col]);
        assert!(validate_dataset(&dataset, XptVersion::V5).is_err());
        assert!(validate_dataset(&dataset, XptVersion::V8).is_ok());
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
