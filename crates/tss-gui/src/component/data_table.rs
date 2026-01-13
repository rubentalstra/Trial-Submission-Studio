//! Data table component.
//!
//! Paginated table for displaying large datasets.

use iced::widget::{button, column, container, row, rule, scrollable, space, text};
use iced::{Border, Element, Length};
use iced_fonts::lucide;

use crate::theme::{
    GRAY_50, GRAY_100, GRAY_200, GRAY_500, GRAY_600, GRAY_700, GRAY_800, SPACING_SM,
    TABLE_CELL_PADDING_X, TABLE_CELL_PADDING_Y, WHITE, button_ghost,
};

// =============================================================================
// TABLE COLUMN
// =============================================================================

/// Column definition for data table.
pub struct TableColumn {
    /// Column header text
    pub header: String,
    /// Column width
    pub width: Length,
}

impl TableColumn {
    /// Create a new column with fixed width.
    pub fn fixed(header: impl Into<String>, width: f32) -> Self {
        Self {
            header: header.into(),
            width: Length::Fixed(width),
        }
    }

    /// Create a new column that fills available space.
    pub fn fill(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: Length::Fill,
        }
    }

    /// Create a new column with proportional width.
    pub fn portion(header: impl Into<String>, portion: u16) -> Self {
        Self {
            header: header.into(),
            width: Length::FillPortion(portion),
        }
    }
}

// =============================================================================
// DATA TABLE
// =============================================================================

/// Creates a paginated data table.
///
/// Displays tabular data with column headers and pagination controls.
///
/// # Arguments
///
/// * `columns` - Column definitions
/// * `rows` - Row data (each row is a Vec of cell strings)
/// * `page` - Current page (0-indexed)
/// * `page_size` - Number of rows per page
/// * `total_rows` - Total number of rows in the dataset
/// * `on_page_change` - Message factory for page changes
///
/// # Example
///
/// ```rust,ignore
/// use tss_gui::component::{data_table, TableColumn};
///
/// let columns = vec![
///     TableColumn::fixed("Variable", 150.0),
///     TableColumn::fill("Description"),
///     TableColumn::fixed("Type", 100.0),
/// ];
///
/// let table = data_table(
///     &columns,
///     rows,
///     state.page,
///     20,
///     total_count,
///     Message::PageChanged,
/// );
/// ```
pub fn data_table<'a, M: Clone + 'a>(
    columns: &'a [TableColumn],
    rows: &'a [Vec<String>],
    page: usize,
    page_size: usize,
    total_rows: usize,
    on_page_change: impl Fn(usize) -> M + Clone + 'a,
) -> Element<'a, M> {
    // Header row
    let header_row = {
        let mut header = row![].spacing(0);
        for col in columns {
            header = header.push(
                container(
                    text(&col.header)
                        .size(12)
                        .color(GRAY_600)
                        .font(iced::Font::DEFAULT),
                )
                .width(col.width)
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
    let end = (start + page_size).min(rows.len());
    let visible_rows = if start < rows.len() {
        &rows[start..end]
    } else {
        &[]
    };

    // Data rows
    let mut data_rows = column![].spacing(0);
    for (row_idx, row_data) in visible_rows.iter().enumerate() {
        let mut data_row = row![].spacing(0);
        for (col_idx, cell) in row_data.iter().enumerate() {
            let width = columns
                .get(col_idx)
                .map(|c| c.width)
                .unwrap_or(Length::Fill);
            let is_even = row_idx % 2 == 0;

            data_row = data_row.push(
                container(text(cell).size(13).color(GRAY_700))
                    .width(width)
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

    let pagination = {
        let prev_enabled = page > 0;
        let next_enabled = page < total_pages.saturating_sub(1);

        let prev_button = button(lucide::chevron_left().size(14).color(if prev_enabled {
            GRAY_700
        } else {
            GRAY_500
        }))
        .on_press_maybe(if prev_enabled {
            Some(on_page_change.clone()(page - 1))
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
            Some(on_page_change(page + 1))
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

        row![
            space::horizontal(),
            prev_button,
            page_info,
            next_button,
            space::horizontal(),
        ]
        .spacing(SPACING_SM)
        .align_y(iced::Alignment::Center)
    };

    // Assemble table
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
    .spacing(0)
    .into()
}

/// Creates a simple table without pagination.
///
/// For smaller datasets that don't need pagination.
pub fn simple_table<'a, M: 'a>(
    columns: &'a [TableColumn],
    rows: &'a [Vec<String>],
) -> Element<'a, M> {
    // Header row
    let header_row = {
        let mut header = row![].spacing(0);
        for col in columns {
            header = header.push(
                container(text(&col.header).size(12).color(GRAY_600))
                    .width(col.width)
                    .padding([TABLE_CELL_PADDING_Y, TABLE_CELL_PADDING_X])
                    .style(|_theme| container::Style {
                        background: Some(GRAY_100.into()),
                        ..Default::default()
                    }),
            );
        }
        header
    };

    // Data rows
    let mut data_rows = column![].spacing(0);
    for (row_idx, row_data) in rows.iter().enumerate() {
        let mut data_row = row![].spacing(0);
        for (col_idx, cell) in row_data.iter().enumerate() {
            let width = columns
                .get(col_idx)
                .map(|c| c.width)
                .unwrap_or(Length::Fill);
            let is_even = row_idx % 2 == 0;

            data_row = data_row.push(
                container(text(cell).size(13).color(GRAY_700))
                    .width(width)
                    .padding([TABLE_CELL_PADDING_Y, TABLE_CELL_PADDING_X])
                    .style(move |_theme| container::Style {
                        background: Some(if is_even { WHITE } else { GRAY_50 }.into()),
                        ..Default::default()
                    }),
            );
        }
        data_rows = data_rows.push(data_row);
    }

    column![
        header_row,
        rule::horizontal(1).style(|_theme| rule::Style {
            color: GRAY_200,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        scrollable(data_rows).height(Length::Fill),
    ]
    .spacing(0)
    .into()
}

/// Creates a selectable table row.
///
/// Returns a row element that can be clicked.
pub fn selectable_row<'a, M: Clone + 'a>(
    columns: &[TableColumn],
    cells: &'a [String],
    is_selected: bool,
    on_click: M,
) -> Element<'a, M> {
    let mut data_row = row![].spacing(0);

    for (col_idx, cell) in cells.iter().enumerate() {
        let width = columns
            .get(col_idx)
            .map(|c| c.width)
            .unwrap_or(Length::Fill);

        data_row = data_row.push(
            container(
                text(cell)
                    .size(13)
                    .color(if is_selected { GRAY_800 } else { GRAY_700 }),
            )
            .width(width)
            .padding([TABLE_CELL_PADDING_Y, TABLE_CELL_PADDING_X]),
        );
    }

    let bg_color = if is_selected {
        crate::theme::PRIMARY_100
    } else {
        WHITE
    };

    button(data_row)
        .on_press(on_click)
        .width(Length::Fill)
        .padding(0)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered if !is_selected => Some(GRAY_100.into()),
                _ => Some(bg_color.into()),
            };
            button::Style {
                background: bg,
                border: Border::default(),
                ..Default::default()
            }
        })
        .into()
}
