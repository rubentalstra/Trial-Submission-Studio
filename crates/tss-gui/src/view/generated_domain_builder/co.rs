//! CO (Comments) domain builder view.
//!
//! CO is a **Special-Purpose** domain per SDTM-IG v3.4 Section 5.3.
//! It captures free-text comments that can be linked to records in other domains.
//!
//! ## Required Variables
//! - USUBJID - Subject identifier
//! - COVAL - Comment text (max 200 chars, overflow to COVALn)
//!
//! ## Optional Variables
//! - RDOMAIN - Related domain to link to
//! - IDVAR - Identifying variable name (when linking)
//! - IDVARVAL - Identifying variable value
//! - COREF - Comment reference
//! - CODTC - Date/time of comment
//! - COEVAL - Evaluator

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};
use iced_fonts::lucide;

use crate::component::inputs::form_field;
use crate::message::{CoBuilderState, CoMessage, GeneratedDomainMessage, Message};
use crate::theme::{
    ClinicalColors, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL, button_secondary,
};

use super::common::{entry_list_container, entry_list_empty, entry_row, truncate};
use super::{view_builder_footer, view_header};

// =============================================================================
// CO BUILDER VIEW
// =============================================================================

/// Render the CO (Comments) builder.
pub fn view_co_builder(state: &CoBuilderState) -> Element<'_, Message> {
    let header = view_header("Create Comments Domain", Some("CO"));

    let content = row![
        // Left panel: Entry form
        container(view_co_form(state))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(SPACING_LG),
        // Right panel: Entry list
        container(view_co_entries(&state.entries))
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

/// CO entry form.
fn view_co_form(state: &CoBuilderState) -> Element<'_, Message> {
    let usubjid_field = form_field(
        "Subject ID (USUBJID) *",
        &state.usubjid,
        "Enter subject identifier...",
        |s| Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::UsubjidChanged(s))),
        None,
    );

    let comment_field = form_field(
        "Comment (COVAL) *",
        &state.comment,
        "Enter comment text...",
        |s| Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::CommentChanged(s))),
        None,
    );

    let rdomain_field = form_field(
        "Related Domain (RDOMAIN)",
        &state.rdomain,
        "e.g., AE, CM, LB",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::RdomainChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    let idvar_field = form_field(
        "Identifying Variable (IDVAR)",
        &state.idvar,
        "e.g., AESEQ",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::IdvarChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    let idvarval_field = form_field(
        "Variable Value (IDVARVAL)",
        &state.idvarval,
        "e.g., 1",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::IdvarvalChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    let coref_field = form_field(
        "Comment Reference (COREF)",
        &state.coref,
        "Optional reference...",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::CorefChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    let codtc_field = form_field(
        "Date/Time (CODTC)",
        &state.codtc,
        "YYYY-MM-DD or ISO 8601",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::CodtcChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    let coeval_field = form_field(
        "Evaluator (COEVAL)",
        &state.coeval,
        "e.g., INVESTIGATOR",
        |s| {
            Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::CoevalChanged(
                if s.is_empty() { None } else { Some(s) },
            )))
        },
        None,
    );

    // Can add if required fields are filled
    let can_add = !state.usubjid.trim().is_empty() && !state.comment.trim().is_empty();

    let add_btn = button(
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
            text(if state.editing_index.is_some() {
                "Update Entry"
            } else {
                "Add Entry"
            })
            .size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(
        can_add.then_some(Message::GeneratedDomain(GeneratedDomainMessage::Co(
            CoMessage::AddEntry,
        ))),
    )
    .padding([SPACING_SM, SPACING_MD])
    .style(button_secondary);

    scrollable(
        column![
            usubjid_field,
            Space::new().height(SPACING_MD),
            comment_field,
            Space::new().height(SPACING_XL),
            row![
                container(rdomain_field).width(Length::FillPortion(1)),
                Space::new().width(SPACING_MD),
                container(idvar_field).width(Length::FillPortion(1)),
            ],
            Space::new().height(SPACING_MD),
            row![
                container(idvarval_field).width(Length::FillPortion(1)),
                Space::new().width(SPACING_MD),
                container(coref_field).width(Length::FillPortion(1)),
            ],
            Space::new().height(SPACING_MD),
            row![
                container(codtc_field).width(Length::FillPortion(1)),
                Space::new().width(SPACING_MD),
                container(coeval_field).width(Length::FillPortion(1)),
            ],
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

/// CO entries list.
fn view_co_entries(entries: &[crate::state::CommentEntry]) -> Element<'_, Message> {
    if entries.is_empty() {
        return entry_list_container(
            "Comment Entries",
            0,
            entry_list_empty("No comments added yet. Fill out the form and click \"Add Entry\"."),
        );
    }

    let entry_elements: Vec<Element<'_, Message>> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let primary = format!("{}: {}", entry.usubjid, truncate(&entry.comment, 40));
            let secondary = entry
                .rdomain
                .as_ref()
                .map(|r| format!("Links to {}", r))
                .unwrap_or_else(|| "Standalone comment".to_string());

            entry_row(primary, secondary, idx, |i| {
                Message::GeneratedDomain(GeneratedDomainMessage::Co(CoMessage::RemoveEntry(i)))
            })
        })
        .collect();

    entry_list_container(
        "Comment Entries",
        entries.len(),
        column(entry_elements).spacing(1).into(),
    )
}
