//! Text utilities for SDTM compliance.
//!
//! This module provides utilities for handling text data per SDTMIG v3.4 requirements,
//! particularly Section 4.5.3 which defines rules for handling long text strings.
//!
//! # Key constraints (per SDTMIG 4.5.3):
//! - SAS V5 transport file character variable maximum length: 200 characters
//! - --TEST: 40 characters (200 for IE/TI/TS)
//! - --TESTCD: 8 characters
//! - QNAM: 8 characters
//! - QLABEL: 40 characters

/// Maximum length for SAS V5 transport file character variables.
/// Per SDTMIG 4.2.1, character variables have a max length of 200 bytes.
pub const SAS_V5_MAX_LENGTH: usize = 200;

/// Maximum length for --TEST variable (standard domains).
/// Per SDTMIG 4.5.3.1, --TEST is limited to 40 characters.
pub const TEST_MAX_LENGTH: usize = 40;

/// Maximum length for --TEST variable in exception domains (IE, TI, TS).
/// Per SDTMIG 4.5.3.1, these domains allow up to 200 characters.
pub const TEST_EXCEPTION_MAX_LENGTH: usize = 200;

/// Maximum length for --TESTCD and QNAM variables.
/// Per SDTMIG, these short codes are limited to 8 characters.
pub const TESTCD_MAX_LENGTH: usize = 8;

/// Maximum length for QLABEL variable.
/// Per SDTMIG 4.5.3.1, QLABEL is limited to 40 characters.
pub const QLABEL_MAX_LENGTH: usize = 40;

/// Result of splitting a long text value into multiple parts.
#[derive(Debug, Clone)]
pub struct TextSplitResult {
    /// The parts of the split text. First element goes to parent, rest go to SUPP records.
    pub parts: Vec<String>,
    /// The original full text length.
    pub original_length: usize,
    /// Whether the text was actually split (length > max_length).
    pub was_split: bool,
}

/// Result of splitting a long text for SUPP record generation.
#[derive(Debug, Clone)]
pub struct SuppSplitRecord {
    /// The QNAM value (e.g., "MHTERM", "MHTERM1", "MHTERM2")
    pub qnam: String,
    /// The QLABEL value (same for all parts per SDTMIG)
    pub qlabel: String,
    /// The text content for this record
    pub qval: String,
    /// Whether this is the first (original) part or a continuation
    pub is_continuation: bool,
    /// The sequence number (0 for first/original, 1+ for continuations)
    pub sequence: usize,
}

/// Split a long text string into multiple parts at word boundaries.
///
/// Per SDTMIG v3.4 Section 4.5.3.2:
/// - The first 200 characters go in the parent domain variable
/// - Each additional 200 characters go in SUPP-- records
/// - Text should be split between words to improve readability
///
/// # Arguments
/// * `text` - The text to split
/// * `max_length` - Maximum length for each part (typically 200)
///
/// # Returns
/// A `TextSplitResult` containing the split parts
///
/// # Example
/// ```
/// use sdtm_core::text_utils::split_text_at_word_boundary;
///
/// let long_text = "This is a very long text that exceeds the maximum length.";
/// let result = split_text_at_word_boundary(long_text, 30);
/// assert!(!result.parts.is_empty());
/// for part in &result.parts {
///     assert!(part.len() <= 30);
/// }
/// ```
pub fn split_text_at_word_boundary(text: &str, max_length: usize) -> TextSplitResult {
    let trimmed = text.trim();
    let original_length = trimmed.len();

    if original_length <= max_length {
        return TextSplitResult {
            parts: vec![trimmed.to_string()],
            original_length,
            was_split: false,
        };
    }

    let mut parts = Vec::new();
    let mut remaining = trimmed;

    while !remaining.is_empty() {
        if remaining.len() <= max_length {
            parts.push(remaining.to_string());
            break;
        }

        // Find the best split point at or before max_length
        let split_point = find_word_boundary_split(remaining, max_length);

        let (chunk, rest) = remaining.split_at(split_point);
        parts.push(chunk.trim_end().to_string());
        remaining = rest.trim_start();
    }

    TextSplitResult {
        parts,
        original_length,
        was_split: true,
    }
}

/// Find the best word boundary for splitting text.
///
/// Looks for whitespace or punctuation near the max_length position.
/// If no suitable boundary is found, splits at max_length exactly.
fn find_word_boundary_split(text: &str, max_length: usize) -> usize {
    if text.len() <= max_length {
        return text.len();
    }

    // Look backwards from max_length to find a word boundary
    let search_start = max_length.saturating_sub(30).max(max_length / 2);

    // First, look for whitespace (preferred split point)
    for i in (search_start..=max_length).rev() {
        if let Some(ch) = text.chars().nth(i)
            && ch.is_whitespace()
        {
            return i + 1; // Include the whitespace in the first chunk
        }
    }

    // If no whitespace, look for punctuation
    for i in (search_start..=max_length).rev() {
        if let Some(ch) = text.chars().nth(i)
            && matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | '-' | ')' | ']')
        {
            return i + 1; // Include the punctuation in the first chunk
        }
    }

    // If no suitable boundary found, split at max_length
    // But be careful not to split a UTF-8 multi-byte character
    let mut boundary = max_length;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }

    boundary
}

/// Generate QNAM values for split text parts.
///
/// Per SDTMIG v3.4 Section 4.5.3.2:
/// - First QNAM has no suffix (e.g., "MHTERM")
/// - Subsequent QNAMs have numeric suffix (e.g., "MHTERM1", "MHTERM2")
/// - If original QNAM is already 8 characters, replace last char with digit
///   (e.g., "AEACNOTH" -> "AEACNOT1", "AEACNOT2")
///
/// # Arguments
/// * `base_qnam` - The original QNAM value (e.g., "MHTERM" or "AEACNOTH")
/// * `count` - Number of QNAM values needed (including the first)
///
/// # Returns
/// A vector of QNAM values for each part
pub fn generate_split_qnams(base_qnam: &str, count: usize) -> Vec<String> {
    if count == 0 {
        return Vec::new();
    }

    let mut qnams = vec![base_qnam.to_string()];

    if count == 1 {
        return qnams;
    }

    let base_upper = base_qnam.to_uppercase();

    for i in 1..count {
        let qnam = if base_upper.len() >= 8 {
            // Per SDTMIG 4.5.3.2: If QNAM is already 8 chars, replace last char with digit
            // E.g., AEACNOTH -> AEACNOT1, AEACNOT2
            let prefix: String = base_upper.chars().take(7).collect();
            format!("{}{}", prefix, i)
        } else {
            // Append numeric suffix
            // E.g., MHTERM -> MHTERM1, MHTERM2
            format!("{}{}", base_upper, i)
        };

        // Ensure QNAM doesn't exceed 8 characters
        let truncated: String = qnam.chars().take(8).collect();
        qnams.push(truncated);
    }

    qnams
}

/// Split a long text value and generate SUPP record data.
///
/// Per SDTMIG v3.4 Section 4.5.3.2, this function:
/// 1. Splits text at word boundaries
/// 2. Generates appropriate QNAMs with suffixes
/// 3. Uses the same QLABEL for all parts (they represent one logical value)
///
/// # Arguments
/// * `text` - The text value to split
/// * `qnam` - The base QNAM value
/// * `qlabel` - The QLABEL value (same for all parts)
/// * `max_length` - Maximum length for each part (typically 200)
///
/// # Returns
/// A vector of `SuppSplitRecord` for each part
pub fn split_for_supp_records(
    text: &str,
    qnam: &str,
    qlabel: &str,
    max_length: usize,
) -> Vec<SuppSplitRecord> {
    let split_result = split_text_at_word_boundary(text, max_length);
    let qnams = generate_split_qnams(qnam, split_result.parts.len());

    split_result
        .parts
        .into_iter()
        .enumerate()
        .zip(qnams)
        .map(|((idx, part), qnam)| SuppSplitRecord {
            qnam,
            qlabel: qlabel.to_string(),
            qval: part,
            is_continuation: idx > 0,
            sequence: idx,
        })
        .collect()
}

/// Check if a --TEST value exceeds the allowed length.
///
/// # Arguments
/// * `text` - The test name value
/// * `is_exception_domain` - Whether the domain is IE, TI, or TS
pub fn exceeds_test_max(text: &str, is_exception_domain: bool) -> bool {
    let max_len = if is_exception_domain {
        TEST_EXCEPTION_MAX_LENGTH
    } else {
        TEST_MAX_LENGTH
    };
    text.trim().len() > max_len
}

/// Truncate text to the specified maximum length at a word boundary if possible.
///
/// Used for variables where truncation is the recommended approach per SDTMIG.
pub fn truncate_at_word_boundary(text: &str, max_length: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_length {
        return trimmed.to_string();
    }

    let split_point = find_word_boundary_split(trimmed, max_length);
    trimmed[..split_point].trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_short_text() {
        let result = split_text_at_word_boundary("Hello world", 200);
        assert!(!result.was_split);
        assert_eq!(result.parts.len(), 1);
        assert_eq!(result.parts[0], "Hello world");
    }

    #[test]
    fn test_split_long_text_at_word_boundary() {
        let long_text = "This is a test sentence that is longer than twenty characters and needs to be split at word boundaries for proper SDTM compliance.";
        let result = split_text_at_word_boundary(long_text, 50);

        assert!(result.was_split);
        assert!(result.parts.len() > 1);

        // Each part should be <= 50 characters
        for part in &result.parts {
            assert!(
                part.len() <= 50,
                "Part '{}' exceeds max length ({})",
                part,
                part.len()
            );
        }

        // Joined parts should reconstruct original (with possible spacing differences)
        let rejoined: String = result.parts.join(" ");
        // The rejoined text should contain all words
        assert!(rejoined.contains("SDTM"));
        assert!(rejoined.contains("compliance"));
    }

    #[test]
    fn test_generate_split_qnams_short_base() {
        let qnams = generate_split_qnams("MHTERM", 3);
        assert_eq!(qnams.len(), 3);
        assert_eq!(qnams[0], "MHTERM");
        assert_eq!(qnams[1], "MHTERM1");
        assert_eq!(qnams[2], "MHTERM2");
    }

    #[test]
    fn test_generate_split_qnams_8char_base() {
        // Per SDTMIG 4.5.3.2: If QNAM is 8 chars, replace last char with digit
        let qnams = generate_split_qnams("AEACNOTH", 4);
        assert_eq!(qnams.len(), 4);
        assert_eq!(qnams[0], "AEACNOTH");
        assert_eq!(qnams[1], "AEACNOT1");
        assert_eq!(qnams[2], "AEACNOT2");
        assert_eq!(qnams[3], "AEACNOT3");
    }

    #[test]
    fn test_split_for_supp_records() {
        let long_text = "This is a very long medical history term that exceeds 200 characters and needs to be split into multiple SUPPQUAL records according to SDTMIG Section 4.5.3.2 requirements for proper data submission compliance with SAS V5 transport file format limitations.";

        let records = split_for_supp_records(long_text, "MHTERM", "Medical History Term", 100);

        assert!(records.len() > 1);

        // First record is not a continuation
        assert!(!records[0].is_continuation);
        assert_eq!(records[0].sequence, 0);
        assert_eq!(records[0].qnam, "MHTERM");

        // Subsequent records are continuations
        for (idx, record) in records.iter().enumerate().skip(1) {
            assert!(record.is_continuation);
            assert_eq!(record.sequence, idx);
            assert_eq!(record.qlabel, "Medical History Term");
        }
    }

    #[test]
    fn test_exceeds_test_max() {
        let short_text = "Blood Pressure";
        let long_text = "This is a very long test name that definitely exceeds forty characters";

        // Standard domain
        assert!(!exceeds_test_max(short_text, false));
        assert!(exceeds_test_max(long_text, false));

        // Exception domain (IE, TI, TS) - allows up to 200
        assert!(!exceeds_test_max(long_text, true));
    }

    #[test]
    fn test_truncate_at_word_boundary() {
        let text = "This is a test sentence";
        let truncated = truncate_at_word_boundary(text, 15);

        assert!(truncated.len() <= 15);
        // Should truncate at a word boundary
        assert!(!truncated.ends_with("sen")); // Not mid-word
    }
}
