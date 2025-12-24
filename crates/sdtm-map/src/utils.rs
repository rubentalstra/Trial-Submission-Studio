pub fn normalize_text(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .replace(['_', '-', '.', '/', '\\'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn safe_column_name(raw: &str) -> String {
    raw.trim().to_string()
}
