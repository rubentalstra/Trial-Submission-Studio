# Trial Submission Studio - Architecture Guide

This document describes the overall architecture of the Trial Submission Studio
GUI application, built with Iced 0.14.0.

## Table of Contents

1. [Overview](#overview)
2. [The Elm Architecture](#the-elm-architecture)
3. [Module Structure](#module-structure)
4. [Data Flow](#data-flow)
5. [Application Lifecycle](#application-lifecycle)
6. [Key Design Decisions](#key-design-decisions)

---

## Overview

Trial Submission Studio uses **Iced 0.14.0**, a cross-platform GUI library for
Rust inspired by Elm. The application follows the Elm architecture pattern,
which provides:

- **Predictable state management** - All state changes flow through a single
  `update` function
- **Type-safe UI** - The view is a pure function of state
- **Declarative rendering** - UI is described as data, not imperative commands
- **Async-friendly** - Built-in support for background tasks via `Task`
- **Multi-window support** - Dialogs run as separate windows

### Technology Stack

| Component            | Technology                |
|----------------------|---------------------------|
| GUI Framework        | Iced 0.14.0               |
| Async Runtime        | Tokio                     |
| Graphics Backend     | wgpu (Metal/Vulkan/DX12)  |
| Icon Font            | iced_fonts 0.3.0 (Lucide) |
| Menu (macOS)         | muda (native)             |
| Menu (Windows/Linux) | In-app menu               |

---

## The Elm Architecture

Iced implements the Elm architecture with four core concepts:

### 1. State

The application state is held in the `App` struct:

```rust
pub struct App {
    /// All application state.
    pub state: AppState,
}
```

The `AppState` struct contains all application data:

```rust
pub struct AppState {
    /// Current view and its associated UI state.
    pub view: ViewState,

    /// Loaded study data.
    pub study: Option<Study>,

    /// User settings (persisted to disk).
    pub settings: Settings,

    /// CDISC Controlled Terminology registry.
    pub terminology: Option<TerminologyRegistry>,

    /// Current error message to display (transient).
    pub error: Option<String>,

    /// Whether a background task is running.
    pub is_loading: bool,

    /// Menu dropdown state (Windows/Linux only).
    #[cfg(not(target_os = "macos"))]
    pub menu_dropdown: MenuDropdownState,

    /// Tracks open dialog windows (multi-window mode).
    pub dialog_windows: DialogWindows,

    /// Main window ID.
    pub main_window_id: Option<window::Id>,

    /// Active toast notification.
    pub toast: Option<ToastState>,
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
    // Navigation
    Navigate(ViewState),
    SetWorkflowMode(WorkflowMode),

    // View-specific messages
    Home(HomeMessage),
    DomainEditor(DomainEditorMessage),
    Export(ExportMessage),

    // Dialogs
    Dialog(DialogMessage),

    // Menu
    MenuAction(MenuAction),
    Menu(MenuMessage),
    InitNativeMenu,

    // Multi-window dialog management
    DialogWindowOpened(DialogType, window::Id),
    DialogWindowClosed(window::Id),
    CloseWindow(window::Id),

    // Background task results
    StudyLoaded(StudyLoadResult),
    PreviewReady { domain, result },
    ValidationComplete { domain, report },
    UpdateCheckComplete(Result<Option<UpdateInfo>, String>),
    UpdateReadyToInstall { info, data, verified },

    // Global events
    KeyPressed(Key, Modifiers),
    FolderSelected(Option<PathBuf>),
    DismissError,

    // External actions
    OpenUrl(String),

    // Toast notifications
    Toast(ToastMessage),

    Noop,
}
```

**Rules:**

- Messages are immutable data
- Messages should be descriptive of _what happened_, not _what to do_
- Nested messages for complex features (see `02-message-patterns.md`)

### 3. Update

The update function processes messages and returns new state plus optional
tasks:

```rust
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(view_state) => {
                self.state.view = view_state;
                Task::none()
            }
            Message::Home(msg) => self.handle_home_message(msg),
            Message::DomainEditor(msg) => self.handle_domain_editor_message(msg),
            Message::Export(msg) => self.handle_export_message(msg),
            Message::Dialog(msg) => self.handle_dialog_message(msg),
            // ... more handlers
        }
    }
}
```

**Rules:**

- Update is the _only_ place state changes happen
- Keep update handlers focused and delegate to helper methods
- Return `Task::none()` when no async work is needed
- Return `Task::perform()` or `Task::done()` for async operations

### 4. View

The view function renders the current state as UI elements:

```rust
impl App {
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        // Check if this is a dialog window
        if let Some(dialog_type) = self.state.dialog_windows.dialog_type(id) {
            return match dialog_type {
                DialogType::About => view_about_dialog_content(id),
                DialogType::Settings => view_settings_dialog_content(...),
                // ... other dialogs
            };
        }

        // Main window content
        match &self.state.view {
            ViewState::Home { .. } => view_home(&self.state),
            ViewState::DomainEditor { domain, tab, .. } => {
                view_domain_editor(&self.state, domain, *tab)
            }
            ViewState::Export(_) => view_export(&self.state),
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
├── main.rs                     # Entry point, Iced app runner
├── lib.rs                      # Library exports
│
├── app/                        # Main App struct
│   ├── mod.rs                  # App implementation (new, update, view, etc.)
│   ├── util.rs                 # Utilities (icon loading, study loading)
│   └── handler/                # Message handlers by category
│       ├── mod.rs
│       ├── home.rs             # HomeMessage handler
│       ├── domain_editor.rs    # DomainEditorMessage handler
│       ├── mapping.rs          # MappingMessage handler
│       ├── normalization.rs    # NormalizationMessage handler
│       ├── validation.rs       # ValidationMessage handler
│       ├── preview.rs          # PreviewMessage handler
│       ├── supp.rs             # SuppMessage handler
│       ├── export.rs           # ExportMessage handler
│       ├── dialog.rs           # DialogMessage handler
│       ├── menu.rs             # MenuMessage handler
│       └── keyboard.rs         # Keyboard shortcuts
│
├── message/                    # Message types (see 02-message-patterns.md)
│   ├── mod.rs                  # Root Message enum
│   ├── home.rs                 # HomeMessage
│   ├── domain_editor.rs        # DomainEditorMessage + sub-messages
│   ├── export.rs               # ExportMessage
│   ├── dialog.rs               # DialogMessage + sub-messages
│   └── menu.rs                 # MenuMessage
│
├── state/                      # Application state (see 03-state-management.md)
│   ├── mod.rs                  # AppState, DialogWindows, DialogType
│   ├── study.rs                # Study struct
│   ├── domain_state.rs         # DomainState, SuppColumnConfig
│   ├── view_state.rs           # ViewState, EditorTab, UI state types
│   └── settings.rs             # Settings, RecentStudy, preferences
│
├── view/                       # View functions
│   ├── mod.rs                  # View routing and exports
│   ├── home/                   # Home view
│   │   ├── mod.rs              # view_home router
│   │   ├── welcome.rs          # Welcome screen (no study)
│   │   └── study.rs            # Study overview (study loaded)
│   ├── domain_editor/          # Domain editor tabs
│   │   ├── mod.rs              # view_domain_editor router
│   │   ├── mapping.rs          # Mapping tab
│   │   ├── normalization.rs    # Normalization tab
│   │   ├── validation.rs       # Validation tab
│   │   ├── preview.rs          # Preview tab
│   │   └── supp.rs             # SUPP qualifier tab
│   ├── export.rs               # Export view
│   └── dialog/                 # Dialog windows
│       ├── mod.rs              # Dialog exports
│       ├── about.rs            # About dialog
│       ├── settings.rs         # Settings dialog
│       ├── third_party.rs      # Third-party licenses
│       ├── update.rs           # Update dialog
│       ├── export.rs           # Export progress/complete dialogs
│       └── close_study.rs      # Close study confirmation
│
├── component/                  # Reusable UI components (see 04-component-guide.md)
│   ├── mod.rs                  # Component exports
│   ├── master_detail.rs        # Split pane layout
│   ├── sidebar.rs              # Navigation sidebar
│   ├── tab_bar.rs              # Tab navigation
│   ├── master_panel.rs         # Master panel with header
│   ├── search_filter_bar.rs    # Search + filter bar
│   ├── modal.rs                # Modal dialogs
│   ├── progress_modal.rs       # Progress overlays
│   ├── form_field.rs           # Form inputs
│   ├── search_box.rs           # Search with clear
│   ├── text_field.rs           # Text/textarea fields
│   ├── data_table.rs           # Paginated tables
│   ├── status_badge.rs         # Status indicators
│   ├── metadata_card.rs        # Metadata display
│   ├── status_card.rs          # Status cards
│   ├── empty_state.rs          # Empty/Loading/Error states
│   ├── section_card.rs         # Section containers
│   ├── core_badge.rs           # Core designation badges
│   ├── domain_badge.rs         # Domain code badges
│   ├── progress_bar.rs         # Progress bars
│   ├── domain_card.rs          # Domain cards for home
│   ├── selectable_row.rs       # Selectable list rows
│   ├── variable_list_item.rs   # Variable list items
│   ├── detail_header.rs        # Detail panel headers
│   ├── page_header.rs          # Page headers
│   ├── action_button.rs        # Action buttons
│   ├── toast.rs                # Toast notifications
│   └── icon.rs                 # Icon helpers
│
├── theme/                      # Theming (see 05-theming.md)
│   ├── mod.rs                  # Theme exports
│   ├── palette.rs              # Color definitions
│   ├── spacing.rs              # Spacing constants
│   ├── typography.rs           # Text styles
│   └── clinical.rs             # Professional Clinical theme
│
├── service/                    # Background services
│   ├── mod.rs                  # Service exports
│   ├── export.rs               # Export service
│   ├── preview.rs              # Preview generation
│   └── validation.rs           # Validation service
│
└── menu/                       # Menu system
    ├── mod.rs                  # Menu exports
    ├── common.rs               # Shared menu types
    ├── macos/                  # macOS native menu
    │   ├── mod.rs
    │   ├── menu_bar.rs         # Native menu bar
    │   ├── recent_studies.rs   # Recent studies submenu
    │   └── subscription.rs     # Menu event subscription
    └── desktop/                # Windows/Linux in-app menu
        ├── mod.rs
        ├── menu_bar.rs         # In-app menu bar
        ├── components.rs       # Menu components
        └── state.rs            # Dropdown state
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

```rust
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

// 5. Update handles result, starts study loading:
Message::FolderSelected(Some(path)) => {
self.state.is_loading = true;
Task::perform(
async move { load_study( & path).await },
Message::StudyLoaded
)
}

// 6. Study loads, produces message:
Message::StudyLoaded(Ok((study, terminology)))

// 7. Update stores study in state:
Message::StudyLoaded(Ok((study, terminology))) => {
self.state.study = Some(study);
self.state.terminology = Some(terminology);
self.state.is_loading = false;
self.state.view = ViewState::home();
Task::none()
}
```

---

## Application Lifecycle

### Startup

```rust
pub fn main() -> iced::Result {
    iced::daemon(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
```

1. `main()` calls `iced::daemon()` (multi-window mode)
2. `App::new()` creates initial state and startup tasks
3. Main window is opened explicitly via `window::open()`
4. `App::subscription()` sets up event listeners

### Runtime

1. Iced calls `App::view(id)` to render each window
2. User interactions produce messages
3. Iced calls `App::update()` for each message
4. State changes trigger re-render
5. Subscriptions (keyboard, timers, menu) produce messages
6. Tasks run in background and produce result messages

### Multi-Window Architecture

The application uses Iced's daemon mode for multi-window support:

```rust
pub fn view(&self, id: window::Id) -> Element<'_, Message> {
    // Check if this is a dialog window
    if let Some(dialog_type) = self.state.dialog_windows.dialog_type(id) {
        return match dialog_type {
            DialogType::About => view_about_dialog_content(id),
            DialogType::Settings => view_settings_dialog_content(...),
            // ...
        };
    }

    // Main window content
    // ...
}
```

Dialog windows are tracked in `DialogWindows`:

```rust
pub struct DialogWindows {
    pub about: Option<window::Id>,
    pub settings: Option<(window::Id, SettingsCategory)>,
    pub third_party: Option<(window::Id, ThirdPartyState)>,
    pub update: Option<(window::Id, UpdateState)>,
    pub close_study_confirm: Option<window::Id>,
    pub export_progress: Option<(window::Id, ExportProgressState)>,
    pub export_complete: Option<(window::Id, ExportResult)>,
}
```

### Shutdown

1. User closes main window or triggers quit
2. `Message::DialogWindowClosed(id)` is sent
3. If it's the main window, `iced::exit()` is called
4. Application exits

---

## Key Design Decisions

### 1. Hybrid Menu System

- **macOS**: Native menu via `muda` crate for platform consistency
- **Windows/Linux**: In-app menu bar for cross-platform control

```rust
// macOS: Native menu with subscription for events
#[cfg(target_os = "macos")]
let menu_sub = crate::menu::menu_subscription().map( | action|...);

// Windows/Linux: In-app menu rendered in view
#[cfg(not(target_os = "macos"))]
let content_with_menu = column![menu_bar, content];
```

### 2. Multi-Window Dialogs

Dialogs run as separate windows, not overlays:

- About, Settings, Third-Party, Update dialogs
- Export progress and completion dialogs
- Close study confirmation

This provides:

- Better platform integration
- Cleaner focus management
- Independent window positioning

### 3. ViewState Contains UI State

UI state for each view is embedded in `ViewState`:

```rust
pub enum ViewState {
    Home {
        workflow_mode: WorkflowMode,
        selected_domain_idx: Option<usize>,
        search_filter: String,
    },
    DomainEditor {
        domain: String,
        tab: EditorTab,
        preview_cache: Option<DataFrame>,
        mapping_ui: MappingUiState,
        normalization_ui: NormalizationUiState,
        // ... per-tab UI state
    },
    Export(ExportViewState),
}
```

This keeps UI state localized with its view.

### 4. Nested Message Pattern

Complex features use nested enums for organization:

```rust
Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::AcceptSuggestion(var)))
```

This keeps the root `Message` enum manageable while allowing detailed
sub-messages.

### 5. Handler Modules

Message handlers are organized by feature in `app/handler/`:

```rust
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Home(msg) => self.handle_home_message(msg),
            Message::DomainEditor(msg) => self.handle_domain_editor_message(msg),
            // Each handler is in its own module
        }
    }
}
```

### 6. Toast Notifications

Toasts provide non-intrusive feedback:

```rust
pub struct ToastState {
    pub message: String,
    pub variant: ToastVariant,
    pub action: Option<ToastAction>,
}
```

Toasts auto-dismiss via subscription timer:

```rust
let toast_sub = if self .state.toast.is_some() {
time::every(Duration::from_secs(5))
.map( | _ | Message::Toast(ToastMessage::Dismiss))
} else {
Subscription::none()
};
```

### 7. Light Theme Only (Initially)

The Professional Clinical theme focuses on light mode for:

- Medical/regulatory aesthetic
- Extended reading sessions
- Print-friendly output

Dark mode may be added in future versions.

---

## Next Steps

- **[02-message-patterns.md](./02-message-patterns.md)** - Detailed message
  hierarchy
- **[03-state-management.md](./03-state-management.md)** - State patterns and
  conventions
- **[04-component-guide.md](./04-component-guide.md)** - Building reusable
  components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
