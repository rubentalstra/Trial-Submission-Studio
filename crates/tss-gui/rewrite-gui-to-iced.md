# Trial Submission Studio: egui → Iced 0.14.0 Migration Plan

## Executive Summary

Complete rewrite of the `tss-gui` crate from egui/eframe to Iced 0.14.0,
implementing a Professional Clinical visual style with Teal/Cyan accent colors
and light theme.

**Key Decisions:**

- **Framework**: Iced 0.14.0 (Elm-inspired architecture)
- **Visual Style**: Professional Clinical (clean, precise, medical-inspired)
- **Theme**: Light theme only (initially)
- **Accent Color**: Teal/Cyan (`#009BA6`)
- **Menu System**: Hybrid (native on macOS via muda, in-app on Windows/Linux)
- **Icons**: `iced_fonts` 0.3.0 with Lucide icon set
- **Documentation**: Comprehensive (5 markdown files in `crates/tss-gui/docs/`)

### Iced 0.14.0 Application Pattern (IMPORTANT)

Iced 0.14.0 uses the **builder pattern**, NOT the old `Application` trait:

```rust
// main.rs entry point
pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(Settings {
            window: window::Settings {
                size: Size::new(1280.0, 800.0),
                min_size: Some(Size::new(1024.0, 600.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

// App struct with associated functions (not trait impl)
struct App {
    state: AppState,
}

impl App {
    fn new() -> (Self, Task<Message>) { ... }
    fn update(&mut self, message: Message) -> Task<Message> { ... }
    fn view(&self) -> Element<'_, Message> { ... }
    fn title(&self) -> String { ... }
    fn theme(&self) -> Theme { ... }
    fn subscription(&self) -> Subscription<Message> { ... }
}
```

---

## 1. New Directory Structure

```
crates/tss-gui/
├── Cargo.toml                    # Updated dependencies
├── build.rs                      # Windows resource compilation
├── assets/
│   ├── icon.png
│   └── icon.svg
├── docs/                         # NEW: Comprehensive documentation
│   ├── 01-architecture.md        # Overall architecture guide
│   ├── 02-message-patterns.md    # Message hierarchy documentation
│   ├── 03-state-management.md    # State patterns and conventions
│   ├── 04-component-guide.md     # Reusable component patterns
│   └── 05-theming.md             # Theme/styling guide (STYLE.md equivalent)
└── src/
    ├── lib.rs
    ├── main.rs                   # Entry point
    ├── app.rs                    # Main Application struct
    │
    ├── message/                  # NEW: Message hierarchy
    │   ├── mod.rs                # Root Message enum
    │   ├── home.rs               # HomeMessage
    │   ├── domain_editor.rs      # DomainEditorMessage + tab messages
    │   ├── export.rs             # ExportMessage
    │   ├── dialog.rs             # DialogMessage
    │   └── menu.rs               # MenuMessage
    │
    ├── state/                    # Adapted from current (minimal changes)
    │   ├── mod.rs
    │   ├── app_state.rs          # Root AppState
    │   ├── study_state.rs        # StudyState (mostly unchanged)
    │   ├── domain_state.rs       # DomainState (mostly unchanged)
    │   ├── derived_state.rs      # DerivedState (mostly unchanged)
    │   ├── ui_state.rs           # UiState (mostly unchanged)
    │   └── navigation.rs         # View, EditorTab, WorkflowMode enums
    │
    ├── view/                     # NEW: Iced view functions
    │   ├── mod.rs                # View routing
    │   ├── home.rs               # Home view
    │   ├── export.rs             # Export view
    │   ├── domain_editor/
    │   │   ├── mod.rs            # DomainEditor + tab routing
    │   │   ├── mapping.rs        # Mapping tab
    │   │   ├── transform.rs      # Transform tab
    │   │   ├── validation.rs     # Validation tab
    │   │   ├── preview.rs        # Preview tab
    │   │   └── supp.rs           # SUPP tab
    │   └── dialog/
    │       ├── mod.rs
    │       ├── about.rs
    │       ├── settings.rs
    │       ├── third_party.rs
    │       └── update.rs
    │
    ├── component/                # NEW: Reusable UI components
    │   ├── mod.rs
    │   ├── master_detail.rs      # Master-detail layout
    │   ├── data_table.rs         # Paginated data table
    │   ├── status_badge.rs       # Status indicators
    │   ├── modal.rs              # Modal overlay wrapper
    │   ├── progress_modal.rs     # Progress with cancellation
    │   ├── tab_bar.rs            # Tab navigation
    │   ├── sidebar.rs            # Sidebar navigation
    │   ├── search_box.rs         # Search input
    │   ├── form_field.rs         # Form field with validation
    │   └── icon.rs               # Icon wrapper
    │
    ├── theme/                    # NEW: Professional Clinical theme
    │   ├── mod.rs
    │   ├── clinical.rs           # ClinicalTheme implementation
    │   ├── palette.rs            # Color palette definitions
    │   ├── spacing.rs            # Spacing constants
    │   └── typography.rs         # Text styles
    │
    ├── service/                  # Adapted from current
    │   ├── mod.rs
    │   ├── preview.rs            # Preview computation (Task::perform)
    │   ├── export.rs             # Export with progress (Task::sip)
    │   ├── study_loader.rs       # Study loading (Task::perform)
    │   └── update_checker.rs     # Update checking
    │
    ├── menu/                     # Adapted: Hybrid approach
    │   ├── mod.rs
    │   ├── native.rs             # macOS native menu (muda)
    │   └── in_app.rs             # Windows/Linux in-app menu
    │
    └── settings/                 # Unchanged logic
        ├── mod.rs
        ├── persistence.rs
        └── defaults.rs
```

---

## 2. Cargo.toml Changes

```toml
[package]
name = "tss-gui"
version.workspace = true
edition.workspace = true

[dependencies]
# Iced framework (replace eframe/egui)
# NOTE: Iced 0.14.0 uses builder pattern: iced::application(new, update, view).run()
#
# Feature flags explained:
#   - tokio:    Async runtime for Task::perform, Task::sip (preview, export, updates)
#   - image:    Image loading with codecs (PNG icons, potential charts)
#   - svg:      SVG rendering for vector icons/graphics
#   - markdown: Markdown widget (replaces egui_commonmark for changelogs/licenses)
#   - lazy:     Lazy widget rendering (performance optimization)
#   - advanced: Advanced widget capabilities
#
# Default features (auto-included): wgpu, tiny-skia, thread-pool, wayland, x11,
#                                   web-colors, crisp, linux-theme-detection
iced = { version = "0.14.0", features = [
    "tokio", # Async runtime for background tasks
    "image", # Image loading with codecs
    "svg", # SVG icon support
    "markdown", # Markdown rendering (changelogs, licenses)
    "lazy", # Performance: lazy widget rendering
    "advanced", # Advanced widget capabilities
] }
iced_fonts = { version = "0.3.0", features = ["lucide"] }  # Lucide icons

# Workspace crates (unchanged)
tss-model.workspace = true
tss-standards.workspace = true
tss-ingest.workspace = true
tss-map.workspace = true
tss-normalization.workspace = true
tss-validate.workspace = true
tss-output.workspace = true
tss-updater.workspace = true
xportrs.workspace = true

# Utilities (unchanged)
directories = "6.0"
toml = "0.9"
tracing.workspace = true
open = "5.3"
rfd = "0.16"
image = "0.25"

# Menu (macOS only, Windows/Linux use in-app)
muda = "0.17"
crossbeam-channel = "0.5"

# Async runtime (required by Iced)
tokio = { version = "1", features = ["rt-multi-thread", "sync"] }
async-stream = "0.3"

[target.'cfg(target_os = "macos")'.dependencies]
# No longer need winit directly

[target.'cfg(windows)'.build-dependencies]
winresource = "0.1"
```

---

## 3. Message Hierarchy

```rust
// message/mod.rs - Root Message enum
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(View),

    // View-specific
    Home(HomeMessage),
    DomainEditor(DomainEditorMessage),
    Export(ExportMessage),

    // Dialogs
    Dialog(DialogMessage),

    // Menu
    Menu(MenuMessage),

    // Background tasks
    Task(TaskMessage),

    // Global events
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    Tick,
}
```

**Sub-messages** (see `02-message-patterns.md` for full details):

- `HomeMessage`: WorkflowModeSelected, OpenStudy, RecentStudy, CloseStudy,
  DomainClicked
- `DomainEditorMessage`: TabSelected, Mapping(...), Transform(...),
  Validation(...), Preview(...), Supp(...)
- `ExportMessage`: DomainToggled, FormatChanged, StartExport, CancelExport,
  Progress, Complete
- `DialogMessage`: About(...), Settings(...), ThirdParty(...), Update(...)
- `TaskMessage`: StudyLoaded, PreviewReady, ValidationComplete,
  UpdateCheckComplete

---

## 4. Theme Definition

### Color Palette (Professional Clinical)

```rust
// Primary - Teal/Cyan
pub const PRIMARY_50: Color = rgb(0.88, 0.97, 0.98);  // #E0F7FA - Lightest tint
pub const PRIMARY_100: Color = rgb(0.70, 0.92, 0.95);  // #B3EBF2
pub const PRIMARY_200: Color = rgb(0.50, 0.85, 0.90);  // #80D9E6
pub const PRIMARY_300: Color = rgb(0.30, 0.78, 0.82);  // #4DC7D1
pub const PRIMARY_400: Color = rgb(0.15, 0.70, 0.75);  // #26B3BF
pub const PRIMARY_500: Color = rgb(0.00, 0.61, 0.65);  // #009BA6 - Main accent
pub const PRIMARY_600: Color = rgb(0.00, 0.52, 0.56);  // #00858E
pub const PRIMARY_700: Color = rgb(0.00, 0.44, 0.47);  // #007078
pub const PRIMARY_800: Color = rgb(0.00, 0.35, 0.38);  // #005A61
pub const PRIMARY_900: Color = rgb(0.00, 0.27, 0.29);  // #00454A - Darkest shade

// Semantic Colors
pub const SUCCESS: Color = rgb(0.20, 0.70, 0.40);  // #33B366 - Green
pub const WARNING: Color = rgb(0.95, 0.65, 0.05);  // #F2A60D - Amber
pub const ERROR: Color = rgb(0.85, 0.25, 0.25);  // #D94040 - Red
pub const INFO: Color = rgb(0.25, 0.55, 0.85);  // #408CD9 - Blue

// Neutral Grays
pub const GRAY_50: Color = rgb(0.98, 0.98, 0.99);  // #FAFAFE - Background
pub const GRAY_100: Color = rgb(0.95, 0.95, 0.97);  // #F2F2F7 - Surface
pub const GRAY_200: Color = rgb(0.90, 0.90, 0.93);  // #E6E6ED - Border
pub const GRAY_300: Color = rgb(0.82, 0.82, 0.86);  // #D1D1DB - Divider
pub const GRAY_400: Color = rgb(0.65, 0.65, 0.70);  // #A6A6B3 - Placeholder
pub const GRAY_500: Color = rgb(0.50, 0.50, 0.55);  // #80808C - Secondary text
pub const GRAY_600: Color = rgb(0.40, 0.40, 0.45);  // #666673 - Muted text
pub const GRAY_700: Color = rgb(0.30, 0.30, 0.35);  // #4D4D59 - Body text
pub const GRAY_800: Color = rgb(0.20, 0.20, 0.24);  // #33333D - Headings
pub const GRAY_900: Color = rgb(0.10, 0.10, 0.12);  // #1A1A1F - Primary text
```

### Spacing System

```rust
pub const SPACING_XS: f32 = 4.0;   // Tight spacing
pub const SPACING_SM: f32 = 8.0;   // Small gaps
pub const SPACING_MD: f32 = 16.0;  // Default padding
pub const SPACING_LG: f32 = 24.0;  // Section spacing
pub const SPACING_XL: f32 = 32.0;  // Large gaps
pub const SPACING_XXL: f32 = 48.0;  // Page margins

pub const BORDER_RADIUS_SM: f32 = 4.0;   // Buttons, inputs
pub const BORDER_RADIUS_MD: f32 = 6.0;   // Cards
pub const BORDER_RADIUS_LG: f32 = 8.0;   // Modals
```

### Typography

```rust
pub const FONT_SIZE_CAPTION: f32 = 11.0;  // Labels, hints
pub const FONT_SIZE_SMALL: f32 = 12.0;  // Secondary text
pub const FONT_SIZE_BODY: f32 = 14.0;  // Default text
pub const FONT_SIZE_SUBTITLE: f32 = 16.0;  // Emphasized text
pub const FONT_SIZE_TITLE: f32 = 20.0;  // Section headers
pub const FONT_SIZE_HEADING: f32 = 24.0;  // Page headers
pub const FONT_SIZE_DISPLAY: f32 = 32.0;  // Hero text
```

---

## 5. Key Component Patterns

### Master-Detail Layout (Mapping, Export screens)

```rust
pub fn master_detail<'a, M: 'a>(
    master: Element<'a, M>,
    detail: Element<'a, M>,
    master_width: f32,
) -> Element<'a, M> {
    row![
        container(scrollable(master))
            .width(master_width)
            .height(Fill),
        vertical_rule(1),
        container(scrollable(detail))
            .width(Fill)
            .height(Fill),
    ].into()
}
```

### Modal Dialog (using float widget)

```rust
pub fn modal<'a, M: Clone + 'a>(
    base: Element<'a, M>,
    content: Element<'a, M>,
) -> Element<'a, M> {
    stack![
        base,
        float(
            opaque(backdrop()),  // Semi-transparent overlay
            center(card(content))
        )
    ].into()
}
```

### Background Tasks with Progress

```rust
// Using Task::sip for progress streaming
let (task, handle) = Task::sip(
export_stream(config),
Message::ExportProgress,
Message::ExportComplete,
).abortable();

// Store handle for cancellation
state.tasks.export = Some(handle.abort_on_drop());
```

---

## 6. Implementation Phases

### Phase 1: Foundation (Files 1-15)

**Goal**: Basic app shell with routing and theme

1. Update `Cargo.toml` with Iced dependencies
2. Create `theme/palette.rs` - Color palette
3. Create `theme/spacing.rs` - Spacing constants
4. Create `theme/typography.rs` - Text styles
5. Create `theme/clinical.rs` - ClinicalTheme implementation
6. Create `theme/mod.rs` - Theme exports
7. Create `state/navigation.rs` - View, EditorTab enums
8. Create `message/mod.rs` - Root Message enum
9. Create `message/home.rs` - HomeMessage
10. Create `message/menu.rs` - MenuMessage
11. Create `app.rs` - Main Application struct
12. Update `main.rs` - Entry point
13. Create `view/mod.rs` - View routing
14. Create `component/icon.rs` - Icon helper
15. Run and verify basic window appears

### Phase 2: Components (Files 16-27)

**Goal**: All reusable components ready

16. Create `component/mod.rs` - Component exports
17. Create `component/master_detail.rs`
18. Create `component/modal.rs`
19. Create `component/progress_modal.rs`
20. Create `component/tab_bar.rs`
21. Create `component/sidebar.rs`
22. Create `component/search_box.rs`
23. Create `component/form_field.rs`
24. Create `component/status_badge.rs`
25. Create `component/data_table.rs`
26. Create component tests
27. Create `component/README.md` with usage examples

### Phase 3: Home & Navigation (Files 28-38)

**Goal**: Home screen functional, study loading works

28. Create `view/home.rs` - Home view
29. Update `state/app_state.rs` - Add dialog/task state
30. Port `state/study_state.rs` - Minimal changes
31. Port `state/domain_state.rs` - Minimal changes
32. Port `state/derived_state.rs` - Minimal changes
33. Port `state/ui_state.rs` - Minimal changes
34. Create `service/study_loader.rs` - Task::perform pattern
35. Create `service/mod.rs`
36. Wire up home → study loading
37. Add recent studies list
38. Test study loading end-to-end

### Phase 4: Domain Editor (Files 39-55)

**Goal**: All 5 tabs functional

39. Create `view/domain_editor/mod.rs` - Tab routing
40. Create `message/domain_editor.rs` - Tab messages
41. Create `view/domain_editor/mapping.rs` - Mapping tab (target columns in the
    left side bar, and then you map the source to the target)
42. Create `service/preview.rs` - Preview computation
43. Create `view/domain_editor/preview.rs` - Preview tab
44. Create `view/domain_editor/validation.rs` - Validation tab
45. Create `view/domain_editor/transform.rs` - Transform tab (it's now
    Normalization)
46. Create `view/domain_editor/supp.rs` - SUPP tab
47. Wire up mapping state changes
48. Wire up preview rebuilding
49. Wire up validation display
50. Test domain editor navigation
51. Test mapping workflow
52. Test preview generation
53. Add keyboard navigation
54. Test tab switching
55. Polish domain editor UX

### Phase 5: Export & Dialogs (Files 56-72)

**Goal**: Export functional, all dialogs working

56. Create `message/export.rs` - Export messages
57. Create `view/export.rs` - Export view
58. Create `service/export.rs` - Task::sip pattern
59. Wire up export progress modal
60. Create `message/dialog.rs` - Dialog messages
61. Create `view/dialog/mod.rs` - Dialog routing
62. Create `view/dialog/about.rs`
63. Create `view/dialog/settings.rs` - Tabbed settings
64. Create `view/dialog/third_party.rs`
65. Create `view/dialog/update.rs`
66. Create `service/update_checker.rs`
67. Port `settings/mod.rs` - Unchanged
68. Port `settings/persistence.rs` - Unchanged
69. Test export workflow
70. Test settings persistence
71. Test update checking
72. Test all dialogs

### Phase 6: Menu & Polish (Files 73-85)

**Goal**: Production-ready release

73. Create `menu/mod.rs` - Menu state
74. Create `menu/native.rs` - macOS muda integration
75. Create `menu/in_app.rs` - Windows/Linux menu
76. Wire up menu subscriptions
77. Add keyboard shortcuts
78. Add responsive layouts (Sensor widget)
79. Create `docs/01-architecture.md`
80. Create `docs/02-message-patterns.md`
81. Create `docs/03-state-management.md`
82. Create `docs/04-component-guide.md`
83. Create `docs/05-theming.md`
84. Final testing all platforms
85. Remove old egui code

---

## 7. Documentation Files to Create

### `docs/05-theming.md` (STYLE.md equivalent)

This will be the comprehensive style guide similar to the DRFW example,
containing:

1. **Design Principles**
    - Professional Clinical aesthetic
    - Clarity over decoration
    - Consistent depth hierarchy
    - Performance-first patterns

2. **Color System**
    - Full palette definitions
    - Semantic color usage
    - Accessible contrast ratios

3. **Typography**
    - Font sizes and weights
    - Text hierarchy
    - Monospace for data

4. **Spacing & Layout**
    - Spacing scale
    - Component spacing rules
    - Master-detail patterns

5. **Component Styles**
    - Button variants (primary, secondary, danger, ghost)
    - Container styles (card, modal, sidebar)
    - Form elements
    - Status badges

6. **Icons**
    - iced_fonts 0.3.0 with Lucide icons (`iced_fonts::lucide::*`)
    - Icon sizing conventions
    - Icon color customization

7. **Animation & Transitions**
    - Minimal animation philosophy
    - Loading states
    - Progress indicators

---

## 8. Files to Modify (Critical)

| File                            | Action  | Notes                            |
|---------------------------------|---------|----------------------------------|
| `crates/tss-gui/Cargo.toml`     | Replace | New Iced dependencies            |
| `crates/tss-gui/src/main.rs`    | Rewrite | Iced app initialization          |
| `crates/tss-gui/src/app.rs`     | Rewrite | Application trait impl           |
| `crates/tss-gui/src/state/*`    | Adapt   | Minimal changes, add TaskHandles |
| `crates/tss-gui/src/settings/*` | Keep    | Unchanged logic                  |

---

## 9. Files to Delete (After Migration)

After successful migration and testing:

- `src/views/` (egui views)
- `src/menu.rs` (old menu implementation)
- `src/theme.rs` (old spacing constants)
- Any egui-specific dependencies

---

## 10. Verification Plan

### Unit Tests

- [ ] Theme color contrast validation
- [ ] Message routing coverage
- [ ] State update correctness

### Integration Tests

- [ ] Study loading flow
- [ ] Mapping workflow
- [ ] Export with progress
- [ ] Settings persistence

### Manual Testing Checklist

- [ ] App launches on macOS
- [ ] App launches on Windows
- [ ] App launches on Linux
- [ ] Native menu works (macOS)
- [ ] In-app menu works (Windows/Linux)
- [ ] Open study folder
- [ ] Navigate between domains
- [ ] Map variables
- [ ] View preview
- [ ] View validation issues
- [ ] Configure SUPP
- [ ] Export domains
- [ ] Cancel export
- [ ] Change settings
- [ ] Check for updates
- [ ] View about dialog
- [ ] View third-party licenses
- [ ] Keyboard shortcuts work
- [ ] Window resize handles correctly

---

## 11. Risk Mitigation

| Risk                   | Mitigation                          |
|------------------------|-------------------------------------|
| Iced breaking changes  | Pin exact version 0.14.0            |
| Performance regression | Profile preview/export, use async   |
| Platform-specific bugs | Test all 3 platforms early          |
| Theme inconsistency    | Create style guide first, follow it |
| Missing Iced features  | Float widget + custom components    |

---

## 12. Dependencies Changed

### Removed

- `eframe` 0.33.3
- `egui` 0.33.3
- `egui_extras` 0.33.3
- `egui_phosphor` 0.11.0
- `egui_commonmark` 0.22.0
- `winit` 0.30.12 (macOS specific)

### Added

- `iced` 0.14.0
- `iced_fonts` 0.3.0 with Lucide feature
- `tokio` 1.x
- `async-stream` 0.3

### Kept

- `muda` 0.17.1
- `crossbeam-channel` 0.5.15
- `directories` 6.0
- `toml` 0.9.8
- `image` 0.25
- `rfd` 0.16.0
- `open` 5.3
- All workspace crates unchanged
