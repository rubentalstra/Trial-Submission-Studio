//! RELSPEC (Related Specimens) domain builder view.
//!
//! RELSPEC is a **Relationship** domain per SDTM-IG v3.4 Section 8.6.
//! It tracks specimen parent/child relationships and hierarchy levels.
//!
//! ## Required Variables
//! - USUBJID - Subject identifier
//! - REFID - Specimen reference identifier
//!
//! ## Optional Variables
//! - SPEC - Specimen type
//! - PARENT - Parent specimen REFID

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::inputs::form_field;
use crate::message::{GeneratedDomainMessage, Message, RelspecBuilderState, RelspecMessage};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, button_secondary,
};

use super::common::{entry_list_container, entry_list_empty, entry_row};
use super::{view_builder_footer, view_header};

// =============================================================================
// RELSPEC BUILDER VIEW
// =============================================================================

/// Render the RELSPEC (Related Specimens) builder.
pub fn view_relspec_builder(state: &RelspecBuilderState) -> Element<'_, Message> {
    let header = view_header("Create Related Specimens Domain", Some("RELSPEC"));

    let content = row![
        // Left panel: Entry form
        container(view_relspec_form(state))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(SPACING_LG),
        // Right panel: Entry list
        container(view_relspec_entries(&state.entries))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(SPACING_LG)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.clinical().background_secondary.into()),
                ..Default::default()
            }),
    ]
    .height(Length::Fill);

    let footer = view_builder_footer(state.entries.len(), !state.entries.is_empty());

    column![header, content, footer,]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// =============================================================================
// FORM
// =============================================================================

/// RELSPEC entry form.
fn view_relspec_form(state: &RelspecBuilderState) -> Element<'_, Message> {
    let usubjid_field = form_field(
        "Subject ID (USUBJID) *",
        &state.usubjid,
        "Enter subject identifier...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relspec(
                RelspecMessage::UsubjidChanged(s),
            ))
        },
        None,
    );

    let refid_field = form_field(
        "Specimen Reference ID (REFID) *",
        &state.refid,
        "Unique specimen identifier...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relspec(
                RelspecMessage::RefidChanged(s),
            ))
        },
        None,
    );

    let spec_field = form_field(
        "Specimen Type (SPEC)",
        &state.spec,
        "e.g., BLOOD, URINE, TISSUE",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relspec(
                RelspecMessage::SpecChanged(if s.is_empty() { None } else { Some(s) }),
            ))
        },
        None,
    );

    let parent_field = form_field(
        "Parent Specimen (PARENT)",
        &state.parent,
        "REFID of parent specimen (if derived)",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relspec(
                RelspecMessage::ParentChanged(if s.is_empty() { None } else { Some(s) }),
            ))
        },
        None,
    );

    // Hierarchy notice
    let hierarchy_notice = container(
        row![
            container(lucide::info().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().primary.base.color),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("LEVEL will be auto-calculated: 1 for collected samples, parent LEVEL + 1 for derived")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.clinical().text_secondary),
                }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([SPACING_SM, SPACING_MD])
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().primary.weak.color.into()),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Can add if required fields are filled
    let can_add = !state.usubjid.trim().is_empty() && !state.refid.trim().is_empty();

    let add_btn =
        button(
            row![
                container(lucide::plus().size(14)).style(move |theme: &Theme| container::Style {
                    text_color: Some(if can_add {
                        theme.extended_palette().primary.base.color
                    } else {
                        theme.clinical().text_muted
                    }),
                    ..Default::default()
                }),
                Space::new().width(SPACING_SM),
                text("Add Specimen").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press_maybe(can_add.then_some(Message::GeneratedDomain(
            GeneratedDomainMessage::Relspec(RelspecMessage::AddEntry),
        )))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    scrollable(
        column![
            usubjid_field,
            Space::new().height(SPACING_MD),
            refid_field,
            Space::new().height(SPACING_MD),
            spec_field,
            Space::new().height(SPACING_MD),
            parent_field,
            Space::new().height(SPACING_XL),
            hierarchy_notice,
            Space::new().height(SPACING_XL),
            add_btn,
        ]
        .width(Length::Fill),
    )
    .into()
}

// =============================================================================
// ENTRY LIST
// =============================================================================

/// RELSPEC entries list.
fn view_relspec_entries(entries: &[crate::state::RelspecEntry]) -> Element<'_, Message> {
    if entries.is_empty() {
        return entry_list_container(
            "Specimen Entries",
            0,
            entry_list_empty(
                "No specimens added yet. Fill out the form and click \"Add Specimen\".",
            ),
        );
    }

    let entry_elements: Vec<Element<'_, Message>> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let primary = format!("{}: {}", entry.usubjid, entry.refid);
            let secondary = match (&entry.spec, &entry.parent) {
                (Some(spec), Some(parent)) => format!("{} (derived from {})", spec, parent),
                (Some(spec), None) => format!("{} (collected)", spec),
                (None, Some(parent)) => format!("Derived from {}", parent),
                (None, None) => "Collected specimen".to_string(),
            };

            entry_row(primary, secondary, idx, |i| {
                Message::GeneratedDomain(GeneratedDomainMessage::Relspec(
                    RelspecMessage::RemoveEntry(i),
                ))
            })
        })
        .collect();

    entry_list_container(
        "Specimen Entries",
        entries.len(),
        column(entry_elements).spacing(1).into(),
    )
}
