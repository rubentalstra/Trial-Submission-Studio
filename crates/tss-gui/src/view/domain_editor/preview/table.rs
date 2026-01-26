//! Data table components for the Preview tab.
//!
//! Contains the table view, header row, data rows, and column width calculation.

use std::collections::HashSet;

use iced::widget::{Space, column, container, row, scrollable, text, text::Wrapping};
use iced::{Alignment, Border, Element, Length, Theme};
use polars::prelude::DataFrame;

use crate::message::Message;
use crate::state::{PreviewUiState, SourceDomainState};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_MD, SPACING_SM, SPACING_XS, ThemeConfig,
};

use super::helpers::{
    CELL_PADDING_X, CELL_PADDING_Y, CHAR_WIDTH, MAX_COL_WIDTH, MIN_COL_WIDTH, format_anyvalue,
    not_collected_colors,
};
use super::pagination::{view_pagination, view_rows_per_page_selector};

// =============================================================================
// DATA TABLE VIEW
// =============================================================================

/// Display the actual data table from DataFrame with horizontal scrolling.
pub(super) fn view_data_table<'a>(
    config: &ThemeConfig,
    df: &DataFrame,
    preview_ui: &PreviewUiState,
    source: &SourceDomainState,
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
    let not_collected_cols: HashSet<&str> = source
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

// =============================================================================
// COLUMN WIDTH CALCULATION
// =============================================================================

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

// =============================================================================
// TABLE CONTENT BUILDING
// =============================================================================

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
