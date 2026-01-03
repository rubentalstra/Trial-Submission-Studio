# tss-gui

The desktop application crate providing the graphical user interface.

## Overview

`tss-gui` is the main entry point for Trial Submission Studio, built with egui/eframe.

## Responsibilities

- Application window and layout
- User interaction handling
- Navigation between workflow steps
- Data visualization
- File dialogs and system integration

## Dependencies

```toml
[dependencies]
eframe = "0.29"
egui = "0.29"
tss-ingest = { path = "../tss-ingest" }
tss-map = { path = "../tss-map" }
tss-validate = { path = "../tss-validate" }
tss-output = { path = "../tss-output" }
tss-updater = { path = "../tss-updater" }
```

## Architecture

### Application Structure

```
tss-gui/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Application state
│   ├── views/
│   │   ├── mod.rs
│   │   ├── import.rs     # Import view
│   │   ├── mapping.rs    # Mapping view
│   │   ├── validation.rs # Validation view
│   │   └── export.rs     # Export view
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── data_grid.rs  # Data table widget
│   │   └── mapping.rs    # Mapping connection widget
│   └── state/
│       ├── mod.rs
│       └── workflow.rs   # Workflow state machine
└── assets/
    ├── icon.svg
    └── icon.png
```

### State Management

The application uses a centralized state pattern:

```rust
pub struct App {
    workflow: WorkflowState,
    data: Option<DataFrame>,
    mappings: Vec<Mapping>,
    validation_results: Vec<ValidationResult>,
}
```

### View Pattern

Each view implements a common trait:

```rust
pub trait View {
    fn ui(&mut self, ctx: &egui::Context, state: &mut AppState);
    fn title(&self) -> &str;
}
```

## Key Components

### Main Window

- Menu bar with file operations
- Sidebar navigation
- Main content area
- Status bar

### Data Grid

Custom widget for displaying large datasets:

- Virtual scrolling for performance
- Column sorting
- Row selection
- Type-aware formatting

### Mapping Interface

Visual mapping between source and target:

- Drag-and-drop connections
- Match confidence display
- Automatic suggestions

### Validation Panel

Results display with:

- Severity filtering
- Row highlighting
- Quick navigation to issues

## Configuration

### Settings Storage

User preferences stored in:

- macOS: `~/Library/Application Support/trial-submission-studio/`
- Windows: `%APPDATA%\trial-submission-studio\`
- Linux: `~/.config/trial-submission-studio/`

### Configurable Options

- Theme (light/dark)
- Recent files
- Export preferences
- Validation strictness

## Running

```bash
# Development
cargo run --package tss-gui

# Release
cargo run --release --package tss-gui
```

## Testing

```bash
cargo test --package tss-gui
```

GUI testing is limited; focus on:

- State transitions
- Data transformations
- Integration with other crates

## See Also

- [Architecture Overview](../overview.md)
- [tss-ingest](tss-ingest.md) - Data loading
- [tss-output](tss-output.md) - Export functionality
