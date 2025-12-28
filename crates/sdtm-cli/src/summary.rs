use std::cmp::Ordering;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::{UTF8_FULL, UTF8_FULL_CONDENSED};
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, Table, Width,
};

use sdtm_model::Severity;

use crate::types::{DomainDataCheck, DomainSummary, StudyResult};

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
    print_data_checks(result);
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
        header_cell("CT"),
        header_cell("Code"),
        header_cell("Message"),
        header_cell("Values"),
    ]);
    apply_issue_table_style(&mut table);
    align_column(&mut table, 1, CellAlignment::Center);
    align_column(&mut table, 3, CellAlignment::Center);
    align_column(&mut table, 4, CellAlignment::Center);
    for (domain, issue) in issues {
        let (message, values) = split_values(&issue.message);
        let message = highlight_count_in_message(message, issue.count, issue.severity);
        let domain_cell = domain_cell(&domain);
        table.add_row(vec![
            domain_cell,
            severity_cell(issue.severity),
            Cell::new(issue.variable.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(issue.ct_source.clone().unwrap_or_else(|| "-".to_string())),
            Cell::new(issue.code.clone()),
            Cell::new(message),
            values_cell(values),
        ]);
    }
    println!();
    println!("Issues:");
    println!("{table}");
}

fn print_data_checks(result: &StudyResult) {
    if result.data_checks.is_empty() {
        return;
    }
    let mut table = Table::new();
    table.set_header(vec![
        header_cell("Domain"),
        header_cell("CSV Rows"),
        header_cell("XPT Rows"),
        header_cell("Delta"),
    ]);
    apply_data_check_table_style(&mut table);
    align_column(&mut table, 1, CellAlignment::Right);
    align_column(&mut table, 2, CellAlignment::Right);
    align_column(&mut table, 3, CellAlignment::Right);
    for check in ordered_data_checks(&result.data_checks) {
        table.add_row(vec![
            domain_cell(&check.domain_code),
            Cell::new(check.csv_rows),
            xpt_count_cell(check.xpt_rows),
            delta_cell(check.csv_rows, check.xpt_rows),
        ]);
    }
    println!();
    println!("Data Check:");
    println!("{table}");
}

fn xpt_count_cell(count: Option<usize>) -> Cell {
    match count {
        Some(value) => Cell::new(value),
        None => dim_cell("-"),
    }
}

fn delta_cell(csv_rows: usize, xpt_rows: Option<usize>) -> Cell {
    let Some(xpt_rows) = xpt_rows else {
        return dim_cell("-");
    };
    let delta = xpt_rows as isize - csv_rows as isize;
    if delta > 0 {
        Cell::new(format!("+{delta}")).fg(Color::Green)
    } else if delta < 0 {
        Cell::new(delta).fg(Color::Red)
    } else {
        dim_cell(0)
    }
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
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(200);
}

fn apply_data_check_table_style(table: &mut Table) {
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(80);
    if table.column_count() >= 4 {
        table.set_constraints(vec![
            ColumnConstraint::UpperBoundary(Width::Fixed(10)),
            ColumnConstraint::LowerBoundary(Width::Fixed(9)),
            ColumnConstraint::LowerBoundary(Width::Fixed(9)),
            ColumnConstraint::LowerBoundary(Width::Fixed(8)),
        ]);
    }
}

fn align_column(table: &mut Table, index: usize, alignment: CellAlignment) {
    if let Some(column) = table.column_mut(index) {
        column.set_cell_alignment(alignment);
    }
}

fn ordered_summaries(summaries: &[DomainSummary]) -> Vec<&DomainSummary> {
    let mut ordered: Vec<&DomainSummary> = summaries.iter().collect();
    ordered.sort_by(|a, b| summary_sort_key(&a.domain_code).cmp(&summary_sort_key(&b.domain_code)));
    ordered
}

fn ordered_data_checks(checks: &[DomainDataCheck]) -> Vec<&DomainDataCheck> {
    let mut ordered: Vec<&DomainDataCheck> = checks.iter().collect();
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

fn severity_cell(severity: Severity) -> Cell {
    match severity {
        Severity::Reject => Cell::new("REJECT")
            .fg(Color::Red)
            .add_attribute(Attribute::Bold),
        Severity::Error => Cell::new("ERROR").fg(Color::Red),
        Severity::Warning => Cell::new("WARN").fg(Color::Yellow),
    }
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Reject => 3,
        Severity::Error => 2,
        Severity::Warning => 1,
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

fn split_values(message: &str) -> (String, String) {
    match message.rsplit_once(" values: ") {
        Some((head, tail)) => (head.to_string(), tail.to_string()),
        None => (message.to_string(), "-".to_string()),
    }
}

fn values_cell(value: String) -> Cell {
    if value == "-" {
        dim_cell(value)
    } else {
        Cell::new(value)
    }
}

fn highlight_count_in_message(message: String, count: Option<u64>, severity: Severity) -> String {
    let Some(count) = count else {
        return message;
    };
    if !io::stdout().is_terminal() {
        return message;
    }
    let count_str = count.to_string();
    let replacement = format!(
        "{}{}{}",
        ansi_severity_color(severity),
        count_str,
        ANSI_RESET
    );
    replace_first_count(&message, &count_str, &replacement).unwrap_or(message)
}

fn replace_first_count(message: &str, count: &str, replacement: &str) -> Option<String> {
    for (idx, _) in message.match_indices(count) {
        let end = idx + count.len();
        let left_ok = idx == 0
            || message[..idx]
                .chars()
                .last()
                .map(|ch| !ch.is_ascii_digit())
                .unwrap_or(true);
        let right_ok = end == message.len()
            || message[end..]
                .chars()
                .next()
                .map(|ch| !ch.is_ascii_digit())
                .unwrap_or(true);
        if left_ok && right_ok {
            let mut updated = String::new();
            updated.push_str(&message[..idx]);
            updated.push_str(replacement);
            updated.push_str(&message[end..]);
            return Some(updated);
        }
    }
    None
}

fn ansi_severity_color(severity: Severity) -> &'static str {
    match severity {
        Severity::Reject | Severity::Error => ANSI_RED,
        Severity::Warning => ANSI_YELLOW,
    }
}

const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RESET: &str = "\x1b[0m";

fn dim_cell<T: ToString>(value: T) -> Cell {
    Cell::new(value).fg(Color::DarkGrey)
}
