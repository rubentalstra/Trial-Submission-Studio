//! Streaming XPT reader with observation iterator.
//!
//! This module provides a streaming reader that parses headers once
//! and iterates over observations without loading all data into memory.

use std::io::{BufReader, Read, Seek, SeekFrom};

use super::observation::{observation_length, parse_observation};
use crate::XptVersion;
use crate::error::{Result, XptError};
use crate::header::{
    LabelSectionType, RECORD_LEN, align_to_record, is_label_header, parse_dataset_label,
    parse_dataset_name, parse_dataset_type, parse_labelv8_data, parse_labelv9_data,
    parse_namestr_len, parse_namestr_records, parse_variable_count, validate_dscrptr_header,
    validate_library_header, validate_member_header, validate_namestr_header, validate_obs_header,
};
use crate::types::{Observation, XptColumn, XptReaderOptions};

/// Dataset metadata (without observation data).
///
/// Contains all information needed to interpret observations.
#[derive(Debug, Clone)]
pub struct DatasetMeta {
    /// Dataset name (e.g., "DM", "AE").
    pub name: String,
    /// Optional dataset label.
    pub label: Option<String>,
    /// Dataset type (e.g., "DATA").
    pub dataset_type: Option<String>,
    /// Column definitions.
    pub columns: Vec<XptColumn>,
    /// Detected XPT version.
    pub version: XptVersion,
    /// Bytes per observation row.
    pub observation_length: usize,
}

/// Streaming XPT reader.
///
/// Parses headers once and provides an iterator over observations.
/// This allows reading large files without loading all data into memory.
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// use tss_xpt::reader::StreamingReader;
///
/// let file = File::open("large.xpt").unwrap();
/// let mut reader = StreamingReader::new(file).unwrap();
///
/// println!("Dataset: {}", reader.meta().name);
/// println!("Columns: {:?}", reader.meta().columns.iter().map(|c| &c.name).collect::<Vec<_>>());
///
/// // Stream observations one at a time
/// for obs in reader.observations() {
///     let obs = obs.unwrap();
///     // Process observation...
/// }
/// ```
pub struct StreamingReader<R: Read> {
    reader: BufReader<R>,
    meta: DatasetMeta,
    options: XptReaderOptions,
    /// Byte offset where observations start.
    obs_start: u64,
    /// Current observation index.
    current_obs: usize,
    /// Total number of observations (if known).
    total_obs: Option<usize>,
}

impl<R: Read> StreamingReader<R> {
    /// Get the dataset metadata.
    #[must_use]
    pub fn meta(&self) -> &DatasetMeta {
        &self.meta
    }

    /// Get the detected XPT version.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.meta.version
    }

    /// Get the column definitions.
    #[must_use]
    pub fn columns(&self) -> &[XptColumn] {
        &self.meta.columns
    }

    /// Get the number of columns.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.meta.columns.len()
    }

    /// Get total observations if known (requires Seek).
    #[must_use]
    pub fn total_observations(&self) -> Option<usize> {
        self.total_obs
    }
}

impl<R: Read + Seek> StreamingReader<R> {
    /// Create a new streaming reader.
    ///
    /// Parses all headers and positions at the start of observation data.
    pub fn new(reader: R) -> Result<Self> {
        Self::with_options(reader, XptReaderOptions::default())
    }

    /// Create a streaming reader with options.
    pub fn with_options(reader: R, options: XptReaderOptions) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        #[allow(unused_assignments)]
        let mut offset = 0u64;

        // Read library header
        let mut record = [0u8; RECORD_LEN];
        reader.read_exact(&mut record)?;
        let version = validate_library_header(&record)?;
        offset += RECORD_LEN as u64;

        // Skip library real header and modified header
        reader.seek(SeekFrom::Current(RECORD_LEN as i64 * 2))?;
        offset += RECORD_LEN as u64 * 2;

        // Member header
        reader.read_exact(&mut record)?;
        let _member_version = validate_member_header(&record)?;
        let namestr_len = parse_namestr_len(&record)?;
        offset += RECORD_LEN as u64;

        // DSCRPTR header
        reader.read_exact(&mut record)?;
        let _dscrptr_version = validate_dscrptr_header(&record)?;
        offset += RECORD_LEN as u64;

        // Member data
        reader.read_exact(&mut record)?;
        let dataset_name = parse_dataset_name(&record, version)?;
        offset += RECORD_LEN as u64;

        // Member second
        reader.read_exact(&mut record)?;
        let dataset_label = parse_dataset_label(&record);
        let dataset_type = parse_dataset_type(&record);
        offset += RECORD_LEN as u64;

        // NAMESTR header
        reader.read_exact(&mut record)?;
        let _namestr_version = validate_namestr_header(&record)?;
        let var_count = parse_variable_count(&record, version)?;
        offset += RECORD_LEN as u64;

        // Read NAMESTR records
        let namestr_total = var_count
            .checked_mul(namestr_len)
            .ok_or(XptError::ObservationOverflow)?;
        let mut namestr_data = vec![0u8; namestr_total];
        reader.read_exact(&mut namestr_data)?;
        offset += namestr_total as u64;

        // Align to record boundary
        let aligned = align_to_record(offset as usize) as u64;
        if aligned > offset {
            reader.seek(SeekFrom::Current((aligned - offset) as i64))?;
            offset = aligned;
        }

        // Parse columns
        let mut columns = parse_namestr_records(&namestr_data, var_count, namestr_len, version)?;

        // Check for optional LABELV8/V9 section
        reader.read_exact(&mut record)?;
        if let Some(label_type) = is_label_header(&record) {
            offset += RECORD_LEN as u64;

            // Find OBS header by reading records until we find it
            let mut label_data = Vec::new();
            loop {
                reader.read_exact(&mut record)?;
                if validate_obs_header(&record).is_ok() {
                    // Found OBS header, apply label data
                    match label_type {
                        LabelSectionType::V8 => {
                            let _ = parse_labelv8_data(&label_data, &mut columns);
                        }
                        LabelSectionType::V9 => {
                            let _ = parse_labelv9_data(&label_data, &mut columns);
                        }
                        LabelSectionType::None => {}
                    }
                    offset += label_data.len() as u64 + RECORD_LEN as u64;
                    break;
                }
                label_data.extend_from_slice(&record);
            }
        } else {
            // Current record should be OBS header
            let _obs_version = validate_obs_header(&record)?;
            offset += RECORD_LEN as u64;
        }

        // Calculate observation length
        let obs_len = observation_length(&columns).ok_or(XptError::ObservationOverflow)?;

        // Suppress unused offset warning - we track it for debugging but use stream_position()
        let _ = offset;

        // Calculate total observations from remaining file size
        let current_pos = reader.stream_position()?;
        let end_pos = reader.seek(SeekFrom::End(0))?;
        let data_bytes = end_pos.saturating_sub(current_pos) as usize;
        reader.seek(SeekFrom::Start(current_pos))?;

        let total_obs = if obs_len > 0 {
            Some(data_bytes / obs_len)
        } else {
            Some(0)
        };

        let meta = DatasetMeta {
            name: dataset_name,
            label: dataset_label,
            dataset_type,
            columns,
            version,
            observation_length: obs_len,
        };

        Ok(Self {
            reader,
            meta,
            options,
            obs_start: current_pos,
            current_obs: 0,
            total_obs,
        })
    }

    /// Reset to the beginning of observations.
    pub fn reset(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(self.obs_start))?;
        self.current_obs = 0;
        Ok(())
    }

    /// Create an iterator over observations.
    pub fn observations(&mut self) -> ObservationIter<'_, R> {
        ObservationIter { reader: self }
    }

    /// Read all observations into a vector.
    ///
    /// This consumes all remaining observations from the current position.
    pub fn read_all_observations(&mut self) -> Result<Vec<Observation>> {
        let count = self.total_obs.unwrap_or(0);
        let mut observations = Vec::with_capacity(count);

        for obs_result in self.observations() {
            observations.push(obs_result?);
        }

        Ok(observations)
    }

    /// Read the next observation.
    fn read_next_observation(&mut self) -> Result<Option<Observation>> {
        if self.meta.observation_length == 0 {
            return Ok(None);
        }

        // Check if we've reached the expected end
        if let Some(total) = self.total_obs
            && self.current_obs >= total
        {
            return Ok(None);
        }

        // Read observation bytes
        let mut row_bytes = vec![0u8; self.meta.observation_length];
        match self.reader.read_exact(&mut row_bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        }

        // Skip all-space rows (padding at end)
        if row_bytes.iter().all(|&b| b == b' ') {
            return Ok(None);
        }

        let obs = parse_observation(&row_bytes, &self.meta.columns, self.options.trim_strings);
        self.current_obs += 1;

        Ok(Some(obs))
    }
}

/// Iterator over observations in a streaming reader.
pub struct ObservationIter<'a, R: Read + Seek> {
    reader: &'a mut StreamingReader<R>,
}

impl<R: Read + Seek> Iterator for ObservationIter<'_, R> {
    type Item = Result<Observation>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_next_observation() {
            Ok(Some(obs)) => Some(Ok(obs)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self
            .reader
            .total_obs
            .map(|t| t.saturating_sub(self.reader.current_obs));
        (remaining.unwrap_or(0), remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests are in the crate-level tests with real XPT files.
    // These tests verify the basic API works.

    #[test]
    fn test_dataset_meta_debug() {
        let meta = DatasetMeta {
            name: "DM".to_string(),
            label: Some("Demographics".to_string()),
            dataset_type: Some("DATA".to_string()),
            columns: vec![],
            version: XptVersion::V5,
            observation_length: 0,
        };
        let debug = format!("{:?}", meta);
        assert!(debug.contains("DM"));
    }
}
