//! Tests for data utility functions.

use sdtm_transform::{sanitize_qnam, sanitize_test_code, strip_all_quotes, strip_quotes};

#[test]
fn strip_quotes_removes_wrapping_quotes() {
    assert_eq!(strip_quotes("\"hello\""), "hello");
    assert_eq!(strip_quotes("\"world\""), "world");
}

#[test]
fn strip_quotes_trims_whitespace() {
    assert_eq!(strip_quotes("  \"hello\"  "), "hello");
    assert_eq!(strip_quotes("  unquoted  "), "unquoted");
}

#[test]
fn strip_quotes_leaves_partial_quotes() {
    assert_eq!(strip_quotes("\"partial"), "\"partial");
    assert_eq!(strip_quotes("partial\""), "partial\"");
}

#[test]
fn strip_quotes_handles_unquoted() {
    assert_eq!(strip_quotes("unquoted"), "unquoted");
    assert_eq!(strip_quotes(""), "");
}

#[test]
fn strip_all_quotes_removes_all_quotes() {
    assert_eq!(strip_all_quotes("\"hello\""), "hello");
    assert_eq!(strip_all_quotes("he\"llo"), "hello");
    assert_eq!(strip_all_quotes("\"a\"b\"c\""), "abc");
}

#[test]
fn strip_all_quotes_handles_no_quotes() {
    assert_eq!(strip_all_quotes("unquoted"), "unquoted");
    assert_eq!(strip_all_quotes("  trimmed  "), "trimmed");
}

#[test]
fn sanitize_test_code_uppercase_alphanumeric() {
    assert_eq!(sanitize_test_code("weight"), "WEIGHT");
    assert_eq!(sanitize_test_code("SYSBP"), "SYSBP");
    assert_eq!(sanitize_test_code("wt-kg"), "WT_KG"); // 5 chars, fits
    assert_eq!(sanitize_test_code("weight-kg"), "WEIGHT_K"); // 9 chars truncated to 8
}

#[test]
fn sanitize_test_code_truncates_to_8_chars() {
    assert_eq!(sanitize_test_code("verylongname"), "VERYLONG");
}

#[test]
fn sanitize_test_code_prefixes_if_starts_with_digit() {
    assert_eq!(sanitize_test_code("123test"), "T123TEST");
}

#[test]
fn sanitize_test_code_fallback_for_empty() {
    assert_eq!(sanitize_test_code(""), "TEST");
    assert_eq!(sanitize_test_code("---"), "TEST");
}

#[test]
fn sanitize_qnam_uppercase_alphanumeric() {
    assert_eq!(sanitize_qnam("custom"), "CUSTOM");
    assert_eq!(sanitize_qnam("my-value"), "MY_VALUE");
}

#[test]
fn sanitize_qnam_collapses_underscores() {
    assert_eq!(sanitize_qnam("a--b"), "A_B");
    assert_eq!(sanitize_qnam("a---b"), "A_B");
}

#[test]
fn sanitize_qnam_prefixes_if_starts_with_digit() {
    assert_eq!(sanitize_qnam("123value"), "Q123VALU");
}

#[test]
fn sanitize_qnam_fallback_for_empty() {
    assert_eq!(sanitize_qnam(""), "QVAL");
    assert_eq!(sanitize_qnam("---"), "QVAL");
}
