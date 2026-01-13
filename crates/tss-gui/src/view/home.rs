//! Home view for Trial Submission Studio.
//!
//! The home screen displays:
//! - Title and tagline
//! - Workflow mode selector (SDTM/ADaM/SEND)
//! - Open study folder button
//! - Recent studies list
//! - Study overview (when loaded) with domain list

use iced::widget::{Space, button, center, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};

use crate::component::modal;
use crate::message::{HomeMessage, Message};
use crate::state::{AppState, Domain, Study, ViewState, WorkflowMode};
use crate::theme::{
    BORDER_RADIUS_MD, GRAY_100, GRAY_200, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900,
    PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, SUCCESS, WARNING, WHITE,
    button_primary, button_secondary,
};

// =============================================================================
// MAIN HOME VIEW
// =============================================================================

/// Render the home view.
///
/// Shows either the study selector (no study loaded) or the study overview
/// (study loaded with domain list).
pub fn view_home<'a>(state: &'a AppState) -> Element<'a, Message> {
    // Get workflow mode and close_confirm from view state
    let (workflow_mode, close_confirm) = match &state.view {
        ViewState::Home {
            workflow_mode,
            close_confirm,
        } => (*workflow_mode, *close_confirm),
        _ => (WorkflowMode::default(), false),
    };

    let content = if state.study.is_some() {
        view_study_loaded(state, workflow_mode)
    } else {
        view_no_study(state, workflow_mode)
    };

    // Wrap content in a centered container
    let main_content = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(SPACING_XL);

    // Check if we need to show the close study confirmation modal
    if close_confirm {
        view_close_study_modal(main_content.into())
    } else {
        main_content.into()
    }
}

// =============================================================================
// NO STUDY LOADED VIEW
// =============================================================================

/// View when no study is loaded - shows welcome message and study selector.
fn view_no_study<'a>(state: &'a AppState, workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let title = text("Trial Submission Studio").size(32).color(GRAY_900);

    let tagline = text("Transform clinical data to regulatory formats")
        .size(14)
        .color(GRAY_500);

    // Workflow mode selector card
    let selector_card = view_workflow_selector(workflow_mode);

    // Recent studies section
    let recent_studies = view_recent_studies(state);

    column![
        // Center the header
        container(
            column![title, Space::new().height(SPACING_SM), tagline,].align_x(Alignment::Center)
        )
        .width(Length::Fill)
        .center_x(Length::Shrink),
        Space::new().height(SPACING_XL),
        // Selector card centered
        container(selector_card)
            .width(Length::Fill)
            .center_x(Length::Shrink),
        Space::new().height(SPACING_XL),
        recent_studies,
    ]
    .spacing(0)
    .into()
}

/// Workflow mode selector card with dropdown and open button.
fn view_workflow_selector<'a>(current_mode: WorkflowMode) -> Element<'a, Message> {
    let header = text("Select a CDISC Standard").size(16).color(GRAY_800);

    // Mode buttons (simplified - SDTM enabled, others disabled)
    let mode_buttons = row![
        view_mode_button(WorkflowMode::Sdtm, current_mode, true),
        view_mode_button(WorkflowMode::Adam, current_mode, false),
        view_mode_button(WorkflowMode::Send, current_mode, false),
    ]
    .spacing(SPACING_SM);

    // Description for current mode
    let description = text(current_mode.description()).size(13).color(GRAY_500);

    // Open folder button
    let open_button = button(
        row![
            text("\u{f07c}") // folder-open icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(14),
            text("Open Study Folder").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::OpenStudyClicked))
    .padding([12.0, 24.0])
    .style(button_primary);

    let content = column![
        header,
        Space::new().height(SPACING_MD),
        mode_buttons,
        Space::new().height(SPACING_SM),
        description,
        Space::new().height(SPACING_LG),
        open_button,
    ]
    .align_x(Alignment::Center);

    // Card container
    container(content)
        .padding(SPACING_LG)
        .width(Length::Fixed(450.0))
        .style(|_theme| container::Style {
            background: Some(GRAY_100.into()),
            border: Border {
                radius: BORDER_RADIUS_MD.into(),
                color: GRAY_200,
                width: 1.0,
            },
            ..Default::default()
        })
        .into()
}

/// A single workflow mode button.
fn view_mode_button<'a>(
    mode: WorkflowMode,
    current: WorkflowMode,
    enabled: bool,
) -> Element<'a, Message> {
    let is_selected = mode == current;
    let label = if enabled {
        mode.display_name().to_string()
    } else {
        format!("{} (N/A)", mode.display_name())
    };

    let btn = button(text(label).size(13))
        .padding([8.0, 16.0])
        .style(move |theme, status| {
            if is_selected {
                // Selected style
                iced::widget::button::Style {
                    background: Some(PRIMARY_500.into()),
                    text_color: WHITE,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else if !enabled {
                // Disabled style
                iced::widget::button::Style {
                    background: None,
                    text_color: GRAY_500,
                    border: Border {
                        radius: 4.0.into(),
                        color: GRAY_200,
                        width: 1.0,
                    },
                    ..Default::default()
                }
            } else {
                // Normal button style
                button_secondary(theme, status)
            }
        });

    if enabled && !is_selected {
        btn.on_press(Message::SetWorkflowMode(mode)).into()
    } else {
        btn.into()
    }
}

/// Recent studies list section.
fn view_recent_studies<'a>(state: &'a AppState) -> Element<'a, Message> {
    let recent = &state.settings.general.recent_studies;

    if recent.is_empty() {
        return column![].into();
    }

    let header = text("Recent Studies").size(14).color(GRAY_700);

    let mut items = column![].spacing(SPACING_SM);
    for path in recent.iter().take(5) {
        let display = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        let path_clone = path.clone();
        let item = button(
            row![
                text("\u{f07b}") // folder icon
                    .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                    .size(12)
                    .color(GRAY_500),
                text(display).size(13).color(GRAY_700),
            ]
            .spacing(SPACING_SM)
            .align_y(Alignment::Center),
        )
        .on_press(Message::Home(HomeMessage::RecentStudyClicked(path_clone)))
        .padding([6.0, 12.0])
        .style(|_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
                _ => None,
            };
            iced::widget::button::Style {
                background: bg,
                text_color: GRAY_700,
                border: Border::default(),
                ..Default::default()
            }
        });

        items = items.push(item);
    }

    column![header, Space::new().height(SPACING_SM), items,].into()
}

// =============================================================================
// STUDY LOADED VIEW
// =============================================================================

/// View when a study is loaded - shows study info and domain list.
fn view_study_loaded<'a>(state: &'a AppState, workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let study = state.study.as_ref().unwrap();

    // Header with study name and close button
    let header = view_study_header(study, workflow_mode);

    // Domain list
    let domain_list = view_domain_list(study);

    // Export button
    let export_button = button(
        row![
            text("\u{f56e}") // file-export icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(14),
            text("Go to Export").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::GoToExportClicked))
    .padding([10.0, 20.0])
    .style(button_primary);

    column![
        header,
        Space::new().height(SPACING_LG),
        domain_list,
        Space::new().height(SPACING_LG),
        container(export_button)
            .width(Length::Fill)
            .center_x(Length::Shrink),
    ]
    .into()
}

/// Study header with name, mode badge, and close button.
fn view_study_header<'a>(study: &'a Study, mode: WorkflowMode) -> Element<'a, Message> {
    let study_name = text(&study.study_id).size(24).color(GRAY_900);

    // Mode badge
    let mode_badge = container(text(mode.display_name()).size(12).color(WHITE))
        .padding([4.0, 8.0])
        .style(move |_theme| container::Style {
            background: Some(PRIMARY_500.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Path info
    let path_text = text(study.study_folder.display().to_string())
        .size(12)
        .color(GRAY_500);

    // Close button
    let close_button = button(
        row![
            text("\u{f00d}") // times icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(12),
            text("Close Study").size(12),
        ]
        .spacing(4.0)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseStudyClicked))
    .padding([6.0, 12.0])
    .style(button_secondary);

    let header_row = row![
        study_name,
        Space::new().width(SPACING_SM),
        mode_badge,
        Space::new().width(Length::Fill),
        close_button,
    ]
    .align_y(Alignment::Center);

    column![header_row, Space::new().height(4.0), path_text,]
        .spacing(0)
        .into()
}

/// Domain list showing all discovered domains.
fn view_domain_list<'a>(study: &'a Study) -> Element<'a, Message> {
    let header = row![
        text("\u{f1c0}") // database icon
            .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
            .size(14)
            .color(GRAY_600),
        text("Discovered Domains").size(14).color(GRAY_700),
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    let domain_codes = study.domain_codes_dm_first();

    let mut domain_items = column![].spacing(SPACING_SM);

    for code in domain_codes {
        if let Some(domain) = study.domain(code) {
            let item = view_domain_item(code, domain);
            domain_items = domain_items.push(item);
        }
    }

    let scrollable_list = scrollable(domain_items)
        .height(Length::FillPortion(1))
        .width(Length::Fill);

    column![
        header,
        Space::new().height(SPACING_MD),
        container(scrollable_list)
            .width(Length::Fill)
            .height(Length::Fixed(400.0)),
    ]
    .into()
}

/// A single domain item in the list.
fn view_domain_item<'a>(code: &'a str, domain: &'a Domain) -> Element<'a, Message> {
    let display_name = domain.display_name(code);
    let row_count = domain.row_count();
    let is_complete = domain.is_mapping_complete();
    let is_touched = domain.is_touched();

    // Status icon
    let (icon_char, icon_color) = if is_complete {
        ('\u{f058}', SUCCESS) // check-circle
    } else if is_touched {
        ('\u{f303}', WARNING) // pencil-alt
    } else {
        ('\u{f111}', GRAY_500) // circle
    };

    let status_icon = text(icon_char.to_string())
        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
        .size(14)
        .color(icon_color);

    let name_text = text(display_name).size(14).color(GRAY_800);
    let rows_text = text(format!("{} rows", row_count)).size(12).color(GRAY_500);

    let code_owned = code.to_string();
    let item_button = button(
        row![
            status_icon,
            name_text,
            Space::new().width(Length::Fill),
            rows_text,
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center)
        .padding([SPACING_SM, SPACING_MD]),
    )
    .on_press(Message::Home(HomeMessage::DomainClicked(code_owned)))
    .width(Length::Fill)
    .style(|_theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => Some(GRAY_100.into()),
            _ => None,
        };
        iced::widget::button::Style {
            background: bg,
            text_color: GRAY_800,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    item_button.into()
}

// =============================================================================
// CLOSE STUDY MODAL
// =============================================================================

/// Close study confirmation modal.
fn view_close_study_modal<'a>(base: Element<'a, Message>) -> Element<'a, Message> {
    let warning_icon = text("\u{f071}") // exclamation-triangle
        .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
        .size(48)
        .color(WARNING);

    let title = text("Close Study?").size(18).color(GRAY_900);

    let message = text("All unsaved mapping progress will be lost.")
        .size(14)
        .color(GRAY_600);

    let cancel_button = button(text("Cancel").size(14))
        .on_press(Message::Home(HomeMessage::CloseStudyCancelled))
        .padding([10.0, 20.0])
        .style(button_secondary);

    let confirm_button = button(
        row![
            text("\u{f2ed}") // trash-alt icon
                .font(iced::Font::with_name("Font Awesome 6 Free Solid"))
                .size(12),
            text("Close Study").size(14),
        ]
        .spacing(SPACING_SM)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Home(HomeMessage::CloseStudyConfirmed))
    .padding([10.0, 20.0])
    .style(|_theme, _status| iced::widget::button::Style {
        background: Some(iced::Color::from_rgb(0.75, 0.22, 0.17).into()), // Red
        text_color: WHITE,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let buttons = row![cancel_button, confirm_button,]
        .spacing(SPACING_MD)
        .align_y(Alignment::Center);

    let content = column![
        warning_icon,
        Space::new().height(SPACING_MD),
        title,
        Space::new().height(SPACING_SM),
        message,
        Space::new().height(SPACING_LG),
        buttons,
    ]
    .align_x(Alignment::Center);

    modal(
        base,
        "Close Study",
        container(content)
            .padding(SPACING_MD)
            .center_x(Length::Shrink)
            .into(),
        Message::Home(HomeMessage::CloseStudyCancelled),
        vec![],
    )
}
