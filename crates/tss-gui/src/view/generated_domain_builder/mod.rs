//! Generated domain builder view.
//!
//! Entry-point view for building generated domains (CO, RELREC, RELSPEC, RELSUB).
//!
//! ## Module Structure
//!
//! - `mod.rs` - Main dispatcher with domain type selection
//! - `co.rs` - CO (Comments) builder - **Special-Purpose**
//! - `relrec.rs` - RELREC (Related Records) builder - **Relationship**
//! - `relspec.rs` - RELSPEC (Related Specimens) builder - **Relationship**
//! - `relsub.rs` - RELSUB (Related Subjects) builder - **Relationship**
//! - `common.rs` - Shared UI components

pub mod co;
pub mod common;
pub mod relrec;
pub mod relspec;
pub mod relsub;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::message::{GeneratedDomainBuilderState, GeneratedDomainMessage, Message};
use crate::state::GeneratedDomainType;
use crate::theme::{
    BORDER_RADIUS_SM, ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, SPACING_XS,
    button_primary, button_secondary,
};

pub use co::view_co_builder;
pub use relrec::view_relrec_builder;
pub use relspec::view_relspec_builder;
pub use relsub::view_relsub_builder;

// =============================================================================
// MAIN BUILDER VIEW
// =============================================================================

/// Render the generated domain builder view.
///
/// Shows domain type selection if no type selected, otherwise shows the
/// appropriate builder for the selected domain type.
pub fn view_generated_domain_builder(
    builder: &GeneratedDomainBuilderState,
) -> Element<'_, Message> {
    match builder.selected_type {
        None => view_domain_type_selection(),
        Some(GeneratedDomainType::Comments) => view_co_builder(&builder.co),
        Some(GeneratedDomainType::RelatedRecords) => view_relrec_builder(&builder.relrec),
        Some(GeneratedDomainType::RelatedSpecimens) => view_relspec_builder(&builder.relspec),
        Some(GeneratedDomainType::RelatedSubjects) => view_relsub_builder(&builder.relsub),
    }
}

// =============================================================================
// DOMAIN TYPE SELECTION
// =============================================================================

/// Domain type selection screen.
fn view_domain_type_selection<'a>() -> Element<'a, Message> {
    let header = view_header("Create Generated Domain", None);

    let intro = text(
        "Generated domains are created by the application rather than mapped from source data. \
         Select the type of domain you want to create.",
    )
    .size(14)
    .style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_secondary),
    });

    // Special-Purpose Domains section
    let special_purpose_section = column![
        text("Special-Purpose Domains")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_SM),
        view_domain_type_card(
            GeneratedDomainType::Comments,
            "CO",
            "Comments",
            "Free-text comments linked to records in other domains",
            container(lucide::message_square().size(24)),
        ),
    ]
    .spacing(0);

    // Relationship Domains section
    let relationship_section = column![
        text("Relationship Domains")
            .size(14)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.clinical().text_muted),
            }),
        Space::new().height(SPACING_SM),
        row![
            view_domain_type_card(
                GeneratedDomainType::RelatedRecords,
                "RELREC",
                "Related Records",
                "Links records across or within domains",
                container(lucide::link().size(24)),
            ),
            Space::new().width(SPACING_MD),
            view_domain_type_card(
                GeneratedDomainType::RelatedSpecimens,
                "RELSPEC",
                "Related Specimens",
                "Tracks specimen parent/child hierarchy",
                container(lucide::git_branch().size(24)),
            ),
        ]
        .spacing(0),
        Space::new().height(SPACING_MD),
        view_domain_type_card(
            GeneratedDomainType::RelatedSubjects,
            "RELSUB",
            "Related Subjects",
            "Subject-to-subject relationships (e.g., mother/child)",
            container(lucide::users().size(24)),
        ),
    ]
    .spacing(0);

    let content = column![
        intro,
        Space::new().height(SPACING_XL),
        special_purpose_section,
        Space::new().height(SPACING_XL),
        relationship_section,
    ]
    .padding(SPACING_LG);

    let footer = view_selection_footer();

    column![header, content, Space::new().height(Length::Fill), footer,]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Domain type selection card.
fn view_domain_type_card<'a>(
    domain_type: GeneratedDomainType,
    code: &'a str,
    name: &'a str,
    description: &'a str,
    icon: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let icon_container = container(icon).style(|theme: &Theme| container::Style {
        text_color: Some(theme.extended_palette().primary.base.color),
        ..Default::default()
    });

    let badge = container(text(code).size(11).style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_on_accent),
    }))
    .padding([SPACING_XS / 2.0, SPACING_XS])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().primary.base.color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let title_row = row![
        text(name).size(16).style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.base.text),
        }),
        Space::new().width(SPACING_SM),
        badge,
    ]
    .align_y(Alignment::Center);

    let desc = text(description)
        .size(13)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let card_content = row![
        icon_container,
        Space::new().width(SPACING_MD),
        column![title_row, Space::new().height(SPACING_XS), desc,].width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .padding(SPACING_MD);

    button(card_content)
        .on_press(Message::GeneratedDomain(
            GeneratedDomainMessage::SelectDomainType(domain_type),
        ))
        .padding(0)
        .width(Length::FillPortion(1))
        .style(|theme: &Theme, status| {
            let clinical = theme.clinical();
            let base = button::Style {
                background: Some(clinical.background_secondary.into()),
                text_color: theme.extended_palette().background.base.text,
                border: Border {
                    width: 1.0,
                    radius: BORDER_RADIUS_SM.into(),
                    color: clinical.border_default,
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered | button::Status::Pressed => button::Style {
                    background: Some(clinical.background_elevated.into()),
                    border: Border {
                        color: theme.extended_palette().primary.base.color,
                        ..base.border
                    },
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

// =============================================================================
// SHARED HEADER / FOOTER
// =============================================================================

/// Header for builder views.
pub fn view_header<'a>(title: &'a str, domain_badge: Option<&'a str>) -> Element<'a, Message> {
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
    .on_press(Message::GeneratedDomain(GeneratedDomainMessage::Cancel))
    .padding([SPACING_SM, SPACING_MD])
    .style(button_secondary);

    let title_text = text(title).size(20).style(|theme: &Theme| text::Style {
        color: Some(theme.extended_palette().background.base.text),
    });

    let badge = domain_badge.map(|code| {
        container(text(code).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_on_accent),
        }))
        .padding([SPACING_XS / 2.0, SPACING_SM])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().primary.base.color.into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    });

    let mut header_row =
        row![back_btn, Space::new().width(SPACING_MD), title_text,].align_y(Alignment::Center);

    if let Some(b) = badge {
        header_row = header_row.push(Space::new().width(SPACING_SM)).push(b);
    }

    container(header_row.push(Space::new().width(Length::Fill)))
        .padding([SPACING_MD, SPACING_LG])
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.clinical().background_elevated.into()),
            border: Border {
                width: 1.0,
                radius: 0.0.into(),
                color: theme.clinical().border_default,
            },
            ..Default::default()
        })
        .into()
}

/// Footer for domain type selection.
fn view_selection_footer<'a>() -> Element<'a, Message> {
    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::GeneratedDomain(GeneratedDomainMessage::Cancel))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    container(
        row![Space::new().width(Length::Fill), cancel_btn,]
            .padding([SPACING_MD, SPACING_LG])
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_elevated.into()),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: theme.clinical().border_default,
        },
        ..Default::default()
    })
    .into()
}

/// Footer for builder views with create button.
pub fn view_builder_footer<'a>(entry_count: usize, can_create: bool) -> Element<'a, Message> {
    let cancel_btn = button(text("Cancel").size(13))
        .on_press(Message::GeneratedDomain(GeneratedDomainMessage::Cancel))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    let entry_label = if entry_count == 1 {
        format!("{} entry", entry_count)
    } else {
        format!("{} entries", entry_count)
    };

    let count_text = text(entry_label)
        .size(13)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.clinical().text_secondary),
        });

    let create_btn = button(
        row![
            text("Create Domain").size(13),
            Space::new().width(SPACING_SM),
            container(lucide::check().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.clinical().text_on_accent),
                ..Default::default()
            }),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(can_create.then_some(Message::GeneratedDomain(
        GeneratedDomainMessage::CreateDomain,
    )))
    .padding([SPACING_SM, SPACING_XL])
    .style(button_primary);

    container(
        row![
            cancel_btn,
            Space::new().width(Length::Fill),
            count_text,
            Space::new().width(SPACING_MD),
            create_btn,
        ]
        .padding([SPACING_MD, SPACING_LG])
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.clinical().background_elevated.into()),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: theme.clinical().border_default,
        },
        ..Default::default()
    })
    .into()
}
