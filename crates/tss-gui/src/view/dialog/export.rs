//! Export dialog views for Trial Submission Studio.
//!
//! Multi-window dialogs for export operations:
//! - Progress: Shows current export status with cancel option
//! - Completion: Shows results (success, error, or cancelled)

use iced::widget::{Space, button, column, container, progress_bar, row, text};
use iced::window;
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{ExportMessage, Message};
use crate::state::{ExportProgressState, ExportResult};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, button_primary, button_secondary,
};

// =============================================================================
// EXPORT PROGRESS DIALOG
// =============================================================================

/// Render the export progress dialog content for a standalone window.
pub fn view_export_progress_dialog_content<'a>(
    state: &'a ExportProgressState,
    _window_id: window::Id,
) -> Element<'a, Message> {
    let domain_text = state.current_domain.as_deref().unwrap_or("Preparing...");

    // Header with icon
    let header = row![
        container(lucide::loader().size(24)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_secondary),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Exporting...")
            .size(20)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text)
            }),
    ]
    .align_y(Alignment::Center);

    // Current domain and step
    let domain_label = text(domain_text)
        .size(16)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });
    let step_label = text(&state.current_step)
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    // Progress bar
    let progress = container(progress_bar(0.0..=1.0, state.progress)).width(Length::Fixed(320.0));

    // Progress percentage
    let percent = (state.progress * 100.0) as u32;
    let progress_text = text(format!("{}%", percent))
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    // Files written
    let files_text = text(format!("{} files written", state.files_written))
        .size(12)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        });

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
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
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
    _window_id: window::Id,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match result {
        ExportResult::Success {
            output_dir,
            files,
            domains_exported,
            elapsed_ms,
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
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
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
) -> Element<'static, Message> {
    let icon = container(lucide::circle_check().size(56)).style(|theme: &Theme| container::Style {
        text_color: Some(theme.extended_palette().success.base.color),
        ..Default::default()
    });

    let title = text("Export Complete!")
        .size(22)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let files_text = text(format!("{} files written", files_count))
        .size(16)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let domains_text = text(format!(
        "{} domain{} exported in {}ms",
        domains_exported,
        if domains_exported == 1 { "" } else { "s" },
        elapsed_ms
    ))
    .size(13)
    .style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_secondary),
    });

    let output_label = text("Output directory:")
        .size(12)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_muted),
        });

    let output_path = container(text(output_dir.display().to_string()).size(12).style(
        |theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        },
    ))
    .padding([SPACING_SM, SPACING_MD])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_elevated.into()),
        border: Border {
            radius: 4.0.into(),
            color: theme.clinical().border_default,
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
fn view_error_content(message: &str, domain: Option<&str>) -> Element<'static, Message> {
    let icon = container(lucide::circle_x().size(56)).style(|theme: &Theme| container::Style {
        text_color: Some(theme.extended_palette().warning.base.color),
        ..Default::default()
    });

    let title = text("Export Failed")
        .size(22)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let domain_text: Element<'static, Message> = if let Some(d) = domain {
        text(format!("Domain: {}", d))
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            })
            .into()
    } else {
        Space::new().into()
    };

    let error_container =
        container(
            text(message.to_string())
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().warning.base.color),
                }),
        )
        .padding([SPACING_SM, SPACING_MD])
        .max_width(350)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().status_warning_light.into()),
            border: Border {
                radius: 4.0.into(),
                color: theme.extended_palette().warning.base.color,
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
fn view_cancelled_content() -> Element<'static, Message> {
    let icon = container(lucide::circle_slash().size(56)).style(|theme: &Theme| container::Style {
        text_color: Some(theme.clinical().text_muted),
        ..Default::default()
    });

    let title = text("Export Cancelled")
        .size(22)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let message = text("The export operation was cancelled.")
        .size(14)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

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
