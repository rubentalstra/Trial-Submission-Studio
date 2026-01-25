use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

/// Pre-processed IG content - primarily text chunks from the PDF documents
pub struct IgIndex {
    sdtm: IgContent,
    send: IgContent,
    adam: IgContent,
}

/// Content from a single Implementation Guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgContent {
    pub name: String,
    pub version: String,
    /// Text chunks extracted from the PDF, preserving context and page references
    pub chunks: Vec<TextChunk>,
}

/// A chunk of text from the IG document
///
/// These are the building blocks for search - each chunk represents
/// a meaningful section of the guidance document (a paragraph, a rule,
/// an explanation, a table with its context, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    /// Section/chapter heading this chunk belongs to
    pub heading: String,
    /// Page number(s) in the original PDF
    pub page: u32,
    /// The actual text content - this is the prose, rules, guidance
    pub content: String,
    /// Optional: domain code if this chunk relates to a specific domain
    pub domain: Option<String>,
    /// Optional: variable name if this chunk discusses a specific variable
    pub variable: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub ig: String,
    pub heading: String,
    pub page: u32,
    pub content: String,
    pub domain: Option<String>,
    pub variable: Option<String>,
    pub score: f32,
}

impl IgIndex {
    /// Load pre-processed IG data (embedded at compile time)
    pub fn load() -> anyhow::Result<Self> {
        let sdtm: IgContent = serde_json::from_str(include_str!("../data/sdtm-ig-v3.4.json"))?;
        let send: IgContent = serde_json::from_str(include_str!("../data/send-ig-v3.1.1.json"))?;
        let adam: IgContent = serde_json::from_str(include_str!("../data/adam-ig-v1.3.json"))?;

        Ok(Self { sdtm, send, adam })
    }

    pub fn section_count(&self) -> usize {
        self.sdtm.chunks.len() + self.send.chunks.len() + self.adam.chunks.len()
    }

    pub fn domain_count(&self) -> usize {
        // Count unique domains mentioned across all chunks
        let mut domains = std::collections::HashSet::new();
        for ig in [&self.sdtm, &self.send, &self.adam] {
            for chunk in &ig.chunks {
                if let Some(d) = &chunk.domain {
                    domains.insert(d.clone());
                }
            }
        }
        domains.len()
    }

    /// Full-text search across IGs
    ///
    /// This searches the prose content of the guidance documents,
    /// returning relevant sections with their context.
    pub fn search(&self, query: &str, ig: &str, limit: usize) -> Vec<SearchResult> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        if keywords.is_empty() {
            return Vec::new();
        }

        // Case-insensitive keyword matching
        let lower_keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
        let ac = match AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .build(&lower_keywords)
        {
            Ok(ac) => ac,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        let igs_to_search: Vec<(&str, &IgContent)> = match ig.to_lowercase().as_str() {
            "sdtm" => vec![("SDTM-IG v3.4", &self.sdtm)],
            "send" => vec![("SEND-IG v3.1.1", &self.send)],
            "adam" => vec![("ADaM-IG v1.3", &self.adam)],
            _ => vec![
                ("SDTM-IG v3.4", &self.sdtm),
                ("SEND-IG v3.1.1", &self.send),
                ("ADaM-IG v1.3", &self.adam),
            ],
        };

        for (ig_name, ig_content) in igs_to_search {
            for chunk in &ig_content.chunks {
                let matches: Vec<_> = ac.find_iter(&chunk.content).collect();
                if !matches.is_empty() {
                    // Score based on how many distinct keywords matched
                    let unique_patterns: std::collections::HashSet<_> =
                        matches.iter().map(|m| m.pattern().as_usize()).collect();
                    let score = unique_patterns.len() as f32 / lower_keywords.len() as f32;

                    results.push(SearchResult {
                        ig: ig_name.to_string(),
                        heading: chunk.heading.clone(),
                        page: chunk.page,
                        content: truncate_around_match(&chunk.content, &query, 600),
                        domain: chunk.domain.clone(),
                        variable: chunk.variable.clone(),
                        score,
                    });
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.total_cmp(&a.score));
        results.truncate(limit);
        results
    }

    /// Get all chunks related to a specific domain
    pub fn get_domain(&self, domain: &str, ig: &str) -> Option<Vec<TextChunk>> {
        let ig_content = match ig.to_lowercase().as_str() {
            "sdtm" => &self.sdtm,
            "send" => &self.send,
            "adam" => &self.adam,
            _ => return None,
        };

        let domain_upper = domain.to_uppercase();
        let chunks: Vec<TextChunk> = ig_content
            .chunks
            .iter()
            .filter(|c| {
                c.domain
                    .as_ref()
                    .is_some_and(|d| d.eq_ignore_ascii_case(&domain_upper))
            })
            .cloned()
            .collect();

        if chunks.is_empty() {
            None
        } else {
            Some(chunks)
        }
    }

    /// Get all chunks that mention a specific variable
    pub fn get_variable(&self, variable: &str, domain: Option<&str>) -> Vec<TextChunk> {
        let variable_upper = variable.to_uppercase();
        let mut results = Vec::new();

        for ig in [&self.sdtm, &self.send, &self.adam] {
            for chunk in &ig.chunks {
                // Check if this chunk is about the variable
                let var_matches = chunk
                    .variable
                    .as_ref()
                    .is_some_and(|v| v.eq_ignore_ascii_case(&variable_upper));

                // Or if the variable is mentioned prominently in the content
                let content_mentions = chunk.content.to_uppercase().contains(&variable_upper);

                if var_matches || content_mentions {
                    // Filter by domain if specified
                    if let Some(d) = domain {
                        if !chunk
                            .domain
                            .as_ref()
                            .is_some_and(|cd| cd.eq_ignore_ascii_case(d))
                        {
                            continue;
                        }
                    }
                    results.push(chunk.clone());
                }
            }
        }

        results
    }
}

/// Truncate content, trying to show the most relevant part around the first match
fn truncate_around_match(content: &str, query: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        return content.to_string();
    }

    // Find where the first query word appears
    let first_keyword = query.split_whitespace().next().unwrap_or("");
    let lower_content = content.to_lowercase();
    let lower_keyword = first_keyword.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_keyword) {
        // Center the window around the match
        let half_len = max_len / 2;
        let start = pos.saturating_sub(half_len);
        let end = (start + max_len).min(content.len());

        let mut result = String::new();
        if start > 0 {
            result.push_str("...");
        }
        result.push_str(&content[start..end]);
        if end < content.len() {
            result.push_str("...");
        }
        result
    } else {
        // Fallback: just truncate from the start
        format!("{}...", &content[..max_len])
    }
}
