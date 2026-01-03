//! CSV header parsing and normalization.

/// Result of CSV header analysis.
#[derive(Debug, Clone)]
pub struct CsvHeaders {
    /// Normalized column names (trimmed, no empty).
    pub columns: Vec<String>,
    /// Optional labels if double-header was detected.
    pub labels: Option<Vec<String>>,
    /// Number of rows to skip before data (1 for single header, 2 for double).
    pub skip_rows: usize,
}

impl CsvHeaders {
    /// Creates a new CsvHeaders with single header (no labels).
    pub fn single(columns: Vec<String>) -> Self {
        Self {
            columns,
            labels: None,
            skip_rows: 1,
        }
    }

    /// Creates a new CsvHeaders with double header (labels + columns).
    pub fn double(labels: Vec<String>, columns: Vec<String>) -> Self {
        Self {
            columns,
            labels: Some(labels),
            skip_rows: 2,
        }
    }

    /// Returns the number of columns.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if there are no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns the label for a column if available.
    pub fn label_for(&self, column: &str) -> Option<&str> {
        let idx = self.columns.iter().position(|c| c == column)?;
        self.labels
            .as_ref()
            .and_then(|labels| labels.get(idx).map(String::as_str))
    }
}

/// Normalizes a header value by trimming whitespace.
pub fn normalize_header(value: &str) -> String {
    value.trim().to_string()
}

/// Parses a CSV line into fields, handling quoted values.
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                // Check for escaped quote ("")
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(normalize_header(&current));
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Don't forget the last field
    fields.push(normalize_header(&current));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_headers_single() {
        let headers = CsvHeaders::single(vec!["A".to_string(), "B".to_string()]);
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.skip_rows, 1);
        assert!(headers.labels.is_none());
    }

    #[test]
    fn test_csv_headers_double() {
        let headers = CsvHeaders::double(
            vec!["Label A".to_string(), "Label B".to_string()],
            vec!["A".to_string(), "B".to_string()],
        );
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.skip_rows, 2);
        assert!(headers.labels.is_some());
        assert_eq!(headers.label_for("A"), Some("Label A"));
        assert_eq!(headers.label_for("C"), None); // Not found
    }

    #[test]
    fn test_csv_headers_single_no_labels() {
        let headers = CsvHeaders::single(vec!["A".to_string(), "B".to_string()]);
        assert_eq!(headers.label_for("A"), None); // No labels in single-header mode
    }

    #[test]
    fn test_normalize_header() {
        assert_eq!(normalize_header("  hello  "), "hello");
        assert_eq!(normalize_header("hello"), "hello");
    }

    #[test]
    fn test_parse_csv_line_simple() {
        let result = parse_csv_line("a,b,c");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_csv_line_quoted() {
        let result = parse_csv_line("\"hello, world\",b,c");
        assert_eq!(result, vec!["hello, world", "b", "c"]);
    }

    #[test]
    fn test_parse_csv_line_escaped_quotes() {
        let result = parse_csv_line("\"he said \"\"hello\"\"\",b");
        assert_eq!(result, vec!["he said \"hello\"", "b"]);
    }

    #[test]
    fn test_parse_csv_line_trimmed() {
        let result = parse_csv_line("  a  ,  b  ");
        assert_eq!(result, vec!["a", "b"]);
    }
}
