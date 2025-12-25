use std::cmp::Ordering;
use std::path::PathBuf;

use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::{UTF8_FULL, UTF8_FULL_CONDENSED};
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, Table, Width,
};

use sdtm_model::IssueSeverity;

use crate::types::{DomainSummary, StudyResult};

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
        header_cell("Domain"),
        header_cell("Description"),
        header_cell("Records"),
        header_cell("XPT"),
        header_cell("XML"),
        header_cell("SAS"),
        header_cell("Errors"),
        header_cell("Warnings"),
    ]);
    apply_summary_table_style(&mut table);
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
        let domain_cell = domain_cell(&summary.domain_code);
        let description_cell =
            description_cell(&summary.description, is_supp_domain(&summary.domain_code));
        table.add_row(vec![
            domain_cell,
            description_cell,
            Cell::new(summary.records),
            output_cell(summary.outputs.xpt.as_ref()),
            output_cell(summary.outputs.dataset_xml.as_ref()),
            output_cell(summary.outputs.sas.as_ref()),
            count_cell(errors, Color::Red),
            count_cell(warnings, Color::Yellow),
        ]);
    }
    table.add_row(vec![
        Cell::new("TOTAL")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("All domains")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
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
        header_cell("Domain"),
        header_cell("Severity"),
        header_cell("Variable"),
        header_cell("Code"),
        header_cell("Count"),
        header_cell("Rule"),
        header_cell("Category"),
        header_cell("Message"),
        header_cell("Examples"),
    ]);
    apply_issue_table_style(&mut table);
    align_column(&mut table, 1, CellAlignment::Center);
    align_column(&mut table, 3, CellAlignment::Center);
    align_column(&mut table, 4, CellAlignment::Right);
    for (domain, issue) in issues {
        let (message, examples) = split_examples(&issue.message);
        let count_cell = issue_count_cell(issue.count, issue.severity);
        let domain_cell = domain_cell(&domain);
        table.add_row(vec![
            domain_cell,
            severity_cell(issue.severity),
            Cell::new(issue.variable.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(issue.code.clone()),
            count_cell,
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
        Some(_) => Cell::new("✓")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
        None => dim_cell("-"),
    }
}

fn count_cell(count: Option<usize>, color: Color) -> Cell {
    match count {
        Some(value) if value > 0 => Cell::new(value).fg(color).add_attribute(Attribute::Bold),
        Some(value) => dim_cell(value),
        None => dim_cell("-"),
    }
}

pub fn apply_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
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

fn apply_summary_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(165);
    if table.column_count() >= 8 {
        table.set_constraints(vec![
            ColumnConstraint::UpperBoundary(Width::Fixed(10)),
            ColumnConstraint::UpperBoundary(Width::Percentage(45)),
            ColumnConstraint::LowerBoundary(Width::Fixed(7)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::LowerBoundary(Width::Fixed(7)),
            ColumnConstraint::LowerBoundary(Width::Fixed(9)),
        ]);
    }
}

fn apply_issue_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(200);
    if table.column_count() >= 9 {
        table.set_constraints(vec![
            ColumnConstraint::UpperBoundary(Width::Fixed(10)),
            ColumnConstraint::UpperBoundary(Width::Fixed(9)),
            ColumnConstraint::UpperBoundary(Width::Fixed(12)),
            ColumnConstraint::UpperBoundary(Width::Fixed(10)),
            ColumnConstraint::LowerBoundary(Width::Fixed(5)),
            ColumnConstraint::UpperBoundary(Width::Fixed(10)),
            ColumnConstraint::UpperBoundary(Width::Fixed(12)),
            ColumnConstraint::UpperBoundary(Width::Percentage(45)),
            ColumnConstraint::UpperBoundary(Width::Percentage(30)),
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
        IssueSeverity::Reject => Cell::new("REJECT")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold),
        IssueSeverity::Error => Cell::new("ERROR").fg(Color::Red),
        IssueSeverity::Warning => Cell::new("WARN").fg(Color::Yellow),
    }
}

fn issue_count_cell(count: Option<u64>, severity: IssueSeverity) -> Cell {
    match count {
        Some(value) => Cell::new(value).fg(severity_color(severity)),
        None => dim_cell("-"),
    }
}

fn severity_rank(severity: IssueSeverity) -> u8 {
    match severity {
        IssueSeverity::Reject => 3,
        IssueSeverity::Error => 2,
        IssueSeverity::Warning => 1,
    }
}

fn severity_color(severity: IssueSeverity) -> Color {
    match severity {
        IssueSeverity::Reject => Color::Red,
        IssueSeverity::Error => Color::Red,
        IssueSeverity::Warning => Color::Yellow,
    }
}

fn header_cell(label: &str) -> Cell {
    Cell::new(label)
        .fg(Color::Cyan)
        .add_attribute(Attribute::Bold)
}

fn is_supp_domain(code: &str) -> bool {
    let upper = code.to_uppercase();
    upper.starts_with("SUPP") && upper.len() > 4
}

fn domain_cell(code: &str) -> Cell {
    if is_supp_domain(code) {
        Cell::new(format!("  -> {}", code)).fg(Color::DarkGrey)
    } else {
        Cell::new(code)
            .fg(Color::Blue)
            .add_attribute(Attribute::Bold)
    }
}

fn description_cell(description: &str, is_supp: bool) -> Cell {
    if is_supp {
        Cell::new(description).fg(Color::DarkGrey)
    } else {
        Cell::new(description)
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
