//! PDF Processing Tool for CDISC Implementation Guides
//!
//! Two-pass extraction with full TUI:
//! 1. First pass: Identify section boundaries and their domains
//! 2. Second pass: Chunk content within sections, inheriting domain context
//!
//! Usage:
//!   cargo run --bin process_pdfs

use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::stdout;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// =============================================================================
// TUI State
// =============================================================================

/// Main application state for the TUI
struct App {
    igs: Vec<IgState>,
    current_ig: usize,
    start_time: Instant,
    total_chunks: usize,
    total_domains: HashSet<String>,
    spinner_frame: usize,
    is_complete: bool,
}

impl App {
    fn new(ig_configs: &[(&str, &str, &str, &str, &[&str])]) -> Self {
        let igs = ig_configs
            .iter()
            .map(|(_, name, version, _, _)| IgState {
                name: name.to_string(),
                version: version.to_string(),
                status: IgStatus::Waiting,
                chunks: 0,
                domains: 0,
                elapsed: None,
            })
            .collect();

        Self {
            igs,
            current_ig: 0,
            start_time: Instant::now(),
            total_chunks: 0,
            total_domains: HashSet::new(),
            spinner_frame: 0,
            is_complete: false,
        }
    }

    fn completed_count(&self) -> usize {
        self.igs
            .iter()
            .filter(|ig| matches!(ig.status, IgStatus::Complete | IgStatus::Skipped))
            .count()
    }

    fn advance_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    fn current_spinner(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame]
    }
}

/// State for a single Implementation Guide
struct IgState {
    name: String,
    version: String,
    status: IgStatus,
    chunks: usize,
    domains: usize,
    elapsed: Option<Duration>,
}

/// Processing status for an IG
#[derive(Clone)]
enum IgStatus {
    Waiting,
    Processing(Phase),
    Complete,
    Error(String),
    Skipped,
}

/// Processing phase
#[derive(Clone, Copy)]
enum Phase {
    Extracting,
    IdentifyingSections,
    CreatingChunks,
    Saving,
}

impl Phase {
    fn description(&self) -> &'static str {
        match self {
            Phase::Extracting => "Extracting text from PDF...",
            Phase::IdentifyingSections => "Pass 1: Identifying sections...",
            Phase::CreatingChunks => "Pass 2: Creating chunks...",
            Phase::Saving => "Saving JSON output...",
        }
    }
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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
// Section Validation Functions
// =============================================================================

/// Validate that a section number follows CDISC IG conventions
/// Valid: "1.1", "6.3.5.2", "4.2.1.6"
/// Invalid: "0.056", "24.6", "137.1", "1.126819661"
fn is_valid_section_number(num_str: &str) -> bool {
    let segments: Vec<&str> = num_str.split('.').collect();

    // Must have 2-6 segments (e.g., "1.1" to "1.2.3.4.5.6")
    if segments.len() < 2 || segments.len() > 6 {
        return false;
    }

    for (i, seg) in segments.iter().enumerate() {
        // Each segment must be a valid integer
        let Ok(n) = seg.parse::<u32>() else {
            return false;
        };

        // First segment (chapter): must be 1-9
        if i == 0 && (n == 0 || n > 9) {
            return false;
        }

        // Subsequent segments: must be 1-50
        if i > 0 && n > 50 {
            return false;
        }

        // Reject scientific notation (leading zeros like "056")
        if seg.len() > 1 && seg.starts_with('0') {
            return false;
        }
    }

    true
}

/// Validate that a heading title looks like prose, not table data
/// Valid: "Demographics Domain", "Use of USUBJID", "Purpose"
/// Invalid: "MdFI QUANTITATIVE FLUORESCENCE", "CR CR ACE"
fn is_valid_heading_title(title: &str) -> bool {
    let trimmed = title.trim();

    // Must be reasonable length
    if trimmed.len() < 3 || trimmed.len() > 100 {
        return false;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();

    // Single-word titles allowed if properly capitalized (e.g., "Purpose")
    if words.len() == 1 {
        let word = words[0];
        return word.len() >= 3
            && word.chars().next().unwrap().is_uppercase()
            && word.chars().skip(1).any(|c| c.is_lowercase());
    }

    // Multi-word: first word must have lowercase letters (not ALL-CAPS)
    let first = words[0];
    if first.len() > 2
        && first
            .chars()
            .all(|c| c.is_uppercase() || !c.is_alphabetic())
    {
        // ALL-CAPS first word is OK only if second word is proper case
        if let Some(second) = words.get(1) {
            let second_proper = second
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && second.chars().skip(1).any(|c| c.is_lowercase());
            if !second_proper {
                return false;
            }
        }
    }

    // Reject if >50% of words are ALL-CAPS (table data pattern)
    let all_caps_count = words
        .iter()
        .filter(|w| w.len() > 1 && w.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()))
        .count();
    if words.len() > 2 && all_caps_count > words.len() / 2 {
        return false;
    }

    // Reject scientific/measurement patterns
    let bad_patterns = ["GRCh", "BAU/", "MdFI", "/mL", "GENE WITH", "/kg"];
    if bad_patterns.iter().any(|p| trimmed.contains(p)) {
        return false;
    }

    // Reject repeated consecutive words (e.g., "CR CR ACE", "PR PR ACE")
    for window in words.windows(2) {
        if window[0] == window[1] {
            return false;
        }
    }

    true
}

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
// TUI Rendering
// =============================================================================

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: header, IG list, progress, stats
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header
        Constraint::Min(7),    // IG List
        Constraint::Length(5), // Progress
        Constraint::Length(3), // Stats
    ])
    .split(area);

    render_header(frame, chunks[0]);
    render_ig_list(frame, chunks[1], app);
    render_progress(frame, chunks[2], app);
    render_stats(frame, chunks[3], app);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" CDISC Implementation Guide Processor ")
        .title_style(Style::default().bold().fg(Color::Cyan));

    let text = Paragraph::new(vec![
        Line::from(vec![Span::raw("  Two-Pass Section-First Extraction")]),
        Line::from(vec![
            Span::raw("  Press "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" or "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" to quit"),
        ]),
    ])
    .style(Style::default().fg(Color::Gray))
    .block(block);

    frame.render_widget(text, area);
}

fn render_ig_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .igs
        .iter()
        .enumerate()
        .map(|(i, ig)| {
            let (symbol, style) = match &ig.status {
                IgStatus::Complete => ("✓", Style::default().fg(Color::Green)),
                IgStatus::Processing(_) => {
                    if i == app.current_ig {
                        (app.current_spinner(), Style::default().fg(Color::Yellow))
                    } else {
                        ("⠸", Style::default().fg(Color::Yellow))
                    }
                }
                IgStatus::Waiting => (" ", Style::default().fg(Color::DarkGray)),
                IgStatus::Error(_) => ("✗", Style::default().fg(Color::Red)),
                IgStatus::Skipped => ("○", Style::default().fg(Color::DarkGray)),
            };

            let status_text = match &ig.status {
                IgStatus::Complete => format!(
                    "{} chunks   {} domains          {:.1}s",
                    ig.chunks,
                    ig.domains,
                    ig.elapsed.map(|d| d.as_secs_f32()).unwrap_or(0.0)
                ),
                IgStatus::Processing(phase) => phase.description().to_string(),
                IgStatus::Waiting => "Waiting...".to_string(),
                IgStatus::Error(e) => format!("Error: {}", e),
                IgStatus::Skipped => "PDF not found".to_string(),
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", symbol), style),
                Span::styled(
                    format!("{:<18}", format!("{} v{}", ig.name, ig.version)),
                    if matches!(ig.status, IgStatus::Processing(_)) {
                        Style::default().bold().fg(Color::White)
                    } else {
                        style
                    },
                ),
                Span::styled(status_text, Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Implementation Guides "),
    );

    frame.render_widget(list, area);
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Progress ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate progress
    let (ratio, phase_text) = if app.is_complete {
        (1.0, "Complete! Press any key to exit...".to_string())
    } else if let Some(ig) = app.igs.get(app.current_ig) {
        match &ig.status {
            IgStatus::Processing(phase) => {
                let base_progress = app.current_ig as f64 / app.igs.len() as f64;
                let phase_progress = match phase {
                    Phase::Extracting => 0.25,
                    Phase::IdentifyingSections => 0.5,
                    Phase::CreatingChunks => 0.75,
                    Phase::Saving => 0.9,
                };
                let ratio = base_progress + (phase_progress / app.igs.len() as f64);
                (ratio, format!("Current: {} v{}", ig.name, ig.version))
            }
            _ => {
                let ratio = app.completed_count() as f64 / app.igs.len() as f64;
                (ratio, "Processing...".to_string())
            }
        }
    } else {
        (0.0, "Initializing...".to_string())
    };

    let progress_area =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

    // Progress bar
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(ratio.min(1.0))
        .label(format!("{:.0}%", ratio * 100.0));

    frame.render_widget(gauge, progress_area[0]);

    // Phase text
    let phase_para = Paragraph::new(phase_text).style(Style::default().fg(Color::Gray));

    frame.render_widget(phase_para, progress_area[1]);
}

fn render_stats(frame: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.start_time.elapsed().as_secs_f32();
    let stats_text = format!(
        "Files: {}/{}  │  Chunks: {}  │  Domains: {}  │  Time: {:.1}s",
        app.completed_count(),
        app.igs.len(),
        app.total_chunks,
        app.total_domains.len(),
        elapsed
    );

    let stats = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(stats, area);
}

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() -> Result<()> {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pdf_dir = base_path.join("pdfs");
    let data_dir = base_path.join("data");

    fs::create_dir_all(&data_dir)?;

    // Each entry: (pdf_name, ig_name, version, output_name, domains)
    let igs: &[(&str, &str, &str, &str, &[&str])] = &[
        (
            "SDTMIG_v3.4.pdf",
            "SDTM-IG",
            "3.4",
            "sdtm-ig-v3.4.json",
            SDTM_DOMAINS,
        ),
        (
            "SENDIG_v3.1.1.pdf",
            "SEND-IG",
            "3.1.1",
            "send-ig-v3.1.1.json",
            SEND_DOMAINS,
        ),
        (
            "ADaMIG_v1.3.pdf",
            "ADaM-IG",
            "1.3",
            "adam-ig-v1.3.json",
            ADAM_DOMAINS,
        ),
    ];

    // Initialize terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    terminal.clear()?;

    // Create app state
    let mut app = App::new(igs);
    let mut user_quit = false;

    // Initial render
    terminal.draw(|f| ui(f, &app))?;

    // Process each IG
    'processing: for (i, (pdf_name, ig_name, version, output_name, domains)) in
        igs.iter().enumerate()
    {
        app.current_ig = i;
        let ig_start = Instant::now();

        let pdf_path = pdf_dir.join(pdf_name);
        let output_path = data_dir.join(output_name);

        if !pdf_path.exists() {
            app.igs[i].status = IgStatus::Skipped;
            terminal.draw(|f| ui(f, &app))?;
            continue;
        }

        // Phase 1: Extract text
        app.igs[i].status = IgStatus::Processing(Phase::Extracting);
        terminal.draw(|f| ui(f, &app))?;

        let text = match extract_with_animation(&mut terminal, &mut app, &pdf_path) {
            Ok(Some(t)) => t,
            Ok(None) => {
                user_quit = true;
                break 'processing;
            }
            Err(e) => {
                app.igs[i].status = IgStatus::Error(format!("{:#}", e));
                terminal.draw(|f| ui(f, &app))?;
                continue;
            }
        };

        // Phase 2: Identify sections
        app.igs[i].status = IgStatus::Processing(Phase::IdentifyingSections);
        terminal.draw(|f| ui(f, &app))?;

        let sections = identify_sections(&text, domains);
        if animate_briefly(&mut terminal, &mut app)? {
            user_quit = true;
            break 'processing;
        }

        // Phase 3: Create chunks
        app.igs[i].status = IgStatus::Processing(Phase::CreatingChunks);
        terminal.draw(|f| ui(f, &app))?;

        let chunks = chunk_sections(&sections, domains);
        if animate_briefly(&mut terminal, &mut app)? {
            user_quit = true;
            break 'processing;
        }

        // Phase 4: Save
        app.igs[i].status = IgStatus::Processing(Phase::Saving);
        terminal.draw(|f| ui(f, &app))?;

        let content = IgContent {
            name: ig_name.to_string(),
            version: version.to_string(),
            chunks,
        };

        let domains_found: HashSet<String> = content
            .chunks
            .iter()
            .filter_map(|c| c.domain.clone())
            .collect();

        let json = serde_json::to_string_pretty(&content)?;
        fs::write(&output_path, &json)?;

        // Update stats
        app.total_chunks += content.chunks.len();
        app.total_domains.extend(domains_found.clone());

        app.igs[i].chunks = content.chunks.len();
        app.igs[i].domains = domains_found.len();
        app.igs[i].elapsed = Some(ig_start.elapsed());
        app.igs[i].status = IgStatus::Complete;

        terminal.draw(|f| ui(f, &app))?;
    }

    // Show final state and wait for exit (unless user already quit)
    if !user_quit {
        app.is_complete = true;
        terminal.draw(|f| ui(f, &app))?;
        wait_for_exit()?;
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Extract text with spinner animation, returns None if user quit
fn extract_with_animation(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    path: &Path,
) -> Result<Option<String>> {
    let (tx, rx) = mpsc::channel();
    let path = path.to_owned();

    // Spawn extraction in background thread
    thread::spawn(move || {
        let result = pdf_extract::extract_text(&path);
        tx.send(result).ok();
    });

    // Animate while waiting
    loop {
        // Check for quit
        if check_for_quit()? {
            return Ok(None);
        }

        match rx.try_recv() {
            Ok(result) => {
                return Ok(Some(result.context("PDF extraction failed")?));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Thread panicked or dropped sender without sending
                anyhow::bail!("PDF extraction thread failed unexpectedly");
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Still processing, continue animation
                app.advance_spinner();
                terminal.draw(|f| ui(f, app))?;
                thread::sleep(Duration::from_millis(80));
            }
        }
    }
}

/// Brief animation for fast phases, returns true if user wants to quit
fn animate_briefly(terminal: &mut DefaultTerminal, app: &mut App) -> Result<bool> {
    for _ in 0..3 {
        if check_for_quit()? {
            return Ok(true);
        }
        app.advance_spinner();
        terminal.draw(|f| ui(f, app))?;
        thread::sleep(Duration::from_millis(80));
    }
    Ok(false)
}

/// Check if user pressed quit key (q or Escape), non-blocking
fn check_for_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(0))?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
        && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    {
        return Ok(true);
    }
    Ok(false)
}

/// Wait for any key press to exit
fn wait_for_exit() -> Result<()> {
    loop {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            return Ok(());
        }
    }
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
            let num_str = number.as_str();
            let title_str = title.as_str().trim();

            // Validate section number structure (reject table data like "24.6", "137.1")
            if !is_valid_section_number(num_str) {
                continue;
            }

            // Validate title looks like prose heading (reject "MdFI QUANTITATIVE FLUORESCENCE")
            if !is_valid_heading_title(title_str) {
                continue;
            }

            let heading = format!("{} {}", num_str, title_str);

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
    if !current_chunk.is_empty()
        && let Some(chunk) = create_chunk(
            heading,
            &current_chunk,
            section_domain,
            domains,
            *next_index,
            section_parent,
        )
    {
        *next_index += 1;
        chunks.push(chunk);
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

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_section_numbers() {
        assert!(is_valid_section_number("1.1"));
        assert!(is_valid_section_number("6.3.5.2"));
        assert!(is_valid_section_number("4.2.1.6.2"));
        assert!(is_valid_section_number("9.1"));
        assert!(is_valid_section_number("1.2.3.4.5.6"));
    }

    #[test]
    fn test_invalid_section_numbers() {
        assert!(!is_valid_section_number("0.056299177")); // Starts with 0
        assert!(!is_valid_section_number("24.6")); // Chapter > 9
        assert!(!is_valid_section_number("137.1")); // Chapter > 9
        assert!(!is_valid_section_number("1.126819661")); // Segment > 50
        assert!(!is_valid_section_number("1")); // Single segment
        assert!(!is_valid_section_number("1.2.3.4.5.6.7")); // Too deep (7 levels)
    }

    #[test]
    fn test_valid_heading_titles() {
        assert!(is_valid_heading_title("Purpose"));
        assert!(is_valid_heading_title("Demographics Domain"));
        assert!(is_valid_heading_title("Use of Subject and USUBJID"));
        assert!(is_valid_heading_title("SDTM Core Designations"));
        assert!(is_valid_heading_title("Adverse Events (AE)"));
    }

    #[test]
    fn test_invalid_heading_titles() {
        assert!(!is_valid_heading_title("MdFI QUANTITATIVE FLUORESCENCE"));
        assert!(!is_valid_heading_title("GRCh38.p12 ACTB GENE WITH"));
        assert!(!is_valid_heading_title("BAU/mL 10 mm"));
        assert!(!is_valid_heading_title("CR CR ACE"));
        assert!(!is_valid_heading_title("PR PR ACE"));
    }
}
