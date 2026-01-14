//! Study view - displayed when a study is loaded.
//!
//! Shows study info header, domain cards with progress, and export action.

use std::collections::HashMap;

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};
use iced_fonts::lucide;

use crate::component::{DomainCard, PageHeader};
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, Study, WorkflowMode};
use crate::theme::{
    GRAY_500, GRAY_600, GRAY_700, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL,
    WHITE, button_primary, button_secondary,
};

/// Render the study view (study loaded).
pub fn view_study<'a>(
    state: &'a AppState,
    study: &'a Study,
    workflow_mode: WorkflowMode,
    validation_summaries: &'a HashMap<String, (usize, usize)>,
) -> Element<'a, Message> {
    // Page header with study info
    let header = view_header(study, workflow_mode);

    // Study path info
    let path_info = view_path_info(study);

    // Domain cards section
    let domains_section = view_domains(state, study, validation_summaries);

    // Export button
    let export_section = view_export_action();

    // Build the main content
    let content = column![
        path_info,
        Space::new().height(SPACING_LG),
        domains_section,
        Space::new().height(SPACING_XL),
        export_section,
        Space::new().height(SPACING_LG),
    ]
    .padding(iced::Padding {
        top: 0.0,
        right: SPACING_LG,
        bottom: SPACING_LG,
        left: SPACING_LG,
    })
    .width(Length::Fill);

    // Scrollable content area
    let scrollable_content = scrollable(content).height(Length::Fill);

    // Full page layout
    column![header, scrollable_content,]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the page header with study info.
fn view_header<'a>(study: &Study, workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let total_rows = study.total_rows();
    let domain_count = study.domain_count();

    // Close button
    let close_btn = button(
        row![
            lucide::x().size(14).color(GRAY_700),
            Space::new().width(SPACING_SM),
            text("Close Study").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseStudyClicked))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_secondary);

    PageHeader::new(&study.study_id)
        .badge(workflow_mode.display_name(), PRIMARY_500)
        .meta("Domains", domain_count.to_string())
        .meta("Total rows", format_number(total_rows))
        .trailing(close_btn)
        .view()
}

/// Render the study path info.
fn view_path_info<'a>(study: &Study) -> Element<'a, Message> {
    let path_str = study.study_folder.display().to_string();

    row![
        lucide::folder().size(14).color(GRAY_500),
        Space::new().width(SPACING_SM),
        text(path_str).size(12).color(GRAY_500),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Render the domains section with cards.
fn view_domains<'a>(
    state: &'a AppState,
    study: &'a Study,
    validation_summaries: &'a HashMap<String, (usize, usize)>,
) -> Element<'a, Message> {
    let domain_codes = study.domain_codes_dm_first();

    // Section header with domain count
    let header = row![
        text("Domains").size(16).color(GRAY_700),
        Space::new().width(SPACING_SM),
        container(
            text(format!("{}", domain_codes.len()))
                .size(12)
                .color(GRAY_600)
        )
        .padding([2.0, 8.0])
        .style(|_| container::Style {
            background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.05).into()),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    // Domain cards
    let mut cards_column = column![].spacing(SPACING_SM).width(Length::Fill);

    for code in domain_codes {
        if let Some(domain) = state.domain(code) {
            let summary = domain.summary();
            let row_count = domain.row_count();

            // Calculate progress
            let progress = if summary.total_variables > 0 {
                summary.mapped as f32 / summary.total_variables as f32
            } else {
                0.0
            };

            // Get validation summary (if available)
            let validation = validation_summaries.get(code).copied();

            // Get display name from domain
            let display_name = domain.display_name(code);

            let card = DomainCard::new(
                code,
                display_name,
                Message::Home(HomeMessage::DomainClicked(code.to_string())),
            )
            .row_count(row_count)
            .progress(progress)
            .validation_opt(validation)
            .view();

            cards_column = cards_column.push(card);
        }
    }

    column![header, Space::new().height(SPACING_MD), cards_column,]
        .width(Length::Fill)
        .into()
}

/// Render the export action button.
fn view_export_action<'a>() -> Element<'a, Message> {
    let export_btn = button(
        row![
            lucide::download().size(16).color(WHITE),
            Space::new().width(SPACING_SM),
            text("Go to Export").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::GoToExportClicked))
    .padding([SPACING_SM, SPACING_LG])
    .style(button_primary);

    container(export_btn)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Format a number with thousands separators.
fn format_number(n: usize) -> String {
    if n < 1000 {
        return n.to_string();
    }

    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    result
}
