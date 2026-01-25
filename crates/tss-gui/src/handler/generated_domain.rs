//! Generated domain builder message handler.
//!
//! Handles:
//! - Domain type selection (CO, RELREC, RELSPEC, RELSUB)
//! - Form field updates for each domain type
//! - Entry management (add, remove, edit)
//! - Domain generation and creation

use iced::Task;

use super::MessageHandler;
use crate::message::Message;
use crate::message::generated_domains::{
    CoMessage, GeneratedDomainBuilderState, GeneratedDomainMessage, RelrecMessage, RelspecMessage,
    RelsubMessage,
};
use crate::service::generated_domains as generation_service;
use crate::state::{AppState, ViewState};

/// Handler for generated domain builder messages.
pub struct GeneratedDomainHandler;

impl MessageHandler<GeneratedDomainMessage> for GeneratedDomainHandler {
    fn handle(&self, state: &mut AppState, msg: GeneratedDomainMessage) -> Task<Message> {
        // Get the builder state from view state
        let builder = match &mut state.view {
            ViewState::GeneratedDomainBuilder { builder, .. } => builder,
            _ => return Task::none(),
        };

        match msg {
            GeneratedDomainMessage::SelectDomainType(domain_type) => {
                builder.selected_type = Some(domain_type);
                Task::none()
            }

            GeneratedDomainMessage::Cancel => {
                state.view = ViewState::home();
                Task::none()
            }

            GeneratedDomainMessage::CreateDomain => handle_create_domain(state),

            // Delegate to domain-specific handlers
            GeneratedDomainMessage::Co(msg) => handle_co_message(builder, msg),
            GeneratedDomainMessage::Relrec(msg) => handle_relrec_message(builder, msg),
            GeneratedDomainMessage::Relspec(msg) => handle_relspec_message(builder, msg),
            GeneratedDomainMessage::Relsub(msg) => handle_relsub_message(builder, msg),
        }
    }
}

// =============================================================================
// CO MESSAGE HANDLER
// =============================================================================

fn handle_co_message(builder: &mut GeneratedDomainBuilderState, msg: CoMessage) -> Task<Message> {
    match msg {
        CoMessage::UsubjidChanged(val) => {
            builder.co.usubjid = val;
        }
        CoMessage::CommentChanged(val) => {
            builder.co.comment = val;
        }
        CoMessage::RdomainChanged(val) => {
            builder.co.rdomain = val.unwrap_or_default();
        }
        CoMessage::IdvarChanged(val) => {
            builder.co.idvar = val.unwrap_or_default();
        }
        CoMessage::IdvarvalChanged(val) => {
            builder.co.idvarval = val.unwrap_or_default();
        }
        CoMessage::CorefChanged(val) => {
            builder.co.coref = val.unwrap_or_default();
        }
        CoMessage::CodtcChanged(val) => {
            builder.co.codtc = val.unwrap_or_default();
        }
        CoMessage::CoevalChanged(val) => {
            builder.co.coeval = val.unwrap_or_default();
        }
        CoMessage::AddEntry => {
            if let Some(entry) = builder.co.build_entry() {
                if let Some(index) = builder.co.editing_index {
                    if index < builder.co.entries.len() {
                        builder.co.entries[index] = entry;
                    }
                } else {
                    builder.co.entries.push(entry);
                }
                builder.co.clear_form();
            }
        }
        CoMessage::RemoveEntry(index) => {
            if index < builder.co.entries.len() {
                builder.co.entries.remove(index);
            }
        }
        CoMessage::EditEntry(index) => {
            builder.co.load_entry(index);
        }
    }
    Task::none()
}

// =============================================================================
// RELREC MESSAGE HANDLER
// =============================================================================

fn handle_relrec_message(
    builder: &mut GeneratedDomainBuilderState,
    msg: RelrecMessage,
) -> Task<Message> {
    match msg {
        RelrecMessage::RelidChanged(val) => {
            builder.relrec.relid = val;
        }
        RelrecMessage::UsubjidChanged(val) => {
            builder.relrec.usubjid = val.unwrap_or_default();
        }
        RelrecMessage::RdomainChanged(val) => {
            builder.relrec.rdomain = val;
        }
        RelrecMessage::IdvarChanged(val) => {
            builder.relrec.idvar = val;
        }
        RelrecMessage::IdvarvalChanged(val) => {
            builder.relrec.idvarval = val.unwrap_or_default();
        }
        RelrecMessage::ReltypeChanged(val) => {
            builder.relrec.reltype = val.unwrap_or_default();
        }
        RelrecMessage::AddEntry => {
            if let Some(entry) = builder.relrec.build_entry() {
                builder.relrec.entries.push(entry);
                builder.relrec.clear_form();
            }
        }
        RelrecMessage::RemoveEntry(index) => {
            if index < builder.relrec.entries.len() {
                builder.relrec.entries.remove(index);
            }
        }
    }
    Task::none()
}

// =============================================================================
// RELSPEC MESSAGE HANDLER
// =============================================================================

fn handle_relspec_message(
    builder: &mut GeneratedDomainBuilderState,
    msg: RelspecMessage,
) -> Task<Message> {
    match msg {
        RelspecMessage::UsubjidChanged(val) => {
            builder.relspec.usubjid = val;
        }
        RelspecMessage::RefidChanged(val) => {
            builder.relspec.refid = val;
        }
        RelspecMessage::SpecChanged(val) => {
            builder.relspec.spec = val.unwrap_or_default();
        }
        RelspecMessage::ParentChanged(val) => {
            builder.relspec.parent = val.unwrap_or_default();
        }
        RelspecMessage::AddEntry => {
            if let Some(entry) = builder.relspec.build_entry() {
                builder.relspec.entries.push(entry);
                builder.relspec.clear_form();
            }
        }
        RelspecMessage::RemoveEntry(index) => {
            if index < builder.relspec.entries.len() {
                builder.relspec.entries.remove(index);
            }
        }
    }
    Task::none()
}

// =============================================================================
// RELSUB MESSAGE HANDLER
// =============================================================================

fn handle_relsub_message(
    builder: &mut GeneratedDomainBuilderState,
    msg: RelsubMessage,
) -> Task<Message> {
    match msg {
        RelsubMessage::UsubjidChanged(val) => {
            builder.relsub.usubjid = val;
        }
        RelsubMessage::RsubjidChanged(val) => {
            builder.relsub.rsubjid = val;
        }
        RelsubMessage::SrelChanged(val) => {
            builder.relsub.srel = val;
        }
        RelsubMessage::AddEntry => {
            if let Some(entry) = builder.relsub.build_entry() {
                builder.relsub.entries.push(entry);
                builder.relsub.clear_form();
            }
        }
        RelsubMessage::RemoveEntry(index) => {
            if index < builder.relsub.entries.len() {
                builder.relsub.entries.remove(index);
            }
        }
    }
    Task::none()
}

// =============================================================================
// DOMAIN CREATION
// =============================================================================

fn handle_create_domain(state: &mut AppState) -> Task<Message> {
    // Extract builder state
    let (domain_type, entries) = match &state.view {
        ViewState::GeneratedDomainBuilder { builder, .. } => {
            let Some(domain_type) = builder.selected_type else {
                return Task::none();
            };
            (domain_type, builder.get_entries())
        }
        _ => return Task::none(),
    };

    if entries.is_empty() {
        return Task::none();
    }

    // Get study ID
    let Some(study) = &state.study else {
        return Task::none();
    };

    let study_id = study.study_id.clone();

    // Get domain definition from standards
    let config = tss_standards::StandardsConfig::sdtm_only();
    let standards = match tss_standards::StandardsRegistry::load(&config) {
        Ok(s) => s,
        Err(e) => {
            state.error = Some(crate::error::GuiError::Operation {
                operation: "Load Standards".to_string(),
                reason: e.to_string(),
            });
            return Task::none();
        }
    };

    let Some(definition) = generation_service::get_domain_definition(domain_type, &standards)
    else {
        state.error = Some(crate::error::GuiError::Operation {
            operation: format!("Generate {} Domain", domain_type.label()),
            reason: format!("Domain definition not found for {}", domain_type.code()),
        });
        return Task::none();
    };

    // Generate the domain
    let result = generation_service::generate_relationship_domain(
        domain_type,
        &study_id,
        entries,
        definition,
    );

    match result {
        Ok(generated_state) => {
            let domain_code = domain_type.code();

            // Add to study
            if let Some(study) = &mut state.study {
                study.add_domain(
                    domain_code,
                    crate::state::DomainState::Generated(generated_state),
                );
            }

            // Mark as dirty
            state.dirty_tracker.mark_dirty();

            // Navigate back to home
            state.view = ViewState::home();
        }
        Err(e) => {
            tracing::error!("Failed to generate {} domain: {}", domain_type.label(), e);
            state.error = Some(crate::error::GuiError::Operation {
                operation: format!("Generate {} Domain", domain_type.label()),
                reason: e.to_string(),
            });
        }
    }

    Task::none()
}
