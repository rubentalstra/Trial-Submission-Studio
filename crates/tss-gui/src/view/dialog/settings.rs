//! Settings dialog view.
//!
//! Master-detail layout with category sidebar and settings content.

use iced::widget::{
    Space, button, column, container, pick_list, radio, row, rule, scrollable, slider, text,
    toggler,
};
use iced::window;
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{
    DeveloperSettingsMessage, DialogMessage, DisplaySettingsMessage, ExportSettingsMessage,
    GeneralSettingsMessage, Message, SettingsCategory, SettingsMessage, UpdateSettingsMessage,
    ValidationSettingsMessage,
};
use crate::state::{AssignmentMode, ExportFormat, Settings, XptVersion};
use crate::theme::{
    AccessibilityMode, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, SPACING_XS,
    ThemeConfig, ThemeMode, button_primary,
};

/// Width of the category sidebar.
const SIDEBAR_WIDTH: f32 = 200.0;

/// Render the Settings dialog content for a standalone window (multi-window mode).
///
/// This is the content that appears in a separate dialog window.
pub fn view_settings_dialog_content<'a>(
    settings: &'a Settings,
    active_category: SettingsCategory,
    window_id: window::Id,
) -> Element<'a, Message> {
    let content = view_dialog_content_for_window(settings, active_category, window_id);

    // Wrap in a styled container for the window
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            ..Default::default()
        })
        .into()
}

/// Dialog content for window mode with window-specific close action.
fn view_dialog_content_for_window<'a>(
    settings: &'a Settings,
    active_category: SettingsCategory,
    window_id: window::Id,
) -> Element<'a, Message> {
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

/// Dialog header with title.
fn view_header<'a>() -> Element<'a, Message> {
    row![
        Space::new().width(SPACING_LG),
        container(lucide::settings().size(18)).style(|theme: &Theme| container::Style {
            text_color: Some(theme.clinical().text_secondary),
            ..Default::default()
        }),
        Space::new().width(SPACING_SM),
        text("Settings")
            .size(18)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .padding([SPACING_MD, 0.0])
    .into()
}

/// Master-detail layout: category sidebar + settings content.
fn view_master_detail<'a>(
    settings: &'a Settings,
    active_category: SettingsCategory,
) -> Element<'a, Message> {
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
            lucide::sliders_horizontal().size(16),
            active_category == SettingsCategory::General,
        ),
        view_sidebar_item(
            SettingsCategory::Export,
            lucide::file_output().size(16),
            active_category == SettingsCategory::Export,
        ),
        view_sidebar_item(
            SettingsCategory::Display,
            lucide::monitor().size(16),
            active_category == SettingsCategory::Display,
        ),
        view_sidebar_item(
            SettingsCategory::Updates,
            lucide::refresh_cw().size(16),
            active_category == SettingsCategory::Updates,
        ),
        view_sidebar_item(
            SettingsCategory::Developer,
            lucide::code().size(16),
            active_category == SettingsCategory::Developer,
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
    let icon_container = container(icon.into()).style(move |theme: &Theme| container::Style {
        text_color: Some(if is_active {
            theme.extended_palette().background.base.text
        } else {
            theme.clinical().text_secondary
        }),
        ..Default::default()
    });

    let label_text = text(category.label())
        .size(14)
        .style(move |theme: &Theme| text::Style {
            color: Some(if is_active {
                theme.extended_palette().background.base.text
            } else {
                theme.clinical().text_secondary
            }),
        });

    let content = row![icon_container, Space::new().width(SPACING_SM), label_text,]
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_MD]);

    let item = container(content)
        .width(Length::Fill)
        .style(move |theme: &Theme| container::Style {
            background: Some(if is_active {
                theme.clinical().accent_primary_light.into()
            } else {
                Color::TRANSPARENT.into()
            }),
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
        SettingsCategory::Display => view_display_settings(settings),
        SettingsCategory::Updates => view_update_settings(settings),
        SettingsCategory::Developer => view_developer_settings(settings),
        SettingsCategory::Validation => view_validation_settings(settings),
    };

    container(content)
        .padding(SPACING_LG)
        .width(Length::Fill)
        .into()
}

/// General settings section.
fn view_general_settings(settings: &Settings) -> Element<'_, Message> {
    let header_rows_section = column![
        text("CSV Header Rows")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Number of header rows in source CSV files")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
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
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Minimum confidence score for auto-mapping suggestions")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
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
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(SPACING_XS);

    // Assignment mode section
    let assignment_mode_section = column![
        text("Assignment Mode")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("How to assign source files to domains")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XS),
        row![
            // TODO: use the label here from the enum right??
            radio(
                "Drag and Drop",
                AssignmentMode::DragAndDrop,
                Some(settings.general.assignment_mode),
                |v| Message::Dialog(DialogMessage::Settings(SettingsMessage::General(
                    GeneralSettingsMessage::AssignmentModeChanged(v),
                )))
            ),
            Space::new().width(SPACING_MD),
            radio(
                "Click to Assign",
                AssignmentMode::ClickToAssign,
                Some(settings.general.assignment_mode),
                |v| Message::Dialog(DialogMessage::Settings(SettingsMessage::General(
                    GeneralSettingsMessage::AssignmentModeChanged(v),
                )))
            ),
        ],
    ]
    .spacing(SPACING_XS);

    column![
        section_header("General Settings"),
        Space::new().height(SPACING_MD),
        header_rows_section,
        Space::new().height(SPACING_LG),
        threshold_section,
        Space::new().height(SPACING_LG),
        assignment_mode_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Export settings section.
fn view_export_settings(settings: &Settings) -> Element<'_, Message> {
    let format_section = column![
        text("Default Export Format")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Format used when exporting domain data")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
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
        text("XPT Version")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("SAS Transport file version for XPT exports")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
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

    let sdtm_ig_section = column![
        text("SDTM-IG Version")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Implementation Guide version for Dataset-XML and Define-XML")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XS),
        pick_list(
            crate::state::SdtmIgVersion::ALL.to_vec(),
            Some(settings.export.sdtm_ig_version),
            |v| Message::Dialog(DialogMessage::Settings(SettingsMessage::Export(
                ExportSettingsMessage::SdtmIgVersionChanged(v),
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
        Space::new().height(SPACING_LG),
        sdtm_ig_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Display settings section.
fn view_display_settings(settings: &Settings) -> Element<'_, Message> {
    // Theme mode section
    let theme_mode_section = column![
        text("Appearance")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Choose light or dark mode, or follow system preference")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XS),
        pick_list(
            ThemeMode::ALL.to_vec(),
            Some(settings.display.theme_mode),
            |mode| Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                DisplaySettingsMessage::ThemeModeChanged(mode),
            )))
        )
        .width(Length::Fixed(200.0)),
    ]
    .spacing(SPACING_XS);

    // Accessibility mode section
    let accessibility_section = column![
        text("Color Vision Accessibility")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Optimize colors for different types of color vision")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XS),
        pick_list(
            AccessibilityMode::ALL.to_vec(),
            Some(settings.display.accessibility_mode),
            |mode| Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                DisplaySettingsMessage::AccessibilityModeChanged(mode),
            )))
        )
        .width(Length::Fixed(250.0)),
        Space::new().height(SPACING_SM),
        // Live color preview
        view_theme_preview(
            settings.display.theme_mode,
            settings.display.accessibility_mode,
        ),
    ]
    .spacing(SPACING_XS);

    // Preview rows section
    let preview_rows_section = column![
        text("Preview Rows")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        text("Number of rows to show in data preview")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_XS),
        row![
            radio(
                "25",
                25usize,
                Some(settings.display.preview_rows_per_page),
                |v| {
                    Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                        DisplaySettingsMessage::PreviewRowsChanged(v),
                    )))
                }
            ),
            Space::new().width(SPACING_MD),
            radio(
                "50",
                50usize,
                Some(settings.display.preview_rows_per_page),
                |v| {
                    Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                        DisplaySettingsMessage::PreviewRowsChanged(v),
                    )))
                }
            ),
            Space::new().width(SPACING_MD),
            radio(
                "100",
                100usize,
                Some(settings.display.preview_rows_per_page),
                |v| {
                    Message::Dialog(DialogMessage::Settings(SettingsMessage::Display(
                        DisplaySettingsMessage::PreviewRowsChanged(v),
                    )))
                }
            ),
        ],
    ]
    .spacing(SPACING_XS);

    column![
        section_header("Display Settings"),
        Space::new().height(SPACING_MD),
        theme_mode_section,
        Space::new().height(SPACING_LG),
        accessibility_section,
        Space::new().height(SPACING_LG),
        preview_rows_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Live preview showing the status colors for the selected theme/accessibility combination.
fn view_theme_preview(
    theme_mode: ThemeMode,
    accessibility_mode: AccessibilityMode,
) -> Element<'static, Message> {
    // TODO: is this really intentional, because it already updates on change of the settings and everything so why do we need to create a new theme here?
    // Create a temporary Theme to resolve preview colors
    // This is intentional - we want to show preview colors for the selected settings,
    // not the currently active theme
    let config = ThemeConfig::new(theme_mode, accessibility_mode);
    let preview_theme = config.to_theme(false);
    let preview_palette = preview_theme.extended_palette();
    let preview_clinical = preview_theme.clinical();

    let preview_bg = preview_clinical.background_elevated;
    let text_color = preview_clinical.text_on_accent;

    let success_color = preview_palette.success.base.color;
    let warning_color = preview_palette.warning.base.color;
    let error_color = preview_palette.danger.base.color;

    let success_box = container(
        row![
            lucide::circle_check().size(12).color(text_color),
            Space::new().width(4),
            text("Success").size(11).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(move |_| container::Style {
        background: Some(success_color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let warning_box = container(
        row![
            lucide::triangle_alert().size(12).color(text_color),
            Space::new().width(4),
            text("Warning").size(11).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(move |_| container::Style {
        background: Some(warning_color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let error_box = container(
        row![
            lucide::circle_x().size(12).color(text_color),
            Space::new().width(4),
            text("Error").size(11).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding([4, 8])
    .style(move |_| container::Style {
        background: Some(error_color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    container(
        row![
            success_box,
            Space::new().width(8),
            warning_box,
            Space::new().width(8),
            error_box,
        ]
        .padding(SPACING_SM),
    )
    .style(move |_| container::Style {
        background: Some(preview_bg.into()),
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Update settings section.
fn view_update_settings(settings: &Settings) -> Element<'_, Message> {
    let check_on_startup_section = row![
        column![
            text("Check on Startup")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text("Automatically check for updates when the application starts")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .width(Length::Fill),
        toggler(settings.updates.check_on_startup).on_toggle(|v| Message::Dialog(
            DialogMessage::Settings(SettingsMessage::Updates(
                UpdateSettingsMessage::CheckOnStartupToggled(v),
            ))
        )),
    ]
    .align_y(Alignment::Center);

    let channel_description = settings.updates.channel.description();
    let channel_section = row![
        column![
            text("Update Channel")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text(channel_description)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
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
        check_on_startup_section,
        Space::new().height(SPACING_SM),
        channel_section,
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Developer settings section.
fn view_developer_settings(settings: &Settings) -> Element<'_, Message> {
    let bypass_validation_section = row![
        column![
            text("Bypass Validation Errors")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text("Allow export even with validation errors (use with caution)")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
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
            text("Developer Mode")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text("Enable additional debugging features")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
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

/// Validation settings section.
fn view_validation_settings(settings: &Settings) -> Element<'_, Message> {
    let rules = &settings.validation.rules;

    let strict_mode = row![
        column![
            text("Strict Mode")
                .size(14)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.base.text),
                }),
            text("Treat warnings as errors")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .width(Length::Fill),
        toggler(settings.validation.strict_mode).on_toggle(|v| {
            Message::Dialog(DialogMessage::Settings(SettingsMessage::Validation(
                ValidationSettingsMessage::StrictModeToggled(v),
            )))
        }),
    ]
    .align_y(Alignment::Center);

    column![
        section_header("Validation Settings"),
        Space::new().height(SPACING_MD),
        strict_mode,
        Space::new().height(SPACING_LG),
        text("Validation Rules")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().height(SPACING_SM),
        view_rule_toggle(
            "Required Variables",
            "Check that required variables are present",
            rules.check_required_variables,
            "check_required_variables"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Expected Variables",
            "Check that expected variables are present",
            rules.check_expected_variables,
            "check_expected_variables"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Data Types",
            "Check data types match expected types",
            rules.check_data_types,
            "check_data_types"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "ISO 8601 Format",
            "Check date/time format compliance",
            rules.check_iso8601_format,
            "check_iso8601_format"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Sequence Uniqueness",
            "Check sequence number uniqueness",
            rules.check_sequence_uniqueness,
            "check_sequence_uniqueness"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Text Length",
            "Check text length against CDISC limits",
            rules.check_text_length,
            "check_text_length"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Identifier Nulls",
            "Check identifier nulls (STUDYID, USUBJID)",
            rules.check_identifier_nulls,
            "check_identifier_nulls"
        ),
        Space::new().height(SPACING_XS),
        view_rule_toggle(
            "Controlled Terminology",
            "Check controlled terminology values",
            rules.check_controlled_terminology,
            "check_controlled_terminology"
        ),
    ]
    .spacing(SPACING_SM)
    .into()
}

/// Helper to create rule toggle rows.
fn view_rule_toggle(
    label: &'static str,
    description: &'static str,
    enabled: bool,
    rule_id: &'static str,
) -> Element<'static, Message> {
    row![
        column![
            text(label).size(14).style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
            text(description)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_muted),
                }),
        ]
        .width(Length::Fill),
        toggler(enabled).on_toggle(move |v| {
            Message::Dialog(DialogMessage::Settings(SettingsMessage::Validation(
                ValidationSettingsMessage::RuleToggled {
                    rule_id: rule_id.to_string(),
                    enabled: v,
                },
            )))
        }),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Section header helper.
fn section_header(title: &str) -> Element<'_, Message> {
    text(title)
        .size(16)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        })
        .into()
}
