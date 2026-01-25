# Social Preview Image Generator

A standalone Rust tool that generates social preview images for Trial Submission Studio.

## Overview

This tool generates PNG social preview images from configuration, using the project's logo SVG and Inter fonts. It
produces images suitable for Open Graph (OG) and Twitter Card metadata.

## Prerequisites

- Rust toolchain (1.92+)
- The project's logo SVG file at `docs/src/images/logo.svg`

## Usage

From the project root directory:

```bash
cargo run --release --manifest-path generate-social-images/Cargo.toml
```

Or from within the `generate-social-images/` directory:

```bash
cargo run --release
```

## Output

By default, the tool generates two images in `docs/src/images/` (for mdbook):

| File                                                   | Dimensions | Use Case                 |
|--------------------------------------------------------|------------|--------------------------|
| `trial-submission-studio-social-preview_1280x640.png`  | 1280x640   | Standard OG/Twitter      |
| `trial-submission-studio-social-preview_2560x1280.png` | 2560x1280  | High-resolution displays |

## Configuration

All text content, colors, paths, and output sizes are configured in `config.toml`.

### Text Section

```toml
[text]
repo_path = ["rubentalstra", "Trial-Submission-Studio"]
title = ["Trial", "Submission", "Studio"]
tagline = ["Transform clinical trial data into", "CDISC-ready submissions."]
standards = ["SDTM", "ADaM", "SEND"]
output_formats = ["XPT", "Dataset-XML", "Define-XML"]
```

- `repo_path`: Repository path segments, joined with ` / `
- `title`: Title lines, each displayed on a new line
- `tagline`: Tagline lines, each displayed on a new line
- `standards`: Standard names, joined with ` * `
- `output_formats`: Output format names, joined with ` * `

The standards and output formats are displayed together, separated by `  |  `.

### Paths Section

```toml
[paths]
logo_svg = "docs/src/images/logo.svg"
output_dir = "docs/src/images"
filename_prefix = "trial-submission-studio-social-preview"
```

- `logo_svg`: Path to the logo SVG file (relative to project root)
- `output_dir`: Directory for output images (relative to project root)
- `filename_prefix`: Prefix for output filenames

### Colors Section

```toml
[colors]
background = "#FFFFFF"
title = "#1a1a2e"
tagline = "#1a1a2e"
secondary = "#4a4a4a"
accent_blue = "#144678"
dot_blue = "#144678"
dot_red = "#cb4544"
dot_yellow = "#edaa00"
dot_teal = "#a1d0ca"
```

All colors are specified as hex values.

### Output Section

```toml
[output]
sizes = [[1280, 640], [2560, 1280]]
```

Each size is specified as `[width, height]`. Add more sizes as needed.

## Image Layout

```
+--------------------------------------------------------------+
| rubentalstra / Trial-Submission-Studio                       |
|                                                              |
| Trial                                                        |
| Submission                                       +-----+     |
| Studio                                           | TSS |     |
| ____________                                     +-----+     |
| Transform clinical trial data into                           |
| CDISC-ready submissions.                                     |
| ****  (blue, red, yellow, teal dots)                         |
| SDTM * ADaM * SEND  |  XPT * Dataset-XML * Define-XML        |
+--------------------------------------------------------------+
```

## Customization

### Changing Text

Edit the arrays in `config.toml` under `[text]`. Each array element becomes a separate line (for title/tagline) or is
joined with separators (for repo_path, standards, output_formats).

### Changing Colors

Update the hex color values in `config.toml` under `[colors]`.

### Changing Logo

Update the `logo_svg` path in `config.toml` to point to a different SVG file.

### Adding Output Sizes

Add new `[width, height]` pairs to the `sizes` array in `config.toml`.

## Fonts

The tool uses the Inter variable font (`Inter-VariableFont_opsz,wght.ttf`) with full support for the `wght` (weight)
axis:

- **Title**: Rendered at weight 700 (bold)
- **Other text**: Rendered at weight 400 (regular)

The font is included in the `fonts/` directory and is licensed under the SIL Open Font License.

## Dependencies

- `image` - Image creation and manipulation
- `imageproc` - Drawing primitives
- `swash` - Font loading and text rendering with variable font support
- `resvg` - SVG rendering
- `toml` - Configuration file parsing
- `serde` - Serialization/deserialization
