# Trial Submission Studio - Components

This directory contains reusable UI components for the Trial Submission Studio
GUI.

## Overview

Components are **functions that return `Element<M>`**, not custom widgets. This
provides:

- Simple composition
- Type-safe message passing
- Easy customization per use case

## Component Types

1. **Function Components**: Simple functions like `panel()`, `domain_badge()`
2. **Builder Components**: Chainable structs like `EmptyState`, `PageHeader`

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
& state.study_name,
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
& state.search_query,
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

### Feedback Components (Builder Pattern)

#### `EmptyState`

Empty state with icon, title, description, and optional action button.

```rust
use tss_gui::component::EmptyState;
use iced_fonts::lucide;

EmptyState::new(lucide::folder_open().size(48), "No Study Loaded")
.description("Open a study folder to get started")
.action("Open Folder", Message::OpenFolder)
.centered()
.view()
```

#### `LoadingState`

Loading spinner with title and description.

```rust
use tss_gui::component::LoadingState;

LoadingState::new("Building Preview")
.description("Applying mappings and normalization rules...")
.centered()
.view()
```

#### `ErrorState`

Error display with retry option.

```rust
use tss_gui::component::ErrorState;

ErrorState::new("Export Failed")
.message( & error_details)
.retry(Message::RetryExport)
.centered()
.view()
```

#### `NoFilteredResults`

Empty state for filtered/searched lists.

```rust
use tss_gui::component::NoFilteredResults;

NoFilteredResults::new("No columns match filter")
.hint("Try adjusting your search")
.height(150.0)
.view()
```

### Header Components (Builder Pattern)

#### `PageHeader`

Page header with back button, badge, title, and metadata.

```rust
use tss_gui::component::PageHeader;

PageHeader::new("Demographics")
.back(Message::BackClicked)
.badge("DM", PRIMARY_500)
.meta("Rows", "150")
.meta("Progress", "85%")
.view()
```

#### `page_header_simple`

Simple header without metadata.

```rust
use tss_gui::component::page_header_simple;

page_header_simple("Settings", Some(Message::BackClicked))
```

### Section Components

#### `SectionCard`

Titled section card with optional icon.

```rust
use tss_gui::component::SectionCard;
use iced_fonts::lucide;

SectionCard::new("Variable Info", metadata_content)
.icon(lucide::info().size(14))
.view()
```

#### `panel` / `status_panel`

Panel wrappers for content grouping.

```rust
use tss_gui::component::{panel, status_panel};

// Basic panel
panel(content)

// Status panel with border color
status_panel(content, SUCCESS, Some(SUCCESS_LIGHT))
```

### Badge Components

#### `domain_badge` / `domain_badge_small`

Domain code badges (e.g., DM, AE, VS).

```rust
use tss_gui::component::{domain_badge, domain_badge_small};

domain_badge("DM")       // Standard size
domain_badge_small("AE") // Compact size
```

#### `core_badge` / `core_badge_if_important`

CDISC core designation badges (Req/Exp/Perm).

```rust
use tss_gui::component::{core_badge, core_badge_if_important};
use tss_model::sdtm::CoreDesignation;

core_badge(CoreDesignation::Required)   // Always shows "Req" in red
core_badge_if_important(designation)    // Only shows if Required/Expected
```

### List Components (Builder Pattern)

#### `SelectableRow`

Selectable row for master lists with hover/selection states.

```rust
use tss_gui::component::SelectableRow;

SelectableRow::new("STUDYID", Message::Selected(0))
.secondary("Study Identifier")
.leading(lucide::check().size(12).color(SUCCESS))
.trailing(core_badge(CoreDesignation::Required))
.selected(idx == selected_idx)
.view()
```

#### `DomainListItem`

Specialized domain list item for home view.

```rust
use tss_gui::component::DomainListItem;

DomainListItem::new("DM", "Demographics", Message::DomainClicked("DM".into()))
.row_count(150)
.complete(true)
.touched(true)
.view()
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
& columns,
rows,
state.page,
20,  // page_size
total_count,
Message::PageChanged,
);
```

### Helper Components

#### Icons with `iced_fonts::lucide`

Use Lucide icons directly from `iced_fonts`:

```rust
use iced_fonts::lucide;

// Direct usage - each function returns a Text widget
let folder = lucide::folder();
let check = lucide::check();
let search = lucide::search();

// With custom size
let large_icon = lucide::folder().size(24);

// With color
let colored = lucide::check().size(16).color(SUCCESS);
```

## Design Guidelines

1. **Generic Message Type**: Always use generic `M` for flexibility
   ```rust
   pub fn my_component<'a, M: Clone + 'a>(...) -> Element<'a, M>
   ```

2. **Closures for Message Factories**: Use closures to allow callers to define
   messages
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

All components use the theme constants from `crate::theme`:

- **Colors**: `PRIMARY_500`, `GRAY_600`, `SUCCESS`, `WARNING`, `ERROR`, etc.
- **Spacing**: `SPACING_SM`, `SPACING_MD`, `SPACING_LG`, `SPACING_XL`
- **Border Radius**: `BORDER_RADIUS_SM`, `BORDER_RADIUS_MD`
- **Button Styles**: `button_primary`, `button_secondary`

See `docs/05-theming.md` for the complete style guide.

## File Structure

```
component/
├── mod.rs              # Exports all components
├── master_detail.rs    # Split pane layout
├── sidebar.rs          # Vertical navigation
├── tab_bar.rs          # Horizontal tabs
├── modal.rs            # Modal dialogs
├── progress_modal.rs   # Progress overlays
├── form_field.rs       # Form inputs
├── search_box.rs       # Search with clear
├── status_badge.rs     # Status indicators
├── data_table.rs       # Paginated tables
├── empty_state.rs      # EmptyState, LoadingState, ErrorState
├── section_card.rs     # SectionCard, panel, status_panel
├── domain_badge.rs     # Domain code badges
├── core_badge.rs       # Core designation badges
├── selectable_row.rs   # SelectableRow, DomainListItem
└── page_header.rs      # PageHeader builder
```
