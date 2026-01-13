//! SUPP (Supplemental Qualifiers) tab view.
//!
//! The SUPP tab allows configuration of supplemental qualifier domains
//! for columns that don't map to standard SDTM variables.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length};

use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, ViewState};
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM, SUCCESS,
    button_primary,
};

// =============================================================================
// MAIN SUPP TAB VIEW
// =============================================================================

/// Render the SUPP configuration tab content.
pub fn view_supp_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Get UI state
    let _supp_ui = match &state.view {
        ViewState::DomainEditor { supp_ui, .. } => supp_ui,
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_supp_header(domain_code);

    // Content placeholder
    let source_columns = domain.source.column_names();
    let content = if source_columns.is_empty() {
        view_no_columns_state()
    } else {
        view_supp_placeholder(source_columns.len())
    };

    column![header, Space::new().height(SPACING_MD), content,]
        .spacing(0)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// SUPP tab header.
fn view_supp_header<'a>(domain_code: &str) -> Element<'a, Message> {
    let title = text("Supplemental Qualifiers").size(18).color(GRAY_900);

    let subtitle = text(format!(
        "Configure SUPP{} domain for unmapped columns",
        domain_code
    ))
    .size(13)
    .color(GRAY_600);

    let refresh_button = button(
        row![
            text("\u{f021}") // refresh icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(12),
            text("Refresh").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::CancelEditing,
    )))
    .padding([8.0, 16.0])
    .style(button_primary);

    row![
        column![title, Space::new().height(4.0), subtitle,],
        Space::new().width(Length::Fill),
        refresh_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

// =============================================================================
// STATES
// =============================================================================

/// Empty state when there are no columns.
fn view_no_columns_state<'a>() -> Element<'a, Message> {
    container(
        column![
            text("\u{f058}") // check-circle
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(48)
                .color(SUCCESS),
            Space::new().height(SPACING_MD),
            text("All Columns Mapped").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("There are no unmapped columns that need SUPP configuration")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

/// Placeholder SUPP content.
fn view_supp_placeholder<'a>(column_count: usize) -> Element<'a, Message> {
    container(
        column![
            text("\u{f1c0}") // database icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(48)
                .color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("SUPP Configuration").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text(format!("{} source columns available", column_count))
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            text("Available SUPP fields:").size(13).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("• QNAM - Qualifier Variable Name (max 8 chars)")
                .size(12)
                .color(GRAY_500),
            text("• QLABEL - Qualifier Variable Label (max 40 chars)")
                .size(12)
                .color(GRAY_500),
            text("• QORIG - Origin (CRF, Derived, Assigned)")
                .size(12)
                .color(GRAY_500),
            text("• QEVAL - Evaluator (optional)")
                .size(12)
                .color(GRAY_500),
            Space::new().height(SPACING_MD),
            text("Full SUPP configuration interface coming soon")
                .size(11)
                .color(GRAY_400),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(400.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .style(|_theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
