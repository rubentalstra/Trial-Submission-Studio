//! Settings dialog view.
//!
//! Master-detail layout with category sidebar and settings content.

use iced::widget::{
    Space, button, column, container, pick_list, radio, row, rule, scrollable, slider, text,
    toggler,
};
use iced::window;
use iced::{Alignment, Border, Color, Element, Length};
use iced_fonts::lucide;

use crate::message::{
    DeveloperSettingsMessage, DialogMessage, DisplaySettingsMessage, ExportSettingsMessage,
    GeneralSettingsMessage, Message, SettingsCategory, SettingsMessage, UpdateSettingsMessage,
};
use crate::state::{ExportFormat, Settings, XptVersion};
use crate::theme::{
    BORDER_RADIUS_LG, GRAY_100, GRAY_500, GRAY_700, GRAY_800, GRAY_900, PRIMARY_100, SPACING_LG,
    SPACING_MD, SPACING_SM, SPACING_XL, SPACING_XS, WHITE, button_primary,
};

/// Width of the category sidebar.
const SIDEBAR_WIDTH: f32 = 200.0;

/// Render the Settings dialog.
pub fn view_settings_dialog(
    settings: &Settings,
    active_category: SettingsCategory,
) -> Element<Message> {
    let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
            ..Default::default()
        });

    let dialog_content = view_dialog_content(settings, active_category);

    let dialog = container(dialog_content)
        .width(720)
        .height(500)
        .style(|_| container::Style {
            background: Some(WHITE.into()),
            border: Border {
                radius: BORDER_RADIUS_LG.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

    let centered_dialog = container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Shrink)
        .center_y(Length::Shrink);

    iced::widget::stack![backdrop, centered_dialog].into()
}

/// Render the Settings dialog content for a standalone window (multi-window mode).
///
/// This is the content that appears in a separate dialog window.
pub fn view_settings_dialog_content(
    settings: &Settings,
    active_category: SettingsCategory,
    window_id: window::Id,
) -> Element<Message> {
    let content = view_dialog_content_for_window(settings, active_category, window_id);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(GRAY_100.into()),
            ..Default::default()
        })
        .into()
}

/// Dialog content for window mode with window-specific close action.
fn view_dialog_content_for_window(
    settings: &Settings,
    active_category: SettingsCategory,
    window_id: window::Id,
) -> Element<Message> {
    let header = view_header();
    let body = view_master_detail(settings, active_category);
    let footer = view_footer_for_window(window_id);

    column![
        header,
        rule::horizontal(1),
        body,
        rule::horizontal(1),
        footer,
    ]
    .into()
}

/// Dialog footer for window mode (close window instead of dismiss dialog).
fn view_footer_for_window<'a>(window_id: window::Id) -> Element<'a, Message> {
    let reset_btn = button(text("Reset to Defaults").size(13))
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::ResetToDefaults,
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::CloseWindow(window_id))
        .padding([SPACING_SM, SPACING_MD]);

    let apply_btn = button(text("Apply & Close").size(13))
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::Apply,
        )))
        .padding([SPACING_SM, SPACING_XL])
        .style(button_primary);

    row![
        reset_btn,
        Space::new().width(Length::Fill),
        cancel_btn,
        Space::new().width(SPACING_SM),
        apply_btn,
    ]
    .padding([SPACING_MD, SPACING_LG])
    .align_y(Alignment::Center)
    .into()
}

/// Main dialog content with header, master-detail, and footer.
fn view_dialog_content(settings: &Settings, active_category: SettingsCategory) -> Element<Message> {
    let header = view_header();
    let body = view_master_detail(settings, active_category);
    let footer = view_footer();

    column![
        header,
        rule::horizontal(1),
        body,
        rule::horizontal(1),
        footer,
    ]
    .into()
}

/// Dialog header with title.
fn view_header<'a>() -> Element<'a, Message> {
    row![
        Space::new().width(SPACING_LG),
        lucide::settings().size(18).color(GRAY_700),
        Space::new().width(SPACING_SM),
        text("Settings").size(18).color(GRAY_900),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_MD, 0.0])
    .into()
}

/// Master-detail layout: category sidebar + settings content.
fn view_master_detail(settings: &Settings, active_category: SettingsCategory) -> Element<Message> {
    let sidebar = view_sidebar(active_category);
    let content = view_category_content(settings, active_category);

    row![
        container(sidebar).width(SIDEBAR_WIDTH),
        rule::vertical(1),
        container(scrollable(content).height(Length::Fill)).width(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}

/// Category sidebar with navigation items.
fn view_sidebar<'a>(active_category: SettingsCategory) -> Element<'a, Message> {
    column![
        view_sidebar_item(
            SettingsCategory::General,
            lucide::sliders_horizontal(),
            active_category == SettingsCategory::General
        ),
        view_sidebar_item(
            SettingsCategory::Export,
            lucide::file_output(),
            active_category == SettingsCategory::Export
        ),
        view_sidebar_item(
            SettingsCategory::Display,
            lucide::monitor(),
            active_category == SettingsCategory::Display
        ),
        view_sidebar_item(
            SettingsCategory::Updates,
            lucide::refresh_cw(),
            active_category == SettingsCategory::Updates
        ),
        view_sidebar_item(
            SettingsCategory::Developer,
            lucide::code(),
            active_category == SettingsCategory::Developer
        ),
    ]
    .spacing(SPACING_XS)
    .padding([SPACING_MD, SPACING_SM])
    .into()
}

/// Single sidebar navigation item.
fn view_sidebar_item<'a>(
    category: SettingsCategory,
    icon: impl Into<Element<'a, Message>>,
    is_active: bool,
) -> Element<'a, Message> {
    let text_color = if is_active { GRAY_900 } else { GRAY_700 };
    let bg_color = if is_active {
        PRIMARY_100
    } else {
        Color::TRANSPARENT
    };

    let content = row![
        icon.into(),
        Space::new().width(SPACING_SM),
        text(category.label()).size(14).color(text_color),
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_SM, SPACING_MD]);

    let item = container(content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    button(item)
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::CategorySelected(category),
        )))
        .padding(0)
        .style(|_, _| button::Style {
            background: None,
            ..Default::default()
        })
        .into()
}

/// Content area for the active category.
fn view_category_content<'a>(
    settings: &'a Settings,
    category: SettingsCategory,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match category {
        SettingsCategory::General => view_general_settings(settings),
        SettingsCategory::Export => view_export_settings(settings),
        SettingsCategory::Display => view_display_settings(),
        SettingsCategory::Updates => view_update_settings(settings),
        SettingsCategory::Developer => view_developer_settings(settings),
        SettingsCategory::Validation => view_validation_settings(),
    };

    container(content)
        .padding(SPACING_LG)
        .width(Length::Fill)
        .into()
}

/// General settings section.
fn view_general_settings(settings: &Settings) -> Element<Message> {
    let header_rows_section = column![
        text("CSV Header Rows").size(14).color(GRAY_800),
        text("Number of header rows in source CSV files")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_XS),
        row![
            radio("1 row", 1usize, Some(settings.general.header_rows), |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::General(
                    GeneralSettingsMessage::HeaderRowsChanged(v),
                )))
            }),
            Space::new().width(SPACING_MD),
            radio("2 rows", 2usize, Some(settings.general.header_rows), |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::General(
                    GeneralSettingsMessage::HeaderRowsChanged(v),
                )))
            }),
        ],
    ]
    .spacing(SPACING_XS);

    // Confidence threshold slider
    let threshold = settings.general.mapping_confidence_threshold;
    let threshold_section = column![
        text("Mapping Confidence Threshold")
            .size(14)
            .color(GRAY_800),
        text("Minimum confidence score for auto-mapping suggestions")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_XS),
        row![
            slider(0.0..=1.0, threshold, |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::General(
                    GeneralSettingsMessage::ConfidenceThresholdChanged(v),
                )))
            })
            .step(0.05)
            .width(Length::Fixed(200.0)),
            Space::new().width(SPACING_SM),
            text(format!("{:.0}%", threshold * 100.0))
                .size(14)
                .color(GRAY_700),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(SPACING_XS);

    column![
        section_header("General Settings"),
        Space::new().height(SPACING_MD),
        header_rows_section,
        Space::new().height(SPACING_LG),
        threshold_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Export settings section.
fn view_export_settings(settings: &Settings) -> Element<Message> {
    let format_section = column![
        text("Default Export Format").size(14).color(GRAY_800),
        text("Format used when exporting domain data")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_XS),
        radio(
            ExportFormat::Xpt.label(),
            ExportFormat::Xpt,
            Some(settings.export.default_format),
            |f| Message::Dialog(DialogMessage::Settings(SettingsMessage::Export(
                ExportSettingsMessage::DefaultFormatChanged(f),
            )))
        ),
        radio(
            ExportFormat::DatasetXml.label(),
            ExportFormat::DatasetXml,
            Some(settings.export.default_format),
            |f| Message::Dialog(DialogMessage::Settings(SettingsMessage::Export(
                ExportSettingsMessage::DefaultFormatChanged(f),
            )))
        ),
    ]
    .spacing(SPACING_XS);

    let xpt_version_section = column![
        text("XPT Version").size(14).color(GRAY_800),
        text("SAS Transport file version for XPT exports")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_XS),
        radio(
            XptVersion::V5.display_name(),
            XptVersion::V5,
            Some(settings.export.xpt_version),
            |v| Message::Dialog(DialogMessage::Settings(SettingsMessage::Export(
                ExportSettingsMessage::DefaultXptVersionChanged(v),
            )))
        ),
        radio(
            XptVersion::V8.display_name(),
            XptVersion::V8,
            Some(settings.export.xpt_version),
            |v| Message::Dialog(DialogMessage::Settings(SettingsMessage::Export(
                ExportSettingsMessage::DefaultXptVersionChanged(v),
            )))
        ),
    ]
    .spacing(SPACING_XS);

    column![
        section_header("Export Settings"),
        Space::new().height(SPACING_MD),
        format_section,
        Space::new().height(SPACING_LG),
        xpt_version_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Display settings section.
fn view_display_settings<'a>() -> Element<'a, Message> {
    let preview_rows_section = column![
        text("Preview Rows").size(14).color(GRAY_800),
        text("Number of rows to show in data preview")
            .size(12)
            .color(GRAY_500),
        Space::new().height(SPACING_XS),
        row![
            radio("25", 25usize, Some(50), |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                    DisplaySettingsMessage::PreviewRowsChanged(v),
                )))
            }),
            Space::new().width(SPACING_MD),
            radio("50", 50usize, Some(50), |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                    DisplaySettingsMessage::PreviewRowsChanged(v),
                )))
            }),
            Space::new().width(SPACING_MD),
            radio("100", 100usize, Some(50), |v| {
                Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                    DisplaySettingsMessage::PreviewRowsChanged(v),
                )))
            }),
        ],
    ]
    .spacing(SPACING_XS);

    column![
        section_header("Display Settings"),
        Space::new().height(SPACING_MD),
        preview_rows_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Update settings section.
fn view_update_settings(settings: &Settings) -> Element<Message> {
    let enable_section = row![
        column![
            text("Enable Update Checking").size(14).color(GRAY_800),
            text("Check for updates when the application starts")
                .size(12)
                .color(GRAY_500),
        ]
        .width(Length::Fill),
        toggler(settings.updates.enabled).on_toggle(|v| Message::Dialog(DialogMessage::Settings(
            SettingsMessage::Updates(UpdateSettingsMessage::EnabledToggled(v),)
        ))),
    ]
    .align_y(Alignment::Center);

    let channel_section = row![
        column![
            text("Update Channel").size(14).color(GRAY_800),
            text(settings.updates.channel.description())
                .size(12)
                .color(GRAY_500),
        ]
        .width(Length::Fill),
        pick_list(
            tss_updater::UpdateChannel::all(),
            Some(settings.updates.channel),
            |channel| Message::Dialog(DialogMessage::Settings(SettingsMessage::Updates(
                UpdateSettingsMessage::ChannelChanged(channel),
            ))),
        ),
    ]
    .align_y(Alignment::Center);

    column![
        section_header("Update Settings"),
        Space::new().height(SPACING_MD),
        enable_section,
        Space::new().height(SPACING_SM),
        channel_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Developer settings section.
fn view_developer_settings(settings: &Settings) -> Element<Message> {
    let bypass_validation_section = row![
        column![
            text("Bypass Validation Errors").size(14).color(GRAY_800),
            text("Allow export even with validation errors (use with caution)")
                .size(12)
                .color(GRAY_500),
        ]
        .width(Length::Fill),
        toggler(settings.developer.bypass_validation).on_toggle(|v| Message::Dialog(
            DialogMessage::Settings(SettingsMessage::Developer(
                DeveloperSettingsMessage::BypassValidationToggled(v),
            ))
        )),
    ]
    .align_y(Alignment::Center);

    let dev_mode_section = row![
        column![
            text("Developer Mode").size(14).color(GRAY_800),
            text("Enable additional debugging features")
                .size(12)
                .color(GRAY_500),
        ]
        .width(Length::Fill),
        toggler(settings.developer.developer_mode).on_toggle(|v| Message::Dialog(
            DialogMessage::Settings(SettingsMessage::Developer(
                DeveloperSettingsMessage::DeveloperModeToggled(v),
            ))
        )),
    ]
    .align_y(Alignment::Center);

    column![
        section_header("Developer Settings"),
        Space::new().height(SPACING_MD),
        bypass_validation_section,
        Space::new().height(SPACING_MD),
        dev_mode_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Validation settings section (placeholder).
fn view_validation_settings<'a>() -> Element<'a, Message> {
    column![
        section_header("Validation Settings"),
        Space::new().height(SPACING_MD),
        text("Validation rule configuration coming soon...")
            .size(13)
            .color(GRAY_500),
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Section header helper.
fn section_header(title: &str) -> Element<Message> {
    text(title).size(16).color(GRAY_900).into()
}

/// Dialog footer with action buttons.
fn view_footer<'a>() -> Element<'a, Message> {
    let reset_btn = button(text("Reset to Defaults").size(13))
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::ResetToDefaults,
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::Close,
        )))
        .padding([SPACING_SM, SPACING_MD]);

    let apply_btn = button(text("Apply").size(13))
        .on_press(Message::Dialog(DialogMessage::Settings(
            SettingsMessage::Apply,
        )))
        .padding([SPACING_SM, SPACING_XL])
        .style(button_primary);

    row![
        reset_btn,
        Space::new().width(Length::Fill),
        cancel_btn,
        Space::new().width(SPACING_SM),
        apply_btn,
    ]
    .padding([SPACING_MD, SPACING_LG])
    .align_y(Alignment::Center)
    .into()
}
