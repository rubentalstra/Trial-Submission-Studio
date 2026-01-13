# Trial Submission Studio - Theming Guide

This document describes the Professional Clinical theme and styling conventions for Trial Submission Studio.

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing System](#spacing-system)
5. [Component Styling](#component-styling)
6. [Icons](#icons)
7. [Depth & Elevation](#depth--elevation)
8. [Accessibility](#accessibility)
9. [Theme Implementation](#theme-implementation)

---

## Design Philosophy

### Professional Clinical Aesthetic

The Professional Clinical theme is designed for medical/regulatory applications where:

1. **Clarity over decoration** - Every element serves a purpose
2. **Precision matters** - Clean lines, consistent spacing, readable data
3. **Extended use comfort** - Light theme optimized for all-day work sessions
4. **Trust and professionalism** - Conveys reliability for clinical data handling

### Core Principles

| Principle | Implementation |
|-----------|----------------|
| **Clean** | Minimal shadows, subtle borders, generous whitespace |
| **Precise** | Consistent spacing, aligned grids, sharp corners |
| **Calm** | Muted palette, teal accents, no jarring colors |
| **Readable** | High contrast text, appropriate font sizes, monospace for data |

### Visual Language

- **Primary action**: Teal/Cyan (`#009BA6`)
- **Background**: Cool grays with slight blue undertone
- **Text**: Near-black on white/light gray backgrounds
- **Borders**: Subtle, 1px, light gray
- **Shadows**: Minimal, used only for elevation hierarchy

---

## Color System

### Primary Colors (Teal/Cyan)

The primary color palette is used for interactive elements, active states, and emphasis:

```rust
// Primary - Teal/Cyan
pub const PRIMARY_50:  Color = Color::from_rgb(0.88, 0.97, 0.98);  // #E0F7FA - Lightest tint
pub const PRIMARY_100: Color = Color::from_rgb(0.70, 0.92, 0.95);  // #B3EBF2
pub const PRIMARY_200: Color = Color::from_rgb(0.50, 0.85, 0.90);  // #80D9E6
pub const PRIMARY_300: Color = Color::from_rgb(0.30, 0.78, 0.82);  // #4DC7D1
pub const PRIMARY_400: Color = Color::from_rgb(0.15, 0.70, 0.75);  // #26B3BF
pub const PRIMARY_500: Color = Color::from_rgb(0.00, 0.61, 0.65);  // #009BA6 - Main accent
pub const PRIMARY_600: Color = Color::from_rgb(0.00, 0.52, 0.56);  // #00858E
pub const PRIMARY_700: Color = Color::from_rgb(0.00, 0.44, 0.47);  // #007078
pub const PRIMARY_800: Color = Color::from_rgb(0.00, 0.35, 0.38);  // #005A61
pub const PRIMARY_900: Color = Color::from_rgb(0.00, 0.27, 0.29);  // #00454A - Darkest shade
```

### Usage

| Variant | Use Case |
|---------|----------|
| `PRIMARY_50` | Selected row background, subtle highlights |
| `PRIMARY_100` | Active tab background, hover states |
| `PRIMARY_500` | Primary buttons, links, active indicators |
| `PRIMARY_600` | Button hover state |
| `PRIMARY_700` | Button pressed state, text on light backgrounds |
| `PRIMARY_800` | Text on white backgrounds |

### Semantic Colors

For status, feedback, and validation states:

```rust
// Success - Green
pub const SUCCESS:       Color = Color::from_rgb(0.20, 0.70, 0.40);  // #33B366
pub const SUCCESS_LIGHT: Color = Color::from_rgb(0.85, 0.95, 0.88);  // #D9F2E0

// Warning - Amber
pub const WARNING:       Color = Color::from_rgb(0.95, 0.65, 0.05);  // #F2A60D
pub const WARNING_LIGHT: Color = Color::from_rgb(1.00, 0.96, 0.85);  // #FFF5D9

// Error - Red
pub const ERROR:       Color = Color::from_rgb(0.85, 0.25, 0.25);  // #D94040
pub const ERROR_LIGHT: Color = Color::from_rgb(0.99, 0.90, 0.90);  // #FCE6E6

// Info - Blue
pub const INFO:       Color = Color::from_rgb(0.25, 0.55, 0.85);  // #408CD9
pub const INFO_LIGHT: Color = Color::from_rgb(0.90, 0.95, 1.00);  // #E6F2FF
```

### Usage

| Color | Use Case |
|-------|----------|
| `SUCCESS` | Completed status, valid mappings, passed validation |
| `WARNING` | Warnings, attention needed, partial completion |
| `ERROR` | Errors, validation failures, required fields |
| `INFO` | Informational messages, tips, neutral status |

### Neutral Grays

For backgrounds, borders, and text hierarchy:

```rust
// Neutral Grays (cool undertone)
pub const WHITE:    Color = Color::from_rgb(1.00, 1.00, 1.00);  // #FFFFFF
pub const GRAY_50:  Color = Color::from_rgb(0.98, 0.98, 0.99);  // #FAFAFE - Background
pub const GRAY_100: Color = Color::from_rgb(0.95, 0.95, 0.97);  // #F2F2F7 - Surface
pub const GRAY_200: Color = Color::from_rgb(0.90, 0.90, 0.93);  // #E6E6ED - Border
pub const GRAY_300: Color = Color::from_rgb(0.82, 0.82, 0.86);  // #D1D1DB - Divider
pub const GRAY_400: Color = Color::from_rgb(0.65, 0.65, 0.70);  // #A6A6B3 - Placeholder
pub const GRAY_500: Color = Color::from_rgb(0.50, 0.50, 0.55);  // #80808C - Secondary text
pub const GRAY_600: Color = Color::from_rgb(0.40, 0.40, 0.45);  // #666673 - Muted text
pub const GRAY_700: Color = Color::from_rgb(0.30, 0.30, 0.35);  // #4D4D59 - Body text
pub const GRAY_800: Color = Color::from_rgb(0.20, 0.20, 0.24);  // #33333D - Headings
pub const GRAY_900: Color = Color::from_rgb(0.10, 0.10, 0.12);  // #1A1A1F - Primary text
```

### Usage

| Variant | Use Case |
|---------|----------|
| `WHITE` | Card backgrounds, modal backgrounds |
| `GRAY_50` | Page background, alternate rows |
| `GRAY_100` | Panel backgrounds, header backgrounds |
| `GRAY_200` | Borders, dividers |
| `GRAY_300` | Disabled borders |
| `GRAY_400` | Placeholder text, disabled text |
| `GRAY_500` | Secondary text, captions |
| `GRAY_600` | Labels, descriptions |
| `GRAY_700` | Body text |
| `GRAY_800` | Headings, strong text |
| `GRAY_900` | Primary text, titles |

---

## Typography

### Font Sizes

Consistent scale for text hierarchy:

```rust
// Typography scale
pub const FONT_SIZE_CAPTION:  f32 = 11.0;  // Labels, hints, table headers
pub const FONT_SIZE_SMALL:    f32 = 12.0;  // Secondary text, badges
pub const FONT_SIZE_BODY:     f32 = 14.0;  // Default text, form inputs
pub const FONT_SIZE_SUBTITLE: f32 = 16.0;  // Emphasized text, card titles
pub const FONT_SIZE_TITLE:    f32 = 20.0;  // Section headers
pub const FONT_SIZE_HEADING:  f32 = 24.0;  // Page headers
pub const FONT_SIZE_DISPLAY:  f32 = 32.0;  // Hero text, empty states
```

### Text Styles

| Style | Size | Weight | Color | Use |
|-------|------|--------|-------|-----|
| Display | 32px | Bold | `GRAY_900` | Hero text, welcome messages |
| Heading | 24px | Bold | `GRAY_900` | Page titles |
| Title | 20px | Semi-bold | `GRAY_800` | Section headers |
| Subtitle | 16px | Medium | `GRAY_800` | Card titles, emphasis |
| Body | 14px | Regular | `GRAY_700` | Default text |
| Small | 12px | Regular | `GRAY_600` | Secondary info |
| Caption | 11px | Regular | `GRAY_500` | Labels, hints |

### Monospace

Use monospace font for:
- Variable names and codes
- File paths
- Data values
- Code snippets

```rust
// Monospace text helper
pub fn mono_text<'a, M: 'a>(content: &str) -> Element<'a, M> {
    text(content)
        .font(Font::MONOSPACE)
        .size(FONT_SIZE_BODY)
        .into()
}
```

---

## Spacing System

### Spacing Scale

Consistent spacing values for margins, padding, and gaps:

```rust
// Spacing scale (base unit: 4px)
pub const XS:  f32 = 4.0;   // Tight: between related items
pub const SM:  f32 = 8.0;   // Small: compact spacing
pub const MD:  f32 = 16.0;  // Medium: default padding
pub const LG:  f32 = 24.0;  // Large: section spacing
pub const XL:  f32 = 32.0;  // Extra large: major gaps
pub const XXL: f32 = 48.0;  // Page margins, hero spacing
```

### Usage Guidelines

| Spacing | Use Case |
|---------|----------|
| `XS` (4px) | Icon-to-text gap, inline element spacing |
| `SM` (8px) | Button padding, form field gaps |
| `MD` (16px) | Card padding, section margins, default gap |
| `LG` (24px) | Between sections, modal padding |
| `XL` (32px) | Page margins, major section breaks |
| `XXL` (48px) | Hero sections, empty state padding |

### Border Radius

```rust
// Border radius scale
pub const RADIUS_SM: f32 = 4.0;   // Buttons, badges, inputs
pub const RADIUS_MD: f32 = 6.0;   // Cards, list items
pub const RADIUS_LG: f32 = 8.0;   // Modals, panels
pub const RADIUS_XL: f32 = 12.0;  // Large cards, dialogs
```

### Example: Card Layout

```rust
fn card_example<'a, M: 'a>(title: &str, body: &str) -> Element<'a, M> {
    container(
        column![
            text(title).size(FONT_SIZE_SUBTITLE),
            text(body).size(FONT_SIZE_BODY).color(palette::GRAY_600),
        ]
        .spacing(spacing::SM)
    )
    .padding(spacing::MD)
    .style(|_| container::Style {
        background: Some(palette::WHITE.into()),
        border: iced::Border {
            radius: spacing::RADIUS_MD.into(),
            color: palette::GRAY_200,
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}
```

---

## Component Styling

### Buttons

#### Primary Button

Teal background, white text. For main actions.

```rust
fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(palette::PRIMARY_500.into()),
        text_color: palette::WHITE,
        border: iced::Border {
            radius: spacing::RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette::PRIMARY_600.into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(palette::PRIMARY_700.into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(palette::GRAY_300.into()),
            text_color: palette::GRAY_500,
            ..base
        },
        _ => base,
    }
}
```

#### Secondary Button

Outline style, teal border. For secondary actions.

```rust
fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(palette::WHITE.into()),
        text_color: palette::PRIMARY_600,
        border: iced::Border {
            radius: spacing::RADIUS_SM.into(),
            color: palette::PRIMARY_500,
            width: 1.0,
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette::PRIMARY_50.into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(palette::PRIMARY_100.into()),
            ..base
        },
        _ => base,
    }
}
```

#### Danger Button

Red for destructive actions.

```rust
fn danger_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(palette::ERROR.into()),
        text_color: palette::WHITE,
        border: iced::Border {
            radius: spacing::RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.75, 0.20, 0.20).into()),
            ..base
        },
        _ => base,
    }
}
```

#### Ghost Button

Text only, minimal styling.

```rust
fn ghost_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: palette::PRIMARY_600,
        border: iced::Border::default(),
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette::GRAY_100.into()),
            ..base
        },
        _ => base,
    }
}
```

### Text Inputs

```rust
fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: palette::WHITE.into(),
        border: iced::Border {
            radius: spacing::RADIUS_SM.into(),
            color: palette::GRAY_300,
            width: 1.0,
        },
        placeholder: palette::GRAY_400,
        value: palette::GRAY_900,
        selection: palette::PRIMARY_100,
        ..Default::default()
    };

    match status {
        text_input::Status::Focused => text_input::Style {
            border: iced::Border {
                color: palette::PRIMARY_500,
                width: 2.0,
                ..base.border
            },
            ..base
        },
        text_input::Status::Hovered => text_input::Style {
            border: iced::Border {
                color: palette::GRAY_400,
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: palette::GRAY_100.into(),
            value: palette::GRAY_500,
            ..base
        },
        _ => base,
    }
}
```

### Containers

#### Card Container

```rust
fn card_container_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::WHITE.into()),
        border: iced::Border {
            radius: spacing::RADIUS_MD.into(),
            color: palette::GRAY_200,
            width: 1.0,
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}
```

#### Panel Container

```rust
fn panel_container_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::GRAY_50.into()),
        border: iced::Border {
            radius: 0.0.into(),
            color: palette::GRAY_200,
            width: 1.0,
        },
        ..Default::default()
    }
}
```

---

## Icons

### Using iced_fonts

Trial Submission Studio uses `iced_fonts` 0.3.0 which includes:
- Font Awesome (solid, regular, brands)
- Bootstrap Icons
- Nerd Fonts

### Font Awesome Icons

```rust
use iced_fonts::fa;

// Icon constants (Font Awesome Solid)
pub fn icon_folder() -> Text<'static, Theme> {
    text(fa::FOLDER.to_string()).font(fa::FONT)
}

pub fn icon_file() -> Text<'static, Theme> {
    text(fa::FILE.to_string()).font(fa::FONT)
}

pub fn icon_check() -> Text<'static, Theme> {
    text(fa::CHECK.to_string()).font(fa::FONT)
}

pub fn icon_warning() -> Text<'static, Theme> {
    text(fa::TRIANGLE_EXCLAMATION.to_string()).font(fa::FONT)
}

pub fn icon_error() -> Text<'static, Theme> {
    text(fa::CIRCLE_XMARK.to_string()).font(fa::FONT)
}

pub fn icon_search() -> Text<'static, Theme> {
    text(fa::MAGNIFYING_GLASS.to_string()).font(fa::FONT)
}

pub fn icon_export() -> Text<'static, Theme> {
    text(fa::FILE_EXPORT.to_string()).font(fa::FONT)
}
```

### Icon Sizing

| Size | Use Case |
|------|----------|
| 12px | Inline with small text, badges |
| 14px | Inline with body text |
| 16px | Default icon size, buttons |
| 20px | Prominent icons, headers |
| 24px | Large icons, empty states |
| 32px | Hero icons |

### Icon Colors

- Default: Inherit from parent text color
- Active/Selected: `PRIMARY_500`
- Success: `SUCCESS`
- Warning: `WARNING`
- Error: `ERROR`
- Muted: `GRAY_400`

---

## Depth & Elevation

### Shadow Levels

Minimal shadows for subtle depth:

```rust
// Level 0: No shadow (flat)
pub const SHADOW_NONE: Shadow = Shadow::default();

// Level 1: Subtle (cards, buttons)
pub const SHADOW_SM: Shadow = Shadow {
    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
    offset: Vector::new(0.0, 1.0),
    blur_radius: 2.0,
};

// Level 2: Medium (dropdowns, popovers)
pub const SHADOW_MD: Shadow = Shadow {
    color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
    offset: Vector::new(0.0, 2.0),
    blur_radius: 4.0,
};

// Level 3: Large (modals)
pub const SHADOW_LG: Shadow = Shadow {
    color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
    offset: Vector::new(0.0, 4.0),
    blur_radius: 12.0,
};

// Level 4: Extra large (dialogs)
pub const SHADOW_XL: Shadow = Shadow {
    color: Color::from_rgba(0.0, 0.0, 0.0, 0.16),
    offset: Vector::new(0.0, 8.0),
    blur_radius: 24.0,
};
```

### Elevation Hierarchy

| Layer | Use | Shadow |
|-------|-----|--------|
| 0 | Page background | None |
| 1 | Cards, panels | `SHADOW_SM` |
| 2 | Dropdowns, popovers | `SHADOW_MD` |
| 3 | Sticky headers | `SHADOW_MD` |
| 4 | Modals, dialogs | `SHADOW_LG` |
| 5 | Toasts, notifications | `SHADOW_XL` |

---

## Accessibility

### Color Contrast

All text colors meet WCAG 2.1 AA standards:

| Text | Background | Contrast Ratio |
|------|------------|----------------|
| `GRAY_900` | `WHITE` | 15.3:1 |
| `GRAY_800` | `WHITE` | 11.5:1 |
| `GRAY_700` | `WHITE` | 8.5:1 |
| `GRAY_600` | `WHITE` | 5.7:1 |
| `PRIMARY_600` | `WHITE` | 4.8:1 |
| `WHITE` | `PRIMARY_500` | 4.5:1 |

### Focus States

All interactive elements have visible focus indicators:

```rust
// Focus ring style
fn focus_ring() -> iced::Border {
    iced::Border {
        color: palette::PRIMARY_500,
        width: 2.0,
        radius: spacing::RADIUS_SM.into(),
    }
}
```

### Keyboard Navigation

- All interactive elements are keyboard accessible
- Tab order follows visual order
- Focus is clearly visible
- Escape closes modals/dialogs

### Text Sizing

- Minimum body text: 14px
- Minimum caption text: 11px
- Line height: 1.5 for body text
- Text remains readable at 200% zoom

---

## Theme Implementation

### Creating the Clinical Theme

```rust
// theme/clinical.rs

use iced::{Color, Theme};
use iced::theme::Palette;

/// Create the Professional Clinical light theme
pub fn clinical_light() -> Theme {
    Theme::custom(
        "Professional Clinical".to_string(),
        Palette {
            background: GRAY_50,
            text: GRAY_900,
            primary: PRIMARY_500,
            success: SUCCESS,
            danger: ERROR,
        },
    )
}
```

### Applying the Theme

```rust
// In app.rs
impl App {
    pub fn theme(&self) -> Theme {
        clinical_light()
    }
}
```

### Component-Specific Styles

Override default styles using style functions:

```rust
// Custom button style
button("Click me")
    .style(primary_button_style)

// Custom container style
container(content)
    .style(card_container_style)

// Custom text input style
text_input("Placeholder", &value)
    .style(text_input_style)
```

### Helper Module

```rust
// theme/mod.rs

mod clinical;
mod palette;
mod spacing;
mod typography;

pub use clinical::clinical_light;
pub use palette::*;
pub use spacing::*;
pub use typography::*;
```

---

## Quick Reference

### Color Cheat Sheet

```
Background:     GRAY_50     #FAFAFE
Surface:        GRAY_100    #F2F2F7
Card:           WHITE       #FFFFFF
Border:         GRAY_200    #E6E6ED
Text Primary:   GRAY_900    #1A1A1F
Text Secondary: GRAY_600    #666673
Accent:         PRIMARY_500 #009BA6
Success:        SUCCESS     #33B366
Warning:        WARNING     #F2A60D
Error:          ERROR       #D94040
```

### Spacing Cheat Sheet

```
XS:  4px   - Icon gaps, tight spacing
SM:  8px   - Button padding, small gaps
MD:  16px  - Default padding, card margins
LG:  24px  - Section spacing
XL:  32px  - Page margins
XXL: 48px  - Hero sections
```

### Typography Cheat Sheet

```
Caption:  11px  - Labels, hints
Small:    12px  - Secondary text
Body:     14px  - Default text
Subtitle: 16px  - Emphasis
Title:    20px  - Section headers
Heading:  24px  - Page headers
Display:  32px  - Hero text
```

---

## Next Steps

- **[01-architecture.md](./01-architecture.md)** - Overall architecture guide
- **[02-message-patterns.md](./02-message-patterns.md)** - Message hierarchy
- **[03-state-management.md](./03-state-management.md)** - State patterns
- **[04-component-guide.md](./04-component-guide.md)** - Component patterns
