# Trial Submission Studio - Message Patterns

This document describes the message hierarchy and patterns used in Trial
Submission Studio.

## Table of Contents

1. [Message Philosophy](#message-philosophy)
2. [Root Message Enum](#root-message-enum)
3. [Nested Message Pattern](#nested-message-pattern)
4. [Message Categories](#message-categories)
5. [View-Specific Messages](#view-specific-messages)
6. [Naming Conventions](#naming-conventions)
7. [Common Patterns](#common-patterns)
8. [Anti-Patterns](#anti-patterns)

---

## Message Philosophy

Messages in Iced serve as the **communication channel** between the view (UI)
and the update logic. They should:

1. **Describe what happened**, not what to do
2. **Be immutable data** - messages are values, not commands
3. **Carry minimal payload** - only data needed to process the event
4. **Be exhaustively handled** - Rust's match ensures all cases covered

### Good Message Names

```rust
// Describes what the user did
HomeMessage::OpenStudyClicked
MappingMessage::VariableSelected(index)
ExportMessage::FormatChanged(format)
```

### Poor Message Names

```rust
// Describes implementation detail
Message::SetStudyAndNavigateHome(study)  // Too imperative
Message::DoExport                         // Vague
Message::UpdateUI                         // Side-effect focused
```

---

## Root Message Enum

The root `Message` enum in `message/mod.rs` is the entry point for all events:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // =========================================================================
    // Navigation
    // =========================================================================
    /// Navigate to a different view
    Navigate(ViewState),

    /// Change the workflow mode (SDTM, ADaM, SEND)
    SetWorkflowMode(WorkflowMode),

    // =========================================================================
    // View-specific messages
    // =========================================================================
    /// Home view messages
    Home(HomeMessage),

    /// Domain editor messages (includes all tabs)
    DomainEditor(DomainEditorMessage),

    /// Export view messages
    Export(ExportMessage),

    // =========================================================================
    // Dialogs
    // =========================================================================
    /// Dialog messages (About, Settings, ThirdParty, Update)
    Dialog(DialogMessage),

    // =========================================================================
    // Menu
    // =========================================================================
    /// Unified menu action (from native or in-app menu)
    MenuAction(MenuAction),

    /// Legacy menu messages
    Menu(MenuMessage),

    /// Initialize native menu (startup task on macOS)
    InitNativeMenu,

    // =========================================================================
    // Multi-window dialog management
    // =========================================================================
    /// A dialog window was opened
    DialogWindowOpened(DialogType, window::Id),

    /// A dialog window was closed
    DialogWindowClosed(window::Id),

    /// Request to close a specific window
    CloseWindow(window::Id),

    // =========================================================================
    // Background task results
    // =========================================================================
    /// Study loading completed (includes study and terminology registry)
    StudyLoaded(StudyLoadResult),

    /// Preview computation completed for a domain
    PreviewReady {
        domain: String,
        result: Result<DataFrame, String>,
    },

    /// Validation completed for a domain
    ValidationComplete {
        domain: String,
        report: ValidationReport,
    },

    /// Update check completed
    UpdateCheckComplete(Result<Option<UpdateInfo>, String>),

    /// Update is ready to install
    UpdateReadyToInstall {
        info: UpdateInfo,
        data: Arc<Vec<u8>>,
        verified: bool,
    },

    // =========================================================================
    // Global events
    // =========================================================================
    /// Keyboard event
    KeyPressed(Key, Modifiers),

    /// File dialog returned a folder selection
    FolderSelected(Option<PathBuf>),

    /// Dismiss error message
    DismissError,

    // =========================================================================
    // External actions
    // =========================================================================
    /// Open a URL in the system browser
    OpenUrl(String),

    // =========================================================================
    // Toast notifications
    // =========================================================================
    /// Toast notification messages
    Toast(ToastMessage),

    /// No operation - used for placeholder actions
    Noop,
}
```

---

## Nested Message Pattern

Complex features use nested enums to organize related messages:

### Two-Level Nesting

```rust
// Root -> Feature
Message::Home(HomeMessage::OpenStudyClicked)
Message::Export(ExportMessage::StartExport)
Message::Dialog(DialogMessage::About(AboutMessage::Close))
```

### Three-Level Nesting

```rust
// Root -> Feature -> Sub-feature
Message::DomainEditor(DomainEditorMessage::Mapping(MappingMessage::AcceptSuggestion(var)))
Message::Dialog(DialogMessage::Settings(SettingsMessage::General(GeneralSettingsMessage::HeaderRowsChanged(2))))
```

### When to Nest

| Situation                | Recommendation          |
|--------------------------|-------------------------|
| Feature has 2-3 messages | Keep flat in parent     |
| Feature has 4+ messages  | Create sub-enum         |
| Messages share context   | Group in sub-enum       |
| Tab or panel specific    | Create sub-enum per tab |

---

## Message Categories

### 1. Navigation Messages

Control which view is displayed:

```rust
Message::Navigate(ViewState::home())
Message::Navigate(ViewState::domain_editor("DM", EditorTab::Mapping))
Message::Navigate(ViewState::export())
```

### 2. User Interaction Messages

Respond to clicks, selections, input:

```rust
// Home view
HomeMessage::OpenStudyClicked
HomeMessage::RecentStudyClicked(path)
HomeMessage::DomainClicked(domain_code)
HomeMessage::GoToExportClicked

// Mapping tab
MappingMessage::VariableSelected(index)
MappingMessage::SearchChanged(query)
MappingMessage::AcceptSuggestion(variable)
MappingMessage::ManualMap { variable, column }
```

### 3. Form Input Messages

Handle text input and selections:

```rust
// Settings
GeneralSettingsMessage::HeaderRowsChanged(rows)
ExportSettingsMessage::DefaultOutputDirChanged(path)

// SUPP tab
SuppMessage::QnamChanged(value)
SuppMessage::QlabelChanged(value)
SuppMessage::QorigChanged(origin)
```

### 4. Toggle/Switch Messages

Boolean state changes:

```rust
ValidationSettingsMessage::StrictModeToggled(enabled)
MappingMessage::FilterUnmappedToggled(enabled)
MappingMessage::FilterRequiredToggled(enabled)
UpdateSettingsMessage::AutoCheckToggled(enabled)
```

### 5. Task Result Messages

Results from background operations:

```rust
Message::StudyLoaded(Result<(Study, TerminologyRegistry), String>)
Message::PreviewReady { domain, result }
Message::ValidationComplete { domain, report }
Message::UpdateReadyToInstall { info, data, verified }
```

### 6. Window Management Messages

Multi-window dialog lifecycle:

```rust
Message::DialogWindowOpened(DialogType::About, window_id)
Message::DialogWindowClosed(window_id)
Message::CloseWindow(window_id)
```

---

## View-Specific Messages

### HomeMessage

```rust
pub enum HomeMessage {
    // Study selection
    OpenStudyClicked,
    StudyFolderSelected(PathBuf),
    RecentStudyClicked(PathBuf),
    CloseStudyClicked,
    CloseStudyConfirmed,
    CloseStudyCancelled,

    // Navigation
    DomainClicked(String),
    GoToExportClicked,

    // Recent studies management
    RemoveFromRecent(PathBuf),
    ClearAllRecentStudies,
    PruneStaleStudies,
}
```

### DomainEditorMessage

```rust
pub enum DomainEditorMessage {
    /// Switch to a different tab
    TabSelected(EditorTab),

    /// Go back to home view
    BackClicked,

    /// Tab-specific messages
    Mapping(MappingMessage),
    Normalization(NormalizationMessage),
    Validation(ValidationMessage),
    Preview(PreviewMessage),
    Supp(SuppMessage),
}
```

### MappingMessage

```rust
pub enum MappingMessage {
    // Selection
    VariableSelected(usize),
    SearchChanged(String),
    SearchCleared,

    // Mapping actions
    AcceptSuggestion(String),
    ClearMapping(String),
    ManualMap { variable: String, column: String },

    // Not Collected workflow
    MarkNotCollected { variable: String },
    NotCollectedReasonChanged(String),
    NotCollectedSave { variable: String, reason: String },
    NotCollectedCancel,
    EditNotCollectedReason { variable: String, current_reason: String },
    ClearNotCollected(String),

    // Omitted status (Perm variables only)
    MarkOmitted(String),
    ClearOmitted(String),

    // Filters
    FilterUnmappedToggled(bool),
    FilterRequiredToggled(bool),
}
```

### SuppMessage

```rust
pub enum SuppMessage {
    // Navigation & filtering
    ColumnSelected(String),
    SearchChanged(String),
    FilterModeChanged(SuppFilterMode),

    // Field editing
    QnamChanged(String),
    QlabelChanged(String),
    QorigChanged(SuppOrigin),
    QevalChanged(String),

    // Actions
    AddToSupp,
    Skip,
    UndoAction,

    // Edit mode (for included columns)
    StartEdit,
    SaveEdit,
    CancelEdit,
}
```

### DialogMessage

```rust
pub enum DialogMessage {
    /// About dialog
    About(AboutMessage),

    /// Settings dialog
    Settings(SettingsMessage),

    /// Third-party licenses
    ThirdParty(ThirdPartyMessage),

    /// Update dialog
    Update(UpdateMessage),
}
```

---

## Naming Conventions

### Message Enum Names

```rust
// Pattern: {Feature}Message
HomeMessage
DomainEditorMessage
ExportMessage
DialogMessage
MappingMessage      // Sub-feature of DomainEditor
SettingsMessage     // Sub-feature of Dialog
```

### Variant Names

| Action Type  | Pattern            | Examples                                 |
|--------------|--------------------|------------------------------------------|
| Button click | `{Action}Clicked`  | `OpenStudyClicked`, `StartExportClicked` |
| Selection    | `{Item}Selected`   | `VariableSelected`, `TabSelected`        |
| Input change | `{Field}Changed`   | `SearchChanged`, `QnamChanged`           |
| Toggle       | `{Feature}Toggled` | `StrictModeToggled`, `AutoCheckToggled`  |
| Navigation   | `GoTo{View}`       | `GoToExportClicked`, `BackClicked`       |
| Task result  | `{Task}Complete`   | `ValidationComplete`, `StudyLoaded`      |
| Clear/Reset  | `Clear{Item}`      | `ClearMapping`, `ClearSearch`            |

### Payload Naming

```rust
// Single value: just the type
VariableSelected(usize)
FormatChanged(ExportFormat)

// Multiple values: use named fields
ManualMap { variable: String, column: String }
NotCollectedSave { variable: String, reason: String }

// Complex results: use Result or custom struct
StudyLoaded(Result<(Study, TerminologyRegistry), String>)
PreviewReady { domain: String, result: Result<DataFrame, String> }
```

---

## Common Patterns

### 1. List Selection Pattern

```rust
pub enum MappingMessage {
    /// User selected a variable in the list
    VariableSelected(usize),

    /// Clear search filter
    SearchCleared,

    /// Search text changed
    SearchChanged(String),
}

// Handler in app/handler/mapping.rs
fn handle_mapping(&mut self, msg: MappingMessage) -> Task<Message> {
    match msg {
        MappingMessage::VariableSelected(idx) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                mapping_ui.selected_idx = Some(idx);
            }
            Task::none()
        }
        MappingMessage::SearchChanged(query) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                mapping_ui.search_filter = query;
                mapping_ui.selected_idx = None; // Clear selection on search
            }
            Task::none()
        }
        MappingMessage::SearchCleared => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut self.state.view {
                mapping_ui.search_filter.clear();
            }
            Task::none()
        }
    }
}
```

### 2. Multi-Window Dialog Pattern

```rust
pub enum AboutMessage {
    /// Open the About dialog window
    Open,

    /// Close the About dialog window
    Close,

    /// Open external link
    OpenWebsite,
    OpenGitHub,
}

// Handler opens a new window
fn handle_about(&mut self, msg: AboutMessage) -> Task<Message> {
    match msg {
        AboutMessage::Open => {
            // Check if already open
            if self.state.dialog_windows.about.is_some() {
                return Task::none();
            }
            // Open new window
            let settings = window::Settings {
                size: Size::new(400.0, 300.0),
                resizable: false,
                ..Default::default()
            };
            let (id, task) = window::open(settings);
            task.map(move |_| Message::DialogWindowOpened(DialogType::About, id))
        }
        AboutMessage::Close => {
            if let Some(id) = self.state.dialog_windows.about {
                self.state.dialog_windows.about = None;
                return window::close(id);
            }
            Task::none()
        }
        AboutMessage::OpenWebsite => {
            let _ = open::that("https://trialsubmissionstudio.com");
            Task::none()
        }
        // ...
    }
}
```

### 3. Async Task Pattern

```rust
pub enum ExportMessage {
    /// Start the export process
    StartExport,

    /// Progress update (from background task)
    Progress { domain: String, step: String, progress: f32 },

    /// Export completed (from background task)
    Complete(Result<ExportResult, String>),
}

// Handler
fn handle_export(&mut self, msg: ExportMessage) -> Task<Message> {
    match msg {
        ExportMessage::StartExport => {
            let config = self.build_export_config();

            // Open progress dialog window
            let (id, open_task) = window::open(progress_window_settings());

            // Chain: open window, then start export
            open_task
                .map(move |_| Message::DialogWindowOpened(DialogType::ExportProgress, id))
                .chain(Task::perform(
                    async move { run_export(config).await },
                    |result| Message::Export(ExportMessage::Complete(result))
                ))
        }
        ExportMessage::Progress { domain, step, progress } => {
            if let Some((_, ref mut state)) = self.state.dialog_windows.export_progress {
                state.current_domain = Some(domain);
                state.current_step = step;
                state.progress = progress;
            }
            Task::none()
        }
        ExportMessage::Complete(result) => {
            // Close progress window, open complete window
            // ...
            Task::none()
        }
    }
}
```

### 4. Settings Edit Pattern

```rust
pub enum SettingsMessage {
    /// Switch settings category
    CategorySelected(SettingsCategory),

    /// Category-specific messages
    General(GeneralSettingsMessage),
    Export(ExportSettingsMessage),
    Validation(ValidationSettingsMessage),
    Update(UpdateSettingsMessage),
    Display(DisplaySettingsMessage),
    Developer(DeveloperSettingsMessage),

    /// Save all settings
    Save,

    /// Reset to defaults
    ResetToDefaults,
}

// Settings changes are applied directly to state.settings
// and saved on SettingsMessage::Save
```

---

## Anti-Patterns

### 1. Imperative Message Names

```rust
// BAD: Describes what to do
Message::SetViewToHome
Message::LoadStudyFromPath(path)
Message::UpdateMappingState

// GOOD: Describes what happened
Message::Navigate(ViewState::home())
Message::StudyLoaded(result)
MappingMessage::VariableSelected(idx)
```

### 2. Giant Payloads

```rust
// BAD: Too much data in message
Message::UpdateEverything {
study: Study,
settings: Settings,
view: ViewState,
}

// GOOD: Specific, minimal payload
Message::StudyLoaded(Result<(Study, TerminologyRegistry), String>)
```

### 3. Side Effects in Message Creation

```rust
// BAD: Side effect in view
button("Save").on_press({
save_to_disk(); // Side effect!
Message::Saved
})

// GOOD: Side effect in update
button("Save").on_press(Message::SaveClicked)

// In update:
Message::SaveClicked => {
Task::perform(
async { save_to_disk().await },
Message::SaveComplete
)
}
```

### 4. Mixing Window ID in State Path

```rust
// BAD: Window ID duplicated
Message::DialogContent {
window_id: window::Id,  // Already tracked in dialog_windows
content: DialogContent,
}

// GOOD: Use dialog_windows to track window IDs
Message::DialogWindowOpened(DialogType::About, id)
// Later: self.state.dialog_windows.dialog_type(id)
```

---

## Message Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interaction                          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Root Message Enum                           │
│  Message::Home(HomeMessage::OpenStudyClicked)                   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         App::update()                            │
│  match message {                                                 │
│      Message::Home(msg) => self.handle_home_message(msg),       │
│      ...                                                         │
│  }                                                               │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Handler (app/handler/*.rs)                     │
│  fn handle_home_message(&mut self, msg: HomeMessage)            │
│  match msg {                                                     │
│      HomeMessage::OpenStudyClicked => Task::perform(...),       │
│      ...                                                         │
│  }                                                               │
└─────────────────────────────┬───────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              │                               │
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│     State Update         │     │    Background Task      │
│  self.state.view = ...   │     │  Task::perform(...)     │
└─────────────────────────┘     └────────────┬────────────┘
                                             │
                                             ▼
                                ┌─────────────────────────┐
                                │    Result Message       │
                                │  Message::StudyLoaded   │
                                └─────────────────────────┘
```

---

## Next Steps

- **[03-state-management.md](./03-state-management.md)** - State organization and
  patterns
- **[04-component-guide.md](./04-component-guide.md)** - Building reusable
  components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
