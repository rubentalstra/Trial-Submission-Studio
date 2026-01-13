//! Export dialog views for Trial Submission Studio.
//!
//! Multi-window dialogs for export operations:
//! - Progress: Shows current export status with cancel option
//! - Completion: Shows results (success, error, or cancelled)

use iced::widget::{Space, button, column, container, progress_bar, row, text};
use iced::window;
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::message::{ExportMessage, Message};
use crate::state::{ExportProgressState, ExportResult};
use crate::theme::{
    GRAY_100, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, SPACING_LG, SPACING_MD, SPACING_SM,
    SUCCESS, WARNING, WHITE, button_primary, button_secondary,
};

// =============================================================================
// EXPORT PROGRESS DIALOG
// =============================================================================

/// Render the export progress dialog content for a standalone window.
pub fn view_export_progress_dialog_content(
    state: &ExportProgressState,
    window_id: window::Id,
) -> Element<Message> {
    let domain_text = state.current_domain.as_deref().unwrap_or("Preparing...");

    // Header with icon
    let header = row![
        lucide::loader().size(24).color(GRAY_700),
        Space::new().width(SPACING_SM),
        text("Exporting...").size(20).color(GRAY_900),
    ]
    .align_y(Alignment::Center);

    // Current domain and step
    let domain_label = text(domain_text).size(16).color(GRAY_800);
    let step_label = text(&state.current_step).size(14).color(GRAY_600);

    // Progress bar
    let progress = container(progress_bar(0.0..=1.0, state.progress)).width(Length::Fixed(320.0));

    // Progress percentage
    let percent = (state.progress * 100.0) as u32;
    let progress_text = text(format!("{}%", percent)).size(14).color(GRAY_600);

    // Files written
    let files_text = text(format!("{} files written", state.files_written))
        .size(12)
        .color(GRAY_500);

    // Cancel button
    let cancel_button = button(
        row![
            lucide::x().size(14),
            Space::new().width(SPACING_SM),
            text("Cancel").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::CancelExport))
    .padding([10.0, 20.0])
    .style(button_secondary);

    let content = column![
        header,
        Space::new().height(SPACING_LG),
        domain_label,
        Space::new().height(SPACING_SM),
        step_label,
        Space::new().height(SPACING_MD),
        progress,
        Space::new().height(SPACING_SM),
        row![progress_text, Space::new().width(SPACING_MD), files_text,].align_y(Alignment::Center),
        Space::new().height(SPACING_LG),
        cancel_button,
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

// =============================================================================
// EXPORT COMPLETION DIALOG
// =============================================================================

/// Render the export completion dialog content for a standalone window.
pub fn view_export_complete_dialog_content<'a>(
    result: &'a ExportResult,
    window_id: window::Id,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match result {
        ExportResult::Success {
            output_dir,
            files,
            domains_exported,
            elapsed_ms,
            warnings,
        } => view_success_content(output_dir, files.len(), *domains_exported, *elapsed_ms),
        ExportResult::Error { message, domain } => view_error_content(message, domain.as_deref()),
        ExportResult::Cancelled => view_cancelled_content(),
    };

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

/// Success state content.
fn view_success_content(
    output_dir: &std::path::Path,
    files_count: usize,
    domains_exported: usize,
    elapsed_ms: u64,
) -> Element<Message> {
    let icon = lucide::circle_check().size(56).color(SUCCESS);

    let title = text("Export Complete!").size(22).color(GRAY_900);

    let files_text = text(format!("{} files written", files_count))
        .size(16)
        .color(GRAY_700);

    let domains_text = text(format!(
        "{} domain{} exported in {}ms",
        domains_exported,
        if domains_exported == 1 { "" } else { "s" },
        elapsed_ms
    ))
    .size(13)
    .color(GRAY_600);

    let output_label = text("Output directory:").size(12).color(GRAY_500);

    let output_path = container(
        text(output_dir.display().to_string())
            .size(12)
            .color(GRAY_700),
    )
    .padding([SPACING_SM, SPACING_MD])
    .style(|_| container::Style {
        background: Some(WHITE.into()),
        border: Border {
            radius: 4.0.into(),
            color: Color::from_rgb(0.9, 0.9, 0.9),
            width: 1.0,
        },
        ..Default::default()
    });

    // Action buttons
    let open_folder_button = button(
        row![
            lucide::folder_open().size(14),
            Space::new().width(SPACING_SM),
            text("Show in Folder").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::OpenOutputFolder))
    .padding([10.0, 16.0])
    .style(button_secondary);

    let done_button = button(
        row![
            lucide::check().size(14),
            Space::new().width(SPACING_SM),
            text("Done").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::DismissCompletion))
    .padding([10.0, 20.0])
    .style(button_primary);

    let buttons = row![
        open_folder_button,
        Space::new().width(SPACING_MD),
        done_button,
    ]
    .align_y(Alignment::Center);

    column![
        icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        files_text,
        Space::new().height(SPACING_SM),
        domains_text,
        Space::new().height(SPACING_LG),
        output_label,
        Space::new().height(SPACING_SM),
        output_path,
        Space::new().height(SPACING_LG),
        buttons,
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Error state content.
fn view_error_content<'a>(message: &'a str, domain: Option<&'a str>) -> Element<'a, Message> {
    let icon = lucide::circle_x().size(56).color(WARNING);

    let title = text("Export Failed").size(22).color(GRAY_900);

    let domain_text: Element<'a, Message> = if let Some(d) = domain {
        text(format!("Domain: {}", d))
            .size(14)
            .color(GRAY_700)
            .into()
    } else {
        Space::new().into()
    };

    let error_container = container(text(message).size(13).color(WARNING))
        .padding([SPACING_SM, SPACING_MD])
        .max_width(350)
        .style(|_| container::Style {
            background: Some(Color::from_rgb(1.0, 0.95, 0.95).into()),
            border: Border {
                radius: 4.0.into(),
                color: WARNING,
                width: 1.0,
            },
            ..Default::default()
        });

    // Action buttons
    let retry_button = button(
        row![
            lucide::refresh_cw().size(14),
            Space::new().width(SPACING_SM),
            text("Retry").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::RetryExport))
    .padding([10.0, 16.0])
    .style(button_secondary);

    let close_button = button(
        row![
            lucide::x().size(14),
            Space::new().width(SPACING_SM),
            text("Close").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::DismissCompletion))
    .padding([10.0, 20.0])
    .style(button_primary);

    let buttons = row![retry_button, Space::new().width(SPACING_MD), close_button,]
        .align_y(Alignment::Center);

    column![
        icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        domain_text,
        Space::new().height(SPACING_MD),
        error_container,
        Space::new().height(SPACING_LG),
        buttons,
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}

/// Cancelled state content.
fn view_cancelled_content<'a>() -> Element<'a, Message> {
    let icon = lucide::circle_slash().size(56).color(GRAY_500);

    let title = text("Export Cancelled").size(22).color(GRAY_900);

    let message = text("The export operation was cancelled.")
        .size(14)
        .color(GRAY_600);

    let close_button = button(
        row![
            lucide::x().size(14),
            Space::new().width(SPACING_SM),
            text("Close").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::DismissCompletion))
    .padding([10.0, 20.0])
    .style(button_primary);

    column![
        icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        message,
        Space::new().height(SPACING_LG),
        close_button,
    ]
    .align_x(Alignment::Center)
    .padding(SPACING_LG)
    .into()
}
