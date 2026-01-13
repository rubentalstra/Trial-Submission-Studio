# Trial Submission Studio - State Management

This document describes state organization and patterns used in Trial Submission Studio.

## Table of Contents

1. [State Philosophy](#state-philosophy)
2. [State Hierarchy](#state-hierarchy)
3. [State Categories](#state-categories)
4. [Domain State vs UI State](#domain-state-vs-ui-state)
5. [Derived State](#derived-state)
6. [State Update Patterns](#state-update-patterns)
7. [Common Patterns](#common-patterns)
8. [Anti-Patterns](#anti-patterns)

---

## State Philosophy

State management in Iced follows these principles:

1. **Single source of truth** - All state lives in the `App` struct
2. **Immutable updates** - State is updated only in `update()`, never in `view()`
3. **Minimal state** - Only store what you need, derive the rest
4. **Separation of concerns** - Domain data vs UI state vs derived/cached data

### State Ownership

```rust
pub struct App {
    // =========================================================================
    // Navigation State
    // =========================================================================
    /// Current view/screen being displayed
    view: View,

    /// Current workflow mode (SDTM, ADaM, SEND)
    workflow_mode: WorkflowMode,

    // =========================================================================
    // Domain State
    // =========================================================================
    /// Loaded study (None if no study loaded)
    study: Option<StudyState>,

    /// Application settings (persisted to disk)
    settings: Settings,

    // =========================================================================
    // UI State
    // =========================================================================
    /// All UI-specific state (selection, scroll, dialogs)
    ui: UiState,

    // =========================================================================
    // Cached/Derived State
    // =========================================================================
    /// Controlled terminology registry (loaded lazily)
    ct_registry: Option<TerminologyRegistry>,
}
```

---

## State Hierarchy

### Overview

```
App
├── view: View                      # Navigation state
├── workflow_mode: WorkflowMode     # Current mode
├── study: Option<StudyState>       # Domain data
│   ├── path: PathBuf               # Study folder path
│   ├── metadata: StudyMetadata     # Study info
│   └── domains: HashMap<String, DomainState>
│       ├── source: DomainSource    # Original data
│       ├── mapping: MappingState   # Column mappings
│       └── derived: DerivedState   # Cached computations
├── settings: Settings              # Persisted settings
└── ui: UiState                     # UI-only state
    ├── home: HomeUiState
    ├── domain_editors: HashMap<String, DomainEditorUiState>
    ├── export: ExportUiState
    ├── settings: SettingsUiState
    ├── about: AboutUiState
    └── update: UpdateUiState
```

### Navigation State

```rust
/// Current view/screen
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum View {
    #[default]
    Home,
    DomainEditor {
        domain: String,
        tab: EditorTab,
    },
    Export,
}

/// Editor tabs within DomainEditor view
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    #[default]
    Mapping,
    Transform,
    Validation,
    Preview,
    Supp,
}

/// Workflow mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowMode {
    #[default]
    Sdtm,
    Adam,
    Send,
}
```

---

## State Categories

### 1. Navigation State

Controls what's displayed on screen:

```rust
// Current view
view: View

// View helpers
impl View {
    pub fn is_home(&self) -> bool { ... }
    pub fn is_domain_editor(&self) -> bool { ... }
    pub fn is_export(&self) -> bool { ... }
    pub fn current_domain(&self) -> Option<&str> { ... }
    pub fn current_tab(&self) -> Option<EditorTab> { ... }
}
```

### 2. Domain State

Business data that represents the actual clinical trial data:

```rust
/// State for a single study
pub struct StudyState {
    /// Path to study folder
    pub path: PathBuf,

    /// Study metadata (name, description, etc.)
    pub metadata: StudyMetadata,

    /// Domain states keyed by domain code (e.g., "DM", "AE")
    domains: HashMap<String, DomainState>,
}

/// State for a single domain
pub struct DomainState {
    /// Domain code (e.g., "DM", "AE")
    pub code: String,

    /// Original source data
    pub source: DomainSource,

    /// Column mapping state
    pub mapping: MappingState,

    /// Cached/computed state
    pub derived: DerivedState,
}

/// Mapping state for a domain
pub struct MappingState {
    /// Column mappings: source_column -> target_variable
    pub mappings: HashMap<String, String>,

    /// Unmapped source columns
    pub unmapped: Vec<String>,

    /// Mapping confidence scores
    pub confidence: HashMap<String, f64>,
}
```

### 3. UI State

Visual-only state that doesn't affect domain data:

```rust
/// All UI state (separated from domain state)
pub struct UiState {
    /// Home view UI state
    pub home: HomeUiState,

    /// Per-domain editor UI state
    domain_editors: HashMap<String, DomainEditorUiState>,

    /// Export view UI state
    pub export: ExportUiState,

    /// Settings dialog UI state
    pub settings: SettingsUiState,

    /// About dialog UI state
    pub about: AboutUiState,

    /// Update dialog UI state
    pub update: UpdateUiState,
}

/// UI state for a domain editor
pub struct DomainEditorUiState {
    /// Mapping tab UI state
    pub mapping: MappingUiState,

    /// Transform tab UI state
    pub transform: TransformUiState,

    /// Validation tab UI state
    pub validation: ValidationUiState,

    /// Preview tab UI state
    pub preview: PreviewUiState,

    /// SUPP tab UI state
    pub supp: SuppUiState,
}

/// Mapping tab UI state
pub struct MappingUiState {
    /// Currently selected variable index
    pub selected_idx: Option<usize>,

    /// Search filter text
    pub search_filter: String,

    /// Show only unmapped toggle
    pub filter_unmapped: bool,

    /// Scroll position (for virtual scrolling)
    pub scroll_offset: f32,
}
```

### 4. Persisted State

Settings that survive app restarts:

```rust
/// Application settings (persisted to disk)
pub struct Settings {
    /// General settings
    pub general: GeneralSettings,

    /// Export settings
    pub export: ExportSettings,

    /// Validation settings
    pub validation: ValidationSettings,

    /// Update settings
    pub update: UpdateSettings,
}

/// General settings
pub struct GeneralSettings {
    /// Number of header rows in source files
    pub header_rows: usize,

    /// Recent studies (most recent first)
    pub recent_studies: Vec<PathBuf>,

    /// Maximum recent studies to remember
    pub max_recent: usize,
}
```

---

## Domain State vs UI State

### Why Separate?

| Domain State         | UI State      |
|----------------------|---------------|
| Business data        | Visual state  |
| Persisted with study | Ephemeral     |
| Shared across views  | View-specific |
| Affects output       | Display only  |

### Example: Mapping Tab

```rust
// Domain state (affects export output)
pub struct MappingState {
    pub mappings: HashMap<String, String>,  // The actual mappings
}

// UI state (visual only)
pub struct MappingUiState {
    pub selected_idx: Option<usize>,        // Which row is highlighted
    pub search_filter: String,              // Search box text
    pub scroll_offset: f32,                 // Scroll position
}
```

### Access Pattern

```rust
impl App {
    fn view_mapping(&self, domain: &str) -> Element<'_, Message> {
        // Get domain state (the data)
        let domain_state = self.study.as_ref()
            .and_then(|s| s.get_domain(domain));

        // Get UI state (the visual state)
        let ui_state = &self.ui.domain_editor(domain).mapping;

        // Use both to render
        // ...
    }
}
```

---

## Derived State

Computed data that's cached for performance:

```rust
/// Cached/computed state for a domain
pub struct DerivedState {
    /// Cached preview DataFrame
    pub preview: Option<DataFrame>,

    /// Cached validation results
    pub validation: Option<ValidationReport>,

    /// SUPP configuration (derived from source data)
    pub supp_config: Option<SuppConfig>,

    /// Last computation timestamp
    pub computed_at: Option<Instant>,
}
```

### Invalidation Pattern

When source data changes, invalidate derived state:

```rust
impl App {
    /// Invalidate preview when mapping changes
    fn invalidate_preview(&mut self, domain_code: &str) {
        if let Some(domain) = self.domain_mut(domain_code) {
            domain.derived.preview = None;
            domain.derived.validation = None;
            domain.derived.computed_at = None;
        }

        // Also clear UI rebuild state
        self.ui.domain_editor(domain_code).preview.is_rebuilding = false;
        self.ui.domain_editor(domain_code).preview.error = None;
    }
}
```

### Lazy Computation Pattern

Only compute derived state when needed:

```rust
impl App {
    fn handle_preview_tab_selected(&mut self, domain: &str) -> Task<Message> {
        // Check if preview needs rebuilding
        let needs_rebuild = self.domain(domain)
            .map(|d| d.derived.preview.is_none())
            .unwrap_or(false);

        if needs_rebuild {
            // Set rebuilding flag
            self.ui.domain_editor(domain).preview.is_rebuilding = true;

            // Spawn background task
            let domain = domain.to_string();
            let config = self.build_preview_config(&domain);

            Task::perform(
                async move { compute_preview(config).await },
                move |result| Message::PreviewReady {
                    domain: domain.clone(),
                    result,
                }
            )
        } else {
            Task::none()
        }
    }
}
```

---

## State Update Patterns

### 1. Direct State Mutation

For simple, synchronous updates:

```rust, no_run
Message::Navigate(view) => {
    self.view = view;
    Task::none()
}

MappingMessage::VariableSelected(idx) => {
    self.ui.domain_editor(domain).mapping.selected_idx = Some(idx);
    Task::none()
}
```

### 2. Async State Update

For updates that require background work:

```rust, no_run
HomeMessage::OpenStudyClicked => {
    // Start file dialog (async)
    Task::perform(
        async { rfd::AsyncFileDialog::new().pick_folder().await },
        |folder| Message::FolderSelected(folder.map(|f| f.path().to_path_buf()))
    )
}

Message::FolderSelected(Some(path)) => {
    // Start study loading (async)
    Task::perform(
        async move { load_study(&path).await },
        Message::StudyLoaded
    )
}

Message::StudyLoaded(Ok(study)) => {
    // Final state update (sync)
    self.study = Some(study);
    self.view = View::Home;
    Task::none()
}
```

### 3. Chained Updates

When one update triggers another:

```rust, no_run
MappingMessage::AcceptSuggestion(variable) => {
    // Update mapping
    if let Some(domain) = self.current_domain() {
        self.apply_mapping(&domain, &variable);

        // Invalidate dependent state
        self.invalidate_preview(&domain);

        // Move to next unmapped variable
        self.advance_selection(&domain);
    }
    Task::none()
}
```

### 4. Batched Updates

When multiple state changes happen together:

```rust, no_run
Message::StudyLoaded(Ok(study)) => {
    // Batch all related state changes
    self.study = Some(study);
    self.ui.clear_domain_editors();
    self.ui.export.reset();
    self.view = View::Home;

    // Load CT registry if needed
    if self.ct_registry.is_none() {
        return Task::perform(
            async { load_ct_registry().await },
            Message::CtRegistryLoaded
        );
    }

    Task::none()
}
```

---

## Common Patterns

### 1. Optional State Access

Safely access nested optional state:

```rust
// Get domain state
pub fn domain(&self, code: &str) -> Option<&DomainState> {
    self.study.as_ref()?.get_domain(code)
}

// Get mutable domain state
pub fn domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
    self.study.as_mut()?.get_domain_mut(code)
}

// Use in update
fn handle_mapping_change(&mut self, domain: &str, change: MappingChange) {
    if let Some(d) = self.domain_mut(domain) {
        d.mapping.apply_change(change);
    }
}
```

### 2. UI State Factory

Create UI state on-demand for domains:

```rust
impl UiState {
    /// Get or create UI state for a domain editor
    pub fn domain_editor(&mut self, domain: &str) -> &mut DomainEditorUiState {
        self.domain_editors
            .entry(domain.to_string())
            .or_insert_with(DomainEditorUiState::default)
    }

    /// Clear all domain editor UI state (on study change)
    pub fn clear_domain_editors(&mut self) {
        self.domain_editors.clear();
    }
}
```

### 3. Settings with Pending Changes

Edit settings without immediate persistence:

```rust
pub struct SettingsUiState {
    /// Whether settings dialog is open
    open: bool,

    /// Pending settings (edited but not applied)
    pending: Option<Settings>,

    /// Active tab in settings dialog
    pub active_tab: SettingsTab,
}

impl SettingsUiState {
    /// Open settings dialog with current settings
    pub fn open(&mut self, current: &Settings) {
        self.open = true;
        self.pending = Some(current.clone());
    }

    /// Close settings dialog
    /// Returns the new settings if apply=true
    pub fn close(&mut self, apply: bool) -> Option<Settings> {
        self.open = false;
        if apply {
            self.pending.take()
        } else {
            self.pending = None;
            None
        }
    }

    /// Get pending settings for editing
    pub fn pending_mut(&mut self) -> Option<&mut Settings> {
        self.pending.as_mut()
    }
}
```

### 4. Export Progress State

Track async operation progress:

```rust
pub struct ExportUiState {
    /// Current export phase
    pub phase: ExportPhase,

    /// Currently exporting domain
    pub current_domain: Option<String>,

    /// Current step description
    pub current_step: String,

    /// Overall progress (0.0 - 1.0)
    pub overall_progress: f32,

    /// Files written so far
    pub written_files: Vec<PathBuf>,

    /// Final result (success or error)
    pub result: Option<Result<ExportResult, ExportError>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportPhase {
    #[default]
    Idle,
    Preparing,
    Exporting,
    Complete,
}

impl ExportUiState {
    /// Reset to initial state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Update progress from background task
    pub fn apply_progress(&mut self, progress: ExportProgress) {
        match progress {
            ExportProgress::StartingDomain(domain) => {
                self.current_domain = Some(domain);
            }
            ExportProgress::Step(step) => {
                self.current_step = step.to_string();
            }
            ExportProgress::OverallProgress(p) => {
                self.overall_progress = p;
            }
        }
    }
}
```

---

## Anti-Patterns

### 1. State in View Functions

```rust
// BAD: Modifying state in view
fn view_mapping(&mut self) -> Element<'_, Message> {
    self.ui.mapping.view_count += 1;  // Side effect!
    // ...
}

// GOOD: Pure view function
fn view_mapping(&self) -> Element<'_, Message> {
    // Only read state, never modify
    // ...
}
```

### 2. Duplicated State

```rust
// BAD: Same data in multiple places
pub struct App {
    current_domain: Option<String>,  // Duplicates View::DomainEditor
    view: View,
}

// GOOD: Single source of truth
pub struct App {
    view: View,  // Contains domain info when in editor
}

// Access via helper
impl App {
    fn current_domain(&self) -> Option<&str> {
        if let View::DomainEditor { domain, .. } = &self.view {
            Some(domain)
        } else {
            None
        }
    }
}
```

### 3. Giant State Structs

```rust
// BAD: Flat structure with everything at top level
pub struct App {
    view: View,
    mapping_selected_idx: Option<usize>,
    mapping_search: String,
    preview_scroll: f32,
    export_phase: ExportPhase,
    export_progress: f32,
    // ... 50 more fields
}

// GOOD: Nested structure organized by concern
pub struct App {
    view: View,
    study: Option<StudyState>,
    ui: UiState,
}
```

### 4. Mixing Domain and UI State

```rust
// BAD: UI state mixed into domain struct
pub struct DomainState {
    pub mapping: MappingState,
    pub selected_variable: Option<usize>,  // This is UI state!
    pub scroll_position: f32,              // This is UI state!
}

// GOOD: Separated concerns
pub struct DomainState {
    pub mapping: MappingState,  // Domain data only
}

pub struct DomainEditorUiState {
    pub mapping: MappingUiState,  // UI state here
}
```

### 5. Forgetting to Invalidate

```rust
// BAD: Changing source without invalidating derived
fn update_mapping(&mut self, domain: &str, mapping: MappingChange) {
    if let Some(d) = self.domain_mut(domain) {
        d.mapping.apply(mapping);
        // Forgot to invalidate preview!
    }
}

// GOOD: Always invalidate dependent state
fn update_mapping(&mut self, domain: &str, mapping: MappingChange) {
    if let Some(d) = self.domain_mut(domain) {
        d.mapping.apply(mapping);
    }
    self.invalidate_preview(domain);  // Always invalidate
}
```

---

## State Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            App State                                     │
│                                                                         │
│  ┌───────────────┐  ┌──────────────────────┐  ┌────────────────────┐  │
│  │   Navigation  │  │    Domain State      │  │     UI State       │  │
│  │               │  │                      │  │                    │  │
│  │  view         │  │  study: StudyState   │  │  home              │  │
│  │  workflow     │  │    domains: HashMap  │  │  domain_editors    │  │
│  │               │  │      mapping         │  │    mapping         │  │
│  │               │  │      source          │  │    transform       │  │
│  │               │  │      derived ←───────┼──┼─── preview         │  │
│  │               │  │                      │  │  export            │  │
│  └───────────────┘  └──────────────────────┘  └────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                        Persisted State                             │ │
│  │  settings: Settings (general, export, validation, update)         │ │
│  └───────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘

Message Flow:
  User Action → Message → update() → State Change → view() → UI Update
                              │
                              └→ Task (async) → Result Message → State Change
```

---

## Next Steps

- **[04-component-guide.md](./04-component-guide.md)** - Building reusable components
- **[05-theming.md](./05-theming.md)** - Professional Clinical theme guide
