//! NAMESTR record parsing and building.
//!
//! The NAMESTR record describes a single variable in an XPT dataset.
//! Each NAMESTR is 140 bytes (or 136 bytes for VAX/VMS).
//!
//! # NAMESTR Structure (140 bytes)
//!
//! ## V5 Structure
//!
//! | Offset | Field   | Type     | Description                    |
//! |--------|---------|----------|--------------------------------|
//! | 0-1    | ntype   | short    | 1=NUMERIC, 2=CHAR              |
//! | 2-3    | nhfun   | short    | Hash (always 0)                |
//! | 4-5    | nlng    | short    | Variable length in observation |
//! | 6-7    | nvar0   | short    | Variable number                |
//! | 8-15   | nname   | char[8]  | Variable name                  |
//! | 16-55  | nlabel  | char[40] | Variable label                 |
//! | 56-63  | nform   | char[8]  | Format name                    |
//! | 64-65  | nfl     | short    | Format field length            |
//! | 66-67  | nfd     | short    | Format decimals                |
//! | 68-69  | nfj     | short    | Justification (0=left, 1=right)|
//! | 70-71  | nfill   | char[2]  | Padding                        |
//! | 72-79  | niform  | char[8]  | Informat name                  |
//! | 80-81  | nifl    | short    | Informat length                |
//! | 82-83  | nifd    | short    | Informat decimals              |
//! | 84-87  | npos    | long     | Position in observation        |
//! | 88-139 | rest    | char[52] | Reserved                       |
//!
//! ## V8 Extended Structure
//!
//! V8 uses the reserved area for extended fields:
//!
//! | Offset  | Field    | Type     | Description                   |
//! |---------|----------|----------|-------------------------------|
//! | 88-119  | longname | char[32] | Long variable name (if > 8)   |
//! | 120-121 | lablen   | short    | Label length indicator        |
//! | 122-139 | rest     | char[18] | Reserved                      |

use super::common::{NAMESTR_LEN, read_i16, read_string, write_i16, write_i32, write_string};
use crate::error::{Result, XptError};
use crate::types::{Justification, XptColumn, XptType, XptVersion};

/// Parse a single NAMESTR record into an XptColumn.
///
/// # Arguments
/// * `data` - Byte slice containing the NAMESTR data
/// * `namestr_len` - Length of NAMESTR (140 or 136 for VAX/VMS)
/// * `index` - Variable index (for error messages)
/// * `version` - XPT version (V5 or V8) for extended field support
///
/// # Returns
/// Parsed `XptColumn` on success.
pub fn parse_namestr(
    data: &[u8],
    namestr_len: usize,
    index: usize,
    version: XptVersion,
) -> Result<XptColumn> {
    if data.len() < namestr_len.min(88) {
        return Err(XptError::invalid_namestr(index, format!("data too short: {} bytes", data.len())));
    }

    // ntype: variable type (1=NUM, 2=CHAR)
    let ntype = read_i16(data, 0);
    let data_type = XptType::from_ntype(ntype)
        .ok_or_else(|| XptError::invalid_namestr(index, format!("invalid ntype: {ntype}")))?;

    // nlng: variable length
    let length = read_i16(data, 4) as u16;
    if length == 0 {
        return Err(XptError::invalid_namestr(index, "variable length is zero"));
    }

    // nname: variable name (8 chars)
    let short_name = read_string(data, 8, 8);
    if short_name.is_empty() {
        return Err(XptError::invalid_namestr(index, "empty variable name"));
    }

    // V8: Check for long name in extended area (offset 88-119)
    let name = if version.supports_long_names() && data.len() >= 120 {
        let long_name = read_string(data, 88, 32);
        if long_name.is_empty() {
            short_name
        } else {
            long_name
        }
    } else {
        short_name
    };

    // nlabel: variable label (40 chars)
    let label = read_string(data, 16, 40);

    // nform: format name (8 chars)
    let format = read_string(data, 56, 8);

    // nfl, nfd: format length and decimals
    let format_length = read_i16(data, 64) as u16;
    let format_decimals = read_i16(data, 66) as u16;

    // nfj: justification
    let justification = Justification::from_nfj(read_i16(data, 68));

    // niform: informat name (8 chars)
    let informat = read_string(data, 72, 8);

    // nifl, nifd: informat length and decimals
    let informat_length = read_i16(data, 80) as u16;
    let informat_decimals = read_i16(data, 82) as u16;

    Ok(XptColumn {
        name,
        label: if label.is_empty() { None } else { Some(label) },
        data_type,
        length,
        format: if format.is_empty() {
            None
        } else {
            Some(format)
        },
        format_length,
        format_decimals,
        informat: if informat.is_empty() {
            None
        } else {
            Some(informat)
        },
        informat_length,
        informat_decimals,
        justification,
    })
}

/// Build a NAMESTR record from an XptColumn.
///
/// # Arguments
/// * `column` - The column definition
/// * `varnum` - Variable number (1-based)
/// * `position` - Position in observation (byte offset)
/// * `version` - XPT version (V5 or V8) for extended field support
///
/// # Returns
/// 140-byte NAMESTR record.
#[must_use]
pub fn build_namestr(
    column: &XptColumn,
    varnum: u16,
    position: u32,
    version: XptVersion,
) -> [u8; NAMESTR_LEN] {
    let mut buf = [0u8; NAMESTR_LEN];

    // ntype: variable type
    write_i16(&mut buf, 0, column.data_type.to_ntype());

    // nhfun: hash function (always 0)
    write_i16(&mut buf, 2, 0);

    // nlng: variable length
    write_i16(&mut buf, 4, column.length as i16);

    // nvar0: variable number
    write_i16(&mut buf, 6, varnum as i16);

    // nname: variable name (8 chars, space-padded)
    // For V8, if name > 8 chars, truncate here and write full name to extended area
    let short_name = if column.name.len() > 8 {
        &column.name[..8]
    } else {
        &column.name
    };
    write_string(&mut buf, 8, short_name, 8);

    // nlabel: variable label (40 chars, space-padded)
    let label = column.label.as_deref().unwrap_or("");
    write_string(&mut buf, 16, label, 40);

    // nform: format name (8 chars)
    let format = column.format.as_deref().unwrap_or("");
    write_string(&mut buf, 56, format, 8);

    // nfl: format length
    write_i16(&mut buf, 64, column.format_length as i16);

    // nfd: format decimals
    write_i16(&mut buf, 66, column.format_decimals as i16);

    // nfj: justification
    write_i16(&mut buf, 68, column.justification.to_nfj());

    // nfill: padding (2 bytes, zeros)
    buf[70] = 0;
    buf[71] = 0;

    // niform: informat name (8 chars)
    let informat = column.informat.as_deref().unwrap_or("");
    write_string(&mut buf, 72, informat, 8);

    // nifl: informat length
    write_i16(&mut buf, 80, column.informat_length as i16);

    // nifd: informat decimals
    write_i16(&mut buf, 82, column.informat_decimals as i16);

    // npos: position in observation
    write_i32(&mut buf, 84, position as i32);

    // V8 extended fields (offset 88-139)
    if version.supports_long_names() {
        // longname: long variable name (32 chars, space-padded) at offset 88-119
        if column.name.len() > 8 {
            write_string(&mut buf, 88, &column.name, 32);
        }

        // lablen: label length indicator at offset 120-121
        if let Some(lbl) = &column.label
            && lbl.len() > 40
        {
            write_i16(&mut buf, 120, lbl.len() as i16);
        }
    }

    buf
}

/// Parse multiple NAMESTR records.
///
/// # Arguments
/// * `data` - Byte slice containing all NAMESTR data
/// * `var_count` - Number of variables
/// * `namestr_len` - Length of each NAMESTR (140 or 136)
/// * `version` - XPT version (V5 or V8) for extended field support
///
/// # Returns
/// Vector of parsed columns.
pub fn parse_namestr_records(
    data: &[u8],
    var_count: usize,
    namestr_len: usize,
    version: XptVersion,
) -> Result<Vec<XptColumn>> {
    let mut columns = Vec::with_capacity(var_count);

    for idx in 0..var_count {
        let offset = idx
            .checked_mul(namestr_len)
            .ok_or(XptError::ObservationOverflow)?;

        let record =
            data.get(offset..offset + namestr_len)
                .ok_or_else(|| XptError::invalid_namestr(idx, "NAMESTR data out of bounds"))?;

        columns.push(parse_namestr(record, namestr_len, idx, version)?);
    }

    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse_numeric_v5() {
        let col = XptColumn::numeric("AGE")
            .with_label("Age in Years")
            .with_length(8);

        let namestr = build_namestr(&col, 1, 0, XptVersion::V5);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5).unwrap();

        assert_eq!(parsed.name, "AGE");
        assert_eq!(parsed.label, Some("Age in Years".to_string()));
        assert_eq!(parsed.data_type, XptType::Num);
        assert_eq!(parsed.length, 8);
    }

    #[test]
    fn test_build_and_parse_character_v5() {
        let col = XptColumn::character("USUBJID", 20)
            .with_label("Unique Subject ID")
            .with_format("$20", 20, 0);

        let namestr = build_namestr(&col, 1, 0, XptVersion::V5);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5).unwrap();

        assert_eq!(parsed.name, "USUBJID");
        assert_eq!(parsed.data_type, XptType::Char);
        assert_eq!(parsed.length, 20);
        assert_eq!(parsed.format, Some("$20".to_string()));
        assert_eq!(parsed.format_length, 20);
    }

    #[test]
    fn test_parse_invalid_ntype() {
        let mut namestr = [0u8; NAMESTR_LEN];
        namestr[0] = 0;
        namestr[1] = 5; // Invalid ntype

        let result = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zero_length() {
        let mut namestr = [0u8; NAMESTR_LEN];
        namestr[1] = 1; // ntype = 1
        namestr[4] = 0;
        namestr[5] = 0; // length = 0

        let result = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_with_format_v5() {
        let col = XptColumn::numeric("VISIT")
            .with_label("Visit Number")
            .with_format("BEST", 8, 2)
            .with_informat("F", 8, 2)
            .with_justification(Justification::Right);

        let namestr = build_namestr(&col, 5, 100, XptVersion::V5);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5).unwrap();

        assert_eq!(parsed.name, col.name);
        assert_eq!(parsed.label, col.label);
        assert_eq!(parsed.format, col.format);
        assert_eq!(parsed.format_length, col.format_length);
        assert_eq!(parsed.format_decimals, col.format_decimals);
        assert_eq!(parsed.informat, col.informat);
        assert_eq!(parsed.informat_length, col.informat_length);
        assert_eq!(parsed.informat_decimals, col.informat_decimals);
        assert_eq!(parsed.justification, col.justification);
    }

    #[test]
    fn test_parse_multiple_namestr_v5() {
        let cols = vec![
            XptColumn::numeric("AGE"),
            XptColumn::character("SEX", 1),
            XptColumn::character("RACE", 40),
        ];

        let mut data = Vec::new();
        let mut position = 0u32;
        for (i, col) in cols.iter().enumerate() {
            let namestr = build_namestr(col, (i + 1) as u16, position, XptVersion::V5);
            data.extend_from_slice(&namestr);
            position += col.length as u32;
        }

        let parsed = parse_namestr_records(&data, 3, NAMESTR_LEN, XptVersion::V5).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].name, "AGE");
        assert_eq!(parsed[1].name, "SEX");
        assert_eq!(parsed[2].name, "RACE");
    }

    #[test]
    fn test_string_padding_v5() {
        let col = XptColumn::numeric("X");
        let namestr = build_namestr(&col, 1, 0, XptVersion::V5);

        // Name should be "X" followed by 7 spaces
        let name_bytes = &namestr[8..16];
        assert_eq!(name_bytes, b"X       ");
    }

    // V8 tests for long names

    #[test]
    fn test_build_and_parse_long_name_v8() {
        // Create a column with a long name (> 8 chars)
        let mut col = XptColumn::numeric("VERYLONGVARIABLENAME");
        col.name = "VERYLONGVARIABLENAME".to_string(); // Ensure full name
        col.label = Some("A Long Variable".to_string());

        let namestr = build_namestr(&col, 1, 0, XptVersion::V8);

        // Verify short name is truncated in base area
        let short_name = &namestr[8..16];
        assert_eq!(&short_name[..8], b"VERYLONG");

        // Verify long name is written to extended area (offset 88-119)
        let long_name_bytes = &namestr[88..120];
        let long_name = String::from_utf8_lossy(long_name_bytes)
            .trim_end()
            .to_string();
        assert_eq!(long_name, "VERYLONGVARIABLENAME");

        // Parse and verify
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V8).unwrap();
        assert_eq!(parsed.name, "VERYLONGVARIABLENAME");
    }

    #[test]
    fn test_v8_short_name_no_extended() {
        // Short name should not use extended area
        let col = XptColumn::numeric("AGE");

        let namestr = build_namestr(&col, 1, 0, XptVersion::V8);

        // Extended area should be zeros (no long name written)
        let long_name_bytes = &namestr[88..120];
        assert!(long_name_bytes.iter().all(|&b| b == 0));

        // Parse and verify short name is used
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V8).unwrap();
        assert_eq!(parsed.name, "AGE");
    }

    #[test]
    fn test_v5_ignores_extended_area() {
        // V5 should ignore extended area even if data is there
        let mut col = XptColumn::numeric("VERYLONGVARIABLENAME");
        col.name = "VERYLONGVARIABLENAME".to_string();

        // Build with V8 (writes long name)
        let namestr = build_namestr(&col, 1, 0, XptVersion::V8);

        // Parse with V5 should use truncated short name
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0, XptVersion::V5).unwrap();
        assert_eq!(parsed.name, "VERYLONG");
    }
}
