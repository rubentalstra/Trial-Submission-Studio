//! Streaming XPT writer for large datasets.
//!
//! This module provides a streaming writer that can write observations
//! incrementally without loading all data into memory.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::XptVersion;
use crate::error::Result;
use crate::header::{
    LabelSectionType, LibraryInfo, RECORD_LEN, build_dscrptr_header, build_labelv8_data,
    build_labelv8_header, build_labelv9_data, build_labelv9_header, build_library_header,
    build_member_header, build_namestr, build_namestr_header, build_obs_header, build_real_header,
    build_second_header, determine_label_section,
};
use crate::types::{Observation, XptColumn, XptWriterOptions};

use super::observation::{encode_observation, observation_length};

/// Metadata for a dataset being written.
///
/// This contains all the information needed to write headers.
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    /// Dataset name.
    pub name: String,
    /// Optional dataset label.
    pub label: Option<String>,
    /// Optional dataset type.
    pub dataset_type: Option<String>,
    /// Column definitions.
    pub columns: Vec<XptColumn>,
}

impl DatasetInfo {
    /// Create new dataset info.
    pub fn new(name: impl Into<String>, columns: Vec<XptColumn>) -> Self {
        Self {
            name: name.into(),
            label: None,
            dataset_type: None,
            columns,
        }
    }

    /// Set the dataset label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the dataset type.
    #[must_use]
    pub fn with_type(mut self, dataset_type: impl Into<String>) -> Self {
        self.dataset_type = Some(dataset_type.into());
        self
    }
}

/// Streaming XPT writer.
///
/// Writes observations incrementally to avoid loading all data into memory.
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// use tss_xpt::{XptColumn, Observation, XptValue};
/// use tss_xpt::writer::{DatasetInfo, StreamingWriter};
///
/// let file = File::create("large.xpt").unwrap();
/// let info = DatasetInfo::new("DM", vec![
///     XptColumn::character("USUBJID", 20),
///     XptColumn::numeric("AGE"),
/// ]);
///
/// let mut writer = StreamingWriter::new(file, info).unwrap();
///
/// // Write observations one at a time
/// for i in 0..1000000 {
///     let obs = Observation::new(vec![
///         XptValue::character(format!("SUBJ-{:06}", i)),
///         XptValue::numeric(30.0 + (i % 50) as f64),
///     ]);
///     writer.write_observation(&obs).unwrap();
/// }
///
/// writer.finish().unwrap();
/// ```
pub struct StreamingWriter<W: Write> {
    writer: BufWriter<W>,
    options: XptWriterOptions,
    columns: Vec<XptColumn>,
    obs_length: usize,
    record_writer: Option<RecordWriterState>,
}

/// Internal state for the record writer between calls.
struct RecordWriterState {
    record: [u8; RECORD_LEN],
    pos: usize,
}

impl<W: Write> StreamingWriter<W> {
    /// Create a new streaming writer.
    ///
    /// Writes all headers immediately, then observations can be added.
    pub fn new(writer: W, info: DatasetInfo) -> Result<Self> {
        Self::with_options(writer, info, XptWriterOptions::default())
    }

    /// Create a streaming writer with options.
    pub fn with_options(writer: W, info: DatasetInfo, options: XptWriterOptions) -> Result<Self> {
        let mut writer = BufWriter::new(writer);
        let version = options.version;

        let lib_info: LibraryInfo = (&options).into();

        // Library headers
        writer.write_all(&build_library_header(version))?;
        writer.write_all(&build_real_header(&lib_info))?;
        writer.write_all(&build_second_header(&lib_info.modified))?;

        // Member headers
        writer.write_all(&build_member_header(version, options.namestr_length))?;
        writer.write_all(&build_dscrptr_header(version))?;

        // Build member data and second records
        let member_data = build_member_data_from_info(&info, &options);
        let member_second = build_member_second_from_info(&info, &options);
        writer.write_all(&member_data)?;
        writer.write_all(&member_second)?;

        // NAMESTR header and records
        writer.write_all(&build_namestr_header(version, info.columns.len()))?;
        write_namestr_records(&mut writer, &info.columns, version)?;

        // V8: Write LABELV8/V9 section if needed
        if version.supports_label_section() {
            write_label_section(&mut writer, &info.columns)?;
        }

        // OBS header
        writer.write_all(&build_obs_header(version))?;

        let obs_length = observation_length(&info.columns);

        Ok(Self {
            writer,
            options,
            columns: info.columns,
            obs_length,
            record_writer: Some(RecordWriterState {
                record: [b' '; RECORD_LEN],
                pos: 0,
            }),
        })
    }

    /// Write a single observation.
    pub fn write_observation(&mut self, observation: &Observation) -> Result<()> {
        let bytes = encode_observation(observation, &self.columns, &self.options);
        self.write_bytes(&bytes)
    }

    /// Write raw observation bytes.
    fn write_bytes(&mut self, mut bytes: &[u8]) -> Result<()> {
        let state = self
            .record_writer
            .as_mut()
            .expect("Writer already finished");

        while !bytes.is_empty() {
            let remaining = RECORD_LEN - state.pos;
            let take = remaining.min(bytes.len());

            state.record[state.pos..state.pos + take].copy_from_slice(&bytes[..take]);
            state.pos += take;
            bytes = &bytes[take..];

            if state.pos == RECORD_LEN {
                self.writer.write_all(&state.record)?;
                state.record = [b' '; RECORD_LEN];
                state.pos = 0;
            }
        }
        Ok(())
    }

    /// Finish writing and flush the output.
    ///
    /// This must be called after all observations have been written.
    pub fn finish(mut self) -> Result<()> {
        if let Some(state) = self.record_writer.take() {
            if state.pos > 0 {
                // Pad remaining bytes with spaces
                let mut record = state.record;
                for idx in state.pos..RECORD_LEN {
                    record[idx] = b' ';
                }
                self.writer.write_all(&record)?;
            }
        }
        self.writer.flush()?;
        Ok(())
    }

    /// Get the observation length in bytes.
    #[must_use]
    pub fn observation_length(&self) -> usize {
        self.obs_length
    }

    /// Get the columns.
    #[must_use]
    pub fn columns(&self) -> &[XptColumn] {
        &self.columns
    }
}

impl StreamingWriter<File> {
    /// Create a streaming writer to a file.
    pub fn create(path: &Path, info: DatasetInfo) -> Result<Self> {
        let file = File::create(path)?;
        Self::new(file, info)
    }

    /// Create a streaming writer to a file with options.
    pub fn create_with_options(
        path: &Path,
        info: DatasetInfo,
        options: XptWriterOptions,
    ) -> Result<Self> {
        let file = File::create(path)?;
        Self::with_options(file, info, options)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

/// Write NAMESTR records for all columns.
fn write_namestr_records<W: Write>(
    writer: &mut W,
    columns: &[XptColumn],
    version: XptVersion,
) -> Result<()> {
    let mut record = [b' '; RECORD_LEN];
    let mut pos = 0usize;
    let mut position = 0u32;

    for (idx, column) in columns.iter().enumerate() {
        let namestr = build_namestr(column, (idx + 1) as u16, position, version);
        position = position.saturating_add(column.length as u32);

        let mut bytes = namestr.as_slice();
        while !bytes.is_empty() {
            let remaining = RECORD_LEN - pos;
            let take = remaining.min(bytes.len());

            record[pos..pos + take].copy_from_slice(&bytes[..take]);
            pos += take;
            bytes = &bytes[take..];

            if pos == RECORD_LEN {
                writer.write_all(&record)?;
                record = [b' '; RECORD_LEN];
                pos = 0;
            }
        }
    }

    // Flush remaining
    if pos > 0 {
        for idx in pos..RECORD_LEN {
            record[idx] = b' ';
        }
        writer.write_all(&record)?;
    }

    Ok(())
}

/// Write LABELV8/V9 section if needed.
fn write_label_section<W: Write>(writer: &mut W, columns: &[XptColumn]) -> Result<()> {
    match determine_label_section(columns) {
        LabelSectionType::None => Ok(()),
        LabelSectionType::V8 => {
            writer.write_all(&build_labelv8_header())?;
            let data = build_labelv8_data(columns);
            if !data.is_empty() {
                write_with_padding(writer, &data)?;
            }
            Ok(())
        }
        LabelSectionType::V9 => {
            writer.write_all(&build_labelv9_header())?;
            let data = build_labelv9_data(columns);
            if !data.is_empty() {
                write_with_padding(writer, &data)?;
            }
            Ok(())
        }
    }
}

/// Write data with 80-byte record padding.
fn write_with_padding<W: Write>(writer: &mut W, data: &[u8]) -> Result<()> {
    let mut record = [b' '; RECORD_LEN];
    let mut pos = 0usize;
    let mut bytes = data;

    while !bytes.is_empty() {
        let remaining = RECORD_LEN - pos;
        let take = remaining.min(bytes.len());

        record[pos..pos + take].copy_from_slice(&bytes[..take]);
        pos += take;
        bytes = &bytes[take..];

        if pos == RECORD_LEN {
            writer.write_all(&record)?;
            record = [b' '; RECORD_LEN];
            pos = 0;
        }
    }

    if pos > 0 {
        for idx in pos..RECORD_LEN {
            record[idx] = b' ';
        }
        writer.write_all(&record)?;
    }

    Ok(())
}

/// Build member data record from DatasetInfo.
fn build_member_data_from_info(info: &DatasetInfo, options: &XptWriterOptions) -> [u8; RECORD_LEN] {
    use crate::header::{format_xpt_datetime, write_string};

    let mut record = [b' '; RECORD_LEN];
    let version = options.version;

    // Write dataset name
    let name_len = version.dataset_name_limit();
    write_string(&mut record, 8, &info.name.to_uppercase(), name_len);

    // Dataset type at position 8+name_len
    let type_offset = 8 + name_len;
    let dtype = info.dataset_type.as_deref().unwrap_or("DATA");
    write_string(&mut record, type_offset, dtype, 8);

    // Created timestamp
    let created_offset = type_offset + 8;
    let created_str = format_xpt_datetime(options.get_created());
    write_string(&mut record, created_offset, &created_str, 16);

    // Modified timestamp
    let modified_offset = created_offset + 16;
    let modified_str = format_xpt_datetime(options.get_modified());
    write_string(&mut record, modified_offset, &modified_str, 16);

    record
}

/// Build member second record from DatasetInfo.
fn build_member_second_from_info(
    info: &DatasetInfo,
    _options: &XptWriterOptions,
) -> [u8; RECORD_LEN] {
    use crate::header::write_string;

    let mut record = [b' '; RECORD_LEN];

    // Dataset label (max 40 chars)
    if let Some(label) = &info.label {
        write_string(&mut record, 0, label, 40);
    }

    // Dataset type label at offset 40
    write_string(&mut record, 40, "DATA", 8);

    record
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::XptValue;

    #[test]
    fn test_dataset_info() {
        let info = DatasetInfo::new("DM", vec![XptColumn::numeric("AGE")])
            .with_label("Demographics")
            .with_type("DATA");

        assert_eq!(info.name, "DM");
        assert_eq!(info.label, Some("Demographics".to_string()));
        assert_eq!(info.dataset_type, Some("DATA".to_string()));
    }

    #[test]
    fn test_streaming_writer_to_vec() {
        let mut output = Vec::new();
        let info = DatasetInfo::new(
            "TEST",
            vec![XptColumn::character("NAME", 8), XptColumn::numeric("AGE")],
        );

        {
            let mut writer = StreamingWriter::new(&mut output, info).unwrap();

            let obs1 = Observation::new(vec![XptValue::character("JOHN"), XptValue::numeric(30.0)]);
            writer.write_observation(&obs1).unwrap();

            let obs2 = Observation::new(vec![XptValue::character("JANE"), XptValue::numeric(25.0)]);
            writer.write_observation(&obs2).unwrap();

            writer.finish().unwrap();
        }

        // Output should be non-empty and a multiple of 80 bytes
        assert!(!output.is_empty());
        assert!(output.len() % RECORD_LEN == 0);
    }
}
