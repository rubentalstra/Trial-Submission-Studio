//! LABELV8/V9 extended label and format sections.
//!
//! These sections are written when V8 format is used and either:
//! - LABELV8: Labels exceed 40 characters (and format names ≤ 8 chars)
//! - LABELV9: Format names exceed 8 characters (includes long labels too)
//!
//! The label section follows after NAMESTR records and before the OBS header.
//!
//! # LABELV8 Record Structure
//!
//! For each variable with label > 40 chars:
//! | Offset | Field   | Type     | Description                    |
//! |--------|---------|----------|--------------------------------|
//! | 0-1    | varnum  | short    | Variable number (1-based)      |
//! | 2-3    | lablen  | short    | Label length                   |
//! | 4+     | label   | char[n]  | Label text (length = lablen)   |
//!
//! # LABELV9 Record Structure
//!
//! For each variable with label > 40 chars OR format > 8 chars:
//! | Offset | Field   | Type     | Description                    |
//! |--------|---------|----------|--------------------------------|
//! | 0-1    | varnum  | short    | Variable number (1-based)      |
//! | 2-3    | namelen | short    | Format name length (0 if ≤8)   |
//! | 4-5    | lablen  | short    | Label length (0 if ≤40)        |
//! | 6-7    | inflen  | short    | Informat name length (0 if ≤8) |
//! | 8+     | format  | char[n]  | Format name (if namelen > 0)   |
//! | ...    | label   | char[n]  | Label text (if lablen > 0)     |
//! | ...    | informat| char[n]  | Informat name (if inflen > 0)  |

use crate::error::Result;
use crate::types::XptColumn;

use super::library::RECORD_LEN;

/// LABELV8 header prefix.
pub const LABELV8_HEADER_PREFIX: &[u8; 48] = b"HEADER RECORD*******LABELV8 HEADER RECORD!!!!!!!";

/// LABELV9 header prefix.
pub const LABELV9_HEADER_PREFIX: &[u8; 48] = b"HEADER RECORD*******LABELV9 HEADER RECORD!!!!!!!";

/// Type of label section needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelSectionType {
    /// No label section needed (all labels ≤40 chars, all formats ≤8 chars).
    None,
    /// LABELV8 section (some labels > 40 chars, but all formats ≤8 chars).
    V8,
    /// LABELV9 section (some formats > 8 chars, or need full V9 support).
    V9,
}

/// Determine which label section type is needed for the given columns.
///
/// # Rules
/// - If any format or informat name > 8 chars: LABELV9
/// - Else if any label > 40 chars: LABELV8
/// - Otherwise: None
#[must_use]
pub fn determine_label_section(columns: &[XptColumn]) -> LabelSectionType {
    let needs_long_format = columns.iter().any(|c| {
        c.format.as_ref().is_some_and(|f| f.len() > 8)
            || c.informat.as_ref().is_some_and(|f| f.len() > 8)
    });

    if needs_long_format {
        return LabelSectionType::V9;
    }

    let needs_long_label = columns
        .iter()
        .any(|c| c.label.as_ref().is_some_and(|l| l.len() > 40));

    if needs_long_label {
        return LabelSectionType::V8;
    }

    LabelSectionType::None
}

/// Build a LABELV8 header record.
#[must_use]
pub fn build_labelv8_header() -> [u8; RECORD_LEN] {
    build_label_header(LABELV8_HEADER_PREFIX)
}

/// Build a LABELV9 header record.
#[must_use]
pub fn build_labelv9_header() -> [u8; RECORD_LEN] {
    build_label_header(LABELV9_HEADER_PREFIX)
}

/// Build a label header with the given prefix.
fn build_label_header(prefix: &[u8; 48]) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];
    record[..48].copy_from_slice(prefix);
    // Rest is space-padded (default)
    record
}

/// Build LABELV8 section data for columns with long labels.
///
/// Returns the data bytes to write after the LABELV8 header.
/// Each entry is: varnum (2 bytes) + lablen (2 bytes) + label (lablen bytes)
#[must_use]
pub fn build_labelv8_data(columns: &[XptColumn]) -> Vec<u8> {
    let mut data = Vec::new();

    for (idx, col) in columns.iter().enumerate() {
        if let Some(label) = &col.label {
            if label.len() > 40 {
                let varnum = (idx + 1) as u16;
                let lablen = label.len() as u16;

                // varnum (2 bytes, big-endian)
                data.extend_from_slice(&varnum.to_be_bytes());
                // lablen (2 bytes, big-endian)
                data.extend_from_slice(&lablen.to_be_bytes());
                // label text
                data.extend_from_slice(label.as_bytes());
            }
        }
    }

    data
}

/// Build LABELV9 section data for columns with long labels or formats.
///
/// Returns the data bytes to write after the LABELV9 header.
/// Each entry contains variable-length fields for long formats, labels, and informats.
#[must_use]
pub fn build_labelv9_data(columns: &[XptColumn]) -> Vec<u8> {
    let mut data = Vec::new();

    for (idx, col) in columns.iter().enumerate() {
        let format_long = col.format.as_ref().is_some_and(|f| f.len() > 8);
        let label_long = col.label.as_ref().is_some_and(|l| l.len() > 40);
        let informat_long = col.informat.as_ref().is_some_and(|f| f.len() > 8);

        if format_long || label_long || informat_long {
            let varnum = (idx + 1) as u16;
            let namelen = if format_long {
                col.format.as_ref().map(|f| f.len()).unwrap_or(0) as u16
            } else {
                0
            };
            let lablen = if label_long {
                col.label.as_ref().map(|l| l.len()).unwrap_or(0) as u16
            } else {
                0
            };
            let inflen = if informat_long {
                col.informat.as_ref().map(|f| f.len()).unwrap_or(0) as u16
            } else {
                0
            };

            // varnum (2 bytes)
            data.extend_from_slice(&varnum.to_be_bytes());
            // namelen (2 bytes)
            data.extend_from_slice(&namelen.to_be_bytes());
            // lablen (2 bytes)
            data.extend_from_slice(&lablen.to_be_bytes());
            // inflen (2 bytes)
            data.extend_from_slice(&inflen.to_be_bytes());

            // format (if namelen > 0)
            if namelen > 0 {
                if let Some(format) = &col.format {
                    data.extend_from_slice(format.as_bytes());
                }
            }

            // label (if lablen > 0)
            if lablen > 0 {
                if let Some(label) = &col.label {
                    data.extend_from_slice(label.as_bytes());
                }
            }

            // informat (if inflen > 0)
            if inflen > 0 {
                if let Some(informat) = &col.informat {
                    data.extend_from_slice(informat.as_bytes());
                }
            }
        }
    }

    data
}

/// Parse LABELV8 section data and update columns with long labels.
///
/// # Arguments
/// * `data` - The label section data (after the header)
/// * `columns` - Mutable slice of columns to update
pub fn parse_labelv8_data(data: &[u8], columns: &mut [XptColumn]) -> Result<()> {
    let mut pos = 0;

    while pos + 4 <= data.len() {
        // varnum (2 bytes)
        let varnum = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // lablen (2 bytes)
        let lablen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // Validate varnum
        if varnum == 0 || varnum > columns.len() {
            break; // Invalid varnum, stop parsing
        }

        // Read label
        if pos + lablen > data.len() {
            break; // Not enough data
        }

        let label = String::from_utf8_lossy(&data[pos..pos + lablen])
            .trim_end()
            .to_string();
        pos += lablen;

        // Update column (1-based to 0-based index)
        if !label.is_empty() {
            columns[varnum - 1].label = Some(label);
        }
    }

    Ok(())
}

/// Parse LABELV9 section data and update columns with long labels/formats.
///
/// # Arguments
/// * `data` - The label section data (after the header)
/// * `columns` - Mutable slice of columns to update
pub fn parse_labelv9_data(data: &[u8], columns: &mut [XptColumn]) -> Result<()> {
    let mut pos = 0;

    while pos + 8 <= data.len() {
        // varnum (2 bytes)
        let varnum = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // namelen (2 bytes)
        let namelen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // lablen (2 bytes)
        let lablen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // inflen (2 bytes)
        let inflen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // Validate varnum
        if varnum == 0 || varnum > columns.len() {
            break; // Invalid varnum, stop parsing
        }

        let col = &mut columns[varnum - 1];

        // Read format (if namelen > 0)
        if namelen > 0 {
            if pos + namelen > data.len() {
                break;
            }
            let format = String::from_utf8_lossy(&data[pos..pos + namelen])
                .trim_end()
                .to_string();
            pos += namelen;
            if !format.is_empty() {
                col.format = Some(format);
            }
        }

        // Read label (if lablen > 0)
        if lablen > 0 {
            if pos + lablen > data.len() {
                break;
            }
            let label = String::from_utf8_lossy(&data[pos..pos + lablen])
                .trim_end()
                .to_string();
            pos += lablen;
            if !label.is_empty() {
                col.label = Some(label);
            }
        }

        // Read informat (if inflen > 0)
        if inflen > 0 {
            if pos + inflen > data.len() {
                break;
            }
            let informat = String::from_utf8_lossy(&data[pos..pos + inflen])
                .trim_end()
                .to_string();
            pos += inflen;
            if !informat.is_empty() {
                col.informat = Some(informat);
            }
        }
    }

    Ok(())
}

/// Validate a LABELV8 header record.
///
/// Returns `true` if the record is a valid LABELV8 header.
#[must_use]
pub fn is_labelv8_header(record: &[u8]) -> bool {
    record.len() >= 48 && record[..48] == *LABELV8_HEADER_PREFIX
}

/// Validate a LABELV9 header record.
///
/// Returns `true` if the record is a valid LABELV9 header.
#[must_use]
pub fn is_labelv9_header(record: &[u8]) -> bool {
    record.len() >= 48 && record[..48] == *LABELV9_HEADER_PREFIX
}

/// Check if a record is any type of label header.
#[must_use]
pub fn is_label_header(record: &[u8]) -> Option<LabelSectionType> {
    if is_labelv8_header(record) {
        Some(LabelSectionType::V8)
    } else if is_labelv9_header(record) {
        Some(LabelSectionType::V9)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::XptType;

    fn make_column(name: &str, label: Option<&str>) -> XptColumn {
        XptColumn {
            name: name.to_string(),
            label: label.map(|s| s.to_string()),
            data_type: XptType::Num,
            length: 8,
            format: None,
            format_length: 0,
            format_decimals: 0,
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification: crate::types::Justification::Right,
        }
    }

    #[test]
    fn test_determine_label_section_none() {
        let columns = vec![
            make_column("AGE", Some("Age in Years")),
            make_column("SEX", Some("Sex")),
        ];
        assert_eq!(determine_label_section(&columns), LabelSectionType::None);
    }

    #[test]
    fn test_determine_label_section_v8() {
        let long_label = "A".repeat(50); // > 40 chars
        let columns = vec![
            make_column("AGE", Some(&long_label)),
            make_column("SEX", Some("Sex")),
        ];
        assert_eq!(determine_label_section(&columns), LabelSectionType::V8);
    }

    #[test]
    fn test_determine_label_section_v9() {
        let mut col = make_column("AGE", Some("Age"));
        col.format = Some("VERYLONGFORMATNAME".to_string()); // > 8 chars
        let columns = vec![col];
        assert_eq!(determine_label_section(&columns), LabelSectionType::V9);
    }

    #[test]
    fn test_determine_label_section_v9_informat() {
        let mut col = make_column("AGE", Some("Age"));
        col.informat = Some("LONGINFORMATNAME".to_string()); // > 8 chars
        let columns = vec![col];
        assert_eq!(determine_label_section(&columns), LabelSectionType::V9);
    }

    #[test]
    fn test_build_labelv8_header() {
        let header = build_labelv8_header();
        assert_eq!(header.len(), RECORD_LEN);
        assert!(is_labelv8_header(&header));
        assert!(!is_labelv9_header(&header));
    }

    #[test]
    fn test_build_labelv9_header() {
        let header = build_labelv9_header();
        assert_eq!(header.len(), RECORD_LEN);
        assert!(is_labelv9_header(&header));
        assert!(!is_labelv8_header(&header));
    }

    #[test]
    fn test_build_and_parse_labelv8_data() {
        let long_label = "This is a very long label that exceeds forty characters limit";
        let mut columns = vec![
            make_column("VAR1", Some("Short")),
            make_column("VAR2", Some(long_label)),
            make_column("VAR3", None),
        ];

        let data = build_labelv8_data(&columns);
        assert!(!data.is_empty());

        // Clear the label to test parsing
        columns[1].label = Some("Short".to_string());

        parse_labelv8_data(&data, &mut columns).unwrap();
        assert_eq!(columns[1].label, Some(long_label.to_string()));
    }

    #[test]
    fn test_build_and_parse_labelv9_data() {
        let long_label = "This is a very long label that exceeds forty characters limit";
        let long_format = "VERYLONGFORMATNAME123";

        let mut col = make_column("VAR1", Some(long_label));
        col.format = Some(long_format.to_string());

        let mut columns = vec![col];

        let data = build_labelv9_data(&columns);
        assert!(!data.is_empty());

        // Clear to test parsing
        columns[0].label = Some("Short".to_string());
        columns[0].format = Some("FMT".to_string());

        parse_labelv9_data(&data, &mut columns).unwrap();
        assert_eq!(columns[0].label, Some(long_label.to_string()));
        assert_eq!(columns[0].format, Some(long_format.to_string()));
    }

    #[test]
    fn test_is_label_header() {
        let v8_header = build_labelv8_header();
        let v9_header = build_labelv9_header();
        let other = [b' '; RECORD_LEN];

        assert_eq!(is_label_header(&v8_header), Some(LabelSectionType::V8));
        assert_eq!(is_label_header(&v9_header), Some(LabelSectionType::V9));
        assert_eq!(is_label_header(&other), None);
    }

    #[test]
    fn test_labelv8_empty_when_no_long_labels() {
        let columns = vec![
            make_column("VAR1", Some("Short label")),
            make_column("VAR2", Some("Another short")),
        ];

        let data = build_labelv8_data(&columns);
        assert!(data.is_empty());
    }

    #[test]
    fn test_labelv9_with_long_informat() {
        let long_informat = "VERYLONGINFORMATNAME";

        let mut col = make_column("VAR1", Some("Short label"));
        col.informat = Some(long_informat.to_string());

        let mut columns = vec![col];

        let data = build_labelv9_data(&columns);
        assert!(!data.is_empty());

        // Clear to test parsing
        columns[0].informat = Some("FMT".to_string());

        parse_labelv9_data(&data, &mut columns).unwrap();
        assert_eq!(columns[0].informat, Some(long_informat.to_string()));
    }
}
