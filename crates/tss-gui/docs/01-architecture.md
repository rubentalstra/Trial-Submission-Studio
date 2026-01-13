# Trial Submission Studio - Architecture Guide

This document describes the overall architecture of the Trial Submission Studio GUI application, built with Iced 0.14.0.

## Table of Contents

1. [Overview](#overview)
2. [The Elm Architecture](#the-elm-architecture)
3. [Module Structure](#module-structure)
4. [Data Flow](#data-flow)
5. [Application Lifecycle](#application-lifecycle)
6. [Key Design Decisions](#key-design-decisions)

---

## Overview

Trial Submission Studio uses **Iced 0.14.0**, a cross-platform GUI library for Rust inspired by Elm. The application
follows the Elm architecture pattern, which provides:

- **Predictable state management** - All state changes flow through a single `update` function
- **Type-safe UI** - The view is a pure function of state
- **Declarative rendering** - UI is described as data, not imperative commands
- **Async-friendly** - Built-in support for background tasks via `Task`

### Technology Stack

| Component            | Technology                      |
|----------------------|---------------------------------|
| GUI Framework        | Iced 0.14.0                     |
| Async Runtime        | Tokio                           |
| Graphics Backend     | wgpu (Metal/Vulkan/DX12)        |
| Icon Font            | iced_fonts 0.3.0 (Font Awesome) |
| Menu (macOS)         | muda (native)                   |
| Menu (Windows/Linux) | In-app menu                     |

---

## The Elm Architecture

Iced implements the Elm architecture with four core concepts:

### 1. State

The application state is a plain Rust struct that holds all data:

```rust
pub struct App {
    view: View,
    study: Option<StudyState>,
    settings: Settings,
    ui: UiState,
}
```

**Rules:**

- State is the single source of truth
- State is never mutated directly from views
- State changes only happen in the `update` function

### 2. Message

Messages represent all possible events and user interactions:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Navigate(View),
    Home(HomeMessage),
    DomainEditor(DomainEditorMessage),
    Export(ExportMessage),
    Dialog(DialogMessage),
    Menu(MenuMessage),
    // ... background task results
    KeyPressed(Key, Modifiers),
    Tick,
}
```

**Rules:**

- Messages are immutable data
- Messages should be descriptive of *what happened*, not *what to do*
- Nested messages for complex features (see `02-message-patterns.md`)

### 3. Update

The update function processes messages and returns new state plus optional tasks:

```rust
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(view) => {
                self.view = view;
                Task::none()
            }
            Message::Home(msg) => self.handle_home_message(msg),
            // ...
        }
    }
}
```

**Rules:**

- Update is the *only* place state changes happen
- Keep update handlers focused and delegate to helper methods
- Return `Task::none()` when no async work is needed
- Return `Task::perform()` or `Task::done()` for async operations

### 4. View

The view function renders the current state as UI elements:

```rust
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        match &self.view {
            View::Home => self.view_home(),
            View::DomainEditor { domain, tab } => self.view_domain_editor(domain, *tab),
            View::Export => self.view_export(),
        }
    }
}
```

**Rules:**

- Views are pure functions - no side effects
- Views only read state, never modify it
- Use composition to build complex UIs from simple components

---

## Module Structure

```
crates/tss-gui/src/
├── main.rs                 # Entry point, Iced app runner
├── app.rs                  # Main App struct, update/view logic
├── lib.rs                  # Library exports
│
├── message/                # Message types (see 02-message-patterns.md)
│   ├── mod.rs              # Root Message enum
│   ├── home.rs             # HomeMessage
│   ├── domain_editor.rs    # DomainEditorMessage + tab messages
│   ├── export.rs           # ExportMessage
│   ├── dialog.rs           # DialogMessage
│   └── menu.rs             # MenuMessage
│
├── state/                  # Application state (see 03-state-management.md)
│   ├── mod.rs              # State exports
│   ├── navigation.rs       # View, EditorTab, WorkflowMode enums
│   ├── app_state.rs        # Root AppState
│   ├── study_state.rs      # Per-study state
│   ├── domain_state.rs     # Per-domain state
│   ├── derived_state.rs    # Computed/cached state
│   └── ui_state.rs         # UI-only state
│
├── view/                   # View functions (Phase 3-5)
│   ├── mod.rs              # View routing
│   ├── home.rs             # Home view
│   ├── export.rs           # Export view
│   └── domain_editor/      # Domain editor tabs
│       ├── mod.rs
│       ├── mapping.rs
│       ├── transform.rs
│       ├── validation.rs
│       ├── preview.rs
│       └── supp.rs
│
├── component/              # Reusable UI components (see 04-component-guide.md)
│   ├── mod.rs
│   ├── master_detail.rs
│   ├── data_table.rs
│   ├── modal.rs
│   ├── tab_bar.rs
│   └── ...
│
├── theme/                  # Theming (see 05-theming.md)
│   ├── mod.rs
│   ├── palette.rs          # Color definitions
│   ├── spacing.rs          # Spacing constants
│   ├── typography.rs       # Text styles
│   └── clinical.rs         # Professional Clinical theme
│
├── service/                # Background services (Phase 3-5)
│   ├── mod.rs
│   ├── study_loader.rs
│   ├── preview.rs
│   ├── export.rs
│   └── update_checker.rs
│
├── menu/                   # Menu system (Phase 6)
│   ├── mod.rs
│   ├── native.rs           # macOS native menu
│   └── in_app.rs           # Windows/Linux menu
│
└── settings/               # Settings persistence
    ├── mod.rs
    └── persistence.rs
```

---

## Data Flow

### User Interaction Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    View     │────▶│   Message   │────▶│   Update    │────▶│    State    │
│  (render)   │     │   (event)   │     │  (logic)    │     │   (data)    │
└─────────────┘     └─────────────┘     └─────────────┘     └──────┬──────┘
       ▲                                                          │
       └──────────────────────────────────────────────────────────┘
                              (re-render)
```

1. **View** renders current state as widgets
2. User interacts with widget (click, type, etc.)
3. Widget produces a **Message**
4. **Update** receives message and modifies state
5. Iced detects state change and re-renders **View**

### Async Task Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Update    │────▶│    Task     │────▶│   Message   │
│  (start)    │     │  (async)    │     │  (result)   │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │   Update    │
                                        │ (complete)  │
                                        └─────────────┘
```

1. **Update** returns a `Task` to perform async work
2. **Task** runs in background (tokio runtime)
3. Task completes and produces a result **Message**
4. **Update** receives result and updates state

### Example: Loading a Study

```rust, no_run
// 1. User clicks "Open Study" button
// 2. View produces message:
Message::Home(HomeMessage::OpenStudyClicked)

// 3. Update handles message:
fn handle_home_message(&mut self, msg: HomeMessage) -> Task<Message> {
    match msg {
        HomeMessage::OpenStudyClicked => {
            // Return task to open file dialog
            Task::perform(
                async { rfd::AsyncFileDialog::new().pick_folder().await },
                |folder| Message::FolderSelected(folder.map(|f| f.path().to_path_buf()))
            )
        }
        // ...
    }
}

// 4. File dialog completes, produces message:
Message::FolderSelected(Some(path))

// 5. Update handles result:
Message::FolderSelected(Some(path)) => {
    // Start loading study in background
    Task::perform(
        async move { load_study(&path).await },
        Message::StudyLoaded
    )
}

// 6. Study loads, produces message:
Message::StudyLoaded(Ok(study))

// 7. Update stores study in state:
Message::StudyLoaded(Ok(study)) => {
    self.study = Some(study);
    self.view = View::Home;
    Task::none()
}
```

---

## Application Lifecycle

### Startup

```rust, no_run
pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window::Settings { ... })
        .run()
}
```

1. `main()` calls `iced::application()` with builder pattern
2. `App::new()` creates initial state and startup tasks
3. Iced creates window and enters event loop
4. `App::subscription()` sets up event listeners

### Runtime

1. Iced calls `App::view()` to render initial UI
2. User interactions produce messages
3. Iced calls `App::update()` for each message
4. State changes trigger re-render
5. Subscriptions (keyboard, timers) produce messages
6. Tasks run in background and produce result messages

### Shutdown

1. User closes window or triggers quit
2. `Message::Menu(MenuMessage::Quit)` is sent
3. Update can perform cleanup (save settings, etc.)
4. Application exits

---

## Key Design Decisions

### 1. Hybrid Menu System

- **macOS**: Native menu via `muda` crate for platform consistency
- **Windows/Linux**: In-app menu for cross-platform control

### 2. Light Theme Only (Initially)

The Professional Clinical theme focuses on light mode for:

- Medical/regulatory aesthetic
- Extended reading sessions
- Print-friendly output

Dark mode may be added in future versions.

### 3. Nested Message Pattern

Complex features use nested enums for organization:

```rust, no_run
Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::AcceptSuggestion(var)))
```

This keeps the root `Message` enum manageable while allowing detailed sub-messages.

### 4. Separation of UI State and Domain State

- **Domain State**: Business data (study, domains, mappings)
- **UI State**: Visual state (selected index, scroll position, dialog open)

This separation enables:

- Cleaner state updates
- Easier testing of business logic
- Clear ownership of concerns

### 5. Component-Based Views

Reusable components encapsulate common patterns:

- `master_detail` - Split pane layout
- `data_table` - Paginated data display
- `modal` - Dialog overlays
- `tab_bar` - Tab navigation

Components are functions that return `Element<Message>`, not custom widgets.

---

## Next Steps

- **[02-message-patterns.md](./02-message-patterns.md)** - Detailed message hierarchy
- **[03-state-management.md](./03-state-management.md)** - State patterns and conventions
- **[04-component-guide.md](./04-component-guide.md)** - Building reusable components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
