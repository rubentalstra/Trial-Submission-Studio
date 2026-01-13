//! Preview tab view.
//!
//! The preview tab displays a paginated data table showing the
//! transformed output data from the mapping and normalization steps.

use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;
use polars::prelude::DataFrame;

use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, PreviewUiState, ViewState};
use crate::theme::{
    GRAY_50, GRAY_100, GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900,
    PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, TABLE_CELL_PADDING_X, TABLE_CELL_PADDING_Y,
    WHITE, button_ghost, button_primary, button_secondary,
};

// =============================================================================
// MAIN PREVIEW TAB VIEW
// =============================================================================

/// Render the preview tab content.
pub fn view_preview_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let _domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return text("Domain not found").size(14).color(GRAY_500).into();
        }
    };

    // Get preview UI state and cached DataFrame
    let (preview_cache, preview_ui) = match &state.view {
        ViewState::DomainEditor {
            preview_cache,
            preview_ui,
            ..
        } => (preview_cache, preview_ui),
        _ => return text("Invalid view state").into(),
    };

    // Header
    let header = view_preview_header(preview_ui);

    // Content based on state
    let content: Element<'a, Message> = if preview_ui.is_rebuilding {
        view_loading_state()
    } else if let Some(error) = &preview_ui.error {
        view_error_state(error.as_str())
    } else if let Some(df) = preview_cache {
        view_data_table(df, preview_ui)
    } else {
        view_empty_state()
    };

    column![header, Space::new().height(SPACING_MD), content,]
        .spacing(0)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Preview header with stats and rebuild button.
fn view_preview_header<'a>(preview_ui: &PreviewUiState) -> Element<'a, Message> {
    let title = text("Data Preview").size(18).color(GRAY_900);

    let subtitle = text("Preview of transformed SDTM output data")
        .size(13)
        .color(GRAY_600);

    let rebuild_button = button(
        row![
            if preview_ui.is_rebuilding {
                lucide::loader().size(12)
            } else {
                lucide::refresh_cw().size(12)
            },
            text(if preview_ui.is_rebuilding {
                "Building..."
            } else {
                "Rebuild Preview"
            })
            .size(14),
        ]
        .spacing(SPACING_SM)
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
        column![title, Space::new().height(4.0), subtitle,],
        Space::new().width(Length::Fill),
        rebuild_button,
    ]
    .align_y(Alignment::Start)
    .into()
}

// =============================================================================
// DATA TABLE VIEW
// =============================================================================

/// Display the actual data table from DataFrame.
fn view_data_table<'a>(df: &DataFrame, preview_ui: &PreviewUiState) -> Element<'a, Message> {
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let total_rows = df.height();
    let page = preview_ui.current_page;
    let page_size = preview_ui.rows_per_page;

    // Build column widths
    let col_widths: Vec<f32> = col_names
        .iter()
        .map(|name| match name.as_str() {
            "USUBJID" => 150.0,
            "STUDYID" => 120.0,
            "DOMAIN" => 60.0,
            _ if name.ends_with("DTC") => 120.0,
            _ if name.ends_with("DY") => 80.0,
            _ if name.ends_with("SEQ") => 80.0,
            _ => 100.0,
        })
        .collect();

    // Header row
    let header_row = {
        let mut header = row![].spacing(0);
        for (name, &width) in col_names.iter().zip(col_widths.iter()) {
            // Clone the name to move into the widget
            let name_owned = name.clone();
            header = header.push(
                container(text(name_owned).size(12).color(GRAY_600))
                    .width(Length::Fixed(width))
                    .padding([TABLE_CELL_PADDING_Y, TABLE_CELL_PADDING_X])
                    .style(|_theme| container::Style {
                        background: Some(GRAY_100.into()),
                        ..Default::default()
                    }),
            );
        }
        header
    };

    // Calculate visible rows
    let start = page * page_size;
    let end = (start + page_size).min(total_rows);

    // Build data rows
    let mut data_rows = column![].spacing(0);
    for row_idx in start..end {
        let mut data_row = row![].spacing(0);
        for (col_idx, col) in df.get_columns().iter().enumerate() {
            let value = col
                .get(row_idx)
                .map_or_else(|_| String::new(), |v| format_anyvalue(&v));
            let width = col_widths.get(col_idx).copied().unwrap_or(100.0);
            let is_even = (row_idx - start) % 2 == 0;

            data_row = data_row.push(
                container(text(value).size(13).color(GRAY_700))
                    .width(Length::Fixed(width))
                    .padding([TABLE_CELL_PADDING_Y, TABLE_CELL_PADDING_X])
                    .style(move |_theme| container::Style {
                        background: Some(if is_even { WHITE } else { GRAY_50 }.into()),
                        ..Default::default()
                    }),
            );
        }
        data_rows = data_rows.push(data_row);
    }

    // Pagination
    let total_pages = if total_rows == 0 {
        1
    } else {
        (total_rows + page_size - 1) / page_size
    };

    let prev_enabled = page > 0;
    let next_enabled = page < total_pages.saturating_sub(1);

    let prev_button = button(lucide::chevron_left().size(14).color(if prev_enabled {
        GRAY_700
    } else {
        GRAY_500
    }))
    .on_press_maybe(if prev_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page - 1),
        )))
    } else {
        None
    })
    .padding([4.0, 10.0])
    .style(button_ghost);

    let next_button = button(lucide::chevron_right().size(14).color(if next_enabled {
        GRAY_700
    } else {
        GRAY_500
    }))
    .on_press_maybe(if next_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page + 1),
        )))
    } else {
        None
    })
    .padding([4.0, 10.0])
    .style(button_ghost);

    let page_info = text(format!(
        "Page {} of {} ({} rows)",
        page + 1,
        total_pages,
        total_rows
    ))
    .size(12)
    .color(GRAY_600);

    let pagination = row![
        Space::new().width(Length::Fill),
        prev_button,
        page_info,
        next_button,
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    // Stats bar
    let num_columns = col_names.len();
    let stats = row![
        text(format!("{} columns", num_columns))
            .size(12)
            .color(GRAY_600),
        text("•").size(12).color(GRAY_400),
        text(format!("{} rows", total_rows))
            .size(12)
            .color(GRAY_600),
        Space::new().width(Length::Fill),
        text("Rows per page:").size(12).color(GRAY_500),
        view_rows_per_page_selector(preview_ui.rows_per_page),
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    // Table container
    let table: Element<'a, Message> = container(
        column![
            header_row,
            rule::horizontal(1).style(|_theme| rule::Style {
                color: GRAY_200,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }),
            scrollable(data_rows).height(Length::Fill),
            rule::horizontal(1).style(|_theme| rule::Style {
                color: GRAY_200,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }),
            container(pagination).padding(SPACING_SM),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        border: Border {
            width: 1.0,
            radius: 4.0.into(),
            color: GRAY_200,
        },
        ..Default::default()
    })
    .into();

    column![stats, Space::new().height(SPACING_SM), table,].into()
}

/// Format a Polars AnyValue for display.
fn format_anyvalue(value: &polars::prelude::AnyValue) -> String {
    use polars::prelude::AnyValue;

    match value {
        AnyValue::Null => String::new(),
        AnyValue::Boolean(b) => if *b { "Y" } else { "N" }.to_string(),
        AnyValue::Int8(n) => n.to_string(),
        AnyValue::Int16(n) => n.to_string(),
        AnyValue::Int32(n) => n.to_string(),
        AnyValue::Int64(n) => n.to_string(),
        AnyValue::UInt8(n) => n.to_string(),
        AnyValue::UInt16(n) => n.to_string(),
        AnyValue::UInt32(n) => n.to_string(),
        AnyValue::UInt64(n) => n.to_string(),
        AnyValue::Float32(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                format!("{:.4}", n)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        }
        AnyValue::Float64(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                format!("{:.4}", n)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        }
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value),
    }
}

/// Rows per page selector buttons.
fn view_rows_per_page_selector<'a>(current: usize) -> Element<'a, Message> {
    let options = [25, 50, 100];

    let buttons: Vec<Element<'a, Message>> = options
        .iter()
        .map(|&n| {
            let is_selected = current == n;
            button(text(format!("{}", n)).size(11))
                .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                    PreviewMessage::RowsPerPageChanged(n),
                )))
                .padding([4.0, 8.0])
                .style(move |_theme, _status| {
                    if is_selected {
                        iced::widget::button::Style {
                            background: Some(PRIMARY_500.into()),
                            text_color: iced::Color::WHITE,
                            border: Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    } else {
                        button_secondary(_theme, _status)
                    }
                })
                .into()
        })
        .collect();

    row(buttons).spacing(4.0).into()
}

// =============================================================================
// STATES
// =============================================================================

/// Loading state while preview is being rebuilt.
fn view_loading_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::loader().size(32).color(PRIMARY_500),
            Space::new().height(SPACING_MD),
            text("Building Preview...").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Applying mappings and normalization rules")
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

/// Error state when preview build failed.
fn view_error_state(error: &str) -> Element<'_, Message> {
    let error_text = error.to_string();

    container(
        column![
            lucide::circle_alert().size(48).color(crate::theme::ERROR),
            Space::new().height(SPACING_MD),
            text("Preview Build Failed").size(16).color(GRAY_800),
            Space::new().height(SPACING_SM),
            text(error_text).size(13).color(GRAY_600),
            Space::new().height(SPACING_LG),
            button(
                row![lucide::refresh_cw().size(12), text("Retry").size(14),]
                    .spacing(SPACING_SM)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RebuildPreview,
            )))
            .padding([10.0, 20.0])
            .style(button_primary),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
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

/// Empty state when no preview is available.
fn view_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::database().size(48).color(GRAY_400),
            Space::new().height(SPACING_MD),
            text("No Preview Available").size(16).color(GRAY_600),
            Space::new().height(SPACING_SM),
            text("Click 'Rebuild Preview' to generate the output preview")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            button(
                row![
                    lucide::refresh_cw().size(12),
                    text("Rebuild Preview").size(14),
                ]
                .spacing(SPACING_SM)
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RebuildPreview,
            )))
            .padding([10.0, 20.0])
            .style(button_primary),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0))
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}
