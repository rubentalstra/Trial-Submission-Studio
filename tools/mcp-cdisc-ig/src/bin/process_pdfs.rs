//! PDF Processing Tool for CDISC Implementation Guides
//!
//! Two-pass extraction:
//! 1. First pass: Identify section boundaries and their domains
//! 2. Second pass: Chunk content within sections, inheriting domain context
//!
//! Usage:
//!   cargo run --bin process_pdfs

use anyhow::{Context, Result};
use lopdf::{Document, content::Content};
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
    heading: String,
    page: u32,
    content: String,
    domain: Option<String>,
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
    start_page: u32,
    domain: Option<String>,
    content: String,
}

/// Text extracted from a single page
struct PageText {
    page: u32,
    text: String,
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
// Known Domains & Variables
// =============================================================================

/// All known CDISC domain codes (from tss-standards CSVs)
const DOMAINS: &[&str] = &[
    // SDTM Interventions
    "AG", "CM", "EC", "EX", "ML", "PR", "SU", // SDTM Events
    "AE", "BE", "CE", "DS", "DV", "HO", "MH", // SDTM Findings
    "BS", "CP", "CV", "DA", "DD", "EG", "FT", "GF", "IE", "IS", "LB", "MB", "MI", "MK", "MS", "NV",
    "OE", "PC", "PE", "PP", "QS", "RE", "RP", "RS", "SC", "SS", "TR", "TU", "UR", "VS",
    // SDTM Findings About
    "FA", "SR", // SDTM Special-Purpose
    "CO", "DM", "SE", "SM", "SV", // SDTM Trial Design
    "TA", "TD", "TE", "TI", "TM", "TS", "TV", // SDTM Study Reference
    "OI", // SDTM Relationship
    "RELREC", "RELSPEC", "RELSUB", "SUPPQUAL", // SEND-specific (not in SDTM)
    "BW", "BG", "CL", "FW", "MA", "OM", "PM", "TF", "TX", "POOLDEF", // ADaM structures
    "ADSL", "BDS", "TTE", // Common ADaM dataset names (conventional)
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

    let igs = [
        (
            "SDTMIG_v3.4.pdf",
            "SDTM Implementation Guide",
            "3.4",
            "sdtm-ig-v3.4.json",
        ),
        (
            "SENDIG_v3.1.1.pdf",
            "SEND Implementation Guide",
            "3.1.1",
            "send-ig-v3.1.1.json",
        ),
        (
            "ADaMIG_v1.3.pdf",
            "ADaM Implementation Guide",
            "1.3",
            "adam-ig-v1.3.json",
        ),
    ];

    for (pdf_name, ig_name, version, output_name) in igs {
        let pdf_path = pdf_dir.join(pdf_name);
        let output_path = data_dir.join(output_name);

        println!("Processing: {}", pdf_name);

        if !pdf_path.exists() {
            println!("  SKIP: PDF not found at {:?}", pdf_path);
            continue;
        }

        match process_ig(&pdf_path, ig_name, version) {
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

fn process_ig(path: &Path, name: &str, version: &str) -> Result<IgContent> {
    // Step 1: Extract all pages
    let pages = extract_pages(path)?;

    // Step 2: First pass - identify sections and their domains
    let sections = identify_sections(&pages);
    println!("  Pass 1: Found {} sections", sections.len());

    // Step 3: Second pass - chunk content within sections
    let chunks = chunk_sections(&sections);
    println!("  Pass 2: Created {} chunks", chunks.len());

    Ok(IgContent {
        name: name.to_string(),
        version: version.to_string(),
        chunks,
    })
}

/// Extract text from all pages of the PDF
fn extract_pages(path: &Path) -> Result<Vec<PageText>> {
    let doc = Document::load(path).with_context(|| format!("Failed to load: {:?}", path))?;

    if doc.is_encrypted() {
        anyhow::bail!("PDF is encrypted and cannot be processed");
    }

    let pages = doc.get_pages();
    println!("  Found {} pages", pages.len());

    let mut result = Vec::with_capacity(pages.len());

    for (page_num, &page_id) in pages.iter() {
        let text = extract_text_from_page(&doc, page_id).unwrap_or_default();
        result.push(PageText {
            page: *page_num,
            text,
        });

        if *page_num % 100 == 0 {
            println!("  ... extracted page {}", page_num);
        }
    }

    let total: usize = result.iter().map(|p| p.text.len()).sum();
    println!("  Extracted {} total characters", total);

    if total == 0 {
        anyhow::bail!("No text extracted - PDF may be image-only");
    }

    Ok(result)
}

/// Extract text from a single PDF page
fn extract_text_from_page(doc: &Document, page_id: lopdf::ObjectId) -> Result<String> {
    let mut text = String::new();

    let content_bytes = doc.get_page_content(page_id)?;
    let content = Content::decode(&content_bytes)?;

    for operation in &content.operations {
        match operation.operator.as_str() {
            "Tj" | "TJ" | "'" | "\"" => {
                for operand in &operation.operands {
                    if let Some(s) = extract_string_from_object(operand) {
                        text.push_str(&s);
                        text.push(' ');
                    }
                }
            }
            "Td" | "TD" | "T*" => {
                if !text.ends_with('\n') && !text.ends_with(' ') {
                    text.push('\n');
                }
            }
            _ => {}
        }
    }

    Ok(text)
}

/// Extract string content from a PDF object
fn extract_string_from_object(obj: &lopdf::Object) -> Option<String> {
    match obj {
        lopdf::Object::String(bytes, _) => {
            // UTF-16BE (BOM marker)
            if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                let utf16: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            Some(u16::from_be_bytes([chunk[0], chunk[1]]))
                        } else {
                            None
                        }
                    })
                    .collect();
                String::from_utf16(&utf16).ok()
            } else {
                // Latin-1 / PDFDocEncoding
                Some(bytes.iter().map(|&b| b as char).collect())
            }
        }
        lopdf::Object::Array(arr) => {
            let mut result = String::new();
            for item in arr {
                if let Some(s) = extract_string_from_object(item) {
                    result.push_str(&s);
                }
            }
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        }
        _ => None,
    }
}

// =============================================================================
// Pass 1: Section Identification
// =============================================================================

/// First pass: Identify all major sections and their associated domains
fn identify_sections(pages: &[PageText]) -> Vec<Section> {
    // Combine all pages into one text with page markers
    let mut full_text = String::new();
    let mut page_boundaries: Vec<(usize, u32)> = Vec::new();

    for page in pages {
        page_boundaries.push((full_text.len(), page.page));
        full_text.push_str(&page.text);
        full_text.push('\n');
    }

    // Find all section headings in the full text
    let mut section_starts: Vec<(usize, String, Option<String>)> = Vec::new();

    for caps in SECTION_HEADING.captures_iter(&full_text) {
        if let (Some(full_match), Some(number), Some(title)) =
            (caps.get(0), caps.get(1), caps.get(2))
        {
            let heading = format!("{} {}", number.as_str(), title.as_str().trim());

            // Skip if it looks like a TOC entry (ends with page number pattern)
            if heading.ends_with(char::is_numeric) && heading.contains("...") {
                continue;
            }

            let domain = extract_domain_from_heading(&heading);
            section_starts.push((full_match.start(), heading, domain));
        }
    }

    // If no sections found, create one big section
    if section_starts.is_empty() {
        let content = normalize_whitespace(&full_text);
        if !content.is_empty() {
            return vec![Section {
                heading: "Document Content".to_string(),
                start_page: 1,
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

        // Find which page this section starts on
        let start_page = page_boundaries
            .iter()
            .rev()
            .find(|(pos, _)| *pos <= *start_pos)
            .map(|(_, page)| *page)
            .unwrap_or(1);

        sections.push(Section {
            heading: heading.clone(),
            start_page,
            domain: domain.clone(),
            content,
        });
    }

    sections
}

/// Extract domain code from a heading like "Demographics Domain (DM)" or "AE - Adverse Events"
fn extract_domain_from_heading(heading: &str) -> Option<String> {
    let heading_upper = heading.to_uppercase();

    // First, try regex pattern for explicit domain mentions
    if let Some(caps) = DOMAIN_IN_HEADING.captures(heading) {
        let code = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str());
        if let Some(code) = code
            && DOMAINS.contains(&code.to_uppercase().as_str())
        {
            return Some(code.to_uppercase());
        }
    }

    // Try to find domain code as a word boundary match
    for domain in DOMAINS {
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
fn chunk_sections(sections: &[Section]) -> Vec<TextChunk> {
    let mut chunks = Vec::new();

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
            section.start_page,
            &section.content,
            section.domain.as_deref(),
        );

        chunks.extend(section_chunks);
    }

    chunks
}

/// Split a section's content into appropriately sized chunks
fn split_into_chunks(
    heading: &str,
    start_page: u32,
    content: &str,
    section_domain: Option<&str>,
) -> Vec<TextChunk> {
    let mut chunks = Vec::new();

    // If content is small enough, return as single chunk
    if content.len() <= 3000 {
        if let Some(chunk) = create_chunk(heading, start_page, content, section_domain) {
            chunks.push(chunk);
        }
        return chunks;
    }

    // Split into smaller chunks at sentence boundaries
    let mut current_chunk = String::new();
    let mut chunk_index = 0;

    for sentence in split_sentences(content) {
        // If adding this sentence would make chunk too large, finalize current chunk
        if !current_chunk.is_empty() && current_chunk.len() + sentence.len() > 2500 {
            let chunk_heading = if chunk_index > 0 {
                format!("{} (cont.)", heading)
            } else {
                heading.to_string()
            };

            if let Some(chunk) =
                create_chunk(&chunk_heading, start_page, &current_chunk, section_domain)
            {
                chunks.push(chunk);
            }

            current_chunk.clear();
            chunk_index += 1;
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(&sentence);
    }

    // Don't forget the last chunk
    if !current_chunk.is_empty() {
        let chunk_heading = if chunk_index > 0 {
            format!("{} (cont.)", heading)
        } else {
            heading.to_string()
        };

        if let Some(chunk) =
            create_chunk(&chunk_heading, start_page, &current_chunk, section_domain)
        {
            chunks.push(chunk);
        }
    }

    chunks
}

/// Create a TextChunk with domain detection
fn create_chunk(
    heading: &str,
    page: u32,
    content: &str,
    section_domain: Option<&str>,
) -> Option<TextChunk> {
    let content = clean_content(content);

    if content.len() < 50 {
        return None;
    }

    // Use section domain if available, otherwise try to detect from content
    let domain = section_domain
        .map(String::from)
        .or_else(|| detect_domain_from_content(&content));

    Some(TextChunk {
        heading: heading.to_string(),
        page,
        content,
        domain,
    })
}

/// Detect domain from content when not available from section heading
fn detect_domain_from_content(content: &str) -> Option<String> {
    let content_upper = content.to_uppercase();

    for domain in DOMAINS {
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
