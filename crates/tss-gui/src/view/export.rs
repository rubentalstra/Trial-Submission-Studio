//! Export view for Trial Submission Studio.
//!
//! Master-detail layout with domain selection (left) and configuration (right).
//! Progress and completion dialogs are shown in separate windows.

use iced::widget::{
    Space, button, checkbox, column, container, radio, row, rule, scrollable, text,
};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::message::{ExportMessage, Message};
use crate::service::export::{domain_has_supp, domain_supp_count};
use crate::state::{
    AppState, DomainState, ExportFormat, ExportViewState, Study, ViewState, XptVersion,
};
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, PRIMARY_500, SPACING_LG,
    SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS, WARNING, WHITE, button_primary,
};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Width of the master (domain list) panel.
const MASTER_WIDTH: f32 = 300.0;

// =============================================================================
// MAIN VIEW FUNCTION
// =============================================================================

/// Render the export view.
///
/// Note: Export progress and completion dialogs are now shown in separate windows,
/// so this view just shows the base export configuration layout.
pub fn view_export(state: &AppState) -> Element<'_, Message> {
    let export_state = match &state.view {
        ViewState::Export(export) => export,
        _ => return text("Invalid view state").into(),
    };

    // Check if we have a study
    let study = match &state.study {
        Some(s) => s,
        None => return view_no_study(),
    };

    // Just show the base export layout
    // Progress and completion are handled by separate dialog windows
    view_export_layout(state, export_state, study)
}

// =============================================================================
// LAYOUT COMPONENTS
// =============================================================================

/// View when no study is loaded.
fn view_no_study<'a>() -> Element<'a, Message> {
    let icon = lucide::folder_open().size(48).color(GRAY_500);

    let content = column![
        icon,
        Space::new().height(SPACING_MD),
        text("No Study Loaded").size(20).color(GRAY_800),
        Space::new().height(SPACING_SM),
        text("Open a study folder to export domains").color(GRAY_500),
        Space::new().height(SPACING_LG),
        button(
            row![
                lucide::arrow_left().size(14),
                Space::new().width(SPACING_XS),
                text("Go Back"),
            ]
            .align_y(Alignment::Center)
        )
        .on_press(Message::Navigate(ViewState::home()))
        .padding([SPACING_SM, SPACING_MD]),
    ]
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Shrink)
        .center_y(Length::Shrink)
        .into()
}

/// Main export layout with header and two-column content.
fn view_export_layout<'a>(
    state: &'a AppState,
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let header = view_header();
    let content = view_two_column_layout(state, export_state, study);

    column![header, rule::horizontal(1), content,]
        .spacing(SPACING_MD)
        .padding(SPACING_LG)
        .into()
}

/// Header with back button and title.
fn view_header<'a>() -> Element<'a, Message> {
    let back_button = button(
        row![
            lucide::arrow_left().size(14),
            Space::new().width(SPACING_XS),
            text("Back"),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Navigate(ViewState::home()))
    .padding([SPACING_XS, SPACING_SM]);

    row![
        back_button,
        Space::new().width(Length::Fill),
        text("Export").size(24).color(GRAY_900),
        Space::new().width(Length::Fill),
        Space::new().width(80), // Balance the back button
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Two-column layout: domain selection (left) and configuration (right).
fn view_two_column_layout<'a>(
    state: &'a AppState,
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let left_panel = view_domain_selection(export_state, study);
    let right_panel = view_export_config(state, export_state, study);

    row![
        container(left_panel).width(MASTER_WIDTH),
        rule::vertical(1),
        container(right_panel).width(Length::Fill),
    ]
    .spacing(SPACING_LG)
    .height(Length::Fill)
    .into()
}

// =============================================================================
// LEFT PANEL: DOMAIN SELECTION
// =============================================================================

/// Domain selection panel.
fn view_domain_selection<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let title = row![
        lucide::database().size(16).color(GRAY_700),
        Space::new().width(SPACING_XS),
        text("Select Domains").size(16).color(GRAY_800),
    ]
    .align_y(Alignment::Center);

    // Selection controls
    let select_all_btn = button(text("Select All").size(12))
        .on_press(Message::Export(ExportMessage::SelectAll))
        .padding([SPACING_XS, SPACING_SM]);

    let deselect_btn = button(text("Clear").size(12))
        .on_press(Message::Export(ExportMessage::DeselectAll))
        .padding([SPACING_XS, SPACING_SM]);

    let controls = row![select_all_btn, Space::new().width(SPACING_SM), deselect_btn,]
        .align_y(Alignment::Center);

    // Domain list
    let domain_codes = study.domain_codes();
    let domain_list: Vec<Element<'a, Message>> = domain_codes
        .iter()
        .filter_map(|code| {
            let domain = study.domain(code)?;
            Some(view_domain_row(code, domain, export_state))
        })
        .collect();

    let domain_list_content = if domain_list.is_empty() {
        column![text("No domains found").color(GRAY_500),]
    } else {
        column(domain_list).spacing(SPACING_XS)
    };

    column![
        title,
        Space::new().height(SPACING_SM),
        controls,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
        scrollable(domain_list_content).height(Length::Fill),
    ]
    .spacing(SPACING_XS)
    .into()
}

/// Single domain row with checkbox, status, and SUPP indicator.
fn view_domain_row<'a>(
    code: &'a str,
    domain: &'a DomainState,
    export_state: &'a ExportViewState,
) -> Element<'a, Message> {
    let is_selected = export_state.is_selected(code);
    let row_count = domain.row_count();

    // Determine status based on mapping progress
    let mapping = &domain.mapping;
    let accepted_count = mapping.all_accepted().len();
    let total_count = mapping.domain().variables.len();
    let mapped_ratio = if total_count > 0 {
        accepted_count as f32 / total_count as f32
    } else {
        0.0
    };

    let status_widget: Element<'a, Message> = if mapped_ratio >= 0.9 {
        lucide::circle_check().size(14).color(SUCCESS).into()
    } else if mapped_ratio >= 0.5 {
        lucide::circle_alert().size(14).color(WARNING).into()
    } else {
        lucide::circle().size(14).color(GRAY_400).into()
    };

    let code_string = code.to_string();
    let checkbox_widget = checkbox(is_selected)
        .on_toggle(move |_| Message::Export(ExportMessage::DomainToggled(code_string.clone())));

    let row_info = text(format!("{} rows", row_count)).size(12).color(GRAY_500);

    let main_row = row![
        checkbox_widget,
        Space::new().width(SPACING_XS),
        text(code).color(GRAY_800),
        Space::new().width(SPACING_SM),
        status_widget,
        Space::new().width(Length::Fill),
        row_info,
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_XS, 0.0]);

    // Check if domain has SUPP columns
    let has_supp = domain_has_supp(domain);

    if has_supp && is_selected {
        let supp_count = domain_supp_count(domain);

        // Show SUPP row below main domain (e.g., "SUPPDM")
        let supp_row = row![
            Space::new().width(28.0), // Indent to align with domain name
            lucide::corner_down_right().size(12).color(GRAY_400),
            Space::new().width(SPACING_XS),
            text(format!("SUPP{}", code)).size(12).color(PRIMARY_500),
            Space::new().width(SPACING_SM),
            text(format!(
                "({} qualifier{})",
                supp_count,
                if supp_count == 1 { "" } else { "s" }
            ))
            .size(11)
            .color(GRAY_500),
        ]
        .align_y(Alignment::Center)
        .padding([2.0, 0.0]);

        container(column![main_row, supp_row].spacing(2.0))
            .style(|_theme| container::Style {
                background: Some(WHITE.into()),
                ..Default::default()
            })
            .into()
    } else {
        container(main_row)
            .style(|_theme| container::Style {
                background: Some(WHITE.into()),
                ..Default::default()
            })
            .into()
    }
}

// =============================================================================
// RIGHT PANEL: EXPORT CONFIGURATION
// =============================================================================

/// Export configuration panel.
fn view_export_config<'a>(
    state: &'a AppState,
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let title = row![
        lucide::settings().size(16).color(GRAY_700),
        Space::new().width(SPACING_XS),
        text("Export Settings").size(16).color(GRAY_800),
    ]
    .align_y(Alignment::Center);

    // Output directory
    let output_section = view_output_directory(export_state, study);

    // Format selection
    let format_section = view_format_selection(state);

    // XPT version (only if XPT selected)
    let xpt_section: Element<'a, Message> =
        if state.settings.export.default_format == ExportFormat::Xpt {
            view_xpt_version(state)
        } else {
            Space::new().into()
        };

    // Define-XML option
    let define_section = view_define_xml_option(state);

    // Export button
    let export_button = view_export_button(export_state);

    column![
        title,
        Space::new().height(SPACING_MD),
        output_section,
        Space::new().height(SPACING_LG),
        format_section,
        Space::new().height(SPACING_MD),
        xpt_section,
        Space::new().height(SPACING_MD),
        define_section,
        Space::new().height(SPACING_LG),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        export_button,
    ]
    .into()
}

/// Output directory section.
fn view_output_directory<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let output_dir = export_state.effective_output_dir(&study.study_folder);

    let label = text("Output Directory").size(14).color(GRAY_700);

    let path_display = container(
        text(output_dir.display().to_string())
            .size(12)
            .color(GRAY_600),
    )
    .padding([SPACING_XS, SPACING_SM])
    .style(|_theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let change_btn = button(text("Change").size(12))
        .on_press(Message::Export(ExportMessage::OutputDirChangeClicked))
        .padding([SPACING_XS, SPACING_SM]);

    column![
        label,
        Space::new().height(SPACING_XS),
        row![path_display, Space::new().width(SPACING_SM), change_btn,].align_y(Alignment::Center),
    ]
    .into()
}

/// Format selection section.
fn view_format_selection(state: &AppState) -> Element<Message> {
    let label = text("Data Format").size(14).color(GRAY_700);

    let current_format = state.settings.export.default_format;

    let format_options = column![
        radio(
            ExportFormat::Xpt.label(),
            ExportFormat::Xpt,
            Some(current_format),
            |f| Message::Export(ExportMessage::FormatChanged(f))
        ),
        text(ExportFormat::Xpt.description())
            .size(11)
            .color(GRAY_500),
        Space::new().height(SPACING_SM),
        radio(
            ExportFormat::DatasetXml.label(),
            ExportFormat::DatasetXml,
            Some(current_format),
            |f| Message::Export(ExportMessage::FormatChanged(f))
        ),
        text(ExportFormat::DatasetXml.description())
            .size(11)
            .color(GRAY_500),
    ]
    .spacing(SPACING_XS);

    column![label, Space::new().height(SPACING_XS), format_options,].into()
}

/// XPT version selection.
fn view_xpt_version(state: &AppState) -> Element<Message> {
    let label = text("XPT Version").size(14).color(GRAY_700);

    let current_version = state.settings.export.xpt_version;

    let version_options = column![
        radio(
            XptVersion::V8.display_name(),
            XptVersion::V8,
            Some(current_version),
            |v| Message::Export(ExportMessage::XptVersionChanged(v))
        ),
        text("Recommended for modern submissions")
            .size(11)
            .color(GRAY_500),
        Space::new().height(SPACING_SM),
        radio(
            XptVersion::V5.display_name(),
            XptVersion::V5,
            Some(current_version),
            |v| Message::Export(ExportMessage::XptVersionChanged(v))
        ),
        text("Maximum compatibility with older software")
            .size(11)
            .color(GRAY_500),
    ]
    .spacing(SPACING_XS);

    column![label, Space::new().height(SPACING_XS), version_options,].into()
}

/// Define-XML info section (always generated).
fn view_define_xml_option(_state: &AppState) -> Element<Message> {
    // Define-XML is always generated - it's required for FDA submissions
    row![
        lucide::circle_check().size(14).color(SUCCESS),
        Space::new().width(SPACING_XS),
        column![
            text("Define-XML will be generated")
                .size(14)
                .color(GRAY_700),
            text("Required metadata for FDA regulatory submissions")
                .size(11)
                .color(GRAY_500),
        ]
        .spacing(2.0),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Export button.
fn view_export_button(export_state: &ExportViewState) -> Element<'_, Message> {
    let count = export_state.selection_count();
    let can_export = export_state.can_export();

    let button_label = if count == 0 {
        String::from("Select domains to export")
    } else {
        format!(
            "Export {} Domain{}",
            count,
            if count == 1 { "" } else { "s" }
        )
    };

    let btn_content = row![
        text(button_label).size(16),
        Space::new().width(SPACING_SM),
        lucide::arrow_right().size(16),
    ]
    .align_y(Alignment::Center);

    let mut btn = button(btn_content).padding([SPACING_SM, SPACING_LG]);

    if can_export {
        btn = btn
            .on_press(Message::Export(ExportMessage::StartExport))
            .style(button_primary);
    }

    btn.into()
}
