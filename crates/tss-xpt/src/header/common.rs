//! Common header utilities for XPT file format.
//!
//! This module provides shared functions and constants used across all header
//! record types (library, member, namestr, label).

/// Standard record length in bytes.
pub const RECORD_LEN: usize = 80;

/// Standard NAMESTR length (140 bytes).
pub const NAMESTR_LEN: usize = 140;

/// VAX/VMS NAMESTR length (136 bytes).
pub const NAMESTR_LEN_VAX: usize = 136;

// ─────────────────────────────────────────────────────────────────────────────
// Header Prefixes - V5 Format
// ─────────────────────────────────────────────────────────────────────────────

/// Library header prefix (V5).
pub const LIBRARY_HEADER_V5: &str = "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!";
/// Member header prefix (V5).
pub const MEMBER_HEADER_V5: &str = "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!";
/// DSCRPTR header prefix (V5).
pub const DSCRPTR_HEADER_V5: &str = "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!";
/// NAMESTR header prefix (V5).
pub const NAMESTR_HEADER_V5: &str = "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!";
/// OBS header prefix (V5).
pub const OBS_HEADER_V5: &str = "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!";

// ─────────────────────────────────────────────────────────────────────────────
// Header Prefixes - V8 Format
// ─────────────────────────────────────────────────────────────────────────────

/// Library header prefix (V8).
pub const LIBRARY_HEADER_V8: &str = "HEADER RECORD*******LIBV8   HEADER RECORD!!!!!!!";
/// Member header prefix (V8).
pub const MEMBER_HEADER_V8: &str = "HEADER RECORD*******MEMBV8  HEADER RECORD!!!!!!!";
/// DSCRPTR header prefix (V8).
pub const DSCRPTR_HEADER_V8: &str = "HEADER RECORD*******DSCPTV8 HEADER RECORD!!!!!!!";
/// NAMESTR header prefix (V8).
pub const NAMESTR_HEADER_V8: &str = "HEADER RECORD*******NAMSTV8 HEADER RECORD!!!!!!!";
/// OBS header prefix (V8).
pub const OBS_HEADER_V8: &str = "HEADER RECORD*******OBSV8   HEADER RECORD!!!!!!!";

// ─────────────────────────────────────────────────────────────────────────────
// Byte I/O Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Read a big-endian i16 from data at the specified offset.
#[inline]
#[must_use]
pub fn read_i16(data: &[u8], offset: usize) -> i16 {
    let bytes = [data[offset], data[offset + 1]];
    i16::from_be_bytes(bytes)
}

/// Read a big-endian u16 from data at the specified offset.
#[inline]
#[must_use]
pub fn read_u16(data: &[u8], offset: usize) -> u16 {
    let bytes = [data[offset], data[offset + 1]];
    u16::from_be_bytes(bytes)
}

/// Read a big-endian i32 from data at the specified offset.
#[inline]
#[must_use]
pub fn read_i32(data: &[u8], offset: usize) -> i32 {
    let bytes = [
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ];
    i32::from_be_bytes(bytes)
}

/// Write a big-endian i16 to buffer at the specified offset.
#[inline]
pub fn write_i16(buf: &mut [u8], offset: usize, value: i16) {
    let bytes = value.to_be_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
}

/// Write a big-endian u16 to buffer at the specified offset.
#[inline]
pub fn write_u16(buf: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_be_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
}

/// Write a big-endian i32 to buffer at the specified offset.
#[inline]
pub fn write_i32(buf: &mut [u8], offset: usize, value: i32) {
    let bytes = value.to_be_bytes();
    buf[offset] = bytes[0];
    buf[offset + 1] = bytes[1];
    buf[offset + 2] = bytes[2];
    buf[offset + 3] = bytes[3];
}

// ─────────────────────────────────────────────────────────────────────────────
// String I/O Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Read a string from a byte slice, trimming trailing spaces and null bytes.
///
/// # Arguments
/// * `data` - Source byte slice
/// * `offset` - Starting offset in bytes
/// * `len` - Number of bytes to read
///
/// # Returns
/// The string with trailing whitespace and null bytes removed.
#[must_use]
pub fn read_string(data: &[u8], offset: usize, len: usize) -> String {
    data.get(offset..offset + len)
        .map(|slice| {
            String::from_utf8_lossy(slice)
                .trim_end_matches(|c: char| c.is_whitespace() || c == '\0')
                .to_string()
        })
        .unwrap_or_default()
}

/// Write a string to a buffer, space-padded to the specified length.
///
/// Non-ASCII characters are replaced with '?'.
///
/// # Arguments
/// * `buf` - Destination buffer
/// * `offset` - Starting offset in buffer
/// * `value` - String value to write
/// * `len` - Total field length (will be space-padded)
pub fn write_string(buf: &mut [u8], offset: usize, value: &str, len: usize) {
    // First, fill with spaces
    for i in 0..len {
        buf[offset + i] = b' ';
    }
    // Then write the string
    for (i, ch) in value.chars().take(len).enumerate() {
        buf[offset + i] = if ch.is_ascii() { ch as u8 } else { b'?' };
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Header Record Building
// ─────────────────────────────────────────────────────────────────────────────

/// Build a fixed header record with the given prefix.
///
/// The format is:
/// - Bytes 0-47: Header prefix (48 chars)
/// - Bytes 48-77: ASCII '0' characters (30 chars)
/// - Bytes 78-79: Spaces (2 chars)
///
/// # Arguments
/// * `prefix` - The header prefix string (must be 48 chars)
///
/// # Returns
/// An 80-byte record array.
#[must_use]
pub fn build_header_record(prefix: &str) -> [u8; RECORD_LEN] {
    let mut record = [b' '; RECORD_LEN];

    // Copy prefix (max 48 bytes)
    let prefix_bytes = prefix.as_bytes();
    let copy_len = prefix_bytes.len().min(48);
    record[..copy_len].copy_from_slice(&prefix_bytes[..copy_len]);

    // Fill bytes 48-77 with '0'
    for byte in record.iter_mut().take(78).skip(48) {
        *byte = b'0';
    }

    // Bytes 78-79 are already spaces

    record
}

// ─────────────────────────────────────────────────────────────────────────────
// Record Alignment
// ─────────────────────────────────────────────────────────────────────────────

/// Align a byte offset to the next 80-byte record boundary.
///
/// # Arguments
/// * `offset` - Current byte offset
///
/// # Returns
/// The offset aligned to the next record boundary.
#[inline]
#[must_use]
pub const fn align_to_record(offset: usize) -> usize {
    if offset.is_multiple_of(RECORD_LEN) {
        offset
    } else {
        offset + (RECORD_LEN - (offset % RECORD_LEN))
    }
}

/// Calculate the number of records needed for a given byte count.
#[inline]
#[must_use]
pub const fn records_needed(bytes: usize) -> usize {
    if bytes == 0 {
        0
    } else {
        bytes.div_ceil(RECORD_LEN)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// String Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Truncate a string to the specified maximum length.
///
/// # Arguments
/// * `s` - The string to truncate
/// * `max_len` - Maximum allowed length
///
/// # Returns
/// The original string if within limit, otherwise truncated.
#[must_use]
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect()
    }
}

/// Normalize a name: trim whitespace and convert to uppercase.
///
/// # Arguments
/// * `name` - The name to normalize
///
/// # Returns
/// Trimmed uppercase string.
#[must_use]
pub fn normalize_name(name: &str) -> String {
    name.trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_i16() {
        let mut buf = [0u8; 4];
        write_i16(&mut buf, 0, 1234);
        write_i16(&mut buf, 2, -5678);
        assert_eq!(read_i16(&buf, 0), 1234);
        assert_eq!(read_i16(&buf, 2), -5678);
    }

    #[test]
    fn test_read_write_i32() {
        let mut buf = [0u8; 4];
        write_i32(&mut buf, 0, 123456789);
        assert_eq!(read_i32(&buf, 0), 123456789);
    }

    #[test]
    fn test_read_string() {
        let data = b"HELLO   \0\0";
        assert_eq!(read_string(data, 0, 10), "HELLO");

        let data = b"TEST";
        assert_eq!(read_string(data, 0, 4), "TEST");

        // Empty string
        let data = b"        ";
        assert_eq!(read_string(data, 0, 8), "");
    }

    #[test]
    fn test_write_string() {
        let mut buf = [0u8; 10];
        write_string(&mut buf, 0, "HELLO", 10);
        assert_eq!(&buf, b"HELLO     ");

        let mut buf = [0u8; 5];
        write_string(&mut buf, 0, "VERYLONGSTRING", 5);
        assert_eq!(&buf, b"VERYL");
    }

    #[test]
    fn test_build_header_record() {
        let record = build_header_record(LIBRARY_HEADER_V5);
        assert_eq!(record.len(), RECORD_LEN);
        assert!(record.starts_with(LIBRARY_HEADER_V5.as_bytes()));

        // Check '0' section
        for byte in &record[48..78] {
            assert_eq!(*byte, b'0');
        }

        // Check trailing spaces
        assert_eq!(record[78], b' ');
        assert_eq!(record[79], b' ');
    }

    #[test]
    fn test_align_to_record() {
        assert_eq!(align_to_record(0), 0);
        assert_eq!(align_to_record(1), 80);
        assert_eq!(align_to_record(80), 80);
        assert_eq!(align_to_record(81), 160);
        assert_eq!(align_to_record(140), 160);
    }

    #[test]
    fn test_records_needed() {
        assert_eq!(records_needed(0), 0);
        assert_eq!(records_needed(1), 1);
        assert_eq!(records_needed(80), 1);
        assert_eq!(records_needed(81), 2);
        assert_eq!(records_needed(160), 2);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 5), "hello");
        assert_eq!(truncate_str("", 5), "");
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("  test  "), "TEST");
        assert_eq!(normalize_name("Hello"), "HELLO");
        assert_eq!(normalize_name("AGE"), "AGE");
    }
}
