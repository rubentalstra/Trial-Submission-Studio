use std::cmp::Ordering;
use std::path::PathBuf;

use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, Table, Width,
};

use sdtm_model::IssueSeverity;

use crate::{DomainSummary, StudyResult};

pub fn print_summary(result: &StudyResult) {
    println!("Study: {}", result.study_id);
    println!("Output: {}", result.output_dir.display());
    if let Some(path) = &result.define_xml {
        println!("Define-XML: {}", path.display());
    }
    if let Some(path) = &result.conformance_report {
        println!("Conformance report: {}", path.display());
    }
    let mut table = Table::new();
    table.set_header(vec![
        "Domain",
        "Description",
        "Records",
        "XPT",
        "XML",
        "SAS",
        "Errors",
        "Warnings",
    ]);
    apply_table_style(&mut table);
    align_column(&mut table, 2, CellAlignment::Right);
    align_column(&mut table, 3, CellAlignment::Center);
    align_column(&mut table, 4, CellAlignment::Center);
    align_column(&mut table, 5, CellAlignment::Center);
    align_column(&mut table, 6, CellAlignment::Right);
    align_column(&mut table, 7, CellAlignment::Right);
    let ordered = ordered_summaries(&result.domains);
    let mut total_records = 0usize;
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    for summary in ordered {
        let (errors, warnings) = match &summary.conformance {
            Some(report) => (Some(report.error_count()), Some(report.warning_count())),
            None => (None, None),
        };
        total_records += summary.records;
        if let Some(count) = errors {
            total_errors += count;
        }
        if let Some(count) = warnings {
            total_warnings += count;
        }
        table.add_row(vec![
            Cell::new(summary.domain_code.clone()),
            Cell::new(summary.description.clone()),
            Cell::new(summary.records),
            output_cell(summary.outputs.xpt.as_ref()),
            output_cell(summary.outputs.dataset_xml.as_ref()),
            output_cell(summary.outputs.sas.as_ref()),
            count_cell(errors, Color::Red),
            count_cell(warnings, Color::Yellow),
        ]);
    }
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new("All domains").add_attribute(Attribute::Bold),
        Cell::new(total_records).add_attribute(Attribute::Bold),
        dim_cell("-"),
        dim_cell("-"),
        dim_cell("-"),
        count_cell(Some(total_errors), Color::Red).add_attribute(Attribute::Bold),
        count_cell(Some(total_warnings), Color::Yellow).add_attribute(Attribute::Bold),
    ]);
    println!("{table}");
    print_issue_table(result);
    if !result.errors.is_empty() {
        eprintln!("Errors:");
        for error in &result.errors {
            eprintln!("- {error}");
        }
    }
}

fn print_issue_table(result: &StudyResult) {
    let mut issues = Vec::new();
    for summary in &result.domains {
        let Some(report) = summary.conformance.as_ref() else {
            continue;
        };
        for issue in &report.issues {
            issues.push((summary.domain_code.clone(), issue.clone()));
        }
    }
    if issues.is_empty() {
        return;
    }
    issues.sort_by(|a, b| {
        let severity = severity_rank(b.1.severity).cmp(&severity_rank(a.1.severity));
        if severity != Ordering::Equal {
            return severity;
        }
        let domain = a.0.cmp(&b.0);
        if domain != Ordering::Equal {
            return domain;
        }
        a.1.code.cmp(&b.1.code)
    });
    let mut table = Table::new();
    table.set_header(vec![
        "Domain", "Severity", "Variable", "Code", "Count", "Rule", "Category", "Message",
        "Examples",
    ]);
    apply_table_style(&mut table);
    align_column(&mut table, 4, CellAlignment::Right);
    for (domain, issue) in issues {
        let (message, examples) = split_examples(&issue.message);
        table.add_row(vec![
            Cell::new(domain),
            severity_cell(issue.severity),
            Cell::new(issue.variable.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(issue.code.clone()),
            Cell::new(
                issue
                    .count
                    .map(|v| v.to_string())
                    .unwrap_or("-".to_string()),
            ),
            Cell::new(issue.rule_id.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(issue.category.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(message),
            example_cell(examples),
        ]);
    }
    println!();
    println!("Issues:");
    println!("{table}");
}

fn output_cell(path: Option<&PathBuf>) -> Cell {
    match path {
        Some(_) => Cell::new("yes").fg(Color::Green),
        None => dim_cell("-"),
    }
}

fn count_cell(count: Option<usize>, color: Color) -> Cell {
    match count {
        Some(value) => Cell::new(value).fg(color),
        None => dim_cell("-"),
    }
}

pub fn apply_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(120);
    if table.column_count() >= 8 {
        table.set_constraints(vec![
            ColumnConstraint::UpperBoundary(Width::Fixed(12)),
            ColumnConstraint::UpperBoundary(Width::Percentage(35)),
            ColumnConstraint::LowerBoundary(Width::Fixed(6)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(6)),
            ColumnConstraint::LowerBoundary(Width::Fixed(8)),
        ]);
    }
}

fn align_column(table: &mut Table, index: usize, alignment: CellAlignment) {
    if let Some(column) = table.column_mut(index) {
        column.set_cell_alignment(alignment);
    }
}

fn ordered_summaries<'a>(summaries: &'a [DomainSummary]) -> Vec<&'a DomainSummary> {
    let mut ordered: Vec<&DomainSummary> = summaries.iter().collect();
    ordered.sort_by(|a, b| summary_sort_key(&a.domain_code).cmp(&summary_sort_key(&b.domain_code)));
    ordered
}

fn summary_sort_key(code: &str) -> (String, u8, String) {
    let upper = code.to_uppercase();
    let is_supp = upper.starts_with("SUPP");
    let base = if is_supp {
        upper.trim_start_matches("SUPP").to_string()
    } else {
        upper.clone()
    };
    (base.clone(), if is_supp { 1 } else { 0 }, upper)
}

fn severity_cell(severity: IssueSeverity) -> Cell {
    match severity {
        IssueSeverity::Reject => Cell::new("reject")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold),
        IssueSeverity::Error => Cell::new("error").fg(Color::Red),
        IssueSeverity::Warning => Cell::new("warning").fg(Color::Yellow),
    }
}

fn severity_rank(severity: IssueSeverity) -> u8 {
    match severity {
        IssueSeverity::Reject => 3,
        IssueSeverity::Error => 2,
        IssueSeverity::Warning => 1,
    }
}

fn split_examples(message: &str) -> (String, String) {
    match message.rsplit_once(" examples: ") {
        Some((head, tail)) => (head.to_string(), tail.to_string()),
        None => (message.to_string(), "-".to_string()),
    }
}

fn example_cell(value: String) -> Cell {
    if value == "-" {
        dim_cell(value)
    } else {
        Cell::new(value)
    }
}

fn dim_cell<T: ToString>(value: T) -> Cell {
    Cell::new(value).fg(Color::DarkGrey)
}
