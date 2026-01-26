//! Preview tab view.
//!
//! The preview tab displays a paginated data table showing the
//! transformed output data from the mapping and normalization steps.
//!
//! Features:
//! - Horizontal and vertical scrolling
//! - Dynamic column widths based on content
//! - Responsive layout that uses available space
//! - Pagination with configurable rows per page

mod helpers;
mod pagination;
mod table;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::DataFrame;

use crate::component::display::{EmptyState, ErrorState, LoadingState};
use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, PreviewUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, ThemeConfig,
    button_primary,
};

use table::view_data_table;

// =============================================================================
// MAIN PREVIEW TAB VIEW
// =============================================================================

/// Render the preview tab content.
pub fn view_preview_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let config = &state.theme_config;

    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            let theme = config.to_theme(false);
            let text_muted = theme.clinical().text_muted;
            return container(text("Domain not found").size(14).color(text_muted))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into();
        }
    };

    // Preview only applies to source domains
    let source = match domain.as_source() {
        Some(s) => s,
        None => {
            let theme = config.to_theme(false);
            let text_muted = theme.clinical().text_muted;
            return container(
                text("Generated domains do not have preview")
                    .size(14)
                    .color(text_muted),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Shrink)
            .center_y(Length::Shrink)
            .into();
        }
    };

    // Get preview UI state and cached DataFrame
    let (preview_cache, preview_ui) = match &state.view {
        ViewState::DomainEditor(editor) => (&editor.preview_cache, &editor.preview_ui),
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_preview_header(config, preview_cache.as_ref(), preview_ui);

    // Content based on state
    let content: Element<'a, Message> = if preview_ui.is_rebuilding {
        view_loading_state()
    } else if let Some(error) = &preview_ui.error {
        view_error_state(error.as_str())
    } else if let Some(df) = preview_cache {
        view_data_table(config, df, preview_ui, source)
    } else {
        view_empty_state(config)
    };

    // Header with padding, table without padding for edge-to-edge look
    let header_section =
        container(column![header, Space::new().height(SPACING_MD),]).padding(iced::Padding {
            top: SPACING_LG,
            right: SPACING_LG,
            bottom: 0.0,
            left: SPACING_LG,
        });

    column![header_section, content,]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Preview header with stats and rebuild button.
fn view_preview_header<'a>(
    config: &ThemeConfig,
    df: Option<&DataFrame>,
    preview_ui: &PreviewUiState,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_primary = theme.extended_palette().background.base.text;
    let text_secondary = theme.clinical().text_secondary;
    let text_muted = theme.clinical().text_muted;
    let text_on_accent = theme.clinical().text_on_accent;
    let bg_secondary = theme.clinical().background_secondary;

    let title = text("Data Preview").size(18).color(text_primary);

    // Stats based on DataFrame
    let stats: Element<'a, Message> = if let Some(df) = df {
        let num_cols = df.width();
        let num_rows = df.height();
        row![
            container(
                row![
                    lucide::table().size(12).color(text_muted),
                    Space::new().width(SPACING_XS),
                    text(format!("{} columns", num_cols))
                        .size(12)
                        .color(text_secondary),
                ]
                .align_y(Alignment::Center)
            )
            .padding([4.0, 8.0])
            .style(move |_: &Theme| container::Style {
                background: Some(bg_secondary.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            container(
                row![
                    lucide::list().size(12).color(text_muted),
                    Space::new().width(SPACING_XS),
                    text(format!("{} rows", num_rows))
                        .size(12)
                        .color(text_secondary),
                ]
                .align_y(Alignment::Center)
            )
            .padding([4.0, 8.0])
            .style(move |_: &Theme| container::Style {
                background: Some(bg_secondary.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        text("No data loaded").size(12).color(text_muted).into()
    };

    let rebuild_button = button(
        row![
            if preview_ui.is_rebuilding {
                lucide::loader().size(14).color(text_on_accent)
            } else {
                lucide::refresh_cw().size(14).color(text_on_accent)
            },
            Space::new().width(SPACING_SM),
            text(if preview_ui.is_rebuilding {
                "Building..."
            } else {
                "Rebuild"
            })
            .size(13)
            .color(text_on_accent),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if preview_ui.is_rebuilding {
        None
    } else {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::RebuildPreview,
        )))
    })
    .padding([8.0, 16.0])
    .style(button_primary);

    row![
        column![title, Space::new().height(SPACING_XS), stats,],
        Space::new().width(Length::Fill),
        rebuild_button,
    ]
    .align_y(Alignment::Center)
    .into()
}

// =============================================================================
// STATE VIEWS
// =============================================================================

/// Loading state while preview is being rebuilt.
fn view_loading_state<'a>() -> Element<'a, Message> {
    LoadingState::new("Building Preview")
        .description("Applying mappings and normalization rules...")
        .centered()
        .view()
}

/// Error state when preview build failed.
fn view_error_state(error: &str) -> Element<'_, Message> {
    ErrorState::new("Preview Build Failed")
        .message(error)
        .retry(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::RebuildPreview,
        )))
        .centered()
        .view()
}

/// Empty state when no preview is available.
fn view_empty_state<'a>(config: &ThemeConfig) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_disabled = theme.clinical().text_disabled;

    EmptyState::new(
        lucide::table().size(48).color(text_disabled),
        "No Preview Available",
    )
    .description("Click 'Rebuild' to generate the transformed data preview")
    .action(
        "Build Preview",
        Message::DomainEditor(DomainEditorMessage::Preview(PreviewMessage::RebuildPreview)),
    )
    .centered()
    .view()
}
