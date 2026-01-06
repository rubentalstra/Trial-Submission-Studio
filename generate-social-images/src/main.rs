use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use resvg::tiny_skia::{self, Pixmap};
use resvg::usvg::{Options, Tree};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::shape::ShapeContext;
use swash::zeno::{Format, Vector};
use swash::FontRef;

#[derive(Debug, Deserialize)]
struct Config {
    text: TextConfig,
    paths: PathsConfig,
    colors: ColorsConfig,
    output: OutputConfig,
}

#[derive(Debug, Deserialize)]
struct TextConfig {
    repo_path: Vec<String>,
    title: Vec<String>,
    tagline: Vec<String>,
    standards: Vec<String>,
    output_formats: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PathsConfig {
    logo_svg: String,
    output_dir: String,
    filename_prefix: String,
}

#[derive(Debug, Deserialize)]
struct ColorsConfig {
    background: String,
    title: String,
    tagline: String,
    secondary: String,
    accent_blue: String,
    dot_blue: String,
    dot_red: String,
    dot_yellow: String,
    dot_teal: String,
}

#[derive(Debug, Deserialize)]
struct OutputConfig {
    sizes: Vec<[u32; 2]>,
}

fn hex_to_rgba(hex: &str) -> Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Rgba([r, g, b, 255])
}

fn load_config(crate_dir: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = crate_dir.join("config.toml");
    if !config_path.exists() {
        return Err(format!("Config file not found: {}", config_path.display()).into());
    }
    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn render_svg_to_image(
    svg_path: &Path,
    target_size: u32,
    font_path: &Path,
) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let svg_data = fs::read(svg_path)?;

    // Load Inter font for SVG text rendering
    let mut options = Options::default();
    options.fontdb_mut().load_font_file(font_path).ok();
    // Also load system fonts as fallback
    options.fontdb_mut().load_system_fonts();

    let tree = Tree::from_data(&svg_data, &options)?;

    let original_size = tree.size();
    let scale = target_size as f32 / original_size.width().max(original_size.height());

    let width = (original_size.width() * scale) as u32;
    let height = (original_size.height() * scale) as u32;

    let mut pixmap = Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let img =
        ImageBuffer::from_raw(width, height, pixmap.take()).ok_or("Failed to create image buffer")?;
    Ok(img)
}

/// Calculate text width using swash
fn measure_text_width(
    text: &str,
    size: f32,
    weight: f32,
    font_data: &[u8],
    shape_context: &mut ShapeContext,
) -> f32 {
    let font = FontRef::from_index(font_data, 0).expect("Failed to create font ref");

    let mut shaper = shape_context
        .builder(font)
        .size(size)
        .variations(&[("wght", weight)])
        .build();

    shaper.add_str(text);

    let mut total_width = 0.0;
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            total_width += glyph.advance;
        }
    });

    total_width
}

/// Draw text using swash with variable font weight support
fn draw_text_swash(
    img: &mut RgbaImage,
    text: &str,
    x: i32,
    y: i32,
    size: f32,
    weight: f32,
    color: Rgba<u8>,
    font_data: &[u8],
    scale_context: &mut ScaleContext,
    shape_context: &mut ShapeContext,
) {
    let font = FontRef::from_index(font_data, 0).expect("Failed to create font ref");

    // Create a shaper with the font
    let mut shaper = shape_context
        .builder(font)
        .size(size)
        .variations(&[("wght", weight)])
        .build();

    // Shape the text
    shaper.add_str(text);

    // Build a scaler for rendering glyphs
    let mut scaler = scale_context
        .builder(font)
        .size(size)
        .variations(&[("wght", weight)])
        .build();

    let mut pen_x = x as f32;
    let pen_y = y as f32 + size; // Baseline offset

    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            // Render the glyph
            let rendered = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .format(Format::Alpha)
            .offset(Vector::new(glyph.x, glyph.y))
            .render(&mut scaler, glyph.id);

            if let Some(rendered_image) = rendered {
                let glyph_x = (pen_x + rendered_image.placement.left as f32) as i32;
                let glyph_y = (pen_y - rendered_image.placement.top as f32) as i32;

                // Draw the glyph pixels onto the image
                for py in 0..rendered_image.placement.height {
                    for px in 0..rendered_image.placement.width {
                        let idx = (py * rendered_image.placement.width + px) as usize;
                        let alpha = rendered_image.data[idx];
                        if alpha > 0 {
                            let dest_x = glyph_x + px as i32;
                            let dest_y = glyph_y + py as i32;
                            if dest_x >= 0
                                && dest_y >= 0
                                && (dest_x as u32) < img.width()
                                && (dest_y as u32) < img.height()
                            {
                                let dest_pixel = img.get_pixel_mut(dest_x as u32, dest_y as u32);
                                // Alpha blend
                                let a = alpha as f32 / 255.0;
                                let inv_a = 1.0 - a;
                                dest_pixel[0] =
                                    (color[0] as f32 * a + dest_pixel[0] as f32 * inv_a) as u8;
                                dest_pixel[1] =
                                    (color[1] as f32 * a + dest_pixel[1] as f32 * inv_a) as u8;
                                dest_pixel[2] =
                                    (color[2] as f32 * a + dest_pixel[2] as f32 * inv_a) as u8;
                                dest_pixel[3] = 255;
                            }
                        }
                    }
                }
            }

            pen_x += glyph.advance;
        }
    });
}

fn generate_image(
    config: &Config,
    width: u32,
    height: u32,
    logo_img: &RgbaImage,
    font_data: &[u8],
    scale_context: &mut ScaleContext,
    shape_context: &mut ShapeContext,
) -> RgbaImage {
    let mut img: RgbaImage =
        ImageBuffer::from_pixel(width, height, hex_to_rgba(&config.colors.background));

    // Scale factor (base design is 1280x640)
    let scale_factor = width as f32 / 1280.0;

    // Colors
    let title_color = hex_to_rgba(&config.colors.title);
    let tagline_color = hex_to_rgba(&config.colors.tagline);
    let secondary_color = hex_to_rgba(&config.colors.secondary);
    let accent_blue = hex_to_rgba(&config.colors.accent_blue);
    let dot_blue = hex_to_rgba(&config.colors.dot_blue);
    let dot_red = hex_to_rgba(&config.colors.dot_red);
    let dot_yellow = hex_to_rgba(&config.colors.dot_yellow);
    let dot_teal = hex_to_rgba(&config.colors.dot_teal);

    // Margins and positions (scaled)
    let margin_left = (50.0 * scale_factor) as i32;
    let margin_top = (35.0 * scale_factor) as i32;

    // Font sizes (scaled)
    let repo_font_size = 26.0 * scale_factor;
    let title_font_size = 100.0 * scale_factor;
    let tagline_font_size = 34.0 * scale_factor;
    let standards_font_size = 26.0 * scale_factor;

    // Font weights
    let weight_regular = 400.0;
    let weight_bold = 700.0;

    // Draw repo path
    let repo_text = config.text.repo_path.join(" / ");
    draw_text_swash(
        &mut img,
        &repo_text,
        margin_left,
        margin_top,
        repo_font_size,
        weight_regular,
        secondary_color,
        font_data,
        scale_context,
        shape_context,
    );

    // Draw title lines (BOLD)
    let title_start_y = margin_top + (55.0 * scale_factor) as i32;
    let title_line_height = (105.0 * scale_factor) as i32;
    for (i, line) in config.text.title.iter().enumerate() {
        let y = title_start_y + (i as i32 * title_line_height);
        draw_text_swash(
            &mut img,
            line,
            margin_left,
            y,
            title_font_size,
            weight_bold,
            title_color,
            font_data,
            scale_context,
            shape_context,
        );
    }

    // Draw underline - width matches "Submission" text, with rounded ends
    let submission_width = measure_text_width(
        "Submission",
        title_font_size,
        weight_bold,
        font_data,
        shape_context,
    );

    // Use even number for thickness to avoid rounding issues with radius
    let underline_thickness = (14.0 * scale_factor).round() as i32;
    let underline_thickness = if underline_thickness % 2 == 1 { underline_thickness + 1 } else { underline_thickness };
    let underline_radius = underline_thickness / 2;

    let underline_center_y = title_start_y
        + (config.text.title.len() as i32 * title_line_height)
        + (12.0 * scale_factor) as i32
        + underline_radius;
    let underline_width = submission_width as i32;

    // Draw rounded pill shape: two circles at ends + rectangle in middle
    let left_center_x = margin_left + underline_radius;
    let right_center_x = margin_left + underline_width - underline_radius;

    // Left rounded end
    draw_filled_circle_mut(&mut img, (left_center_x, underline_center_y), underline_radius, accent_blue);
    // Right rounded end
    draw_filled_circle_mut(&mut img, (right_center_x, underline_center_y), underline_radius, accent_blue);
    // Middle rectangle connecting the two circles
    draw_filled_rect_mut(
        &mut img,
        Rect::at(left_center_x, underline_center_y - underline_radius)
            .of_size((right_center_x - left_center_x) as u32, (underline_radius * 2 + 1) as u32),
        accent_blue,
    );

    let underline_bottom = underline_center_y + underline_radius;

    // Draw tagline lines
    let tagline_start_y = underline_bottom + (20.0 * scale_factor) as i32;
    let tagline_line_height = (42.0 * scale_factor) as i32;
    for (i, line) in config.text.tagline.iter().enumerate() {
        let y = tagline_start_y + (i as i32 * tagline_line_height);
        draw_text_swash(
            &mut img,
            line,
            margin_left,
            y,
            tagline_font_size,
            weight_regular,
            tagline_color,
            font_data,
            scale_context,
            shape_context,
        );
    }

    // Draw 4 colored dots
    let dots_y = tagline_start_y
        + (config.text.tagline.len() as i32 * tagline_line_height)
        + (25.0 * scale_factor) as i32;
    let dot_radius = (10.0 * scale_factor) as i32;
    let dot_spacing = (32.0 * scale_factor) as i32;
    let dot_colors = [dot_blue, dot_red, dot_yellow, dot_teal];

    for (i, &color) in dot_colors.iter().enumerate() {
        let x = margin_left + dot_radius + (i as i32 * dot_spacing);
        draw_filled_circle_mut(&mut img, (x, dots_y), dot_radius, color);
    }

    // Draw standards + output formats line
    let standards_text = config.text.standards.join(" \u{2022} ");
    let formats_text = config.text.output_formats.join(" \u{2022} ");
    let combined_text = format!("{}  |  {}", standards_text, formats_text);
    let standards_y = dots_y + (30.0 * scale_factor) as i32;
    draw_text_swash(
        &mut img,
        &combined_text,
        margin_left,
        standards_y,
        standards_font_size,
        weight_regular,
        secondary_color,
        font_data,
        scale_context,
        shape_context,
    );

    // Draw logo on the right side
    let logo_size = (340.0 * scale_factor) as u32;
    let logo_x = width - logo_size - (80.0 * scale_factor) as u32;
    let logo_y = (height - logo_size) / 2;

    // Scale logo to fit
    let scaled_logo = image::imageops::resize(
        logo_img,
        logo_size,
        logo_size,
        image::imageops::FilterType::Lanczos3,
    );

    // Overlay logo onto main image
    for (x, y, pixel) in scaled_logo.enumerate_pixels() {
        if pixel[3] > 0 {
            let dest_x = logo_x + x;
            let dest_y = logo_y + y;
            if dest_x < width && dest_y < height {
                let dest_pixel = img.get_pixel_mut(dest_x, dest_y);
                // Alpha blending
                let alpha = pixel[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;
                dest_pixel[0] = (pixel[0] as f32 * alpha + dest_pixel[0] as f32 * inv_alpha) as u8;
                dest_pixel[1] = (pixel[1] as f32 * alpha + dest_pixel[1] as f32 * inv_alpha) as u8;
                dest_pixel[2] = (pixel[2] as f32 * alpha + dest_pixel[2] as f32 * inv_alpha) as u8;
                dest_pixel[3] = 255;
            }
        }
    }

    img
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine paths
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_dir.parent().ok_or("Cannot find project root")?;

    println!("Loading configuration...");
    let config = load_config(&crate_dir)?;

    // Load font data
    println!("Loading font...");
    let font_path = crate_dir.join("fonts/Inter-VariableFont_opsz,wght.ttf");
    if !font_path.exists() {
        return Err(format!("Font file not found: {}", font_path.display()).into());
    }
    let font_data = fs::read(&font_path)?;

    // Create swash contexts
    let mut scale_context = ScaleContext::new();
    let mut shape_context = ShapeContext::new();

    // Load and render SVG logo
    println!("Loading logo SVG...");
    let logo_path = project_root.join(&config.paths.logo_svg);
    if !logo_path.exists() {
        return Err(format!("Logo SVG not found: {}", logo_path.display()).into());
    }
    let logo_img = render_svg_to_image(&logo_path, 512, &font_path)?;

    // Create output directory
    let output_dir = project_root.join(&config.paths.output_dir);
    fs::create_dir_all(&output_dir)?;

    // Generate images for each size
    for [width, height] in &config.output.sizes {
        println!("Generating {}x{} image...", width, height);
        let img = generate_image(
            &config,
            *width,
            *height,
            &logo_img,
            &font_data,
            &mut scale_context,
            &mut shape_context,
        );

        let filename = format!("{}_{}x{}.png", config.paths.filename_prefix, width, height);
        let output_path = output_dir.join(&filename);
        img.save(&output_path)?;
        println!("  Saved: {}", output_path.display());
    }

    println!("\nDone! Generated {} images.", config.output.sizes.len());
    Ok(())
}
