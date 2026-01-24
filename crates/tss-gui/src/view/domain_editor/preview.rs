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

use std::collections::HashSet;

use iced::widget::{Space, button, column, container, row, scrollable, text, text::Wrapping};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::DataFrame;

use crate::component::display::{EmptyState, ErrorState, LoadingState};
use crate::message::domain_editor::PreviewMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{AppState, DomainState, PreviewUiState, ViewState};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, ThemeConfig,
    button_ghost, button_primary,
};

// =============================================================================
// SPECIAL COLUMN COLORS (inline functions for theming)
// =============================================================================

/// Get the Not Collected colors for theming
fn not_collected_colors(config: &ThemeConfig) -> NotCollectedColors {
    // Use warning colors as base for "Not Collected" columns
    let theme = config.to_theme(false);
    let warning = theme.extended_palette().warning.base.color;
    let warning_light = theme.clinical().status_warning_light;

    NotCollectedColors {
        bg: warning_light,
        bg_alt: Color {
            a: warning_light.a * 1.1,
            ..warning_light
        },
        header_bg: Color {
            a: warning_light.a * 1.2,
            ..warning_light
        },
        badge_bg: warning,
    }
}

struct NotCollectedColors {
    bg: Color,
    bg_alt: Color,
    header_bg: Color,
    badge_bg: Color,
}

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
        view_data_table(config, df, preview_ui, domain)
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
// DATA TABLE VIEW
// =============================================================================

/// Display the actual data table from DataFrame with horizontal scrolling.
fn view_data_table<'a>(
    config: &ThemeConfig,
    df: &DataFrame,
    preview_ui: &PreviewUiState,
    domain: &DomainState,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let bg_secondary = theme.clinical().background_secondary;
    let border_default = theme.clinical().border_default;

    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(ToString::to_string)
        .collect();
    let total_rows = df.height();
    let page = preview_ui.current_page;
    let page_size = preview_ui.rows_per_page;

    // Get the set of "Not Collected" variable names
    let not_collected_cols: HashSet<&str> = domain
        .mapping
        .all_not_collected()
        .keys()
        .map(String::as_str)
        .collect();

    // Calculate column widths based on content
    let col_widths = calculate_column_widths(df, &col_names);

    // Calculate visible rows
    let start = page * page_size;
    let end = (start + page_size).min(total_rows);

    // Build the complete table (header + data)
    let table_content = build_table_content(
        config,
        df,
        &col_names,
        &col_widths,
        &not_collected_cols,
        start,
        end,
    );

    // Pagination controls
    let pagination = view_pagination(config, page, total_rows, page_size);

    // Rows per page selector
    let rows_selector = view_rows_per_page_selector(config, preview_ui.rows_per_page);

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
    .style(move |_: &Theme| container::Style {
        background: Some(bg_secondary.into()),
        border: Border {
            color: border_default,
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
    .style(move |_: &Theme| container::Style {
        border: Border {
            color: border_default,
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
        .map(|name| {
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
                "DOMAIN" => width.clamp(70.0, 80.0),
                _ if name.ends_with("SEQ") => width.clamp(70.0, 90.0),
                _ if name.ends_with("DY") => width.clamp(70.0, 90.0),
                _ if name.ends_with("DTC") => width.max(110.0),
                _ => width,
            };

            adjusted_width.clamp(MIN_COL_WIDTH, MAX_COL_WIDTH)
        })
        .collect()
}

/// Build the table content (header row + data rows).
fn build_table_content<'a>(
    config: &ThemeConfig,
    df: &DataFrame,
    col_names: &[String],
    col_widths: &[f32],
    not_collected_cols: &HashSet<&str>,
    start: usize,
    end: usize,
) -> Element<'a, Message> {
    // Header row
    let header_row = build_header_row(config, col_names, col_widths, not_collected_cols);

    // Data rows
    let mut data_rows = column![].spacing(0);
    for row_idx in start..end {
        let is_even = (row_idx - start).is_multiple_of(2);
        data_rows = data_rows.push(build_data_row(
            config,
            df,
            col_names,
            col_widths,
            not_collected_cols,
            row_idx,
            is_even,
        ));
    }

    column![header_row, data_rows,].width(Length::Shrink).into()
}

/// Build the header row.
fn build_header_row<'a>(
    config: &ThemeConfig,
    col_names: &[String],
    col_widths: &[f32],
    not_collected_cols: &HashSet<&str>,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_secondary = theme.clinical().text_secondary;
    let text_on_accent = theme.clinical().text_on_accent;
    let bg_secondary = theme.clinical().background_secondary;
    let border_default = theme.clinical().border_default;
    let nc_colors = not_collected_colors(config);

    let mut header = row![].spacing(0);

    for (name, &width) in col_names.iter().zip(col_widths.iter()) {
        let is_not_collected = not_collected_cols.contains(name.as_str());

        // Build cell content with optional "NC" badge
        let cell_content: Element<'a, Message> = if is_not_collected {
            row![
                text(name.clone())
                    .size(12)
                    .color(text_secondary)
                    .wrapping(Wrapping::None)
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    }),
                Space::new().width(SPACING_XS),
                // "NC" badge
                container(text("NC").size(9).color(text_on_accent).font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }))
                .padding([2.0, 4.0])
                .style(move |_: &Theme| container::Style {
                    background: Some(nc_colors.badge_bg.into()),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            text(name.clone())
                .size(12)
                .color(text_secondary)
                .wrapping(Wrapping::None)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                })
                .into()
        };

        // Determine background color based on Not Collected status
        let bg_color = if is_not_collected {
            nc_colors.header_bg
        } else {
            bg_secondary
        };

        let cell = container(cell_content)
            .width(Length::Fixed(width))
            .padding([CELL_PADDING_Y, CELL_PADDING_X])
            .style(move |_: &Theme| container::Style {
                background: Some(bg_color.into()),
                border: Border {
                    color: border_default,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });

        header = header.push(cell);
    }

    container(header)
        .style(move |_: &Theme| container::Style {
            background: Some(bg_secondary.into()),
            ..Default::default()
        })
        .into()
}

/// Build a single data row.
fn build_data_row<'a>(
    config: &ThemeConfig,
    df: &DataFrame,
    col_names: &[String],
    col_widths: &[f32],
    not_collected_cols: &HashSet<&str>,
    row_idx: usize,
    is_even: bool,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_primary = theme.extended_palette().background.base.text;
    let text_disabled = theme.clinical().text_disabled;
    let bg_elevated = theme.clinical().background_elevated;
    let bg_secondary = theme.clinical().background_secondary;
    let border_default = theme.clinical().border_default;
    let nc_colors = not_collected_colors(config);

    let mut data_row = row![].spacing(0);

    for (col_idx, col) in df.get_columns().iter().enumerate() {
        let col_name = col_names.get(col_idx).map(String::as_str).unwrap_or("");
        let is_not_collected = not_collected_cols.contains(col_name);

        // Determine background color based on Not Collected status and alternating rows
        let bg_color = if is_not_collected {
            if is_even {
                nc_colors.bg
            } else {
                nc_colors.bg_alt
            }
        } else if is_even {
            bg_elevated
        } else {
            bg_secondary
        };

        let value = col
            .get(row_idx)
            .map_or_else(|_| String::new(), |v| format_anyvalue(&v));
        let width = col_widths.get(col_idx).copied().unwrap_or(100.0);

        // Check if value is empty/null for styling
        let text_color = if value.is_empty() {
            text_disabled
        } else {
            text_primary
        };
        let display_value = if value.is_empty() {
            "—".to_string()
        } else {
            value
        };

        let cell = container(
            text(display_value)
                .size(13)
                .color(text_color)
                .wrapping(Wrapping::None),
        )
        .width(Length::Fixed(width))
        .padding([CELL_PADDING_Y, CELL_PADDING_X])
        .style(move |_: &Theme| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                color: border_default,
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
fn view_pagination<'a>(
    config: &ThemeConfig,
    page: usize,
    total_rows: usize,
    page_size: usize,
) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_secondary = theme.clinical().text_secondary;
    let text_disabled = theme.clinical().text_disabled;
    let bg_elevated = theme.clinical().background_elevated;
    let border_default = theme.clinical().border_default;

    let total_pages = total_rows.div_ceil(page_size).max(1);

    let prev_enabled = page > 0;
    let next_enabled = page < total_pages.saturating_sub(1);

    // First page button
    let first_button = button(lucide::chevrons_left().size(14).color(if prev_enabled {
        text_secondary
    } else {
        text_disabled
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
        text_secondary
    } else {
        text_disabled
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
        .color(text_secondary),
    )
    .padding([6.0, 12.0])
    .style(move |_: &Theme| container::Style {
        background: Some(bg_elevated.into()),
        border: Border {
            color: border_default,
            width: 1.0,
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    });

    // Next page button
    let next_button = button(lucide::chevron_right().size(14).color(if next_enabled {
        text_secondary
    } else {
        text_disabled
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
        text_secondary
    } else {
        text_disabled
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
fn view_rows_per_page_selector<'a>(config: &ThemeConfig, current: usize) -> Element<'a, Message> {
    let theme = config.to_theme(false);
    let text_secondary = theme.clinical().text_secondary;
    let accent_primary = theme.extended_palette().primary.base.color;
    let bg_elevated = theme.clinical().background_elevated;
    let border_default = theme.clinical().border_default;

    // Create a lighter accent background for selected state
    let accent_light = Color {
        a: 0.15,
        ..accent_primary
    };

    let options = [25, 50, 100, 200];

    let label = text("Rows:").size(12).color(text_secondary);

    let buttons: Vec<Element<'a, Message>> = options
        .iter()
        .map(|&n| {
            let is_selected = current == n;
            button(text(format!("{}", n)).size(11).color(if is_selected {
                accent_primary
            } else {
                text_secondary
            }))
            .on_press(Message::DomainEditor(DomainEditorMessage::Preview(
                PreviewMessage::RowsPerPageChanged(n),
            )))
            .padding([4.0, 8.0])
            .style(move |_: &Theme, _status| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(accent_light.into()),
                        text_color: accent_primary,
                        border: Border {
                            color: accent_primary,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style {
                        background: Some(bg_elevated.into()),
                        text_color: text_secondary,
                        border: Border {
                            color: border_default,
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
