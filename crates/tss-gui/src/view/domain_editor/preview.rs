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

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::DataFrame;

use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, PreviewUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, GRAY_100, GRAY_200, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700,
    GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
    WHITE, button_ghost, button_primary,
};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Minimum column width
const MIN_COL_WIDTH: f32 = 60.0;

/// Maximum column width
const MAX_COL_WIDTH: f32 = 300.0;

/// Padding inside cells
const CELL_PADDING_X: f32 = 12.0;
const CELL_PADDING_Y: f32 = 8.0;

/// Approximate character width for calculating column widths
const CHAR_WIDTH: f32 = 7.5;

// =============================================================================
// MAIN PREVIEW TAB VIEW
// =============================================================================

/// Render the preview tab content.
pub fn view_preview_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let _domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(14).color(GRAY_500))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into();
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
    let header = view_preview_header(preview_cache.as_ref(), preview_ui);

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
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// =============================================================================
// HEADER
// =============================================================================

/// Preview header with stats and rebuild button.
fn view_preview_header<'a>(
    df: Option<&DataFrame>,
    preview_ui: &PreviewUiState,
) -> Element<'a, Message> {
    let title = text("Data Preview").size(18).color(GRAY_900);

    // Stats based on DataFrame
    let stats: Element<'a, Message> = if let Some(df) = df {
        let num_cols = df.width();
        let num_rows = df.height();
        row![
            container(
                row![
                    lucide::table().size(12).color(GRAY_500),
                    Space::new().width(SPACING_XS),
                    text(format!("{} columns", num_cols))
                        .size(12)
                        .color(GRAY_600),
                ]
                .align_y(Alignment::Center)
            )
            .padding([4.0, 8.0])
            .style(|_: &Theme| container::Style {
                background: Some(GRAY_100.into()),
                border: Border {
                    radius: BORDER_RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            container(
                row![
                    lucide::list().size(12).color(GRAY_500),
                    Space::new().width(SPACING_XS),
                    text(format!("{} rows", num_rows)).size(12).color(GRAY_600),
                ]
                .align_y(Alignment::Center)
            )
            .padding([4.0, 8.0])
            .style(|_: &Theme| container::Style {
                background: Some(GRAY_100.into()),
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
        text("No data loaded").size(12).color(GRAY_500).into()
    };

    let rebuild_button = button(
        row![
            if preview_ui.is_rebuilding {
                lucide::loader().size(14).color(WHITE)
            } else {
                lucide::refresh_cw().size(14).color(WHITE)
            },
            Space::new().width(SPACING_SM),
            text(if preview_ui.is_rebuilding {
                "Building..."
            } else {
                "Rebuild"
            })
            .size(13)
            .color(WHITE),
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
// DATA TABLE VIEW
// =============================================================================

/// Display the actual data table from DataFrame with horizontal scrolling.
fn view_data_table<'a>(df: &DataFrame, preview_ui: &PreviewUiState) -> Element<'a, Message> {
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let total_rows = df.height();
    let page = preview_ui.current_page;
    let page_size = preview_ui.rows_per_page;

    // Calculate column widths based on content
    let col_widths = calculate_column_widths(df, &col_names);

    // Calculate visible rows
    let start = page * page_size;
    let end = (start + page_size).min(total_rows);

    // Build the complete table (header + data)
    let table_content = build_table_content(df, &col_names, &col_widths, start, end);

    // Pagination controls
    let pagination = view_pagination(page, total_rows, page_size);

    // Rows per page selector
    let rows_selector = view_rows_per_page_selector(preview_ui.rows_per_page);

    // Bottom bar with pagination and rows selector
    let bottom_bar = container(
        row![
            rows_selector,
            Space::new().width(Length::Fill),
            pagination,
            Space::new().width(Length::Fill),
            // Spacer to balance the rows selector
            Space::new().width(150.0),
        ]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_MD]),
    )
    .style(|_: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            color: GRAY_200,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    });

    // Main table container with border
    let table_container = container(
        column![
            // Scrollable table area (both horizontal and vertical)
            scrollable(
                scrollable(table_content)
                    .direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::new().width(8).scroller_width(6),
                    ))
                    .width(Length::Shrink)
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(8).scroller_width(6),
            ))
            .height(Length::Fill)
            .width(Length::Fill),
            // Bottom bar
            bottom_bar,
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &Theme| container::Style {
        border: Border {
            color: GRAY_200,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    });

    table_container.into()
}

/// Calculate optimal column widths based on header and data content.
fn calculate_column_widths(df: &DataFrame, col_names: &[String]) -> Vec<f32> {
    col_names
        .iter()
        .enumerate()
        .map(|(_col_idx, name)| {
            // Start with header width
            let header_width = (name.len() as f32 * CHAR_WIDTH) + (CELL_PADDING_X * 2.0);

            // Sample some data values to find max width
            let mut max_data_width: f32 = 0.0;
            if let Ok(col) = df.column(name) {
                // Sample first 50 rows for width calculation
                let sample_count = col.len().min(50);
                for i in 0..sample_count {
                    if let Ok(val) = col.get(i) {
                        let val_str = format_anyvalue(&val);
                        let val_width =
                            (val_str.len() as f32 * CHAR_WIDTH) + (CELL_PADDING_X * 2.0);
                        max_data_width = max_data_width.max(val_width);
                    }
                }
            }

            // Use the larger of header or data width, clamped to min/max
            let width = header_width.max(max_data_width);

            // Apply special rules for known column types
            let adjusted_width = match name.as_str() {
                "STUDYID" => width.max(100.0),
                "USUBJID" => width.max(140.0),
                "DOMAIN" => width.max(70.0).min(80.0),
                _ if name.ends_with("SEQ") => width.max(70.0).min(90.0),
                _ if name.ends_with("DY") => width.max(70.0).min(90.0),
                _ if name.ends_with("DTC") => width.max(110.0),
                _ => width,
            };

            adjusted_width.clamp(MIN_COL_WIDTH, MAX_COL_WIDTH)
        })
        .collect()
}

/// Build the table content (header row + data rows).
fn build_table_content<'a>(
    df: &DataFrame,
    col_names: &[String],
    col_widths: &[f32],
    start: usize,
    end: usize,
) -> Element<'a, Message> {
    // Header row
    let header_row = build_header_row(col_names, col_widths);

    // Data rows
    let mut data_rows = column![].spacing(0);
    for row_idx in start..end {
        let is_even = (row_idx - start) % 2 == 0;
        data_rows = data_rows.push(build_data_row(df, col_widths, row_idx, is_even));
    }

    column![header_row, data_rows,].width(Length::Shrink).into()
}

/// Build the header row.
fn build_header_row<'a>(col_names: &[String], col_widths: &[f32]) -> Element<'a, Message> {
    let mut header = row![].spacing(0);

    for (name, &width) in col_names.iter().zip(col_widths.iter()) {
        let cell = container(
            text(name.clone())
                .size(12)
                .color(GRAY_700)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
        )
        .width(Length::Fixed(width))
        .padding([CELL_PADDING_Y, CELL_PADDING_X])
        .style(|_: &Theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                color: GRAY_200,
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        });

        header = header.push(cell);
    }

    container(header)
        .style(|_: &Theme| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

/// Build a single data row.
fn build_data_row<'a>(
    df: &DataFrame,
    col_widths: &[f32],
    row_idx: usize,
    is_even: bool,
) -> Element<'a, Message> {
    let mut data_row = row![].spacing(0);
    let bg_color = if is_even { WHITE } else { GRAY_100 };

    for (col_idx, col) in df.get_columns().iter().enumerate() {
        let value = col
            .get(row_idx)
            .map_or_else(|_| String::new(), |v| format_anyvalue(&v));
        let width = col_widths.get(col_idx).copied().unwrap_or(100.0);

        // Check if value is empty/null for styling
        let text_color = if value.is_empty() { GRAY_400 } else { GRAY_800 };
        let display_value = if value.is_empty() {
            "—".to_string()
        } else {
            value
        };

        let cell = container(text(display_value).size(13).color(text_color))
            .width(Length::Fixed(width))
            .padding([CELL_PADDING_Y, CELL_PADDING_X])
            .style(move |_: &Theme| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    color: GRAY_200,
                    width: 0.5,
                    ..Default::default()
                },
                ..Default::default()
            });

        data_row = data_row.push(cell);
    }

    data_row.into()
}

/// Pagination controls.
fn view_pagination<'a>(page: usize, total_rows: usize, page_size: usize) -> Element<'a, Message> {
    let total_pages = if total_rows == 0 {
        1
    } else {
        (total_rows + page_size - 1) / page_size
    };

    let prev_enabled = page > 0;
    let next_enabled = page < total_pages.saturating_sub(1);

    // First page button
    let first_button = button(lucide::chevrons_left().size(14).color(if prev_enabled {
        GRAY_700
    } else {
        GRAY_400
    }))
    .on_press_maybe(if prev_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(0),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Previous page button
    let prev_button = button(lucide::chevron_left().size(14).color(if prev_enabled {
        GRAY_700
    } else {
        GRAY_400
    }))
    .on_press_maybe(if prev_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page - 1),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Page info
    let start_row = page * page_size + 1;
    let end_row = ((page + 1) * page_size).min(total_rows);
    let page_info = container(
        text(format!(
            "{}-{} of {}",
            if total_rows == 0 { 0 } else { start_row },
            end_row,
            total_rows
        ))
        .size(12)
        .color(GRAY_700),
    )
    .padding([6.0, 12.0])
    .style(|_: &Theme| container::Style {
        background: Some(WHITE.into()),
        border: Border {
            color: GRAY_200,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    });

    // Next page button
    let next_button = button(lucide::chevron_right().size(14).color(if next_enabled {
        GRAY_700
    } else {
        GRAY_400
    }))
    .on_press_maybe(if next_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(page + 1),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    // Last page button
    let last_button = button(lucide::chevrons_right().size(14).color(if next_enabled {
        GRAY_700
    } else {
        GRAY_400
    }))
    .on_press_maybe(if next_enabled {
        Some(Message::DomainEditor(DomainEditorMessage::Preview(
            PreviewMessage::GoToPage(total_pages.saturating_sub(1)),
        )))
    } else {
        None
    })
    .padding([6.0, 8.0])
    .style(button_ghost);

    row![
        first_button,
        prev_button,
        Space::new().width(SPACING_XS),
        page_info,
        Space::new().width(SPACING_XS),
        next_button,
        last_button,
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Rows per page selector.
fn view_rows_per_page_selector<'a>(current: usize) -> Element<'a, Message> {
    let options = [25, 50, 100, 200];

    let label = text("Rows:").size(12).color(GRAY_600);

    let buttons: Vec<Element<'a, Message>> = options
        .iter()
        .map(|&n| {
            let is_selected = current == n;
            button(text(format!("{}", n)).size(11).color(if is_selected {
                PRIMARY_500
            } else {
                GRAY_600
            }))
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RowsPerPageChanged(n),
            )))
            .padding([4.0, 8.0])
            .style(move |_: &Theme, _status| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(PRIMARY_100.into()),
                        text_color: PRIMARY_500,
                        border: Border {
                            color: PRIMARY_500,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style {
                        background: Some(WHITE.into()),
                        text_color: GRAY_600,
                        border: Border {
                            color: GRAY_300,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                }
            })
            .into()
        })
        .collect();

    row![label, Space::new().width(SPACING_SM),]
        .push(row(buttons).spacing(4.0))
        .align_y(Alignment::Center)
        .into()
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

// =============================================================================
// STATES
// =============================================================================

/// Loading state while preview is being rebuilt.
fn view_loading_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::loader().size(40).color(PRIMARY_500),
            Space::new().height(SPACING_LG),
            text("Building Preview").size(18).color(GRAY_800),
            Space::new().height(SPACING_SM),
            text("Applying mappings and normalization rules...")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
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
            Space::new().height(SPACING_LG),
            text("Preview Build Failed").size(18).color(GRAY_800),
            Space::new().height(SPACING_SM),
            container(text(error_text).size(12).color(GRAY_600))
                .padding(SPACING_MD)
                .style(|_: &Theme| container::Style {
                    background: Some(GRAY_100.into()),
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::new().height(SPACING_LG),
            button(
                row![
                    lucide::refresh_cw().size(14).color(WHITE),
                    Space::new().width(SPACING_SM),
                    text("Retry").size(14).color(WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RebuildPreview,
            )))
            .padding([10.0, 24.0])
            .style(button_primary),
        ]
        .align_x(Alignment::Center)
        .max_width(400.0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

/// Empty state when no preview is available.
fn view_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            lucide::table().size(48).color(GRAY_400),
            Space::new().height(SPACING_LG),
            text("No Preview Available").size(18).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text("Click 'Rebuild' to generate the transformed data preview")
                .size(13)
                .color(GRAY_500),
            Space::new().height(SPACING_LG),
            button(
                row![
                    lucide::play().size(14).color(WHITE),
                    Space::new().width(SPACING_SM),
                    text("Build Preview").size(14).color(WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RebuildPreview,
            )))
            .padding([10.0, 24.0])
            .style(button_primary),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}
