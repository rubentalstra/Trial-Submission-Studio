//! PDF Processing Tool for CDISC Implementation Guides
//!
//! Two-pass extraction:
//! 1. First pass: Identify section boundaries and their domains
//! 2. Second pass: Chunk content within sections, inheriting domain context
//!
//! Usage:
//!   cargo run --bin process_pdfs

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// =============================================================================
// Data Structures
// =============================================================================

/// A chunk of text from the IG document
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextChunk {
    /// Unique index for this chunk
    index: usize,
    /// Section heading this chunk belongs to
    heading: String,
    /// The text content
    content: String,
    /// Domain code if applicable
    domain: Option<String>,
    /// Parent chunk index (for continuation chunks split from a larger section)
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_index: Option<usize>,
}

/// Content from a single Implementation Guide
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IgContent {
    name: String,
    version: String,
    chunks: Vec<TextChunk>,
}

/// A detected section in the document
#[derive(Debug, Clone)]
struct Section {
    heading: String,
    domain: Option<String>,
    content: String,
}

// =============================================================================
// Static Patterns (compiled once)
// =============================================================================

/// Pattern for numbered section headings: "6.1.2 Section Title"
/// Looks for section numbers followed by title text
static SECTION_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(\d+(?:\.\d+)+)\s+([A-Z][A-Za-z].{5,80})").unwrap());

/// Pattern for domain mentions in headings: "Demographics Domain (DM)" or "DM - Demographics"
static DOMAIN_IN_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([A-Z]{2,8})\s*(?:[-–—]|Domain|Dataset)\b|\(([A-Z]{2,8})\)").unwrap()
});

/// Whitespace normalization
static WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

// =============================================================================
// Known Domains by IG (only officially supported domains per standard)
// =============================================================================

/// SDTM-IG v3.4 domains (human clinical trials)
const SDTM_DOMAINS: &[&str] = &[
    // Interventions
    "AG", "CM", "EC", "EX", "ML", "PR", "SU", // Events
    "AE", "CE", "DS", "DV", "HO", "MH", // Findings
    "BE", "BS", "CP", "CV", "DA", "DD", "EG", "FT", "GF", "IE", "IS", "LB", "MB", "MI", "MK", "MS",
    "NV", "OE", "PC", "PE", "PP", "QS", "RE", "RP", "RS", "SC", "SS", "TR", "TU", "UR", "VS",
    // Findings About
    "FA", "SR", // Special-Purpose
    "CO", "DM", "SE", "SM", "SV", // Trial Design
    "TA", "TD", "TE", "TI", "TM", "TS", "TV", // Study Reference
    "OI", // Relationship
    "RELREC", "RELSPEC", "RELSUB", "SUPPQUAL",
];

/// SEND-IG v3.1.1 domains (nonclinical/animal studies)
const SEND_DOMAINS: &[&str] = &[
    // SEND-specific domains
    "BG", "BW", "CL", "FW", "LB", "MA", "MI", "OM", "PC", "PM", "PP", "SC", "TF", "TX",
    // Shared with SDTM (applicable to nonclinical)
    "CO", "DM", "DS", "EG", "EX", "SE", "SV", "TA", "TD", "TE", "TI", "TS", "TV",
    // Special structures
    "POOLDEF", "RELREC", "SUPPQUAL",
];

/// ADaM-IG v1.3 structures and common dataset names
const ADAM_DOMAINS: &[&str] = &[
    // ADaM data structures (not domains, but structure types)
    "ADSL", "BDS", "OCCDS", // Common ADaM dataset naming patterns
    "ADAE", "ADCM", "ADEX", "ADLB", "ADPC", "ADPP", "ADTTE", "ADVS",
];

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() -> Result<()> {
    println!("CDISC IG PDF Processor (Two-Pass Section-First)");
    println!("================================================\n");

    let base_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pdf_dir = base_path.join("pdfs");
    let data_dir = base_path.join("data");

    fs::create_dir_all(&data_dir)?;

    // Each entry: (pdf_name, ig_name, version, output_name, domains)
    let igs: &[(&str, &str, &str, &str, &[&str])] = &[
        // Commented out for faster dev iteration - ADaM is only 88 pages
        // (
        //     "SDTMIG_v3.4.pdf",
        //     "SDTM Implementation Guide",
        //     "3.4",
        //     "sdtm-ig-v3.4.json",
        //     SDTM_DOMAINS,
        // ),
        // (
        //     "SENDIG_v3.1.1.pdf",
        //     "SEND Implementation Guide",
        //     "3.1.1",
        //     "send-ig-v3.1.1.json",
        //     SEND_DOMAINS,
        // ),
        (
            "ADaMIG_v1.3.pdf",
            "ADaM Implementation Guide",
            "1.3",
            "adam-ig-v1.3.json",
            ADAM_DOMAINS,
        ),
    ];

    for (pdf_name, ig_name, version, output_name, domains) in igs {
        let pdf_path = pdf_dir.join(pdf_name);
        let output_path = data_dir.join(output_name);

        println!("Processing: {}", pdf_name);

        if !pdf_path.exists() {
            println!("  SKIP: PDF not found at {:?}", pdf_path);
            continue;
        }

        match process_ig(&pdf_path, ig_name, version, domains) {
            Ok(content) => {
                let chunk_count = content.chunks.len();
                let domains_found: std::collections::HashSet<_> = content
                    .chunks
                    .iter()
                    .filter_map(|c| c.domain.as_ref())
                    .collect();
                let total_chars: usize = content.chunks.iter().map(|c| c.content.len()).sum();

                let json = serde_json::to_string_pretty(&content)?;
                fs::write(&output_path, &json)?;

                println!("  -> {} chunks, {} chars", chunk_count, total_chars);
                println!(
                    "  -> {} unique domains: {:?}",
                    domains_found.len(),
                    domains_found
                );
                println!("  -> Saved to {:?}", output_path);
            }
            Err(e) => {
                println!("  ERROR: {:#}", e);
            }
        }
        println!();
    }

    println!("Done!");
    Ok(())
}

// =============================================================================
// Two-Pass Processing
// =============================================================================

fn process_ig(path: &Path, name: &str, version: &str, domains: &[&str]) -> Result<IgContent> {
    // Step 1: Extract all text from PDF
    let text = extract_text(path)?;

    // Step 2: First pass - identify sections and their domains
    let sections = identify_sections(&text, domains);
    println!("  Pass 1: Found {} sections", sections.len());

    // Step 3: Second pass - chunk content within sections
    let chunks = chunk_sections(&sections, domains);
    println!("  Pass 2: Created {} chunks", chunks.len());

    Ok(IgContent {
        name: name.to_string(),
        version: version.to_string(),
        chunks,
    })
}

/// Extract text from the PDF using pdf-extract
/// which properly handles font encodings and ToUnicode maps
fn extract_text(path: &Path) -> Result<String> {
    let text = pdf_extract::extract_text(path)
        .with_context(|| format!("Failed to extract text from {:?}", path))?;

    println!("  Extracted {} total characters", text.len());

    if text.is_empty() {
        anyhow::bail!("No text extracted - PDF may be image-only");
    }

    Ok(text)
}

// =============================================================================
// Pass 1: Section Identification
// =============================================================================

/// First pass: Identify all major sections and their associated domains
fn identify_sections(full_text: &str, domains: &[&str]) -> Vec<Section> {
    // Find all section headings in the full text
    let mut section_starts: Vec<(usize, String, Option<String>)> = Vec::new();

    for caps in SECTION_HEADING.captures_iter(full_text) {
        if let (Some(full_match), Some(number), Some(title)) =
            (caps.get(0), caps.get(1), caps.get(2))
        {
            let heading = format!("{} {}", number.as_str(), title.as_str().trim());

            // Skip if it looks like a TOC entry (ends with page number pattern)
            if heading.ends_with(char::is_numeric) && heading.contains("...") {
                continue;
            }

            let domain = extract_domain_from_heading(&heading, domains);
            section_starts.push((full_match.start(), heading, domain));
        }
    }

    // If no sections found, create one big section
    if section_starts.is_empty() {
        let content = normalize_whitespace(full_text);
        if !content.is_empty() {
            return vec![Section {
                heading: "Document Content".to_string(),
                domain: None,
                content,
            }];
        }
        return Vec::new();
    }

    // Build sections from the boundaries
    let mut sections = Vec::new();

    for (i, (start_pos, heading, domain)) in section_starts.iter().enumerate() {
        let end_pos = section_starts
            .get(i + 1)
            .map(|(pos, _, _)| *pos)
            .unwrap_or(full_text.len());

        let content = &full_text[*start_pos..end_pos];
        let content = normalize_whitespace(content);

        // Skip very short or noise sections
        if content.len() < 100 || is_toc(&content) {
            continue;
        }

        sections.push(Section {
            heading: heading.clone(),
            domain: domain.clone(),
            content,
        });
    }

    sections
}

/// Extract domain code from a heading like "Demographics Domain (DM)" or "AE - Adverse Events"
fn extract_domain_from_heading(heading: &str, domains: &[&str]) -> Option<String> {
    let heading_upper = heading.to_uppercase();

    // First, try regex pattern for explicit domain mentions
    if let Some(caps) = DOMAIN_IN_HEADING.captures(heading) {
        let code = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str());
        if let Some(code) = code
            && domains.contains(&code.to_uppercase().as_str())
        {
            return Some(code.to_uppercase());
        }
    }

    // Try to find domain code as a word boundary match
    for domain in domains {
        // Match patterns like "DM Domain", "The DM", "DM -", "DM–", "(DM)"
        let patterns = [
            format!(r"\b{}\s+(?:Domain|Dataset)", domain),
            format!(r"\b{}\s*[-–—]", domain),
            format!(r"\({}\)", domain),
            format!(r"^\d+(?:\.\d+)*\s+{}\b", domain), // "6.1 DM ..."
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern))
                && re.is_match(&heading_upper)
            {
                return Some(domain.to_string());
            }
        }
    }

    None
}

// =============================================================================
// Pass 2: Chunking Within Sections
// =============================================================================

/// Second pass: Chunk the content within each section
fn chunk_sections(sections: &[Section], domains: &[&str]) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let mut next_index: usize = 0;

    for section in sections {
        // Skip very short sections
        if section.content.len() < 100 {
            continue;
        }

        // Skip table of contents and similar
        if is_toc(&section.content) || is_toc(&section.heading) {
            continue;
        }

        // Split section content into chunks of reasonable size
        let section_chunks = split_into_chunks(
            &section.heading,
            &section.content,
            section.domain.as_deref(),
            domains,
            &mut next_index,
        );

        chunks.extend(section_chunks);
    }

    chunks
}

/// Split a section's content into appropriately sized chunks
fn split_into_chunks(
    heading: &str,
    content: &str,
    section_domain: Option<&str>,
    domains: &[&str],
    next_index: &mut usize,
) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let mut section_parent: Option<usize> = None; // Track first chunk of this section

    // If content is small enough, return as single chunk
    if content.len() <= 3000 {
        if let Some(chunk) =
            create_chunk(heading, content, section_domain, domains, *next_index, None)
        {
            *next_index += 1;
            chunks.push(chunk);
        }
        return chunks;
    }

    // Split into smaller chunks at sentence boundaries
    let mut current_chunk = String::new();

    for sentence in split_sentences(content) {
        // If adding this sentence would make chunk too large, finalize current chunk
        if !current_chunk.is_empty() && current_chunk.len() + sentence.len() > 2500 {
            if let Some(chunk) = create_chunk(
                heading,
                &current_chunk,
                section_domain,
                domains,
                *next_index,
                section_parent,
            ) {
                // First chunk of section becomes the parent for subsequent chunks
                if section_parent.is_none() {
                    section_parent = Some(*next_index);
                }
                *next_index += 1;
                chunks.push(chunk);
            }
            current_chunk.clear();
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(&sentence);
    }

    // Don't forget the last chunk
    if !current_chunk.is_empty() {
        if let Some(chunk) = create_chunk(
            heading,
            &current_chunk,
            section_domain,
            domains,
            *next_index,
            section_parent,
        ) {
            if section_parent.is_none() {
                // This is the only chunk, no parent needed
            }
            *next_index += 1;
            chunks.push(chunk);
        }
    }

    chunks
}

/// Create a TextChunk with domain detection
fn create_chunk(
    heading: &str,
    content: &str,
    section_domain: Option<&str>,
    domains: &[&str],
    index: usize,
    parent_index: Option<usize>,
) -> Option<TextChunk> {
    let content = clean_content(content);

    if content.len() < 50 {
        return None;
    }

    // Use section domain if available, otherwise try to detect from content
    let domain = section_domain
        .map(String::from)
        .or_else(|| detect_domain_from_content(&content, domains));

    Some(TextChunk {
        index,
        heading: heading.to_string(),
        content,
        domain,
        parent_index,
    })
}

/// Detect domain from content when not available from section heading
fn detect_domain_from_content(content: &str, domains: &[&str]) -> Option<String> {
    let content_upper = content.to_uppercase();

    for domain in domains {
        let patterns = [
            format!("{} DOMAIN", domain),
            format!("{} DATASET", domain),
            format!("THE {} ", domain),
        ];

        for pattern in &patterns {
            if content_upper.contains(pattern) {
                return Some(domain.to_string());
            }
        }
    }

    None
}

// =============================================================================
// Helper Functions
// =============================================================================

fn split_sentences(text: &str) -> Vec<String> {
    // Split on sentence boundaries: period/question/exclamation followed by space
    let mut sentences = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        current.push(ch);

        // Check for sentence boundary
        let is_sentence_end = (ch == '.' || ch == '?' || ch == '!')
            && chars.get(i + 1).is_some_and(|&c| c == ' ' || c == '\n');

        if is_sentence_end && current.len() > 20 {
            sentences.push(current.trim().to_string());
            current = String::new();
        }

        // Force split if current gets too long (fallback for bad text)
        if current.len() > 500 {
            // Try to find a good break point
            if let Some(break_pos) = current.rfind([' ', '\n']) {
                let (left, right) = current.split_at(break_pos);
                sentences.push(left.trim().to_string());
                current = right.trim().to_string();
            }
        }
    }

    // Add remainder
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    if sentences.is_empty() {
        vec![text.to_string()]
    } else {
        sentences
    }
}

fn normalize_whitespace(text: &str) -> String {
    WHITESPACE.replace_all(text.trim(), " ").to_string()
}

fn clean_content(text: &str) -> String {
    let text = text.replace('\u{0000}', "");
    normalize_whitespace(&text)
}

fn is_toc(content: &str) -> bool {
    let lower = content.to_lowercase();

    // Explicit TOC indicators
    if lower.contains("table of contents")
        || lower.contains("list of tables")
        || lower.contains("list of figures")
    {
        return true;
    }

    // Dot leaders pattern (common in TOC)
    if content.contains(".....") {
        return true;
    }

    // High ratio of dots to words
    let dot_count = content.matches('.').count();
    let word_count = content.split_whitespace().count();
    if word_count > 10 && (dot_count as f32 / word_count as f32) > 0.5 {
        return true;
    }

    // Many lines ending in numbers (page references)
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 5 {
        let lines_ending_in_number = lines
            .iter()
            .filter(|l| l.trim().chars().last().is_some_and(|c| c.is_ascii_digit()))
            .count();
        if (lines_ending_in_number as f32 / lines.len() as f32) > 0.5 {
            return true;
        }
    }

    false
}
