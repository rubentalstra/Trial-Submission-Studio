//! RELREC (Related Records) domain builder view.
//!
//! RELREC is a **Relationship** domain per SDTM-IG v3.4 Section 8.5.
//! It links records across or within domains using a relationship identifier.
//!
//! ## Required Variables
//! - RELID - Relationship identifier (groups related records)
//! - RDOMAIN - Related domain
//! - IDVAR - Identifying variable (e.g., --SEQ, --GRPID)
//!
//! ## Optional Variables
//! - USUBJID - Subject identifier (null for dataset-level relationships)
//! - IDVARVAL - Identifying variable value
//! - RELTYPE - Relationship type (ONE or MANY)

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::inputs::form_field;
use crate::message::{GeneratedDomainMessage, Message, RelrecBuilderState, RelrecMessage};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, button_secondary,
};

use super::common::{entry_list_container, entry_list_empty, entry_row};
use super::{view_builder_footer, view_header};

// =============================================================================
// RELREC BUILDER VIEW
// =============================================================================

/// Render the RELREC (Related Records) builder.
pub fn view_relrec_builder(state: &RelrecBuilderState) -> Element<'_, Message> {
    let header = view_header("Create Related Records Domain", Some("RELREC"));

    let content = row![
        // Left panel: Entry form
        container(view_relrec_form(state))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(SPACING_LG),
        // Right panel: Entry list
        container(view_relrec_entries(&state.entries))
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

/// RELREC entry form.
fn view_relrec_form(state: &RelrecBuilderState) -> Element<'_, Message> {
    let relid_field = form_field(
        "Relationship ID (RELID) *",
        &state.relid,
        "Groups related records together...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(RelrecMessage::RelidChanged(
                s,
            )))
        },
        None,
    );

    let usubjid_field = form_field(
        "Subject ID (USUBJID)",
        &state.usubjid,
        "Leave empty for dataset-level relationships",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(
                RelrecMessage::UsubjidChanged(if s.is_empty() { None } else { Some(s) }),
            ))
        },
        None,
    );

    let rdomain_field = form_field(
        "Related Domain (RDOMAIN) *",
        &state.rdomain,
        "e.g., AE, CM, LB, EX",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(
                RelrecMessage::RdomainChanged(s),
            ))
        },
        None,
    );

    let idvar_field = form_field(
        "Identifying Variable (IDVAR) *",
        &state.idvar,
        "e.g., AESEQ, CMGRPID",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(RelrecMessage::IdvarChanged(
                s,
            )))
        },
        None,
    );

    let idvarval_field = form_field(
        "Variable Value (IDVARVAL)",
        &state.idvarval,
        "Value of IDVAR for this record",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(
                RelrecMessage::IdvarvalChanged(if s.is_empty() { None } else { Some(s) }),
            ))
        },
        None,
    );

    let reltype_field = form_field(
        "Relationship Type (RELTYPE)",
        &state.reltype,
        "ONE or MANY",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Relrec(
                RelrecMessage::ReltypeChanged(if s.is_empty() { None } else { Some(s) }),
            ))
        },
        None,
    );

    // Help text
    let help_text = text(
        "To link records: create two entries with the same RELID, each pointing to a different domain/record.",
    )
    .size(12)
    .style(|theme: &Theme| text::Style {
        color: Some(theme.clinical().text_muted),
    });

    // Can add if required fields are filled
    let can_add = !state.relid.trim().is_empty()
        && !state.rdomain.trim().is_empty()
        && !state.idvar.trim().is_empty();

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
                text("Add Record Link").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .on_press_maybe(can_add.then_some(Message::GeneratedDomain(
            GeneratedDomainMessage::Relrec(RelrecMessage::AddEntry),
        )))
        .padding([SPACING_SM, SPACING_MD])
        .style(button_secondary);

    scrollable(
        column![
            relid_field,
            Space::new().height(SPACING_MD),
            usubjid_field,
            Space::new().height(SPACING_MD),
            row![
                container(rdomain_field).width(Length::FillPortion(1)),
                Space::new().width(SPACING_MD),
                container(idvar_field).width(Length::FillPortion(1)),
            ],
            Space::new().height(SPACING_MD),
            row![
                container(idvarval_field).width(Length::FillPortion(1)),
                Space::new().width(SPACING_MD),
                container(reltype_field).width(Length::FillPortion(1)),
            ],
            Space::new().height(SPACING_XL),
            help_text,
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

/// RELREC entries list.
fn view_relrec_entries(entries: &[crate::state::RelrecEntry]) -> Element<'_, Message> {
    if entries.is_empty() {
        return entry_list_container(
            "Record Links",
            0,
            entry_list_empty(
                "No record links added yet. Fill out the form and click \"Add Record Link\".",
            ),
        );
    }

    let entry_elements: Vec<Element<'_, Message>> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let primary = format!(
                "[{}] {}.{}",
                entry.relid,
                entry.rdomain,
                entry.idvarval.as_deref().unwrap_or("*")
            );
            let secondary = match &entry.usubjid {
                Some(subj) => format!("Subject: {} | IDVAR: {}", subj, entry.idvar),
                None => format!("Dataset-level | IDVAR: {}", entry.idvar),
            };

            entry_row(primary, secondary, idx, |i| {
                Message::GeneratedDomain(GeneratedDomainMessage::Relrec(
                    RelrecMessage::RemoveEntry(i),
                ))
            })
        })
        .collect();

    entry_list_container(
        "Record Links",
        entries.len(),
        column(entry_elements).spacing(1).into(),
    )
}
