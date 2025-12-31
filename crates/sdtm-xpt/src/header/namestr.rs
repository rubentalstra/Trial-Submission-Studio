//! NAMESTR record parsing and building.
//!
//! The NAMESTR record describes a single variable in an XPT dataset.
//! Each NAMESTR is 140 bytes (or 136 bytes for VAX/VMS).
//!
//! # NAMESTR Structure (140 bytes)
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

use crate::error::{Result, XptError};
use crate::types::{Justification, XptColumn, XptType};

/// Standard NAMESTR length.
pub const NAMESTR_LEN: usize = 140;

/// VAX/VMS NAMESTR length (shorter reserved section).
pub const NAMESTR_LEN_VAX: usize = 136;

/// Parse a single NAMESTR record into an XptColumn.
///
/// # Arguments
/// * `data` - Byte slice containing the NAMESTR data
/// * `namestr_len` - Length of NAMESTR (140 or 136 for VAX/VMS)
/// * `index` - Variable index (for error messages)
///
/// # Returns
/// Parsed `XptColumn` on success.
pub fn parse_namestr(data: &[u8], namestr_len: usize, index: usize) -> Result<XptColumn> {
    if data.len() < namestr_len.min(88) {
        return Err(XptError::InvalidNamestr {
            index,
            message: format!("data too short: {} bytes", data.len()),
        });
    }

    // ntype: variable type (1=NUM, 2=CHAR)
    let ntype = read_i16(data, 0);
    let data_type = XptType::from_ntype(ntype).ok_or_else(|| XptError::InvalidNamestr {
        index,
        message: format!("invalid ntype: {ntype}"),
    })?;

    // nlng: variable length
    let length = read_i16(data, 4) as u16;
    if length == 0 {
        return Err(XptError::InvalidNamestr {
            index,
            message: "variable length is zero".to_string(),
        });
    }

    // nname: variable name (8 chars)
    let name = read_string(data, 8, 8);
    if name.is_empty() {
        return Err(XptError::InvalidNamestr {
            index,
            message: "empty variable name".to_string(),
        });
    }

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
///
/// # Returns
/// 140-byte NAMESTR record.
#[must_use]
pub fn build_namestr(column: &XptColumn, varnum: u16, position: u32) -> [u8; NAMESTR_LEN] {
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
    write_string(&mut buf, 8, &column.name, 8);

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

    // rest: reserved (52 bytes, zeros) - already zero from initialization

    buf
}

/// Parse multiple NAMESTR records.
///
/// # Arguments
/// * `data` - Byte slice containing all NAMESTR data
/// * `var_count` - Number of variables
/// * `namestr_len` - Length of each NAMESTR (140 or 136)
///
/// # Returns
/// Vector of parsed columns.
pub fn parse_namestr_records(
    data: &[u8],
    var_count: usize,
    namestr_len: usize,
) -> Result<Vec<XptColumn>> {
    let mut columns = Vec::with_capacity(var_count);

    for idx in 0..var_count {
        let offset = idx
            .checked_mul(namestr_len)
            .ok_or(XptError::ObservationOverflow)?;

        let record =
            data.get(offset..offset + namestr_len)
                .ok_or_else(|| XptError::InvalidNamestr {
                    index: idx,
                    message: "NAMESTR data out of bounds".to_string(),
                })?;

        columns.push(parse_namestr(record, namestr_len, idx)?);
    }

    Ok(columns)
}

/// Read a big-endian i16 from data.
fn read_i16(data: &[u8], offset: usize) -> i16 {
    let bytes = [data[offset], data[offset + 1]];
    i16::from_be_bytes(bytes)
}

/// Read a big-endian i32 from data.
#[allow(dead_code)]
fn read_i32(data: &[u8], offset: usize) -> i32 {
    let bytes = [
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ];
    i32::from_be_bytes(bytes)
}

/// Read a string from data, trimming trailing spaces.
fn read_string(data: &[u8], offset: usize, len: usize) -> String {
    data.get(offset..offset + len)
        .map(|slice| String::from_utf8_lossy(slice).trim_end().to_string())
        .unwrap_or_default()
}

/// Write a big-endian i16 to buffer.
fn write_i16(buf: &mut [u8], offset: usize, value: i16) {
    let bytes = value.to_be_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
}

/// Write a big-endian i32 to buffer.
fn write_i32(buf: &mut [u8], offset: usize, value: i32) {
    let bytes = value.to_be_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
    buf[offset + 2] = bytes[2];
    buf[offset + 3] = bytes[3];
}

/// Write a string to buffer, space-padded to length.
fn write_string(buf: &mut [u8], offset: usize, value: &str, len: usize) {
    for (i, ch) in value.chars().take(len).enumerate() {
        buf[offset + i] = if ch.is_ascii() { ch as u8 } else { b'?' };
    }
    for i in value.len()..len {
        buf[offset + i] = b' ';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse_numeric() {
        let col = XptColumn::numeric("AGE")
            .with_label("Age in Years")
            .with_length(8);

        let namestr = build_namestr(&col, 1, 0);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0).unwrap();

        assert_eq!(parsed.name, "AGE");
        assert_eq!(parsed.label, Some("Age in Years".to_string()));
        assert_eq!(parsed.data_type, XptType::Num);
        assert_eq!(parsed.length, 8);
    }

    #[test]
    fn test_build_and_parse_character() {
        let col = XptColumn::character("USUBJID", 20)
            .with_label("Unique Subject ID")
            .with_format("$20", 20, 0);

        let namestr = build_namestr(&col, 1, 0);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0).unwrap();

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

        let result = parse_namestr(&namestr, NAMESTR_LEN, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zero_length() {
        let mut namestr = [0u8; NAMESTR_LEN];
        namestr[1] = 1; // ntype = 1
        namestr[4] = 0;
        namestr[5] = 0; // length = 0

        let result = parse_namestr(&namestr, NAMESTR_LEN, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_with_format() {
        let col = XptColumn::numeric("VISIT")
            .with_label("Visit Number")
            .with_format("BEST", 8, 2)
            .with_informat("F", 8, 2)
            .with_justification(Justification::Right);

        let namestr = build_namestr(&col, 5, 100);
        let parsed = parse_namestr(&namestr, NAMESTR_LEN, 0).unwrap();

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
    fn test_parse_multiple_namestr() {
        let cols = vec![
            XptColumn::numeric("AGE"),
            XptColumn::character("SEX", 1),
            XptColumn::character("RACE", 40),
        ];

        let mut data = Vec::new();
        let mut position = 0u32;
        for (i, col) in cols.iter().enumerate() {
            let namestr = build_namestr(col, (i + 1) as u16, position);
            data.extend_from_slice(&namestr);
            position += col.length as u32;
        }

        let parsed = parse_namestr_records(&data, 3, NAMESTR_LEN).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].name, "AGE");
        assert_eq!(parsed[1].name, "SEX");
        assert_eq!(parsed[2].name, "RACE");
    }

    #[test]
    fn test_string_padding() {
        let col = XptColumn::numeric("X");
        let namestr = build_namestr(&col, 1, 0);

        // Name should be "X" followed by 7 spaces
        let name_bytes = &namestr[8..16];
        assert_eq!(name_bytes, b"X       ");
    }
}
