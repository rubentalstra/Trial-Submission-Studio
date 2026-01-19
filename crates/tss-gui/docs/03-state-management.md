# Trial Submission Studio - State Management

This document describes state organization and patterns used in Trial Submission
Studio.

## Table of Contents

1. [State Philosophy](#state-philosophy)
2. [State Architecture](#state-architecture)
3. [AppState - Root State](#appstate---root-state)
4. [ViewState - View + UI State](#viewstate---view--ui-state)
5. [Study and Domain State](#study-and-domain-state)
6. [Dialog Windows State](#dialog-windows-state)
7. [Settings State](#settings-state)
8. [State Update Patterns](#state-update-patterns)
9. [Common Patterns](#common-patterns)
10. [Anti-Patterns](#anti-patterns)

---

## State Philosophy

State management in Trial Submission Studio follows these principles:

1. **Single source of truth** - All state lives in `App.state: AppState`
2. **View-scoped UI state** - UI state lives inside `ViewState` variants
3. **Immutable updates** - State is updated only in `update()`, never in
   `view()`
4. **Derived data on demand** - Compute derived values when needed, don't cache
5. **Multi-window awareness** - Dialog windows track their own IDs and state

### Why View-Scoped UI State?

Instead of separate `UiState` struct, UI state is embedded in `ViewState`:

```rust
// Each view holds its own UI state
pub enum ViewState {
    Home { workflow_mode: WorkflowMode },
    DomainEditor {
        domain: String,
        tab: EditorTab,
        mapping_ui: MappingUiState,      // Tab-specific UI
        validation_ui: ValidationUiState,
        // ...
    },
    Export(ExportViewState),
}
```

Benefits:

- **Automatic cleanup** - Navigation clears transient state
- **Clear ownership** - UI state belongs to its view
- **No synchronization** - No separate state containers to keep in sync

---

## State Architecture

```
App
└── state: AppState
    ├── view: ViewState                 # Current view + its UI state
    │   └── (view-specific UI state)
    │
    ├── study: Option<Study>            # Loaded study data
    │   └── domains: BTreeMap<String, DomainState>
    │       ├── source: DomainSource    # Original CSV data
    │       ├── mapping: ColumnMapping  # Variable mappings
    │       └── validation_cache: ...   # Cached validation
    │
    ├── settings: Settings              # Persisted preferences
    │   ├── general: GeneralSettings
    │   ├── export: ExportSettings
    │   └── ...
    │
    ├── terminology: Option<TerminologyRegistry>  # CDISC CT
    │
    ├── dialog_windows: DialogWindows   # Multi-window tracking
    │   ├── about: Option<window::Id>
    │   ├── settings: Option<(window::Id, SettingsCategory)>
    │   └── ...
    │
    ├── main_window_id: Option<window::Id>
    ├── toast: Option<ToastState>
    ├── error: Option<String>
    └── is_loading: bool
```

---

## AppState - Root State

`AppState` is the root container for all application state:

```rust
#[derive(Default)]
pub struct AppState {
    /// Current view and its associated UI state.
    pub view: ViewState,

    /// Loaded study data (None when no study is open).
    pub study: Option<Study>,

    /// User settings (persisted to disk).
    pub settings: Settings,

    /// CDISC Controlled Terminology registry.
    pub terminology: Option<TerminologyRegistry>,

    /// Current error message to display.
    pub error: Option<String>,

    /// Whether a background task is running.
    pub is_loading: bool,

    /// Menu dropdown state (Windows/Linux only).
    #[cfg(not(target_os = "macos"))]
    pub menu_dropdown: MenuDropdownState,

    /// Tracks open dialog windows.
    pub dialog_windows: DialogWindows,

    /// Main window ID.
    pub main_window_id: Option<window::Id>,

    /// Active toast notification.
    pub toast: Option<ToastState>,
}
```

### Accessors

```rust
impl AppState {
    /// Get domain by code.
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        self.study.as_ref()?.domain(code)
    }

    /// Check if a study is loaded.
    pub fn has_study(&self) -> bool {
        self.study.is_some()
    }
}
```

---

## ViewState - View + UI State

`ViewState` combines navigation with view-specific UI state:

```rust
#[derive(Debug, Clone)]
pub enum ViewState {
    /// Home screen - study selection and overview.
    Home {
        workflow_mode: WorkflowMode,
    },

    /// Domain editor with tabbed interface.
    DomainEditor {
        /// Domain code being edited (e.g., "DM", "AE").
        domain: String,
        /// Active tab.
        tab: EditorTab,
        /// Mapping tab UI state.
        mapping_ui: MappingUiState,
        /// Normalization tab UI state.
        normalization_ui: NormalizationUiState,
        /// Validation tab UI state.
        validation_ui: ValidationUiState,
        /// Preview tab UI state.
        preview_ui: PreviewUiState,
        /// SUPP tab UI state.
        supp_ui: SuppUiState,
        /// Cached preview DataFrame.
        preview_cache: Option<DataFrame>,
    },

    /// Export screen.
    Export(ExportViewState),
}
```

### Constructor Helpers

```rust
impl ViewState {
    /// Create home view state.
    pub fn home() -> Self {
        Self::Home {
            workflow_mode: WorkflowMode::default(),
        }
    }

    /// Create domain editor view state.
    pub fn domain_editor_with_rows(
        domain: impl Into<String>,
        tab: EditorTab,
        rows_per_page: usize,
    ) -> Self {
        Self::DomainEditor {
            domain: domain.into(),
            tab,
            mapping_ui: MappingUiState::default(),
            normalization_ui: NormalizationUiState::default(),
            validation_ui: ValidationUiState::default(),
            preview_ui: PreviewUiState::with_rows_per_page(rows_per_page),
            supp_ui: SuppUiState::default(),
            preview_cache: None,
        }
    }

    /// Create export view state.
    pub fn export() -> Self {
        Self::Export(ExportViewState::default())
    }
}
```

### Tab-Specific UI States

Each editor tab has its own UI state struct:

```rust
/// Mapping tab UI state.
pub struct MappingUiState {
    pub selected_variable: Option<usize>,
    pub search_filter: String,
    pub filter_unmapped: bool,
    pub filter_required: bool,
    pub not_collected_edit: Option<NotCollectedEdit>,
}

/// Validation tab UI state.
pub struct ValidationUiState {
    pub selected_issue: Option<usize>,
    pub severity_filter: SeverityFilter,
}

/// Preview tab UI state.
pub struct PreviewUiState {
    pub current_page: usize,
    pub rows_per_page: usize,
    pub is_rebuilding: bool,
    pub error: Option<String>,
}

/// SUPP tab UI state.
pub struct SuppUiState {
    pub selected_column: Option<String>,
    pub search_filter: String,
    pub filter_mode: SuppFilterMode,
    pub edit_draft: Option<SuppEditDraft>,
}
```

---

## Study and Domain State

### Study

Represents a loaded study folder:

```rust
pub struct Study {
    /// Study identifier (derived from folder name).
    pub study_id: String,

    /// Path to the study folder.
    pub study_folder: PathBuf,

    /// Study metadata (Items.csv, CodeLists.csv) if available.
    pub metadata: Option<StudyMetadata>,

    /// DomainStates indexed by code (e.g., "DM", "AE", "LB").
    domains: BTreeMap<String, DomainState>,
}

impl Study {
    pub fn domain(&self, code: &str) -> Option<&DomainState>;
    pub fn domain_mut(&mut self, code: &str) -> Option<&mut DomainState>;
    pub fn domain_codes_dm_first(&self) -> Vec<&str>;
    pub fn domain_count(&self) -> usize;
    pub fn total_rows(&self) -> usize;
}
```

### DomainState

Holds data and configuration for a single domain:

```rust
pub struct DomainState {
    /// Domain code (e.g., "DM", "AE").
    pub domain_code: String,

    /// Source CSV data.
    pub source: DomainSource,

    /// Variable mappings (source column -> SDTM variable).
    pub mapping: ColumnMapping,

    /// SUPP qualifier configuration.
    pub supp_config: SuppConfig,

    /// Cached validation results.
    pub validation_cache: Option<ValidationReport>,
}
```

### DomainSource

Original CSV data:

```rust
pub struct DomainSource {
    /// Path to the source CSV file.
    pub file_path: PathBuf,

    /// Source DataFrame (Polars).
    pub data: DataFrame,

    /// Column names from source.
    pub columns: Vec<String>,
}
```

---

## Dialog Windows State

Multi-window dialogs track their window IDs and associated state:

```rust
pub struct DialogWindows {
    /// About dialog window ID.
    pub about: Option<window::Id>,

    /// Settings dialog window ID and current category.
    pub settings: Option<(window::Id, SettingsCategory)>,

    /// Third-party licenses dialog.
    pub third_party: Option<(window::Id, ThirdPartyState)>,

    /// Update dialog.
    pub update: Option<(window::Id, UpdateState)>,

    /// Close study confirmation dialog.
    pub close_study_confirm: Option<window::Id>,

    /// Export progress dialog.
    pub export_progress: Option<(window::Id, ExportProgressState)>,

    /// Export completion dialog.
    pub export_complete: Option<(window::Id, ExportResult)>,
}
```

### Helper Methods

```rust
impl DialogWindows {
    /// Check if a window ID belongs to any dialog.
    pub fn is_dialog_window(&self, id: window::Id) -> bool;

    /// Get the dialog type for a window ID.
    pub fn dialog_type(&self, id: window::Id) -> Option<DialogType>;

    /// Close a dialog window by ID.
    pub fn close(&mut self, id: window::Id);
}
```

### Dialog State Examples

```rust
/// Export progress state.
pub struct ExportProgressState {
    pub current_domain: Option<String>,
    pub current_step: String,
    pub progress: f32,
    pub files_written: usize,
}

/// Update dialog state.
pub enum UpdateState {
    Checking,
    NoUpdate,
    Available(UpdateInfo),
    Downloading { progress: f32 },
    ReadyToInstall { info: UpdateInfo, data: Vec<u8>, verified: bool },
    Installing,
    Error(String),
}
```

---

## Settings State

User preferences persisted to disk:

```rust
pub struct Settings {
    pub general: GeneralSettings,
    pub export: ExportSettings,
    pub validation: ValidationSettings,
    pub update: UpdateSettings,
    pub display: DisplaySettings,
    pub developer: DeveloperSettings,
}

impl Settings {
    /// Load settings from disk (or defaults).
    pub fn load() -> Self;

    /// Save settings to disk.
    pub fn save(&self) -> Result<(), io::Error>;
}
```

### Recent Studies

```rust
pub struct GeneralSettings {
    pub recent_studies: Vec<RecentStudy>,
    // ...
}

pub struct RecentStudy {
    pub id: Uuid,
    pub path: PathBuf,
    pub display_name: String,
    pub workflow_type: WorkflowType,
    pub domain_count: usize,
    pub row_count: usize,
    pub last_opened: DateTime<Utc>,
}
```

---

## State Update Patterns

### 1. Direct State Mutation

For simple, synchronous updates:

```rust
Message::Navigate(view_state) => {
self.state.view = view_state;
Task::none()
}

MappingMessage::SearchChanged(query) => {
if let ViewState::DomainEditor { mapping_ui, .. } = & mut self.state.view {
mapping_ui.search_filter = query;
mapping_ui.selected_variable = None;
}
Task::none()
}
```

### 2. Pattern Matching ViewState

Access view-specific state:

```rust
// Read from ViewState
if let ViewState::DomainEditor { domain, tab, mapping_ui,..} = & self .state.view {
// Use domain, tab, mapping_ui
}

// Write to ViewState
if let ViewState::DomainEditor { mapping_ui,..} = & mut self .state.view {
mapping_ui.selected_variable = Some(idx);
}
```

### 3. Async State Update

Background work with `Task::perform`:

```rust
HomeMessage::OpenStudyClicked => {
Task::perform(
async { rfd::AsyncFileDialog::new().pick_folder().await },
| folder | Message::FolderSelected(folder.map( | f | f.path().to_path_buf()))
)
}

Message::FolderSelected(Some(path)) => {
self.state.is_loading = true;
Task::perform(
async move { load_study( & path).await },
Message::StudyLoaded
)
}

Message::StudyLoaded(Ok((study, terminology))) => {
self.state.study = Some(study);
self.state.terminology = Some(terminology);
self.state.is_loading = false;
self.state.view = ViewState::home();
Task::none()
}
```

### 4. Cached Data Invalidation

Clear cached data when source changes:

```rust
MappingMessage::AcceptSuggestion(variable) => {
// Update mapping
if let Some(domain_state) = self.state.study.as_mut()
.and_then( |s | s.domain_mut( & domain))
{
domain_state.mapping.accept_suggestion( & variable);
// Invalidate cached validation
domain_state.validation_cache = None;
}

// Clear preview cache in view
if let ViewState::DomainEditor { preview_cache, .. } = & mut self.state.view {
* preview_cache = None;
}

Task::none()
}
```

### 5. Multi-Window Dialog Opening

```rust
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

// Map task to store window ID
task.map( move | _ | Message::DialogWindowOpened(DialogType::About, id))
}

Message::DialogWindowOpened(DialogType::About, id) => {
self.state.dialog_windows.about = Some(id);
Task::none()
}
```

---

## Common Patterns

### 1. Optional Domain Access

```rust
impl AppState {
    pub fn domain(&self, code: &str) -> Option<&DomainState> {
        self.study.as_ref()?.domain(code)
    }
}

// Usage
if let Some(domain) = self .state.domain("DM") {
// Work with domain
}
```

### 2. Edit Draft Pattern (SUPP)

Temporary edits before committing:

```rust
pub struct SuppUiState {
    /// Edit draft for already-included columns.
    /// When Some, user is editing an included column.
    pub edit_draft: Option<SuppEditDraft>,
}

pub struct SuppEditDraft {
    pub qnam: String,
    pub qlabel: String,
    pub qorig: SuppOrigin,
    pub qeval: String,
}

impl SuppEditDraft {
    pub fn from_config(config: &SuppColumnConfig) -> Self {
        // Copy current values to draft
    }
}

// Save: apply draft to actual config
// Cancel: discard draft
```

### 3. Export Selection State

Track selected domains for export:

```rust
pub struct ExportViewState {
    pub selected_domains: HashSet<String>,
    pub output_dir: Option<PathBuf>,
    pub phase: ExportPhase,
}

impl ExportViewState {
    pub fn toggle_domain(&mut self, domain: &str) {
        if self.selected_domains.contains(domain) {
            self.selected_domains.remove(domain);
        } else {
            self.selected_domains.insert(domain.to_string());
        }
    }

    pub fn can_export(&self) -> bool {
        !self.selected_domains.is_empty() && !self.phase.is_exporting()
    }
}
```

### 4. Toast Notification State

```rust
pub struct ToastState {
    pub message: String,
    pub variant: ToastVariant,
    pub action: Option<ToastAction>,
}

// Show toast
self .state.toast = Some(ToastState::success("Export complete"));

// Auto-dismiss via subscription
let toast_sub = if self .state.toast.is_some() {
time::every(Duration::from_secs(5))
.map( | _ | Message::Toast(ToastMessage::Dismiss))
} else {
Subscription::none()
};
```

---

## Anti-Patterns

### 1. State in View Functions

```rust
// BAD: Modifying state in view
fn view(&mut self) -> Element<'_, Message> {
    self.view_count += 1;  // Side effect!
    // ...
}

// GOOD: Pure view function
fn view(&self) -> Element<'_, Message> {
    // Only read state, never modify
}
```

### 2. Separate UiState Struct

```rust
// BAD: Separate state containers to synchronize
pub struct App {
    view: View,
    ui_state: UiState,  // Must stay in sync with view
}

// GOOD: UI state embedded in view
pub struct App {
    state: AppState,  // state.view contains UI state
}
```

### 3. Caching Derived Data Unnecessarily

```rust
// BAD: Caching computed counts
pub struct StudyState {
    total_rows: usize,  // Must update when data changes
    mapped_count: usize,
}

// GOOD: Compute on demand
impl Study {
    pub fn total_rows(&self) -> usize {
        self.domains.values().map(|d| d.row_count()).sum()
    }
}
```

### 4. Forgetting to Clear Cached State

```rust
// BAD: Updating mapping without invalidating cache
fn update_mapping(&mut self, ...) {
    domain.mapping.update(...);
    // Forgot to clear validation_cache!
}

// GOOD: Always invalidate dependent caches
fn update_mapping(&mut self, ...) {
    domain.mapping.update(...);
    domain.validation_cache = None;

    // Also clear preview cache in view state
    if let ViewState::DomainEditor { preview_cache, .. } = &mut self.state.view {
        *preview_cache = None;
    }
}
```

### 5. Duplicate State Across Windows

```rust
// BAD: Duplicating dialog state
Message::DialogContent {
window_id: window::Id,  // Already in dialog_windows
settings: Settings,     // Should reference state.settings
}

// GOOD: Single source of truth
// dialog_windows tracks window IDs
// state.settings is the single settings instance
```

---

## State Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            App.state: AppState                           │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    ViewState (current view)                      │  │
│  │                                                                   │  │
│  │  Home { workflow_mode }                                          │  │
│  │  DomainEditor { domain, tab, mapping_ui, validation_ui, ... }    │  │
│  │  Export(ExportViewState)                                         │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌──────────────────────────┐  ┌────────────────────────────────────┐ │
│  │     Study (domain data)   │  │      DialogWindows (multi-window) │ │
│  │  study_id, study_folder   │  │  about: Option<window::Id>        │ │
│  │  domains: BTreeMap        │  │  settings: Option<(Id, Category)> │ │
│  │    "DM" -> DomainState    │  │  export_progress: Option<...>     │ │
│  │    "AE" -> DomainState    │  │  ...                              │ │
│  └──────────────────────────┘  └────────────────────────────────────┘ │
│                                                                         │
│  ┌──────────────────────────┐  ┌────────────────────────────────────┐ │
│  │        Settings           │  │         Ephemeral State            │ │
│  │  general, export, ...     │  │  is_loading, error, toast          │ │
│  │  (persisted to disk)      │  │  terminology                       │ │
│  └──────────────────────────┘  └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘

Message Flow:
  User Action -> Message -> update() -> State Change -> view() -> UI Update
                                │
                                └-> Task (async) -> Result Message -> State Change
```

---

## Next Steps

- **[04-component-guide.md](./04-component-guide.md)** - Building reusable
  components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
