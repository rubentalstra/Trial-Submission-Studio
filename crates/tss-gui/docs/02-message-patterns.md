# Trial Submission Studio - Message Patterns

This document describes the message hierarchy and patterns used in Trial Submission Studio.

## Table of Contents

1. [Message Philosophy](#message-philosophy)
2. [Root Message Enum](#root-message-enum)
3. [Nested Message Pattern](#nested-message-pattern)
4. [Message Categories](#message-categories)
5. [Naming Conventions](#naming-conventions)
6. [Common Patterns](#common-patterns)
7. [Anti-Patterns](#anti-patterns)

---

## Message Philosophy

Messages in Iced serve as the **communication channel** between the view (UI) and the update logic. They should:

1. **Describe what happened**, not what to do
2. **Be immutable data** - messages are values, not commands
3. **Carry minimal payload** - only data needed to process the event
4. **Be exhaustively handled** - Rust's match ensures all cases covered

### Good Message Names

```rust, no_run
// Describes what the user did
HomeMessage::OpenStudyClicked
MappingMessage::VariableSelected(index)
ExportMessage::FormatChanged(format)
```

### Poor Message Names

```rust, no_run
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
    Navigate(View),

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
    /// Menu action messages
    Menu(MenuMessage),

    // =========================================================================
    // Background task results
    // =========================================================================
    /// Study loading completed
    StudyLoaded(Result<StudyState, String>),

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

    // =========================================================================
    // Global events
    // =========================================================================
    /// Keyboard event
    KeyPressed(Key, Modifiers),

    /// Periodic tick (for polling, animations)
    Tick,

    /// File dialog returned a folder selection
    FolderSelected(Option<PathBuf>),

    /// No operation - used for placeholder actions
    Noop,
}
```

---

## Nested Message Pattern

Complex features use nested enums to organize related messages:

### Two-Level Nesting

```rust, no_run
// Root → Feature
Message::Home(HomeMessage::OpenStudyClicked)
Message::Export(ExportMessage::StartExport)
Message::Dialog(DialogMessage::CloseAll)
```

### Three-Level Nesting

```rust, no_run
// Root → Feature → Sub-feature
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

```rust, no_run
Message::Navigate(View::Home)
Message::Navigate(View::DomainEditor { domain: "DM".into(), tab: EditorTab::Mapping })
Message::Navigate(View::Export)
```

### 2. User Interaction Messages

Respond to clicks, selections, input:

```rust, no_run
// Home view
HomeMessage::OpenStudyClicked
HomeMessage::RecentStudyClicked(path)
HomeMessage::DomainClicked(domain_code)

// Mapping tab
MappingMessage::VariableSelected(index)
MappingMessage::SearchChanged(query)
MappingMessage::AcceptSuggestion(variable)
MappingMessage::ManualMap { variable, column }
```

### 3. Form Input Messages

Handle text input and selections:

```rust, no_run
// Settings
GeneralSettingsMessage::HeaderRowsChanged(rows)
ExportSettingsMessage::DefaultOutputDirChanged(path)

// SUPP tab
SuppMessage::QnamChanged(value)
SuppMessage::QlabelChanged(value)
```

### 4. Toggle/Switch Messages

Boolean state changes:

```rust, no_run
ValidationSettingsMessage::StrictModeToggled(enabled)
MappingMessage::FilterUnmappedToggled(enabled)
UpdateSettingsMessage::AutoCheckToggled(enabled)
```

### 5. Task Result Messages

Results from background operations:

```rust, no_run
Message::StudyLoaded(Result<StudyState, String>)
Message::PreviewReady { domain, result }
Message::ValidationComplete { domain, report }
ExportMessage::Complete(Result<ExportResult, ExportError>)
```

### 6. Progress Messages

Updates during long operations:

```rust, no_run
ExportMessage::Progress(ExportProgress::StartingDomain(domain))
ExportMessage::Progress(ExportProgress::Step(ExportStep::Validating))
ExportMessage::Progress(ExportProgress::OverallProgress(0.75))
UpdateMessage::InstallProgress(0.5)
```

---

## Naming Conventions

### Message Enum Names

```rust, no_run
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
| Task start   | `Start{Task}`      | `StartExport`, `StartInstall`            |
| Task result  | `{Task}Complete`   | `InstallComplete`, `CheckResult`         |
| Clear/Reset  | `Clear{Item}`      | `ClearMapping`, `ClearSearch`            |

### Payload Naming

```rust, no_run
// Single value: just the type
VariableSelected(usize)
FormatChanged(ExportFormat)

// Multiple values: use named fields
ManualMap { variable: String, column: String }
RuleToggled { index: usize, enabled: bool }

// Complex results: use Result or custom struct
StudyLoaded(Result<StudyState, String>)
Complete(Result<ExportResult, ExportError>)
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

// Handler
fn handle_mapping(&mut self, msg: MappingMessage) -> Task<Message> {
    match msg {
        MappingMessage::VariableSelected(idx) => {
            self.ui.mapping.selected_idx = Some(idx);
            Task::none()
        }
        MappingMessage::SearchChanged(query) => {
            self.ui.mapping.search_filter = query;
            self.ui.mapping.selected_idx = None; // Clear selection on search
            Task::none()
        }
        MappingMessage::SearchCleared => {
            self.ui.mapping.search_filter.clear();
            Task::none()
        }
    }
}
```

### 2. Dialog Open/Close Pattern

```rust
pub enum AboutMessage {
    /// Open the About dialog
    Open,

    /// Close the About dialog
    Close,

    /// Open external link
    OpenWebsite,
    OpenGitHub,
}

// Handler
fn handle_about(&mut self, msg: AboutMessage) -> Task<Message> {
    match msg {
        AboutMessage::Open => {
            self.ui.about.open = true;
            Task::none()
        }
        AboutMessage::Close => {
            self.ui.about.open = false;
            Task::none()
        }
        AboutMessage::OpenWebsite => {
            Task::perform(
                async { open::that("https://example.com") },
                |_| Message::Noop
            )
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

    /// Cancel the export in progress
    CancelExport,

    /// Progress update (from background task)
    Progress(ExportProgress),

    /// Export completed (from background task)
    Complete(Result<ExportResult, ExportError>),
}

// Handler
fn handle_export(&mut self, msg: ExportMessage) -> Task<Message> {
    match msg {
        ExportMessage::StartExport => {
            let config = self.build_export_config();
            self.ui.export.phase = ExportPhase::Exporting;

            // Start background task
            Task::perform(
                async move { run_export(config).await },
                |result| Message::Export(ExportMessage::Complete(result))
            )
        }
        ExportMessage::Progress(progress) => {
            self.ui.export.apply_progress(progress);
            Task::none()
        }
        ExportMessage::Complete(result) => {
            self.ui.export.phase = ExportPhase::Complete;
            self.ui.export.result = Some(result);
            Task::none()
        }
        ExportMessage::CancelExport => {
            // Cancel logic...
            Task::none()
        }
    }
}
```

### 4. Settings Apply/Cancel Pattern

```rust
pub enum SettingsMessage {
    /// Open the Settings dialog
    Open,

    /// Close the Settings dialog (discard changes)
    Close,

    /// Apply changes and close
    Apply,

    /// Reset to default settings
    ResetToDefaults,

    // ... category-specific messages
}

// Handler
fn handle_settings(&mut self, msg: SettingsMessage) -> Task<Message> {
    match msg {
        SettingsMessage::Open => {
            self.ui.settings.pending = Some(self.settings.clone());
            self.ui.settings.open = true;
            Task::none()
        }
        SettingsMessage::Close => {
            self.ui.settings.pending = None;
            self.ui.settings.open = false;
            Task::none()
        }
        SettingsMessage::Apply => {
            if let Some(pending) = self.ui.settings.pending.take() {
                self.settings = pending;
                // Save to disk
                Task::perform(
                    async move { save_settings(&self.settings).await },
                    |_| Message::Noop
                )
            } else {
                Task::none()
            }
        }
        // ...
    }
}
```

---

## Anti-Patterns

### 1. Imperative Message Names

```rust, no_run
// BAD: Describes what to do
Message::SetViewToHome
Message::LoadStudyFromPath(path)
Message::UpdateMappingState

// GOOD: Describes what happened
Message::Navigate(View::Home)
Message::StudyLoaded(result)
MappingMessage::VariableSelected(idx)
```

### 2. Giant Payloads

```rust, no_run
// BAD: Too much data in message
Message::UpdateEverything {
    study: StudyState,
    settings: Settings,
    ui: UiState,
    view: View,
}

// GOOD: Specific, minimal payload
Message::StudyLoaded(Result<StudyState, String>)
```

### 3. Side Effects in Message Creation

```rust, no_run
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

### 4. Duplicate Information

```rust, no_run
// BAD: Domain code in message AND in state path
Message::DomainEditor {
    domain: String,      // Duplicates view state
    msg: DomainEditorMessage,
}

// GOOD: Get domain from current view
Message::DomainEditor(DomainEditorMessage)

// In handler:
if let View::DomainEditor { domain, .. } = &self.view {
    // Use domain from view state
}
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
│      Message::Home(msg) => self.handle_home(msg),               │
│      ...                                                         │
│  }                                                               │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Feature Handler                             │
│  fn handle_home(&mut self, msg: HomeMessage) -> Task<Message>   │
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
│  self.view = View::Home  │     │  Task::perform(...)     │
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

- **[03-state-management.md](./03-state-management.md)** - State organization and patterns
- **[04-component-guide.md](./04-component-guide.md)** - Building reusable components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
