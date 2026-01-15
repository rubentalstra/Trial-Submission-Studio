# Trial Submission Studio - Component Guide

This document describes the reusable UI components and patterns used in Trial
Submission Studio.

## Table of Contents

1. [Component Philosophy](#component-philosophy)
2. [Component Structure](#component-structure)
3. [Core Components](#core-components)
4. [Layout Components](#layout-components)
5. [Form Components](#form-components)
6. [Data Display Components](#data-display-components)
7. [Feedback Components](#feedback-components)
8. [Builder Pattern Components](#builder-pattern-components)
9. [Component Best Practices](#component-best-practices)

---

## Component Philosophy

Components in Iced are **functions that return `Element<Message>`**, not custom
widget types. This approach provides:

1. **Simplicity** - Just functions, no trait implementations
2. **Composability** - Combine components freely
3. **Type safety** - Message type flows through
4. **Flexibility** - Easy to customize per use case

### Component vs Widget

| Component (Function)      | Widget (Struct + Trait)    |
|---------------------------|----------------------------|
| Returns `Element<M>`      | Implements `Widget` trait  |
| Composes built-in widgets | Custom rendering logic     |
| Quick to create           | More complex setup         |
| Use for layout patterns   | Use for novel interactions |

**Our approach**: Use component functions for everything except truly custom
widgets.

---

## Component Structure

### Basic Component Pattern

```rust
// component/status_badge.rs

use iced::widget::{container, text};
use iced::{Element, Length};
use crate::theme::palette;

/// Status badge showing a colored status indicator
pub fn status_badge<'a, M: 'a>(
    label: &str,
    status: Status,
) -> Element<'a, M> {
    let (bg_color, text_color) = match status {
        Status::Success => (palette::SUCCESS_LIGHT, palette::SUCCESS),
        Status::Warning => (palette::WARNING_LIGHT, palette::WARNING),
        Status::Error => (palette::ERROR_LIGHT, palette::ERROR),
        Status::Info => (palette::INFO_LIGHT, palette::INFO),
    };

    container(text(label).size(12))
        .padding([4, 8])
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            text_color: Some(text_color),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Success,
    Warning,
    Error,
    Info,
}
```

### Component with State

```rust
// component/search_box.rs

use iced::widget::{button, container, row, text_input};
use iced::{Element, Length};

/// Search box with clear button
pub fn search_box<'a, M: Clone + 'a>(
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> M + 'a,
    on_clear: M,
) -> Element<'a, M> {
    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(8)
        .width(Length::Fill);

    let clear_button = if value.is_empty() {
        None
    } else {
        Some(
            button(text("×").size(16))
                .on_press(on_clear)
                .padding([4, 8])
                .style(button::secondary),
        )
    };

    let mut content = row![input].spacing(4);
    if let Some(btn) = clear_button {
        content = content.push(btn);
    }

    container(content)
        .width(Length::Fill)
        .into()
}
```

---

## Core Components

### Icons with iced_fonts::lucide

Trial Submission Studio uses `iced_fonts` with the Lucide icon set directly—no
wrapper needed:

```rust
// Direct usage of iced_fonts::lucide icons
use iced_fonts::lucide;

// In your view function, use the icon functions directly:
fn view_example<'a>() -> Element<'a, Message> {
    row![
        lucide::folder(),           // Folder icon
        lucide::file(),             // File icon
        lucide::check(),            // Checkmark icon
        lucide::alert_triangle(),   // Warning icon
        lucide::alert_circle(),     // Error/alert icon
        lucide::loader(),           // Spinner/loading icon
        lucide::search(),           // Search/magnifier icon
        lucide::download(),         // Download icon
        lucide::upload(),           // Upload icon
        lucide::settings(),         // Settings/gear icon
    ]
        .into()
}

// Custom sizing with .size()
fn icon_with_size<'a>() -> Element<'a, Message> {
    lucide::folder().size(24).into()
}

// Styling with text styling methods
fn styled_icon<'a>() -> Element<'a, Message> {
    lucide::check()
        .size(20)
        .color(Color::from_rgb(0.2, 0.8, 0.2))
        .into()
}
```

### Button Variants

```rust
// component/buttons.rs

use iced::widget::{button, row, text};
use iced::{Element, Length};
use crate::theme::palette;

/// Primary action button (teal background)
pub fn primary_button<'a, M: Clone + 'a>(
    label: &str,
    on_press: Option<M>,
) -> Element<'a, M> {
    let btn = button(text(label).center())
        .padding([10, 20])
        .width(Length::Shrink)
        .style(button::primary);

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

/// Secondary button (outline style)
pub fn secondary_button<'a, M: Clone + 'a>(
    label: &str,
    on_press: Option<M>,
) -> Element<'a, M> {
    let btn = button(text(label).center())
        .padding([10, 20])
        .width(Length::Shrink)
        .style(button::secondary);

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

/// Danger button (red for destructive actions)
pub fn danger_button<'a, M: Clone + 'a>(
    label: &str,
    on_press: Option<M>,
) -> Element<'a, M> {
    let btn = button(text(label).center())
        .padding([10, 20])
        .width(Length::Shrink)
        .style(button::danger);

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

/// Ghost button (text only, minimal styling)
pub fn ghost_button<'a, M: Clone + 'a>(
    label: &str,
    on_press: M,
) -> Element<'a, M> {
    button(text(label))
        .on_press(on_press)
        .padding([6, 12])
        .style(button::text)
        .into()
}

/// Icon button (icon only)
pub fn icon_button<'a, M: Clone + 'a>(
    icon: Element<'a, M>,
    on_press: M,
    tooltip: Option<&str>,
) -> Element<'a, M> {
    let btn = button(icon)
        .on_press(on_press)
        .padding(8)
        .style(button::secondary);

    // TODO: Add tooltip support when available
    btn.into()
}
```

---

## Layout Components

### Master-Detail Layout

Split pane with list on left, detail on right:

```rust
// component/master_detail.rs

use iced::widget::{container, horizontal_rule, row, scrollable};
use iced::{Element, Length};
use crate::theme::spacing;

/// Master-detail split layout
///
/// # Arguments
/// * `master` - Left panel content (typically a list)
/// * `detail` - Right panel content (typically details of selected item)
/// * `master_width` - Width of master panel in pixels
pub fn master_detail<'a, M: 'a>(
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    row![
        // Master panel (fixed width)
        container(scrollable(master))
            .width(Length::Fixed(master_width))
            .height(Length::Fill)
            .padding(spacing::MD),
        // Divider
        container(horizontal_rule(1))
            .width(Length::Fixed(1.0))
            .height(Length::Fill),
        // Detail panel (fill remaining)
        container(scrollable(detail))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(spacing::MD),
    ]
        .into()
}

/// Master-detail with header
pub fn master_detail_with_header<'a, M: 'a>(
    header: Element<'a, M>,
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    use iced::widget::column;

    column![
        container(header).padding(spacing::MD),
        horizontal_rule(1),
        master_detail(master, detail, master_width),
    ]
        .into()
}
```

### Tab Bar

Horizontal tab navigation:

```rust
// component/tab_bar.rs

use iced::widget::{button, container, row, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Tab definition
pub struct Tab<M> {
    pub label: String,
    pub message: M,
}

/// Horizontal tab bar
pub fn tab_bar<'a, M: Clone + 'a>(
    tabs: Vec<Tab<M>>,
    active_index: usize,
) -> Element<'a, M> {
    let mut tab_row = row![].spacing(0);

    for (index, tab) in tabs.into_iter().enumerate() {
        let is_active = index == active_index;

        let tab_button = button(
            container(text(&tab.label).size(14))
                .padding([12, 16])
                .center_x(Length::Shrink),
        )
            .on_press(tab.message)
            .style(if is_active {
                tab_style_active
            } else {
                tab_style_inactive
            });

        tab_row = tab_row.push(tab_button);
    }

    container(tab_row)
        .width(Length::Fill)
        .style(tab_bar_container_style)
        .into()
}

fn tab_style_active(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(palette::PRIMARY_50.into()),
        text_color: palette::PRIMARY_600,
        border: iced::Border {
            color: palette::PRIMARY_500,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn tab_style_inactive(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(palette::GRAY_100.into()),
        text_color: palette::GRAY_600,
        border: iced::Border::default(),
        ..Default::default()
    }
}

fn tab_bar_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_100.into()),
        border: iced::Border {
            color: palette::GRAY_200,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}
```

### Sidebar

Vertical navigation sidebar:

```rust
// component/sidebar.rs

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Sidebar item
pub struct SidebarItem<M> {
    pub label: String,
    pub icon: Option<char>,
    pub message: M,
    pub badge: Option<String>,
}

/// Vertical sidebar navigation
pub fn sidebar<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    active_index: Option<usize>,
    width: f32,
) -> Element<'a, M> {
    let mut item_column = column![].spacing(4);

    for (index, item) in items.into_iter().enumerate() {
        let is_active = active_index == Some(index);

        let label = text(&item.label).size(14);

        let item_button = button(
            container(label).padding([8, 12]).width(Length::Fill),
        )
            .on_press(item.message)
            .width(Length::Fill)
            .style(if is_active {
                sidebar_item_active
            } else {
                sidebar_item_inactive
            });

        item_column = item_column.push(item_button);
    }

    container(scrollable(item_column))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(spacing::SM)
        .style(sidebar_container_style)
        .into()
}

fn sidebar_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_50.into()),
        border: iced::Border {
            color: palette::GRAY_200,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn sidebar_item_active(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(palette::PRIMARY_100.into()),
        text_color: palette::PRIMARY_700,
        border: iced::Border {
            color: palette::PRIMARY_500,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn sidebar_item_inactive(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(palette::GRAY_100.into()),
        _ => None,
    };

    button::Style {
        background: bg,
        text_color: palette::GRAY_700,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
```

### Card Container

Elevated card for grouping content:

```rust
// component/card.rs

use iced::widget::container;
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Card container with shadow/elevation
pub fn card<'a, M: 'a>(
    content: Element<'a, M>,
) -> Element<'a, M> {
    container(content)
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(card_style)
        .into()
}

/// Compact card variant
pub fn card_compact<'a, M: 'a>(
    content: Element<'a, M>,
) -> Element<'a, M> {
    container(content)
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(card_style)
        .into()
}

fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        border: iced::Border {
            color: palette::GRAY_200,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}
```

---

## Form Components

### Form Field

Input with label and validation:

```rust
// component/form_field.rs

use iced::widget::{column, container, text, text_input};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Form field with label, input, and optional error
pub fn form_field<'a, M: Clone + 'a>(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> M + 'a,
    error: Option<&str>,
) -> Element<'a, M> {
    let label_text = text(label)
        .size(13)
        .color(palette::GRAY_700);

    let input = text_input(placeholder, value)
        .on_input(on_change)
        .padding(10)
        .width(Length::Fill)
        .style(if error.is_some() {
            text_input_error_style
        } else {
            text_input::default
        });

    let mut content = column![label_text, input].spacing(6);

    if let Some(err) = error {
        let error_text = text(err)
            .size(12)
            .color(palette::ERROR);
        content = content.push(error_text);
    }

    container(content)
        .width(Length::Fill)
        .into()
}

/// Number input field
pub fn number_field<'a, M: Clone + 'a>(
    label: &str,
    value: usize,
    on_change: impl Fn(usize) -> M + 'a,
    min: Option<usize>,
    max: Option<usize>,
) -> Element<'a, M> {
    let value_str = value.to_string();

    form_field(
        label,
        &value_str,
        "0",
        move |s| {
            let parsed = s.parse().unwrap_or(value);
            let clamped = match (min, max) {
                (Some(lo), Some(hi)) => parsed.clamp(lo, hi),
                (Some(lo), None) => parsed.max(lo),
                (None, Some(hi)) => parsed.min(hi),
                (None, None) => parsed,
            };
            on_change(clamped)
        },
        None,
    )
}

fn text_input_error_style(
    theme: &iced::Theme,
    status: text_input::Status,
) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border.color = palette::ERROR;
    style
}
```

### Toggle Switch

Boolean toggle:

```rust
// component/toggle.rs

use iced::widget::{container, row, text, toggler};
use iced::{Element, Length};
use crate::theme::spacing;

/// Toggle switch with label
pub fn toggle<'a, M: Clone + 'a>(
    label: &str,
    value: bool,
    on_toggle: impl Fn(bool) -> M + 'a,
) -> Element<'a, M> {
    row![
        text(label).width(Length::Fill),
        toggler(value).on_toggle(on_toggle),
    ]
        .spacing(spacing::MD)
        .align_y(iced::Alignment::Center)
        .into()
}

/// Toggle with description
pub fn toggle_with_description<'a, M: Clone + 'a>(
    label: &str,
    description: &str,
    value: bool,
    on_toggle: impl Fn(bool) -> M + 'a,
) -> Element<'a, M> {
    use iced::widget::column;
    use crate::theme::palette;

    row![
        column![
            text(label),
            text(description).size(12).color(palette::GRAY_500),
        ]
        .spacing(4)
        .width(Length::Fill),
        toggler(value).on_toggle(on_toggle),
    ]
        .spacing(spacing::MD)
        .align_y(iced::Alignment::Center)
        .into()
}
```

### Dropdown / Pick List

Selection from options:

```rust
// component/dropdown.rs

use iced::widget::{column, container, pick_list, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Dropdown selector with label
pub fn dropdown<'a, T, M>(
    label: &str,
    options: impl Into<std::borrow::Cow<'a, [T]>>,
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    let label_text = text(label)
        .size(13)
        .color(palette::GRAY_700);

    let picker = pick_list(options, selected, on_select)
        .width(Length::Fill)
        .padding(10);

    column![label_text, picker]
        .spacing(6)
        .into()
}
```

---

## Data Display Components

### Data Table

Paginated table for large datasets:

```rust
// component/data_table.rs

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Column definition for data table
pub struct TableColumn {
    pub header: String,
    pub width: Length,
}

/// Data table with headers and pagination
pub fn data_table<'a, M: Clone + 'a>(
    columns: &[TableColumn],
    rows: Vec<Vec<String>>,
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
                container(text(&col.header).size(12).color(palette::GRAY_600))
                    .width(col.width)
                    .padding([8, 12])
                    .style(header_cell_style),
            );
        }
        header
    };

    // Data rows
    let start = page * page_size;
    let end = (start + page_size).min(rows.len());
    let visible_rows = &rows[start..end];

    let mut data_rows = column![].spacing(0);
    for (row_idx, row_data) in visible_rows.iter().enumerate() {
        let mut data_row = row![].spacing(0);
        for (col_idx, cell) in row_data.iter().enumerate() {
            let width = columns.get(col_idx).map(|c| c.width).unwrap_or(Length::Fill);
            data_row = data_row.push(
                container(text(cell).size(13))
                    .width(width)
                    .padding([8, 12])
                    .style(if row_idx % 2 == 0 {
                        row_even_style
                    } else {
                        row_odd_style
                    }),
            );
        }
        data_rows = data_rows.push(data_row);
    }

    // Pagination
    let total_pages = (total_rows + page_size - 1) / page_size;
    let pagination = {
        let prev_button = button(text("<"))
            .on_press_maybe(if page > 0 {
                Some(on_page_change.clone()(page - 1))
            } else {
                None
            })
            .padding([4, 8]);

        let next_button = button(text(">"))
            .on_press_maybe(if page < total_pages - 1 {
                Some(on_page_change(page + 1))
            } else {
                None
            })
            .padding([4, 8]);

        let page_info = text(format!("Page {} of {}", page + 1, total_pages))
            .size(12)
            .color(palette::GRAY_600);

        row![prev_button, page_info, next_button]
            .spacing(spacing::MD)
            .align_y(iced::Alignment::Center)
    };

    column![
        header_row,
        horizontal_rule(1),
        scrollable(data_rows).height(Length::Fill),
        horizontal_rule(1),
        container(pagination).padding(spacing::SM),
    ]
        .spacing(0)
        .into()
}

fn header_cell_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_100.into()),
        ..Default::default()
    }
}

fn row_even_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        ..Default::default()
    }
}

fn row_odd_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_50.into()),
        ..Default::default()
    }
}
```

### List Item

Selectable list item:

```rust
// component/list_item.rs

use iced::widget::{button, container, row, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Selectable list item
pub fn list_item<'a, M: Clone + 'a>(
    primary: &str,
    secondary: Option<&str>,
    is_selected: bool,
    on_click: M,
) -> Element<'a, M> {
    use iced::widget::column;

    let content = if let Some(sec) = secondary {
        column![
            text(primary).size(14),
            text(sec).size(12).color(palette::GRAY_500),
        ]
            .spacing(2)
    } else {
        column![text(primary).size(14)]
    };

    button(container(content).padding([8, 12]).width(Length::Fill))
        .on_press(on_click)
        .width(Length::Fill)
        .style(if is_selected {
            list_item_selected
        } else {
            list_item_normal
        })
        .into()
}

fn list_item_selected(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(palette::PRIMARY_100.into()),
        text_color: palette::PRIMARY_800,
        border: iced::Border {
            color: palette::PRIMARY_300,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn list_item_normal(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(palette::GRAY_100.into()),
        _ => None,
    };

    button::Style {
        background: bg,
        text_color: palette::GRAY_800,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
```

---

## Feedback Components

### Modal Dialog

Overlay modal:

```rust
// component/modal.rs

use iced::widget::{button, center, column, container, opaque, row, stack, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Modal dialog overlay
pub fn modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &str,
    content: Element<'a, M>,
    on_close: M,
    actions: Vec<Element<'a, M>>,
) -> Element<'a, M> {
    let backdrop = container(text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(backdrop_style);

    let header = row![
        text(title).size(18),
        container(text("")).width(Length::Fill),  // Spacer
        button(text("×").size(20))
            .on_press(on_close)
            .style(button::text),
    ]
        .align_y(iced::Alignment::Center);

    let action_row = {
        let mut r = row![].spacing(spacing::SM);
        for action in actions {
            r = r.push(action);
        }
        r
    };

    let dialog = container(
        column![
            header,
            container(content).padding([spacing::MD, 0]),
            action_row,
        ]
            .spacing(spacing::MD),
    )
        .width(Length::Fixed(500.0))
        .padding(spacing::LG)
        .style(dialog_style);

    stack![
        base,
        opaque(backdrop),
        center(dialog),
    ]
        .into()
}

/// Confirmation modal (simple yes/no)
pub fn confirm_modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &str,
    message: &str,
    confirm_label: &str,
    on_confirm: M,
    on_cancel: M,
) -> Element<'a, M> {
    modal(
        base,
        title,
        text(message).into(),
        on_cancel.clone(),
        vec![
            button(text("Cancel")).on_press(on_cancel).into(),
            button(text(confirm_label))
                .on_press(on_confirm)
                .style(button::primary)
                .into(),
        ],
    )
}

fn backdrop_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
        ..Default::default()
    }
}

fn dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}
```

### Progress Modal

Export/loading progress with cancellation:

```rust
// component/progress_modal.rs

use iced::widget::{button, center, column, container, opaque, progress_bar, stack, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};

/// Progress modal with optional cancel button
pub fn progress_modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    title: &str,
    message: &str,
    progress: f32,  // 0.0 to 1.0
    on_cancel: Option<M>,
) -> Element<'a, M> {
    let progress_bar_widget = progress_bar(0.0..=1.0, progress)
        .width(Length::Fill)
        .height(Length::Fixed(8.0));

    let percentage = text(format!("{}%", (progress * 100.0) as u32))
        .size(12)
        .color(palette::GRAY_600);

    let mut content = column![
        text(message).size(14),
        progress_bar_widget,
        percentage,
    ]
        .spacing(spacing::MD);

    if let Some(cancel) = on_cancel {
        content = content.push(
            container(
                button(text("Cancel"))
                    .on_press(cancel)
                    .style(button::secondary),
            )
                .width(Length::Fill)
                .center_x(Length::Shrink),
        );
    }

    let backdrop = container(text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog = container(
        column![text(title).size(18), content].spacing(spacing::MD),
    )
        .width(Length::Fixed(400.0))
        .padding(spacing::LG)
        .style(|_| container::Style {
            background: Some(palette::WHITE.into()),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        });

    stack![base, opaque(backdrop), center(dialog)].into()
}
```

### Toast / Notification

Temporary notification:

```rust
// component/toast.rs

use iced::widget::{container, row, text};
use iced::{Element, Length};
use crate::theme::{palette, spacing};
use crate::component::icon;

/// Toast notification type
#[derive(Debug, Clone, Copy)]
pub enum ToastType {
    Success,
    Warning,
    Error,
    Info,
}

/// Toast notification (typically positioned at bottom)
pub fn toast<'a, M: 'a>(
    message: &str,
    toast_type: ToastType,
) -> Element<'a, M> {
    use iced_fonts::lucide;

    let (bg_color, icon_widget): (_, Element<'a, M>) = match toast_type {
        ToastType::Success => (palette::SUCCESS, lucide::circle_check().size(16).color(palette::WHITE).into()),
        ToastType::Warning => (palette::WARNING, lucide::triangle_alert().size(16).color(palette::WHITE).into()),
        ToastType::Error => (palette::ERROR, lucide::circle_x().size(16).color(palette::WHITE).into()),
        ToastType::Info => (palette::INFO, lucide::info().size(16).color(palette::WHITE).into()),
    };

    container(
        row![
            icon_widget,
            text(message).color(palette::WHITE),
        ]
            .spacing(spacing::SM)
            .align_y(iced::Alignment::Center),
    )
        .padding([spacing::SM, spacing::MD])
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        })
        .into()
}
```

---

## Builder Pattern Components

These components use the builder pattern for flexible, chainable configuration.
They're designed for common UI patterns that appear across multiple views.

### Empty State Components

Located in `component/empty_state.rs`:

#### EmptyState

Full-featured empty state with icon, title, description, and action button:

```rust
use tss_gui::component::EmptyState;
use iced_fonts::lucide;

// Basic usage
EmptyState::new(
    lucide::folder_open().size(48).color(GRAY_400),
    "No Study Loaded"
)
    .view()

// Full featured
EmptyState::new(
    lucide::shield_check().size(48).color(GRAY_400),
    "No Validation Results"
)
    .description("Click 'Re-validate' to check for CDISC conformance issues")
    .action("Run Validation", Message::RunValidation)
    .height(300.0)      // Fixed height
    .centered()         // Center in container
    .view()
```

#### LoadingState

Spinner with loading message:

```rust
use tss_gui::component::LoadingState;

LoadingState::new("Building Preview")
    .description("Applying mappings and normalization rules...")
    .centered()
    .view()
```

#### ErrorState

Error display with optional retry:

```rust
use tss_gui::component::ErrorState;

ErrorState::new("Preview Build Failed")
    .message(&error_string)     // Error details
    .retry(Message::Rebuild)    // Retry button
    .centered()
    .view()
```

#### NoFilteredResults

Compact empty state for filtered lists:

```rust
use tss_gui::component::NoFilteredResults;

NoFilteredResults::new("No columns match filter")
    .hint("Try adjusting your search or filter")
    .height(120.0)
    .view()
```

### Page Header Component

Located in `component/page_header.rs`:

#### PageHeader

Standardized page header with back button, badge, and metadata:

```rust
use tss_gui::component::PageHeader;

PageHeader::new("Demographics")
    .back(Message::BackClicked)                    // Back button
    .badge("DM", PRIMARY_500)                      // Domain badge
    .meta("Rows", domain.row_count().to_string()) // Key-value metadata
    .meta("Progress", format!("{}%", progress))
    .trailing(some_element)                        // Optional trailing element
    .view()
```

#### page_header_simple

Simple function for basic headers:

```rust
use tss_gui::component::page_header_simple;

page_header_simple("Settings", Some(Message::Back))
```

### Section Components

Located in `component/section_card.rs`:

#### SectionCard

Titled card container:

```rust
use tss_gui::component::SectionCard;
use iced_fonts::lucide;

SectionCard::new("Variable Information", content)
    .icon(lucide::info().size(14).color(GRAY_600))
    .view()
```

#### panel / status_panel

Simple wrapper functions:

```rust
use tss_gui::component::{panel, status_panel};

// Basic panel with border
panel(content)

// Status panel with colored border
status_panel(content, SUCCESS, Some(SUCCESS_LIGHT))
```

### Badge Components

#### Domain Badges

Located in `component/domain_badge.rs`:

```rust
use tss_gui::component::{domain_badge, domain_badge_small};

// Standard size badge with primary color
domain_badge("DM")

// Compact badge for tight spaces
domain_badge_small("AE")
```

#### Core Designation Badges

Located in `component/core_badge.rs`:

```rust
use tss_gui::component::{core_badge, core_badge_if_important};
use tss_model::sdtm::CoreDesignation;

// Always shows the badge
core_badge(CoreDesignation::Required)   // Red "Req"
core_badge(CoreDesignation::Expected)   // Amber "Exp"
core_badge(CoreDesignation::Permissible) // Gray "Perm"

// Only shows for Required/Expected (returns empty for Permissible)
core_badge_if_important(designation)
```

### Selectable Row Components

Located in `component/selectable_row.rs`:

#### SelectableRow

Master list item with hover/selection states:

```rust
use tss_gui::component::SelectableRow;
use iced_fonts::lucide;

SelectableRow::new("STUDYID", Message::VariableSelected(idx))
    .secondary("Study Identifier")              // Subtitle
    .leading(lucide::check().size(12))         // Leading element (icon)
    .trailing(core_badge(CoreDesignation::Required)) // Trailing element
    .selected(idx == state.selected_index)      // Selection state
    .view()
```

#### DomainListItem

Specialized for domain lists on home screen:

```rust
use tss_gui::component::DomainListItem;

DomainListItem::new(
    "DM",
    "Demographics",
    Message::DomainClicked("DM".into())
)
    .row_count(150)      // Number of data rows
    .complete(true)      // Mapping complete?
    .touched(true)       // Has been edited?
    .view()
```

---

## Component Best Practices

### 1. Generic Message Type

Always use generic `M` for flexibility:

```rust, no_run
// GOOD: Generic message type
pub fn my_component<'a, M: Clone + 'a>(...) -> Element<'a, M>

// BAD: Tied to specific message type
pub fn my_component<'a>(...) -> Element<'a, Message>
```

### 2. Closure for Message Factories

Use closures for components that need to create messages:

```rust
// GOOD: Closure allows caller to define message
pub fn search_box<'a, M: Clone + 'a>(
    value: &str,
    on_change: impl Fn(String) -> M + 'a,
) -> Element<'a, M>

// BAD: Fixed message variant
pub fn search_box<'a>(value: &str) -> Element<'a, SearchMessage>
```

### 3. Style Functions Over Inline Closures

Define style functions for reuse:

```rust, no_run
// GOOD: Named function for styles
fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        border: iced::Border { radius: 6.0.into(), ..Default::default() },
        ..Default::default()
    }
}

container(content).style(card_style)

// OK: Inline closure for one-off styles
container(content).style(|_| container::Style { ... })
```

### 4. Component Documentation

Document public components:

````rust
/// Search box with clear button
///
/// # Arguments
/// * `value` - Current search text
/// * `placeholder` - Placeholder text when empty
/// * `on_change` - Message factory for text changes
/// * `on_clear` - Message to send when clear button clicked
///
/// # Example
/// ```rust
/// search_box(
///     &self.search,
///     "Search...",
///     MappingMessage::SearchChanged,
///     MappingMessage::SearchCleared,
/// )
/// ```
pub fn search_box<'a, M: Clone + 'a>(...) -> Element<'a, M>
````

### 5. Composability

Components should compose well:

```rust, no_run
// GOOD: Components that work together
let content = column![
    card(form_field("Name", &name, "Enter name", on_name_change, None)),
    card(toggle("Active", is_active, on_toggle)),
    card(dropdown("Type", types, selected_type, on_type_change)),
];

// BAD: Component that does too much
let content = giant_form_component(
    name, is_active, types, selected_type,
    on_name_change, on_toggle, on_type_change,
);
```

---

## Next Steps

- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
