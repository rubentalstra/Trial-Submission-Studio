//! RELSUB (Related Subjects) domain builder view.
//!
//! RELSUB is a **Relationship** domain per SDTM-IG v3.4 Section 8.7.
//! It tracks subject-to-subject relationships (e.g., mother/child, twins).
//!
//! **IMPORTANT**: Per SDTM-IG, RELSUB relationships MUST be bidirectional.
//! When A→B is defined, B→A must also exist with the reciprocal relationship.
//! The generation service handles this automatically.
//!
//! ## Required Variables
//! - USUBJID - Subject identifier
//! - RSUBJID - Related subject identifier
//! - SREL - Subject relationship (from RELSUB codelist)

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::inputs::form_field;
use crate::message::{GeneratedDomainMessage, Message, RelsubBuilderState, RelsubMessage};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, button_secondary,
};

use super::common::{entry_list_container, entry_list_empty, entry_row};
use super::{view_builder_footer, view_header};

// =============================================================================
// RELSUB BUILDER VIEW
// =============================================================================

/// Render the RELSUB (Related Subjects) builder.
pub fn view_relsub_builder(state: &RelsubBuilderState) -> Element<'_, Message> {
    let header = view_header("Create Related Subjects Domain", Some("RELSUB"));

    let content = row![
        // Left panel: Entry form
        container(view_relsub_form(state))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(SPACING_LG),
        // Right panel: Entry list
        container(view_relsub_entries(&state.entries))
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

/// RELSUB entry form.
fn view_relsub_form(state: &RelsubBuilderState) -> Element<'_, Message> {
    let usubjid_field = form_field(
        "Subject ID (USUBJID) *",
        &state.usubjid,
        "Enter subject identifier...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relsub(
                RelsubMessage::UsubjidChanged(s),
            ))
        },
        None,
    );

    let rsubjid_field = form_field(
        "Related Subject ID (RSUBJID) *",
        &state.rsubjid,
        "Enter related subject identifier...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relsub(
                RelsubMessage::RsubjidChanged(s),
            ))
        },
        None,
    );

    let srel_field = form_field(
        "Relationship (SREL) *",
        &state.srel,
        "e.g., MOTHER, BIOLOGICAL",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relsub(RelsubMessage::SrelChanged(
                s,
            )))
        },
        None,
    );

    // Bidirectional notice
    let bidirectional_notice = container(
        row![
            container(lucide::info().size(14)).style(|theme: &Theme| container::Style {
                text_color: Some(theme.extended_palette().primary.base.color),
                ..Default::default()
            }),
            Space::new().width(SPACING_SM),
            text("Reciprocal relationship will be auto-generated per SDTM-IG requirements")
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

    // Common relationship examples
    let examples = text(
        "Common values: MOTHER, BIOLOGICAL | FATHER, BIOLOGICAL | TWIN, MONOZYGOTIC | SIBLING",
    )
    .size(12)
    .style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    });

    // Can add if required fields are filled
    let can_add = !state.usubjid.trim().is_empty()
        && !state.rsubjid.trim().is_empty()
        && !state.srel.trim().is_empty();

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
                text("Add Relationship").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press_maybe(can_add.then_some(Message::GeneratedDomain(
            GeneratedDomainMessage::Relsub(RelsubMessage::AddEntry),
        )))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    scrollable(
        column![
            usubjid_field,
            Space::new().height(SPACING_MD),
            rsubjid_field,
            Space::new().height(SPACING_MD),
            srel_field,
            Space::new().height(SPACING_SM),
            examples,
            Space::new().height(SPACING_XL),
            bidirectional_notice,
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

/// RELSUB entries list.
fn view_relsub_entries(entries: &[crate::state::RelsubEntry]) -> Element<'_, Message> {
    if entries.is_empty() {
        return entry_list_container(
            "Subject Relationships",
            0,
            entry_list_empty(
                "No relationships added yet. Fill out the form and click \"Add Relationship\".",
            ),
        );
    }

    let entry_elements: Vec<Element<'_, Message>> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let primary = format!("{} → {}", entry.usubjid, entry.rsubjid);
            let secondary = entry.srel.clone();

            entry_row(primary, secondary, idx, |i| {
                Message::GeneratedDomain(GeneratedDomainMessage::Relsub(
                    RelsubMessage::RemoveEntry(i),
                ))
            })
        })
        .collect();

    entry_list_container(
        "Subject Relationships",
        entries.len(),
        column(entry_elements).spacing(1).into(),
    )
}
