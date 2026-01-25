//! PDF Processing Tool for CDISC Implementation Guides
//!
//! Extracts text from the IG PDFs using lopdf and creates searchable JSON indexes.
//!
//! Usage:
//!   cargo run --bin process_pdfs

use anyhow::{Context, Result};
use lopdf::{Document, content::Content};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A chunk of text from the IG document
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextChunk {
    heading: String,
    page: u32,
    content: String,
    domain: Option<String>,
    variable: Option<String>,
}

/// Content from a single Implementation Guide
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IgContent {
    name: String,
    version: String,
    chunks: Vec<TextChunk>,
}

/// Known SDTM/SEND/ADaM domain codes for tagging
const DOMAINS: &[&str] = &[
    "DM", "AE", "CE", "DS", "DV", "MH", "CM", "EC", "EX", "SU", "PR", "DA", "DD", "EG", "IE", "LB",
    "MB", "MI", "MO", "MS", "OM", "PC", "PE", "PP", "QS", "RE", "RP", "RS", "SC", "SE", "SM", "SS",
    "TR", "TU", "TV", "VS", "BW", "BG", "CL", "CO", "FW", "MA", "PM", "TF", "ADSL", "ADAE", "ADLB",
    "ADVS", "ADCM", "ADEX", "ADTTE", "RELREC", "SUPPQUAL", "TA", "TE", "TI", "TS",
];

/// Common SDTM variable name patterns
const VARIABLE_PATTERNS: &[&str] = &[
    "USUBJID", "STUDYID", "DOMAIN", "SUBJID", "SITEID", "RFSTDTC", "RFENDTC", "RFXSTDTC",
    "RFXENDTC", "RFICDTC", "RFPENDTC", "DTHDTC", "DTHFL", "ARMCD", "ARM", "ACTARMCD", "ACTARM",
    "COUNTRY", "BRTHDTC", "AGE", "AGEU", "SEX", "RACE", "ETHNIC", "SPECIES", "STRAIN", "SBSTRAIN",
    "AESEQ", "AETERM", "AEDECOD", "AEBODSYS", "AESEV", "AESER", "AEREL", "AEOUT", "AESTDTC",
    "AEENDTC", "VISIT", "VISITNUM", "VISITDY", "EPOCH",
];

fn main() -> Result<()> {
    println!("CDISC IG PDF Processor");
    println!("======================\n");

    let base_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pdf_dir = base_path.join("pdfs");
    let data_dir = base_path.join("data");

    // Ensure data directory exists
    fs::create_dir_all(&data_dir)?;

    // Process each IG
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
            println!("  WARNING: PDF not found at {:?}, skipping", pdf_path);
            continue;
        }

        match process_pdf(&pdf_path, ig_name, version) {
            Ok(content) => {
                let chunk_count = content.chunks.len();
                let total_chars: usize = content.chunks.iter().map(|c| c.content.len()).sum();
                let json = serde_json::to_string_pretty(&content)?;
                fs::write(&output_path, json)?;
                println!(
                    "  Extracted {} chunks ({} chars) -> {:?}",
                    chunk_count, total_chars, output_path
                );
            }
            Err(e) => {
                println!("  ERROR: {:#}", e);
            }
        }
    }

    println!("\nDone!");
    Ok(())
}

fn process_pdf(path: &Path, name: &str, version: &str) -> Result<IgContent> {
    println!("  Loading PDF with lopdf 0.39...");

    // Load the PDF document
    let doc = Document::load(path).with_context(|| format!("Failed to load PDF: {:?}", path))?;

    // Check if encrypted and try to decrypt with empty password
    if doc.is_encrypted() {
        println!("  PDF is encrypted, attempting to decrypt...");
        // lopdf 0.39 should auto-decrypt with empty password on load
        // If still encrypted, the content is not accessible
        if doc.is_encrypted() {
            anyhow::bail!(
                "PDF is encrypted and could not be decrypted. \
                 Content copying may not be allowed."
            );
        }
    }

    // Get page count
    let pages = doc.get_pages();
    let page_count = pages.len();
    println!("  Found {} pages", page_count);

    // Extract text from each page
    let mut all_pages_text: Vec<String> = Vec::with_capacity(page_count);

    for (page_num, &page_id) in pages.iter() {
        let page_text = extract_text_from_page(&doc, page_id).unwrap_or_default();
        if !page_text.is_empty() {
            all_pages_text.push(page_text);
        } else {
            all_pages_text.push(String::new());
        }

        // Progress indicator every 50 pages
        if *page_num % 50 == 0 {
            println!("  Processed {} pages...", page_num);
        }
    }

    let total_chars: usize = all_pages_text.iter().map(|s| s.len()).sum();
    println!("  Extracted {} total characters", total_chars);

    if total_chars == 0 {
        anyhow::bail!("No text extracted from PDF");
    }

    // Split into chunks
    let chunks = extract_chunks(&all_pages_text);
    println!("  Created {} chunks", chunks.len());

    Ok(IgContent {
        name: name.to_string(),
        version: version.to_string(),
        chunks,
    })
}

/// Extract text from a single page using lopdf
fn extract_text_from_page(doc: &Document, page_id: lopdf::ObjectId) -> Result<String> {
    let mut text = String::new();

    // Get the raw page content bytes
    let content_bytes = doc.get_page_content(page_id)?;

    // Parse the content stream
    let content = Content::decode(&content_bytes)?;

    // Parse content stream for text operators
    for operation in &content.operations {
        match operation.operator.as_str() {
            // Text showing operators
            "Tj" | "TJ" | "'" | "\"" => {
                for operand in &operation.operands {
                    if let Some(s) = extract_string_from_object(operand) {
                        text.push_str(&s);
                        text.push(' ');
                    }
                }
            }
            // Text positioning (add newline for readability)
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
            // Try UTF-16BE first (common in PDFs)
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
                // Try as Latin-1 / PDFDocEncoding
                Some(bytes.iter().map(|&b| b as char).collect())
            }
        }
        lopdf::Object::Array(arr) => {
            // TJ operator uses arrays mixing strings and positioning
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

fn extract_chunks(pages: &[String]) -> Vec<TextChunk> {
    let mut chunks = Vec::new();

    // Regex for section headings
    let heading_re =
        Regex::new(r"(?m)^\s*(\d+(?:\.\d+)*)\s+([A-Z][A-Za-z][^\n]{2,80}?)(?:\s*\.\s*\.|\s*$)")
            .unwrap();

    let mut current_heading = "Introduction".to_string();
    let mut current_content = String::new();

    for (page_num, page_text) in pages.iter().enumerate() {
        let page_num = (page_num + 1) as u32;

        // Split into paragraphs
        let paragraphs: Vec<&str> = page_text
            .split("\n\n")
            .flat_map(|p| p.split("\n \n"))
            .collect();

        for para in paragraphs {
            let para = para.trim();

            if para.len() < 15 {
                continue;
            }

            if looks_like_header_footer(para) {
                continue;
            }

            // Check for numbered section heading
            if let Some(caps) = heading_re.captures(para) {
                if current_content.len() > 100 {
                    let chunk = create_chunk(
                        &current_heading,
                        page_num.saturating_sub(1).max(1),
                        &current_content,
                    );
                    if chunk.content.len() > 50 && !looks_like_toc(&chunk.content) {
                        chunks.push(chunk);
                    }
                }
                current_heading = format!("{} {}", &caps[1], caps[2].trim());
                current_content.clear();
                continue;
            }

            // Check for all-caps headings
            if is_likely_heading(para) {
                if current_content.len() > 100 {
                    let chunk = create_chunk(
                        &current_heading,
                        page_num.saturating_sub(1).max(1),
                        &current_content,
                    );
                    if chunk.content.len() > 50 && !looks_like_toc(&chunk.content) {
                        chunks.push(chunk);
                    }
                }
                current_heading = para.to_string();
                current_content.clear();
                continue;
            }

            // Add to current content
            if !current_content.is_empty() {
                current_content.push(' ');
            }
            current_content.push_str(&clean_line(para));

            // Chunk if content is long
            if current_content.len() > 2500 {
                let chunk = create_chunk(&current_heading, page_num, &current_content);
                if chunk.content.len() > 50 && !looks_like_toc(&chunk.content) {
                    chunks.push(chunk);
                }
                current_content.clear();
            }
        }
    }

    // Final chunk
    if current_content.len() > 100 {
        let chunk = create_chunk(&current_heading, pages.len() as u32, &current_content);
        if chunk.content.len() > 50 && !looks_like_toc(&chunk.content) {
            chunks.push(chunk);
        }
    }

    chunks
}

fn is_likely_heading(text: &str) -> bool {
    let text = text.trim();
    if text.len() > 5 && text.len() < 60 {
        let upper_count = text.chars().filter(|c| c.is_uppercase()).count();
        let alpha_count = text.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count > 0 && upper_count as f32 / alpha_count as f32 > 0.8 {
            return true;
        }
    }
    false
}

fn clean_line(text: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(text.trim(), " ").to_string()
}

fn create_chunk(heading: &str, page: u32, content: &str) -> TextChunk {
    let content = clean_content(content);
    let domain = detect_domain(&content, heading);
    let variable = detect_variable(&content, heading);

    TextChunk {
        heading: heading.to_string(),
        page,
        content,
        domain,
        variable,
    }
}

fn clean_content(text: &str) -> String {
    let text = text.replace('\u{0000}', "");
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(text.trim(), " ").to_string()
}

fn detect_domain(content: &str, heading: &str) -> Option<String> {
    let text = format!("{} {}", heading, content).to_uppercase();

    for domain in DOMAINS {
        let pattern = format!(r"\b{}\b", domain);
        if let Ok(re) = Regex::new(&pattern) {
            if re.is_match(&heading.to_uppercase()) {
                return Some(domain.to_string());
            }
        }
    }

    for domain in DOMAINS {
        let patterns = [
            format!("{} DOMAIN", domain),
            format!("{} DATASET", domain),
            format!("THE {} ", domain),
        ];
        for p in &patterns {
            if text.contains(p) {
                return Some(domain.to_string());
            }
        }
    }

    None
}

fn detect_variable(content: &str, heading: &str) -> Option<String> {
    let text = format!("{} {}", heading, content).to_uppercase();

    for var in VARIABLE_PATTERNS {
        if heading.to_uppercase().contains(var) {
            return Some(var.to_string());
        }
    }

    let mut counts: Vec<(&str, usize)> = VARIABLE_PATTERNS
        .iter()
        .map(|v| {
            let pattern = format!(r"\b{}\b", v);
            let count = Regex::new(&pattern)
                .map(|re| re.find_iter(&text).count())
                .unwrap_or(0);
            (*v, count)
        })
        .filter(|(_, c)| *c >= 3)
        .collect();

    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.first().map(|(v, _)| v.to_string())
}

fn looks_like_header_footer(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let text_len = text.len();

    if text_len < 5 || text_len > 150 {
        return false;
    }

    text_lower.contains("© cdisc")
        || text_lower.contains("all rights reserved")
        || (text_lower.contains("implementation guide")
            && (text_lower.contains("cdisc") || text_lower.contains("version")))
        || text.chars().filter(|c| c.is_ascii_digit()).count() > text_len / 2
        || text_lower.starts_with("page ")
        || text_lower.ends_with(" page")
        || (text_lower.contains("cdisc") && text_len < 80)
}

fn looks_like_toc(content: &str) -> bool {
    let dot_count = content.matches('.').count();
    let word_count = content.split_whitespace().count();

    if word_count > 0 {
        let dot_ratio = dot_count as f32 / word_count as f32;
        if dot_ratio > 0.5 {
            return true;
        }
    }

    let lines: Vec<&str> = content.split('\n').collect();
    if lines.len() > 3 {
        let lines_with_trailing_numbers = lines
            .iter()
            .filter(|l| l.trim().chars().last().is_some_and(|c| c.is_ascii_digit()))
            .count();
        if lines_with_trailing_numbers as f32 / lines.len() as f32 > 0.5 {
            return true;
        }
    }

    content.contains(".....")
        || content.to_lowercase().contains("table of contents")
        || content.to_lowercase().contains("list of tables")
        || content.to_lowercase().contains("list of figures")
}
