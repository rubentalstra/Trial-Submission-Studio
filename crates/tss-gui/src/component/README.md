# Trial Submission Studio - Components

This directory contains reusable UI components for the Trial Submission Studio GUI.

## Overview

Components are **functions that return `Element<M>`**, not custom widgets. This provides:
- Simple composition
- Type-safe message passing
- Easy customization per use case

## Components

### Layout Components

#### `master_detail`
Split pane layout with list on left, detail on right.

```rust
use tss_gui::component::master_detail;

let layout = master_detail(
    variable_list,     // Left panel
    variable_details,  // Right panel
    280.0,             // Left panel width
);
```

#### `sidebar`
Vertical navigation sidebar for domain/feature selection.

```rust
use tss_gui::component::{sidebar, SidebarItem};

let items = vec![
    SidebarItem::new("DM", Message::DomainSelected("DM")),
    SidebarItem::new("AE", Message::DomainSelected("AE"))
        .with_badge("3"),  // Error count badge
];

let nav = sidebar(items, Some(0), 280.0);
```

#### `tab_bar`
Horizontal tab navigation.

```rust
use tss_gui::component::{tab_bar, Tab};

let tabs = vec![
    Tab::new("Mapping", Message::TabSelected(0)),
    Tab::new("Transform", Message::TabSelected(1)),
    Tab::new("Validation", Message::TabSelected(2)),
];

let bar = tab_bar(tabs, state.active_tab);
```

### Overlay Components

#### `modal`
Modal dialog with backdrop, title, content, and action buttons.

```rust
use tss_gui::component::modal;

let view = modal(
    base_content,
    "Confirm Action",
    text("Are you sure?").into(),
    Message::CloseModal,
    vec![cancel_btn, confirm_btn],
);
```

#### `confirm_modal`
Pre-built confirmation dialog.

```rust
use tss_gui::component::confirm_modal;

let view = confirm_modal(
    base_content,
    "Delete Variable",
    "Are you sure you want to remove this mapping?",
    "Delete",
    Message::ConfirmDelete,
    Message::CancelDelete,
);
```

#### `progress_modal`
Progress dialog with optional cancel button.

```rust
use tss_gui::component::progress_modal;

let view = progress_modal(
    base_content,
    "Exporting Domains",
    "Processing DM domain...",
    0.45,  // 45% progress
    Some(Message::CancelExport),
);
```

### Form Components

#### `form_field`
Text input with label and optional error message.

```rust
use tss_gui::component::form_field;

let field = form_field(
    "Study Name",
    &state.study_name,
    "Enter study name...",
    Message::StudyNameChanged,
    state.name_error.as_deref(),
);
```

#### `number_field`
Numeric input with validation.

```rust
use tss_gui::component::number_field;

let field = number_field(
    "Header Rows",
    state.header_rows,
    Message::HeaderRowsChanged,
    Some(0),   // min
    Some(10),  // max
);
```

#### `search_box`
Search input with clear button.

```rust
use tss_gui::component::search_box;

let search = search_box(
    &state.search_query,
    "Search variables...",
    Message::SearchChanged,
    Message::SearchCleared,
);
```

### Display Components

#### `status_badge`
Colored status indicator.

```rust
use tss_gui::component::{status_badge, Status};

let badge = status_badge("Mapped", Status::Success);
let warning = status_badge("3 Issues", Status::Warning);
let error = status_badge("Required", Status::Error);
```

#### `data_table`
Paginated data table.

```rust
use tss_gui::component::{data_table, TableColumn};

let columns = vec![
    TableColumn::fixed("Variable", 150.0),
    TableColumn::fill("Description"),
    TableColumn::fixed("Type", 100.0),
];

let table = data_table(
    &columns,
    rows,
    state.page,
    20,  // page_size
    total_count,
    Message::PageChanged,
);
```

### Helper Components

#### `icon`
Font Awesome icon wrapper.

```rust
use tss_gui::component::{icon, icon_folder, icon_check};

// Generic icon with custom size
let custom = icon('\u{f07b}', Some(24.0));

// Convenience functions
let folder = icon_folder();
let check = icon_check();
```

## Design Guidelines

1. **Generic Message Type**: Always use generic `M` for flexibility
   ```rust
   pub fn my_component<'a, M: Clone + 'a>(...) -> Element<'a, M>
   ```

2. **Closures for Message Factories**: Use closures to allow callers to define messages
   ```rust
   pub fn search_box<'a, M: Clone + 'a>(
       value: &str,
       on_change: impl Fn(String) -> M + 'a,
   ) -> Element<'a, M>
   ```

3. **Style Functions**: Define named style functions for reuse
   ```rust
   fn card_style(_theme: &Theme) -> container::Style { ... }
   container(content).style(card_style)
   ```

4. **Composability**: Components should compose well together
   ```rust
   let content = column![
       search_box(&query, "Search...", on_search, on_clear),
       data_table(&cols, rows, page, 20, total, on_page),
   ];
   ```

## Theme Integration

All components use the Professional Clinical theme from `crate::theme`:

- **Colors**: `palette::PRIMARY_500`, `palette::GRAY_600`, etc.
- **Spacing**: `spacing::SPACING_MD`, `spacing::BORDER_RADIUS_SM`
- **Typography**: `typography::FONT_SIZE_BODY`
- **Styles**: `button_primary`, `container_card`, `text_input_default`

See `docs/05-theming.md` for the complete style guide.
