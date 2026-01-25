//! Study view - displayed when a study is loaded.
//!
//! Shows study info header, domain cards with progress, and export action.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::display::DomainCard;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, GeneratedDomainType, Study, WorkflowMode};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL,
    button_primary, button_secondary,
};

/// Render the study view (study loaded).
pub fn view_study<'a>(
    state: &'a AppState,
    study: &'a Study,
    workflow_mode: WorkflowMode,
) -> Element<'a, Message> {
    // Page header with study info
    let header = view_header(study, workflow_mode);

    // Study path info
    let path_info = view_path_info(study);

    // Domain cards section
    let domains_section = view_domains(state, study);

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
fn view_header<'a>(study: &'a Study, workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let total_rows = study.total_rows();
    let domain_count = study.domain_count();

    // Workflow badge
    let badge = container(
        text(workflow_mode.display_name())
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_on_accent),
            }),
    )
    .padding([4.0, 12.0])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().primary.base.color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Metadata items
    let domains_meta =
        text(format!("Domains: {}", domain_count))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            });

    let rows_meta = text(format!("Total rows: {}", format_number(total_rows)))
        .size(12)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        });

    // Close button
    let close_btn = button(
        row![
            container(lucide::x().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_secondary),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Close Project").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseProjectClicked))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_secondary);

    // Build header row
    let header_row = row![
        badge,
        Space::new().width(SPACING_SM),
        text(&study.study_id)
            .size(20)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        Space::new().width(Length::Fill),
        domains_meta,
        Space::new().width(SPACING_MD),
        rows_meta,
        Space::new().width(SPACING_MD),
        close_btn,
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    // Container with background
    container(header_row)
        .width(Length::Fill)
        .padding([SPACING_MD, SPACING_LG])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                width: 0.0,
                radius: 0.0.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Render the study path info.
fn view_path_info<'a>(study: &Study) -> Element<'a, Message> {
    let path_str = study.study_folder.display().to_string();

    row![
        container(lucide::folder().size(14)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_muted),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text(path_str).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Render the domains section with cards.
fn view_domains<'a>(state: &'a AppState, study: &'a Study) -> Element<'a, Message> {
    let domain_codes = study.domain_codes_dm_first();

    // Count source domains and generated domains that have data
    let source_count = domain_codes.len();
    let generated_with_data = GeneratedDomainType::ALL
        .iter()
        .filter(|dt| {
            study
                .domain(dt.code())
                .map(|d| d.row_count() > 0)
                .unwrap_or(false)
        })
        .count();
    let total_count = source_count + generated_with_data;

    // Section header with domain count
    let domain_count_str = format!("{}", total_count);
    let header = row![
        text("Domains").size(16).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        }),
        Space::new().width(SPACING_SM),
        container(
            text(domain_count_str)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                })
        )
        .padding([2.0, 8.0])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_inset.into()),
            border: iced::Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    // Source domain cards
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

            // Get validation summary from DomainState (persists across navigation)
            let validation = domain.validation_summary();

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

    // Generated domain cards (always show CO, RELREC, RELSPEC, RELSUB)
    let generated_section = view_generated_domains(study);

    column![
        header,
        Space::new().height(SPACING_MD),
        cards_column,
        Space::new().height(SPACING_LG),
        generated_section,
    ]
    .width(Length::Fill)
    .into()
}

/// Render the generated domains section (CO, RELREC, RELSPEC, RELSUB).
///
/// These domains are always visible so users can add data during the mapping workflow.
fn view_generated_domains<'a>(study: &'a Study) -> Element<'a, Message> {
    // Section header
    let header = row![
        text("Special-Purpose & Relationship Domains")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
    ]
    .align_y(Alignment::Center);

    let mut cards_column = column![].spacing(SPACING_SM).width(Length::Fill);

    for domain_type in GeneratedDomainType::ALL {
        let code = domain_type.code();
        let label = domain_type.label();

        // Check if domain exists with data
        let (row_count, validation) = if let Some(domain) = study.domain(code) {
            (domain.row_count(), domain.validation_summary())
        } else {
            (0, None)
        };

        // Generated domains always show 0% progress (no mapping needed)
        // or 100% if they have data
        let progress = if row_count > 0 { 1.0 } else { 0.0 };

        let card = DomainCard::new(
            code,
            label,
            Message::Home(HomeMessage::GeneratedDomainClicked(domain_type)),
        )
        .row_count(row_count)
        .progress(progress)
        .validation_opt(validation)
        .view();

        cards_column = cards_column.push(card);
    }

    column![header, Space::new().height(SPACING_SM), cards_column,]
        .width(Length::Fill)
        .into()
}

/// Render the export action button.
fn view_export_action<'a>() -> Element<'a, Message> {
    let export_btn = button(
        row![
            container(lucide::download().size(16)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
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
