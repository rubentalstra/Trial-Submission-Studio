//! Export view for Trial Submission Studio.
//!
//! Master-detail layout with domain selection (left) and configuration (right).
//! Progress and completion dialogs are shown in separate windows.

use iced::widget::{
    Space, button, checkbox, column, container, radio, row, rule, scrollable, text,
};
use iced::{Alignment, Border, Element, Length};
use iced_fonts::lucide;

use crate::component::{
    DetailHeader, EmptyState, MetadataCard, PageHeader, master_detail_with_pinned_header,
};
use crate::message::{ExportMessage, Message};
use crate::service::export::{domain_has_supp, domain_supp_count};
use crate::state::{
    AppState, DomainState, ExportFormat, ExportViewState, Study, ViewState, XptVersion,
};
use crate::theme::{
    GRAY_100, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, MASTER_WIDTH, PRIMARY_500,
    SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, SUCCESS, WARNING, WHITE, button_primary,
    button_secondary,
};

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

    // Use standardized master-detail layout
    view_export_layout(state, export_state, study)
}

// =============================================================================
// LAYOUT
// =============================================================================

/// View when no study is loaded.
fn view_no_study<'a>() -> Element<'a, Message> {
    EmptyState::new(
        lucide::folder_open().size(48).color(GRAY_500),
        "No Study Loaded",
    )
    .description("Open a study folder to export domains")
    .action("Go Back", Message::Navigate(ViewState::home()))
    .centered()
    .view()
}

/// Main export layout using standardized master-detail pattern.
fn view_export_layout<'a>(
    state: &'a AppState,
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    // Page header with back button and metadata
    let header = view_page_header(export_state, study);

    // Master panel header (pinned)
    let master_header = view_master_header(export_state, study);

    // Master panel content (scrollable domain list)
    let master_content = view_domain_list(export_state, study);

    // Detail panel (export settings)
    let detail = view_export_settings(state, export_state, study);

    // Combine header with master-detail layout
    let main_content =
        master_detail_with_pinned_header(master_header, master_content, detail, MASTER_WIDTH);

    column![header, Space::new().height(SPACING_SM), main_content,]
        .height(Length::Fill)
        .into()
}

/// Page header with back button and export metadata.
fn view_page_header<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let total = study.domain_codes().len();
    let selected = export_state.selection_count();

    PageHeader::new("Export Domains")
        .back(Message::Navigate(ViewState::home()))
        .meta("Selected", format!("{}/{}", selected, total))
        .view()
}

// =============================================================================
// MASTER PANEL
// =============================================================================

/// Master panel header with selection controls and stats.
fn view_master_header<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    // Selection controls
    let select_all_btn = button(text("Select All").size(12))
        .on_press(Message::Export(ExportMessage::SelectAll))
        .padding([SPACING_XS, SPACING_SM])
        .style(button_secondary);

    let deselect_btn = button(text("Clear").size(12))
        .on_press(Message::Export(ExportMessage::DeselectAll))
        .padding([SPACING_XS, SPACING_SM])
        .style(button_secondary);

    let controls = row![select_all_btn, Space::new().width(SPACING_XS), deselect_btn,]
        .align_y(Alignment::Center);

    // Stats
    let total = study.domain_codes().len();
    let selected = export_state.selection_count();
    let stats = row![
        text(format!("{}/{}", selected, total))
            .size(12)
            .color(GRAY_600),
        Space::new().width(4.0),
        text("selected").size(11).color(GRAY_500),
    ]
    .align_y(Alignment::Center);

    column![
        controls,
        Space::new().height(SPACING_SM),
        stats,
        Space::new().height(SPACING_SM),
        rule::horizontal(1),
        Space::new().height(SPACING_SM),
    ]
    .into()
}

/// Master panel content: scrollable domain list.
fn view_domain_list<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let domain_codes = study.domain_codes();

    if domain_codes.is_empty() {
        return column![text("No domains found").size(13).color(GRAY_500),].into();
    }

    let domain_items: Vec<Element<'a, Message>> = domain_codes
        .iter()
        .filter_map(|code| {
            let domain = study.domain(code)?;
            Some(view_domain_row(code, domain, export_state))
        })
        .collect();

    column(domain_items).spacing(SPACING_XS).into()
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

    let status_icon: Element<'a, Message> = if mapped_ratio >= 0.9 {
        lucide::circle_check().size(14).color(SUCCESS).into()
    } else if mapped_ratio >= 0.5 {
        lucide::circle_alert().size(14).color(WARNING).into()
    } else {
        lucide::circle().size(14).color(GRAY_400).into()
    };

    let code_string = code.to_string();
    let checkbox_widget = checkbox(is_selected)
        .on_toggle(move |_| Message::Export(ExportMessage::DomainToggled(code_string.clone())));

    let row_info = text(format!("{} rows", row_count)).size(11).color(GRAY_500);

    let main_row = row![
        checkbox_widget,
        Space::new().width(SPACING_XS),
        status_icon,
        Space::new().width(SPACING_XS),
        text(code).size(13).color(GRAY_800),
        Space::new().width(Length::Fill),
        row_info,
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_XS, SPACING_SM]);

    // Check if domain has SUPP columns
    let has_supp = domain_has_supp(domain);

    if has_supp && is_selected {
        let supp_count = domain_supp_count(domain);

        // Show SUPP row below main domain
        let supp_row = row![
            Space::new().width(32.0), // Indent to align
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
        .padding([2.0, SPACING_SM]);

        container(column![main_row, supp_row].spacing(0.0))
            .style(|_| container::Style {
                background: Some(WHITE.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    } else {
        container(main_row)
            .style(|_| container::Style {
                background: Some(WHITE.into()),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }
}

// =============================================================================
// DETAIL PANEL: EXPORT SETTINGS
// =============================================================================

/// Detail panel with export configuration.
fn view_export_settings<'a>(
    state: &'a AppState,
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    // Header
    let header = DetailHeader::new("Export Settings")
        .subtitle("Configure output format and destination")
        .view();

    // Output directory section
    let output_section = view_output_directory(export_state, study);

    // Format selection section
    let format_section = view_format_selection(state);

    // XPT version (only if XPT selected)
    let xpt_section: Element<'a, Message> =
        if state.settings.export.default_format == ExportFormat::Xpt {
            view_xpt_version(state)
        } else {
            Space::new().height(0.0).into()
        };

    // Define-XML info
    let define_section = view_define_xml_info();

    // Export button
    let export_button = view_export_button(export_state);

    // Consistent layout matching other detail panels
    scrollable(column![
        header,
        Space::new().height(SPACING_MD),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        output_section,
        Space::new().height(SPACING_LG),
        format_section,
        Space::new().height(SPACING_LG),
        xpt_section,
        define_section,
        Space::new().height(SPACING_LG),
        rule::horizontal(1),
        Space::new().height(SPACING_MD),
        export_button,
        Space::new().height(SPACING_MD),
    ])
    .height(Length::Fill)
    .into()
}

/// Output directory section.
fn view_output_directory<'a>(
    export_state: &'a ExportViewState,
    study: &'a Study,
) -> Element<'a, Message> {
    let output_dir = export_state.effective_output_dir(&study.study_folder);

    // Section title
    let title = row![
        lucide::folder().size(14).color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("Output Directory").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

    let path_display = container(
        text(output_dir.display().to_string())
            .size(12)
            .color(GRAY_600),
    )
    .padding([SPACING_SM, SPACING_MD])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let change_btn = button(
        row![
            lucide::folder_open().size(12),
            Space::new().width(SPACING_XS),
            text("Change").size(12),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Export(ExportMessage::OutputDirChangeClicked))
    .padding([SPACING_XS, SPACING_SM])
    .style(button_secondary);

    column![
        title,
        Space::new().height(SPACING_XS),
        path_display,
        Space::new().height(SPACING_XS),
        change_btn,
    ]
    .into()
}

/// Format selection section using MetadataCard style.
fn view_format_selection(state: &AppState) -> Element<'_, Message> {
    let current_format = state.settings.export.default_format;

    // Section title
    let title = row![
        lucide::file_output().size(14).color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("Data Format").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

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

    column![title, Space::new().height(SPACING_SM), format_options,].into()
}

/// XPT version selection.
fn view_xpt_version(state: &AppState) -> Element<'_, Message> {
    let current_version = state.settings.export.xpt_version;

    // Section title
    let title = row![
        lucide::settings().size(14).color(GRAY_600),
        Space::new().width(SPACING_SM),
        text("XPT Version").size(14).color(GRAY_700),
    ]
    .align_y(Alignment::Center);

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

    column![
        title,
        Space::new().height(SPACING_SM),
        version_options,
        Space::new().height(SPACING_LG),
    ]
    .into()
}

/// Define-XML info section (always generated).
fn view_define_xml_info<'a>() -> Element<'a, Message> {
    MetadataCard::new()
        .row("Define-XML", "Will be generated automatically")
        .row("Purpose", "Required for FDA regulatory submissions")
        .view()
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
        lucide::download().size(16),
        Space::new().width(SPACING_SM),
        text(button_label).size(14),
    ]
    .align_y(Alignment::Center);

    let mut btn = button(btn_content)
        .padding([SPACING_SM, SPACING_LG])
        .width(Length::Fill);

    if can_export {
        btn = btn
            .on_press(Message::Export(ExportMessage::StartExport))
            .style(button_primary);
    }

    container(btn)
        .width(Length::Fill)
        .center_x(Length::Shrink)
        .into()
}
