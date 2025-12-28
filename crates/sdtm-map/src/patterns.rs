use std::collections::BTreeMap;

use sdtm_model::Domain;

use crate::utils::normalize_text;

/// Common synonyms for SDTM variable names.
/// Maps normalized pattern -> list of potential variable name suffixes.
const VARIABLE_SYNONYMS: &[(&str, &[&str])] = &[
    // Subject identifiers
    ("subject", &["USUBJID", "SUBJID"]),
    ("subject id", &["USUBJID", "SUBJID"]),
    ("subject identifier", &["USUBJID", "SUBJID"]),
    ("unique subject", &["USUBJID"]),
    ("patient", &["USUBJID", "SUBJID"]),
    ("patient id", &["USUBJID", "SUBJID"]),
    // Study identifiers
    ("study", &["STUDYID"]),
    ("study id", &["STUDYID"]),
    ("protocol", &["STUDYID"]),
    // Demographics
    ("age", &["AGE"]),
    ("age in years", &["AGE"]),
    ("sex", &["SEX"]),
    ("gender", &["SEX"]),
    ("race", &["RACE"]),
    ("ethnic", &["ETHNIC"]),
    ("ethnicity", &["ETHNIC"]),
    ("country", &["COUNTRY"]),
    ("birth date", &["BRTHDTC"]),
    ("date of birth", &["BRTHDTC"]),
    // Dates/Times
    ("start date", &["STDTC", "--STDTC"]),
    ("end date", &["ENDTC", "--ENDTC"]),
    ("date time", &["DTC", "--DTC"]),
    ("collection date", &["DTC", "--DTC"]),
    ("reference start", &["RFSTDTC"]),
    ("reference end", &["RFENDTC"]),
    // Visit
    ("visit", &["VISIT"]),
    ("visit name", &["VISIT"]),
    ("visit number", &["VISITNUM"]),
    ("visit day", &["VISITDY"]),
    // Results
    ("result", &["ORRES", "--ORRES"]),
    ("original result", &["ORRES", "--ORRES"]),
    (
        "standard result",
        &["STRESC", "STRESN", "--STRESC", "--STRESN"],
    ),
    ("numeric result", &["STRESN", "--STRESN"]),
    ("character result", &["STRESC", "--STRESC"]),
    ("unit", &["ORRESU", "STRESU", "--ORRESU", "--STRESU"]),
    // Test
    ("test", &["TEST", "--TEST"]),
    ("test name", &["TEST", "--TEST"]),
    ("test code", &["TESTCD", "--TESTCD"]),
    // Category
    ("category", &["CAT", "--CAT"]),
    ("subcategory", &["SCAT", "--SCAT"]),
    // Status
    ("status", &["STAT", "--STAT"]),
    ("reason not done", &["REASND", "--REASND"]),
    // Comments
    ("comment", &["COVAL", "--COM"]),
    ("note", &["COVAL", "--COM"]),
];

/// Build variable patterns including synonyms and label-based patterns.
pub fn build_variable_patterns(domain: &Domain) -> BTreeMap<String, Vec<String>> {
    let mut patterns = BTreeMap::new();
    let domain_prefix = domain.code.trim().to_uppercase();

    for variable in &domain.variables {
        let name = variable.name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let mut values = Vec::new();

        // Add normalized variable name
        values.push(normalize_text(&name));

        // Handle prefix patterns (e.g., AESEQ -> SEQ)
        let name_upper = name.to_uppercase();
        if name_upper.starts_with("--") {
            values.push(normalize_text(&name_upper.replace("--", "")));
        }
        if name_upper.starts_with(&domain_prefix) {
            let suffix = &name_upper[domain_prefix.len()..];
            if !suffix.is_empty() {
                values.push(normalize_text(suffix));
            }
        }

        // Add patterns from variable label
        if let Some(label) = &variable.label {
            let normalized_label = normalize_text(label);
            values.push(normalized_label.clone());

            // Add individual label words as patterns
            for word in normalized_label.split_whitespace() {
                if word.len() > 2 && !is_stopword(word) {
                    values.push(word.to_string());
                }
            }
        }

        patterns.insert(name, values);
    }

    patterns
}

/// Build a synonym lookup map for a domain.
///
/// Returns a map from normalized column names/labels to potential target variables.
pub fn build_synonym_map(domain: &Domain) -> BTreeMap<String, Vec<String>> {
    let mut synonyms = BTreeMap::new();
    let domain_prefix = domain.code.trim().to_uppercase();

    // Add standard synonyms, filtering to those relevant for this domain
    for (pattern, targets) in VARIABLE_SYNONYMS {
        let relevant_targets: Vec<String> = targets
            .iter()
            .filter_map(|t| {
                let target = t.replace("--", &domain_prefix);
                if domain
                    .variables
                    .iter()
                    .any(|v| v.name.eq_ignore_ascii_case(&target))
                {
                    Some(target)
                } else {
                    None
                }
            })
            .collect();

        if !relevant_targets.is_empty() {
            synonyms.insert(pattern.to_string(), relevant_targets);
        }
    }

    // Add label-based synonyms from the domain's variable metadata
    for variable in &domain.variables {
        if let Some(label) = &variable.label {
            let normalized_label = normalize_text(label);

            // Map full label to variable
            synonyms
                .entry(normalized_label.clone())
                .or_insert_with(Vec::new)
                .push(variable.name.clone());

            // Map significant label fragments
            for fragment in significant_fragments(&normalized_label) {
                synonyms
                    .entry(fragment)
                    .or_insert_with(Vec::new)
                    .push(variable.name.clone());
            }
        }
    }

    synonyms
}

/// Extract significant fragments from a label for pattern matching.
fn significant_fragments(label: &str) -> Vec<String> {
    let mut fragments = Vec::new();
    let words: Vec<&str> = label.split_whitespace().collect();

    // Single significant words
    for word in &words {
        if word.len() > 3 && !is_stopword(word) {
            fragments.push(word.to_string());
        }
    }

    // Two-word combinations
    for window in words.windows(2) {
        let combined = format!("{} {}", window[0], window[1]);
        if !is_stopword(window[0]) || !is_stopword(window[1]) {
            fragments.push(combined);
        }
    }

    fragments
}

/// Check if a word is a common stopword that should be ignored.
fn is_stopword(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "a" | "an"
            | "the"
            | "of"
            | "in"
            | "for"
            | "to"
            | "and"
            | "or"
            | "is"
            | "are"
            | "was"
            | "be"
            | "as"
            | "at"
            | "by"
            | "on"
            | "from"
            | "with"
    )
}

/// Match a column name/label against synonyms and return potential target variables.
pub fn match_synonyms(
    column: &str,
    label: Option<&str>,
    synonyms: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut matches = Vec::new();

    // Try column name
    let normalized_column = normalize_text(column);
    if let Some(targets) = synonyms.get(&normalized_column) {
        matches.extend(targets.clone());
    }

    // Try label
    if let Some(label) = label {
        let normalized_label = normalize_text(label);
        if let Some(targets) = synonyms.get(&normalized_label) {
            matches.extend(targets.clone());
        }

        // Try label fragments
        for fragment in significant_fragments(&normalized_label) {
            if let Some(targets) = synonyms.get(&fragment) {
                matches.extend(targets.clone());
            }
        }
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    matches.retain(|x| seen.insert(x.clone()));
    matches
}
