//! Source assignment view for Trial Submission Studio.
//!
//! Two-panel drag-and-drop interface for manually assigning CSV files
//! to CDISC domains.

use iced::widget::{
    Space, button, column, container, mouse_area, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_aw::ContextMenu;
use iced_fonts::lucide;

use crate::message::{Message, SourceAssignmentMessage};
use crate::state::{
    AppState, AssignmentMode, SourceAssignmentUiState, SourceFileEntry, SourceFileStatus,
    TargetDomainEntry, ViewState, WorkflowMode,
};
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
    button_primary, button_secondary,
};

/// Render the source assignment view.
pub fn view_source_assignment(state: &AppState) -> Element<'_, Message> {
    // Extract assignment UI state
    let (workflow_mode, assignment_ui) = match &state.view {
        ViewState::SourceAssignment {
            workflow_mode,
            assignment_ui,
        } => (*workflow_mode, assignment_ui),
        _ => return text("Invalid view state").into(),
    };

    let assignment_mode = state.settings.general.assignment_mode;

    // Header with back button and title
    let header = view_header(workflow_mode);

    // Main content: two-panel layout
    let content = row![
        // Source panel (left)
        view_source_panel(assignment_ui, assignment_mode),
        // Domain panel (right)
        view_domain_panel(assignment_ui, workflow_mode, assignment_mode),
    ]
    .spacing(SPACING_MD)
    .height(Length::Fill);

    // Footer with progress and continue button
    let footer = view_footer(assignment_ui);

    // Full page layout
    let page = column![header, content, footer,]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    // Add loading overlay if creating study
    if assignment_ui.is_creating_study {
        // TODO: Add loading overlay
        container(page)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        container(page)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Render the header with back button and title.
fn view_header<'a>(workflow_mode: WorkflowMode) -> Element<'a, Message> {
    let back_btn = button(
        row![
            container(lucide::arrow_left().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_secondary),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Back").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::SourceAssignment(
        SourceAssignmentMessage::BackClicked,
    ))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_secondary);

    let title = text("Assign Source Files to Domains")
        .size(20)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        });

    let mode_badge = container(text(workflow_mode.display_name()).size(12).style(
        |theme: &Theme| text::Style {
            color: Some(theme.clinical().text_on_accent),
        },
    ))
    .padding([4.0, 10.0])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().primary.base.color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let header_row = row![
        back_btn,
        Space::new().width(SPACING_LG),
        title,
        Space::new().width(SPACING_SM),
        mode_badge,
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACING_SM)
    .align_y(Alignment::Center);

    container(header_row)
        .width(Length::Fill)
        .padding([SPACING_MD, SPACING_LG])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                width: 0.0,
                radius: 0.0.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Render the source files panel (left side).
fn view_source_panel<'a>(
    assignment_ui: &'a SourceAssignmentUiState,
    assignment_mode: AssignmentMode,
) -> Element<'a, Message> {
    // Panel header with folder path
    let folder_path = assignment_ui.folder.display().to_string();
    let truncated_path = truncate_path(&folder_path, 40);

    let header_content = column![
        // Folder path
        row![
            container(lucide::folder().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text(truncated_path)
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
        ]
        .align_y(Alignment::Center),
        Space::new().height(SPACING_SM),
        // Search input
        text_input("Search files...", &assignment_ui.source_search)
            .on_input(
                |s| Message::SourceAssignment(SourceAssignmentMessage::SourceSearchChanged(s))
            )
            .padding(SPACING_SM)
            .width(Length::Fill),
    ];

    let panel_header = container(header_content)
        .width(Length::Fill)
        .padding(SPACING_MD)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                width: 0.0,
                radius: 0.0.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        });

    // File list content
    let filtered_files = assignment_ui.filtered_source_files();
    let marked_files = assignment_ui.marked_files();

    let file_list: Element<'a, Message> = if filtered_files.is_empty() && marked_files.is_empty() {
        // Empty state
        container(
            column![
                container(lucide::circle_check().size(32)).style(|theme: &Theme| {
                    container::Style {
                        text_color: Some(theme.clinical().mapping_mapped),
                        ..Default::default()
                    }
                }),
                Space::new().height(SPACING_SM),
                text("All files categorized")
                    .size(14)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_secondary),
                    }),
                text("Ready to continue")
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.clinical().text_muted),
                    }),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
    } else {
        let mut items = column![].spacing(SPACING_XS);

        // Show available (unassigned, unmarked) files
        for (index, file) in filtered_files {
            items = items.push(view_source_file_item(
                file,
                index,
                assignment_ui,
                assignment_mode,
            ));
        }

        // Show marked files (metadata/skipped)
        if !marked_files.is_empty() {
            items = items.push(Space::new().height(SPACING_SM));
            items = items.push(
                row![
                    rule::horizontal(1).style(|theme: &Theme| rule::Style {
                        color: theme.clinical().border_default,
                        radius: 0.0.into(),
                        fill_mode: rule::FillMode::Full,
                        snap: true,
                    }),
                    Space::new().width(SPACING_SM),
                    text("Marked Files")
                        .size(11)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.clinical().text_muted),
                        }),
                    Space::new().width(SPACING_SM),
                    rule::horizontal(1).style(|theme: &Theme| rule::Style {
                        color: theme.clinical().border_default,
                        radius: 0.0.into(),
                        fill_mode: rule::FillMode::Full,
                        snap: true,
                    }),
                ]
                .align_y(Alignment::Center),
            );
            items = items.push(Space::new().height(SPACING_SM));

            for (index, file) in marked_files {
                items = items.push(view_marked_file_item(file, index));
            }
        }

        scrollable(container(items).padding(SPACING_MD))
            .height(Length::Fill)
            .into()
    };

    // Combined panel
    container(column![panel_header, file_list,])
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Render a single source file item.
///
/// Unassigned files show NO icon - just filename (clean list).
/// Actions via right-click context menu, not inline buttons.
fn view_source_file_item<'a>(
    file: &'a SourceFileEntry,
    index: usize,
    assignment_ui: &'a SourceAssignmentUiState,
    assignment_mode: AssignmentMode,
) -> Element<'a, Message> {
    let is_selected = assignment_ui.selected_file == Some(index);
    let is_dragging = assignment_ui.dragging_file == Some(index);

    let file_name = text(&file.file_stem)
        .size(13)
        .style(move |theme: &Theme| text::Style {
            color: Some(if is_selected || is_dragging {
                theme.extended_palette().primary.base.color
            } else {
                theme.extended_palette().background.base.text
            }),
        });

    // Clean file row: icon space (empty for unassigned) + filename only
    let content = row![
        Space::new().width(20.0), // Icon space (reserved for alignment, empty for unassigned)
        file_name,
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(SPACING_XS);

    // Style for file item container
    let item_style = move |theme: &Theme| container::Style {
        background: if is_selected {
            Some(theme.extended_palette().primary.weak.color.into())
        } else if is_dragging {
            Some(theme.clinical().background_inset.into())
        } else {
            None
        },
        border: Border {
            color: if is_selected {
                theme.extended_palette().primary.base.color
            } else {
                iced::Color::TRANSPARENT
            },
            width: if is_selected { 1.0 } else { 0.0 },
            radius: BORDER_RADIUS_SM.into(),
        },
        ..Default::default()
    };

    let item_container = container(content)
        .width(Length::Fill)
        .padding([SPACING_SM, SPACING_MD])
        .style(item_style);

    // Build context menu content for right-click
    let context_menu_underlay: Element<'a, Message> = match assignment_mode {
        AssignmentMode::DragAndDrop => {
            // Drag-and-drop mode: wrap in mouse_area for drag handling
            mouse_area(item_container)
                .on_press(Message::SourceAssignment(
                    SourceAssignmentMessage::DragStarted { file_index: index },
                ))
                .into()
        }
        AssignmentMode::ClickToAssign => {
            // Click-to-assign mode: wrap in button for click handling
            button(item_container)
                .on_press(Message::SourceAssignment(
                    SourceAssignmentMessage::FileClicked { file_index: index },
                ))
                .padding(0)
                .style(|_, _| iced::widget::button::Style {
                    background: None,
                    border: Border::default(),
                    ..Default::default()
                })
                .into()
        }
    };

    // Wrap in ContextMenu for right-click actions
    ContextMenu::new(context_menu_underlay, move || {
        build_file_context_menu(index)
    })
    .into()
}

/// Build context menu options for an unassigned file.
fn build_file_context_menu(file_index: usize) -> Element<'static, Message> {
    context_menu_container(column![
        context_menu_button(
            lucide::file_text().size(12),
            "Mark as Metadata",
            Message::SourceAssignment(SourceAssignmentMessage::MarkAsMetadata { file_index }),
            |theme: &Theme| theme.clinical().mapping_suggested,
        ),
        context_menu_button(
            lucide::file_x().size(12),
            "Mark as Skipped",
            Message::SourceAssignment(SourceAssignmentMessage::MarkAsSkipped { file_index }),
            |theme: &Theme| theme.clinical().text_muted,
        ),
    ])
}

/// Wrapper container for context menu content.
fn context_menu_container<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(4)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Create a context menu button with icon.
fn context_menu_button<'a>(
    icon: impl Into<Element<'a, Message>>,
    label: &'a str,
    message: Message,
    icon_color_fn: impl Fn(&Theme) -> iced::Color + 'a,
) -> Element<'a, Message> {
    button(
        row![
            container(icon).style(move |theme: &Theme| container::Style {
                text_color: Some(icon_color_fn(theme)),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text(label).size(12).style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().background.base.text),
            }),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(message)
    .padding([SPACING_XS, SPACING_SM])
    .style(context_menu_button_style)
    .into()
}

/// Render a marked file item (metadata or skipped).
///
/// Metadata files show orange `file_text` icon as status indicator.
/// Skipped files show greyed `file_x` icon as status indicator.
/// Unmark action via right-click context menu, not inline button.
fn view_marked_file_item<'a>(file: &'a SourceFileEntry, index: usize) -> Element<'a, Message> {
    let is_metadata = file.status == SourceFileStatus::Metadata;
    let is_skipped = file.status == SourceFileStatus::Skipped;

    // Status icon (orange for metadata, greyed for skipped)
    let icon: Element<'a, Message> = if is_metadata {
        container(lucide::file_text().size(14))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().mapping_suggested),
                ..Default::default()
            })
            .into()
    } else if is_skipped {
        container(lucide::file_x().size(14))
            .style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_muted),
                ..Default::default()
            })
            .into()
    } else {
        Space::new().width(14.0).into()
    };

    let file_name = text(&file.file_stem)
        .size(13)
        .style(move |theme: &Theme| text::Style {
            color: Some(if is_metadata {
                theme.clinical().mapping_suggested
            } else if is_skipped {
                theme.clinical().text_muted
            } else {
                theme.clinical().text_secondary
            }),
        });

    // Clean content: icon + filename only (no inline buttons)
    let content = row![
        icon,
        Space::new().width(SPACING_SM),
        file_name,
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(SPACING_XS);

    let item_container = container(content)
        .width(Length::Fill)
        .padding([SPACING_SM, SPACING_MD])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_inset.into()),
            border: Border {
                radius: BORDER_RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Wrap in ContextMenu for right-click "Unmark" action
    ContextMenu::new(item_container, move || {
        build_marked_file_context_menu(index)
    })
    .into()
}

/// Build context menu options for a marked file (metadata/skipped).
fn build_marked_file_context_menu(file_index: usize) -> Element<'static, Message> {
    context_menu_container(column![context_menu_button(
        lucide::rotate_ccw().size(12),
        "Unmark",
        Message::SourceAssignment(SourceAssignmentMessage::UnmarkFile { file_index }),
        |theme: &Theme| theme.clinical().text_secondary,
    ),])
}

/// Render the domain panel (right side).
fn view_domain_panel<'a>(
    assignment_ui: &'a SourceAssignmentUiState,
    workflow_mode: WorkflowMode,
    assignment_mode: AssignmentMode,
) -> Element<'a, Message> {
    // Panel header with standard info
    let standard_label = match workflow_mode {
        WorkflowMode::Sdtm => "SDTM v3.4 Domains",
        WorkflowMode::Adam => "ADaM v1.3 Datasets",
        WorkflowMode::Send => "SEND v3.1.1 Domains",
    };

    let header_content = column![
        text(standard_label)
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().height(SPACING_SM),
        text_input("Search domains...", &assignment_ui.domain_search)
            .on_input(
                |s| Message::SourceAssignment(SourceAssignmentMessage::DomainSearchChanged(s))
            )
            .padding(SPACING_SM)
            .width(Length::Fill),
    ];

    let panel_header = container(header_content)
        .width(Length::Fill)
        .padding(SPACING_MD)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                width: 0.0,
                radius: 0.0.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        });

    // Domain list grouped by class
    let mut domain_content = column![].spacing(SPACING_MD);

    // Get filtered domains (search applied)
    let filtered_domains = assignment_ui.filtered_domains();

    // Group by class for display
    for (class, domain_codes) in &assignment_ui.domains_by_class {
        // Get domains in this class that pass the filter
        let class_domains: Vec<_> = domain_codes
            .iter()
            .filter_map(|code| filtered_domains.iter().find(|d| &d.code == code).copied())
            .collect();

        if class_domains.is_empty() {
            continue;
        }

        // Class header
        domain_content =
            domain_content.push(text(class).size(11).style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }));

        // Domain items
        for domain in class_domains {
            domain_content =
                domain_content.push(view_domain_bucket(domain, assignment_ui, assignment_mode));
        }
    }

    let domain_list =
        scrollable(container(domain_content).padding(SPACING_MD)).height(Length::Fill);

    // Combined panel
    container(column![panel_header, domain_list,])
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                width: 1.0,
                radius: BORDER_RADIUS_SM.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Render a domain bucket with assigned files.
///
/// In drag-and-drop mode: uses mouse_area for hover/drop detection.
/// In click-to-assign mode: uses button for click handling.
fn view_domain_bucket<'a>(
    domain: &'a TargetDomainEntry,
    assignment_ui: &'a SourceAssignmentUiState,
    assignment_mode: AssignmentMode,
) -> Element<'a, Message> {
    let has_files = !assignment_ui.files_for_domain(&domain.code).is_empty();
    let is_hover_target = assignment_ui.hover_domain.as_ref() == Some(&domain.code);
    let has_selected_file = assignment_ui.selected_file.is_some();

    // Domain header
    let domain_header = row![
        text(&domain.code)
            .size(14)
            .style(move |theme: &Theme| text::Style {
                color: Some(if has_files {
                    theme.extended_palette().background.base.text
                } else {
                    theme.clinical().text_muted
                }),
            }),
        Space::new().width(SPACING_SM),
        text(domain.display_name())
            .size(12)
            .style(move |theme: &Theme| text::Style {
                color: Some(if has_files {
                    theme.clinical().text_secondary
                } else {
                    theme.clinical().text_muted
                }),
            }),
    ]
    .align_y(Alignment::Center);

    // Assigned files list
    let mut content = column![domain_header].spacing(SPACING_XS);

    for (file_idx, file) in assignment_ui
        .source_files
        .iter()
        .enumerate()
        .filter(|(_, f)| f.assigned_domain.as_ref() == Some(&domain.code))
    {
        content = content.push(view_assigned_file(&file.file_stem, file_idx, &domain.code));
    }

    let bucket = container(content)
        .width(Length::Fill)
        .padding(SPACING_MD)
        .style(move |theme: &Theme| container::Style {
            background: Some(if is_hover_target {
                theme.extended_palette().primary.weak.color.into()
            } else if has_files {
                theme.clinical().background_elevated.into()
            } else {
                theme.clinical().background_inset.into()
            }),
            border: Border {
                width: if is_hover_target || (has_selected_file && !has_files) {
                    2.0
                } else {
                    1.0
                },
                radius: BORDER_RADIUS_SM.into(),
                color: if is_hover_target {
                    theme.extended_palette().primary.base.color
                } else if has_selected_file && !has_files {
                    theme.extended_palette().primary.weak.color
                } else {
                    theme.clinical().border_default
                },
            },
            ..Default::default()
        });

    // Capture domain code and dragging file for closures
    let domain_code = domain.code.clone();
    let domain_code_for_click = domain.code.clone();
    let domain_code_for_drop = domain.code.clone();
    let dragging_file_index = assignment_ui.dragging_file;

    match assignment_mode {
        AssignmentMode::DragAndDrop => {
            // Drag-and-drop mode: use mouse_area for hover/drop detection
            let mut area = mouse_area(bucket)
                .on_enter(Message::SourceAssignment(
                    SourceAssignmentMessage::DragOverDomain {
                        domain_code: Some(domain_code),
                    },
                ))
                .on_exit(Message::SourceAssignment(
                    SourceAssignmentMessage::DragOverDomain { domain_code: None },
                ));

            // Only handle release if we're dragging a file
            if let Some(file_index) = dragging_file_index {
                area = area.on_release(Message::SourceAssignment(
                    SourceAssignmentMessage::DroppedOnDomain {
                        file_index,
                        domain_code: domain_code_for_drop,
                    },
                ));
            }

            area.into()
        }
        AssignmentMode::ClickToAssign => {
            // Click-to-assign mode: use button for click handling
            if has_selected_file {
                button(bucket)
                    .on_press(Message::SourceAssignment(
                        SourceAssignmentMessage::DomainClicked {
                            domain_code: domain_code_for_click,
                        },
                    ))
                    .padding(0)
                    .style(|_, _| iced::widget::button::Style {
                        background: None,
                        border: Border::default(),
                        ..Default::default()
                    })
                    .into()
            } else {
                bucket.into()
            }
        }
    }
}

/// Render an assigned file under a domain.
///
/// Shows just the filename (no icon) - location under domain makes status clear.
/// Unassign action via right-click context menu, not inline button.
fn view_assigned_file<'a>(
    file_stem: &'a str,
    file_index: usize,
    domain_code: &'a str,
) -> Element<'a, Message> {
    // Clean content: indent + filename only (no inline buttons)
    let content = row![
        Space::new().width(SPACING_MD), // Indent
        text(format!("{}.csv", file_stem))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_secondary),
            }),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let item_container = container(content)
        .width(Length::Fill)
        .padding([SPACING_XS, 0.0]);

    // Capture domain_code as owned String for the closure
    let domain_code_owned = domain_code.to_string();

    // Wrap in ContextMenu for right-click "Unassign" action
    ContextMenu::new(item_container, move || {
        build_assigned_file_context_menu(file_index, domain_code_owned.clone())
    })
    .into()
}

/// Build context menu options for an assigned file.
fn build_assigned_file_context_menu(
    file_index: usize,
    domain_code: String,
) -> Element<'static, Message> {
    context_menu_container(column![context_menu_button(
        lucide::x().size(12),
        "Unassign",
        Message::SourceAssignment(SourceAssignmentMessage::UnassignFile {
            domain_code,
            file_index,
        }),
        |theme: &Theme| theme.clinical().text_muted,
    ),])
}

/// Render the footer with progress and continue button.
fn view_footer<'a>(assignment_ui: &'a SourceAssignmentUiState) -> Element<'a, Message> {
    let assigned = assignment_ui.assigned_count();
    let metadata = assignment_ui.metadata_count();
    let skipped = assignment_ui.skipped_count();
    let remaining = assignment_ui.remaining_count();

    let can_continue = remaining == 0;

    // Progress summary
    let summary = row![
        text(format!("{} assigned", assigned))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().mapping_mapped),
            }),
        text(" \u{00B7} ")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        text(format!("{} metadata", metadata))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().mapping_suggested),
            }),
        text(" \u{00B7} ")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        text(format!("{} skipped", skipped))
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        text(" \u{00B7} ")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        text(format!("{} remaining", remaining))
            .size(12)
            .style(move |theme: &Theme| text::Style {
                color: Some(if remaining > 0 {
                    theme.clinical().mapping_suggested
                } else {
                    theme.clinical().mapping_mapped
                }),
            }),
    ]
    .align_y(Alignment::Center);

    // Continue button
    let continue_btn = button(
        row![
            text("Continue").size(14),
            Space::new().width(SPACING_SM),
            container(lucide::arrow_right().size(14)).style(move |theme: &Theme| {
                container::Style {
                    text_color: Some(if can_continue {
                        theme.clinical().text_on_accent
                    } else {
                        theme.clinical().text_muted
                    }),
                    ..Default::default()
                }
            }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([SPACING_SM, SPACING_LG])
    .style(if can_continue {
        button_primary
    } else {
        button_disabled
    });

    let continue_btn = if can_continue {
        continue_btn.on_press(Message::SourceAssignment(
            SourceAssignmentMessage::ContinueClicked,
        ))
    } else {
        continue_btn
    };

    let footer_row =
        row![summary, Space::new().width(Length::Fill), continue_btn,].align_y(Alignment::Center);

    container(footer_row)
        .width(Length::Fill)
        .padding([SPACING_MD, SPACING_LG])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_secondary.into()),
            border: Border {
                width: 1.0,
                color: theme.clinical().border_default,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Truncate a path for display.
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    format!("...{}", &path[path.len().saturating_sub(max_len - 3)..])
}

/// Disabled button style.
fn button_disabled(
    theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(theme.clinical().background_inset.into()),
        text_color: theme.clinical().text_muted,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Context menu item button style (hover highlight).
fn context_menu_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
    iced::widget::button::Style {
        background: if is_hovered {
            Some(theme.extended_palette().primary.weak.color.into())
        } else {
            None
        },
        text_color: theme.extended_palette().background.base.text,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
