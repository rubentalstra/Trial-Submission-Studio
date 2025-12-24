#![deny(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use sdtm_standards::StandardsRegistry;

#[derive(Debug, Parser)]
#[command(name = "sdtm")]
#[command(about = "Offline SDTM transpiler (Phase 0 bootstrap)")]
struct Cli {
    /// Path to the offline standards directory.
    #[arg(long, default_value = "standards")]
    standards_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Standards {
        #[command(subcommand)]
        command: StandardsCommand,
    },
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    Validate {
        #[command(subcommand)]
        command: ValidateCommand,
    },
}

#[derive(Debug, Subcommand)]
enum StandardsCommand {
    Verify,
    Summary,
    Doctor {
        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: String,
    },
}

#[derive(Debug, Subcommand)]
enum ValidateCommand {
    Csv {
        /// SDTM domain code (e.g. DM, AE).
        #[arg(long)]
        domain: String,

        /// Input CSV file.
        #[arg(long, value_name = "PATH")]
        input: PathBuf,

        /// Stable source identifier used for deterministic RowId derivation.
        /// Defaults to the input path string.
        #[arg(long)]
        source_id: Option<String>,

        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum RunCommand {
    Csv {
        /// SDTM domain code (e.g. DM, AE).
        #[arg(long)]
        domain: String,

        /// Input CSV file.
        #[arg(long, value_name = "PATH")]
        input: PathBuf,

        /// Stable source identifier used for deterministic RowId derivation.
        /// Defaults to the input path string.
        #[arg(long)]
        source_id: Option<String>,

        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: Option<String>,
    },

    Study {
        /// Directory containing one or more input CSV files.
        #[arg(long, value_name = "PATH")]
        input_dir: PathBuf,

        /// Write machine-readable JSON report to this path. Use '-' for stdout.
        #[arg(long, value_name = "PATH")]
        json: Option<String>,
    },
}

#[derive(Debug, serde::Serialize)]
struct RunReport {
    domain: String,
    input: String,
    ingested_rows: usize,
    ingested_columns: usize,
    mapped_tables: usize,
    validation: sdtm_validate::ValidationReport,
}

#[derive(Debug, serde::Serialize)]
struct StudyFileReport {
    input: String,
    inferred_domain: String,
    inference_reason: String,
    ingested_rows: usize,
    ingested_columns: usize,
    mapped_tables: usize,
    validation: sdtm_validate::ValidationReport,
}

#[derive(Debug, serde::Serialize)]
struct StudySkippedFileReport {
    input: String,
    reason: String,
}

#[derive(Debug, serde::Serialize)]
struct StudyReport {
    input_dir: String,
    files: Vec<StudyFileReport>,
    skipped: Vec<StudySkippedFileReport>,
    total_errors: usize,
    total_warnings: usize,
}

#[derive(Debug, Clone)]
struct DomainInference {
    domain: String,
    reason: String,
}

fn list_csv_files(input_dir: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_csv = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("csv"));
        if is_csv {
            files.push(path);
        }
    }
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(files)
}

fn csv_headers_upper(path: &std::path::Path) -> anyhow::Result<std::collections::BTreeSet<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;
    let headers = reader.headers()?.clone();
    let mut out = std::collections::BTreeSet::new();
    for h in headers.iter() {
        let v = h.trim();
        if !v.is_empty() {
            out.insert(v.to_ascii_uppercase());
        }
    }
    Ok(out)
}

fn infer_domain_for_file(
    registry: &StandardsRegistry,
    path: &std::path::Path,
) -> anyhow::Result<DomainInference> {
    // 1) Filename-based inference.
    let domains: Vec<String> = registry.datasets_by_domain.keys().cloned().collect();

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();

    let tokens: Vec<String> = stem
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    // Prefer the last token (common pattern: *_DM.csv), then fall back.
    for cand in tokens.iter().rev().chain(std::iter::once(&stem)) {
        if registry.datasets_by_domain.contains_key(cand) {
            return Ok(DomainInference {
                domain: cand.clone(),
                reason: format!("filename token '{cand}'"),
            });
        }
    }

    // 2) Header scoring fallback.
    let headers = csv_headers_upper(path)?;

    let mut best: Option<(String, usize, usize, usize)> = None;
    // (domain, required_hits, known_hits, unknown_headers)
    for domain in domains {
        let (required, known) = required_and_known_vars(registry, &domain);

        let required_hits = required.intersection(&headers).count();
        let known_hits = known.intersection(&headers).count();
        let unknown_headers = headers.difference(&known).count();

        let candidate = (domain.clone(), required_hits, known_hits, unknown_headers);

        best = match best {
            None => Some(candidate),
            Some(current) => {
                // Higher required_hits, then higher known_hits, then lower unknown_headers,
                // then lexicographically smallest domain for determinism.
                let (cd, cr, ck, cu) = current;
                let (nd, nr, nk, nu) = candidate;
                let better = nr > cr
                    || (nr == cr && nk > ck)
                    || (nr == cr && nk == ck && nu < cu)
                    || (nr == cr && nk == ck && nu == cu && nd < cd);
                if better {
                    Some((nd, nr, nk, nu))
                } else {
                    Some((cd, cr, ck, cu))
                }
            }
        };
    }

    let Some((domain, required_hits, known_hits, unknown_headers)) = best else {
        return Err(anyhow::anyhow!(
            "no domains available in standards registry"
        ));
    };

    if known_hits == 0 {
        return Err(anyhow::anyhow!(
            "could not infer domain from filename or headers (no known variables matched)"
        ));
    }

    Ok(DomainInference {
        domain: domain.clone(),
        reason: format!(
            "header match (required_hits={required_hits}, known_hits={known_hits}, unknown_headers={unknown_headers})"
        ),
    })
}

fn required_and_known_vars(
    registry: &StandardsRegistry,
    domain: &str,
) -> (
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
) {
    let mut required = std::collections::BTreeSet::new();
    let mut known = std::collections::BTreeSet::new();
    for key in ["*", domain] {
        if let Some(vars) = registry.variables_by_domain.get(key) {
            for v in vars {
                known.insert(v.var.to_ascii_uppercase());
                if v.required.unwrap_or(false) {
                    required.insert(v.var.to_ascii_uppercase());
                }
            }
        }
    }
    (required, known)
}

fn stable_source_id(input_dir: &std::path::Path, path: &std::path::Path) -> String {
    let rel = path.strip_prefix(input_dir).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Standards { command } => match command {
            StandardsCommand::Verify => {
                let (_registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                println!(
                    "OK: verified {} files (SDTM domains={}, SDTMIG domains={}, CT codelists={}, conflicts={})",
                    summary.file_count,
                    summary.domain_count_sdtm,
                    summary.domain_count_sdtmig,
                    summary.codelist_count,
                    summary.conflict_count
                );
                Ok(())
            }
            StandardsCommand::Summary => {
                let (registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                println!("Pins:");
                println!("  SDTM: {}", summary.manifest_pins.sdtm);
                println!("  SDTMIG: {}", summary.manifest_pins.sdtmig);
                println!("  CT: {}", summary.manifest_pins.ct);
                println!();
                println!("Counts:");
                println!("  files: {}", summary.file_count);
                println!("  sdtm domains: {}", summary.domain_count_sdtm);
                println!("  sdtmig domains: {}", summary.domain_count_sdtmig);
                println!("  sdtm variables: {}", summary.variable_count_sdtm);
                println!("  sdtmig variables: {}", summary.variable_count_sdtmig);
                println!("  ct codelists: {}", summary.codelist_count);
                println!();
                println!("Conflicts:");
                println!("  count: {}", registry.conflicts.len());
                Ok(())
            }
            StandardsCommand::Doctor { json } => {
                let (registry, summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                let report = sdtm_standards::DoctorReport::from_verify_summary(
                    &summary,
                    registry.manifest.policy.clone(),
                    registry.files.clone(),
                    registry.conflicts.clone(),
                );

                let out = serde_json::to_string_pretty(&report)?;
                if json == "-" {
                    println!("{}", out);
                } else {
                    std::fs::write(&json, out)?;
                    println!("wrote {}", json);
                }
                Ok(())
            }
        },
        Command::Run { command } => match command {
            RunCommand::Csv {
                domain,
                input,
                source_id,
                json,
            } => {
                let domain_code = sdtm_model::DomainCode::new(domain.clone())?;
                let source_id = source_id.unwrap_or_else(|| input.to_string_lossy().to_string());

                let table = sdtm_ingest::csv_ingest::ingest_csv_file(
                    domain_code,
                    &input,
                    sdtm_ingest::csv_ingest::CsvIngestOptions::new(source_id),
                )?;

                let ingested_rows = table.rows.len();
                let ingested_columns = table.columns.len();

                let mapper = sdtm_map::SimpleMapper::new();
                let mapped = sdtm_core::pipeline::Mapper::map(&mapper, table)?;

                let validator =
                    sdtm_validate::StandardsValidator::from_standards_dir(&cli.standards_dir)?;
                let validation = validator.validate_with_report(&mapped);

                let report = RunReport {
                    domain: domain.clone(),
                    input: input.to_string_lossy().to_string(),
                    ingested_rows,
                    ingested_columns,
                    mapped_tables: mapped.len(),
                    validation: validation.clone(),
                };

                if let Some(json) = json {
                    let out = serde_json::to_string_pretty(&report)?;
                    if json == "-" {
                        println!("{}", out);
                    } else {
                        std::fs::write(&json, out)?;
                        println!("wrote {}", json);
                    }
                } else {
                    println!(
                        "{}: pipeline (rows={}, cols={}, mapped_tables={}, errors={}, warnings={})",
                        domain,
                        report.ingested_rows,
                        report.ingested_columns,
                        report.mapped_tables,
                        report.validation.errors,
                        report.validation.warnings
                    );
                }

                if report.validation.errors > 0 {
                    return Err(anyhow::anyhow!("validation failed"));
                }
                Ok(())
            }

            RunCommand::Study { input_dir, json } => {
                let (registry, _summary) =
                    sdtm_standards::StandardsRegistry::verify_and_load(&cli.standards_dir)
                        .map_err(|e| anyhow::anyhow!(e))?;

                let files = list_csv_files(&input_dir)?;
                if files.is_empty() {
                    return Err(anyhow::anyhow!(
                        "no .csv files found under {}",
                        input_dir.to_string_lossy()
                    ));
                }

                let mapper = sdtm_map::SimpleMapper::new();

                let mut report = StudyReport {
                    input_dir: input_dir.to_string_lossy().to_string(),
                    files: Vec::new(),
                    skipped: Vec::new(),
                    total_errors: 0,
                    total_warnings: 0,
                };

                for path in files {
                    let inference = match infer_domain_for_file(&registry, &path) {
                        Ok(v) => v,
                        Err(e) => {
                            let reason = e.to_string();
                            report.skipped.push(StudySkippedFileReport {
                                input: path.to_string_lossy().to_string(),
                                reason: reason.clone(),
                            });
                            if json.is_none() {
                                println!("{}: skipped ({})", path.to_string_lossy(), reason);
                            }
                            continue;
                        }
                    };
                    let domain_code = sdtm_model::DomainCode::new(inference.domain.clone())?;

                    let source_id = stable_source_id(&input_dir, &path);

                    let table = sdtm_ingest::csv_ingest::ingest_csv_file(
                        domain_code,
                        &path,
                        sdtm_ingest::csv_ingest::CsvIngestOptions::new(source_id),
                    )?;

                    let ingested_rows = table.rows.len();
                    let ingested_columns = table.columns.len();

                    let mapped = sdtm_core::pipeline::Mapper::map(&mapper, table)?;
                    let mut validation = sdtm_validate::ValidationReport::default();
                    for t in &mapped {
                        let r = sdtm_validate::validate_table_against_standards(&registry, t);
                        validation.errors += r.errors;
                        validation.warnings += r.warnings;
                        validation.issues.extend(r.issues);
                    }

                    report.total_errors += validation.errors;
                    report.total_warnings += validation.warnings;

                    report.files.push(StudyFileReport {
                        input: path.to_string_lossy().to_string(),
                        inferred_domain: inference.domain.clone(),
                        inference_reason: inference.reason.clone(),
                        ingested_rows,
                        ingested_columns,
                        mapped_tables: mapped.len(),
                        validation: validation.clone(),
                    });
                }

                if report.files.is_empty() {
                    return Err(anyhow::anyhow!(
                        "no SDTM dataset CSVs could be inferred under {} (skipped={})",
                        input_dir.to_string_lossy(),
                        report.skipped.len()
                    ));
                }

                if let Some(json) = json {
                    let out = serde_json::to_string_pretty(&report)?;
                    if json == "-" {
                        println!("{}", out);
                    } else {
                        std::fs::write(&json, out)?;
                        println!("wrote {}", json);
                    }
                } else {
                    for f in &report.files {
                        println!(
                            "{}: inferred_domain={} ({}) rows={} cols={} errors={} warnings={}",
                            f.input,
                            f.inferred_domain,
                            f.inference_reason,
                            f.ingested_rows,
                            f.ingested_columns,
                            f.validation.errors,
                            f.validation.warnings
                        );
                    }
                    println!(
                        "study: total errors={} warnings={} files={} skipped={}",
                        report.total_errors,
                        report.total_warnings,
                        report.files.len(),
                        report.skipped.len()
                    );
                }

                if report.total_errors > 0 {
                    return Err(anyhow::anyhow!("validation failed"));
                }
                Ok(())
            }
        },
        Command::Validate { command } => match command {
            ValidateCommand::Csv {
                domain,
                input,
                source_id,
                json,
            } => {
                let domain_code = sdtm_model::DomainCode::new(domain.clone())?;
                let source_id = source_id.unwrap_or_else(|| input.to_string_lossy().to_string());

                let table = sdtm_ingest::csv_ingest::ingest_csv_file(
                    domain_code,
                    &input,
                    sdtm_ingest::csv_ingest::CsvIngestOptions::new(source_id),
                )?;

                let validator =
                    sdtm_validate::StandardsValidator::from_standards_dir(&cli.standards_dir)?;
                let report = validator.validate_with_report(&[table]);

                if let Some(json) = json {
                    let out = serde_json::to_string_pretty(&report)?;
                    if json == "-" {
                        println!("{}", out);
                    } else {
                        std::fs::write(&json, out)?;
                        println!("wrote {}", json);
                    }
                } else {
                    println!(
                        "{}: conformance issues (errors={}, warnings={})",
                        domain, report.errors, report.warnings
                    );
                }

                if report.errors > 0 {
                    return Err(anyhow::anyhow!("validation failed"));
                }
                Ok(())
            }
        },
    }
}
