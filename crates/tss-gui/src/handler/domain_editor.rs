//! Domain editor message handlers.
//!
//! Handles:
//! - Tab navigation (Map, Normalize, Validate, Preview, SUPP)
//! - Delegates to specific sub-handlers for each tab

use iced::Task;

use super::MessageHandler;
use crate::message::Message;
use crate::message::domain_editor::{
    DomainEditorMessage, MappingMessage, NormalizationMessage, PreviewMessage, SuppMessage,
    ValidationMessage,
};
use crate::service::preview::{PreviewInput, compute_preview};
use crate::service::validation::{ValidationInput, compute_validation};
use crate::state::{
    AppState, EditorTab, NotCollectedEdit, SuppAction, SuppColumnConfig, SuppEditDraft, ViewState,
};

/// Handler for domain editor messages.
pub struct DomainEditorHandler;

impl MessageHandler<DomainEditorMessage> for DomainEditorHandler {
    fn handle(&self, state: &mut AppState, msg: DomainEditorMessage) -> Task<Message> {
        match msg {
            DomainEditorMessage::TabSelected(tab) => {
                if let ViewState::DomainEditor {
                    tab: current_tab, ..
                } = &mut state.view
                {
                    *current_tab = tab;
                }
                Task::none()
            }

            DomainEditorMessage::BackClicked => {
                state.view = ViewState::home();
                Task::none()
            }

            DomainEditorMessage::Mapping(mapping_msg) => handle_mapping_message(state, mapping_msg),

            DomainEditorMessage::Normalization(norm_msg) => {
                handle_normalization_message(state, norm_msg)
            }

            DomainEditorMessage::Validation(validation_msg) => {
                handle_validation_message(state, validation_msg)
            }

            DomainEditorMessage::Preview(preview_msg) => handle_preview_message(state, preview_msg),

            DomainEditorMessage::Supp(supp_msg) => handle_supp_message(state, supp_msg),
        }
    }
}

// =============================================================================
// MAPPING HANDLER
// =============================================================================

fn handle_mapping_message(state: &mut AppState, msg: MappingMessage) -> Task<Message> {
    // Get current domain code
    let domain_code = match &state.view {
        ViewState::DomainEditor { domain, .. } => domain.clone(),
        _ => return Task::none(),
    };

    match msg {
        MappingMessage::VariableSelected(idx) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.selected_variable = Some(idx);
            }
            Task::none()
        }

        MappingMessage::SearchChanged(text) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.search_filter = text;
            }
            Task::none()
        }

        MappingMessage::SearchCleared => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.search_filter.clear();
            }
            Task::none()
        }

        MappingMessage::AcceptSuggestion(variable) => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                // Explicit error handling for mapping operations (#273)
                if let Err(e) = domain.mapping.accept_suggestion(&variable) {
                    tracing::error!(variable = %variable, error = %e, "Failed to accept suggestion");
                }
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::ClearMapping(variable) => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                domain.mapping.clear_assignment(&variable);
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::ManualMap { variable, column } => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                // Explicit error handling for mapping operations (#273)
                if let Err(e) = domain.mapping.accept_manual(&variable, &column) {
                    tracing::error!(variable = %variable, column = %column, error = %e, "Failed to accept manual mapping");
                }
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::MarkNotCollected { variable } => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                    variable,
                    reason: String::new(),
                });
            }
            Task::none()
        }

        MappingMessage::NotCollectedReasonChanged(reason) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view
                && let Some(edit) = &mut mapping_ui.not_collected_edit
            {
                edit.reason = reason;
            }
            Task::none()
        }

        MappingMessage::NotCollectedSave { variable, reason } => {
            if reason.trim().is_empty() {
                return Task::none();
            }
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                let _ = domain.mapping.mark_not_collected(&variable, &reason);
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor {
                mapping_ui,
                preview_cache,
                ..
            } = &mut state.view
            {
                mapping_ui.not_collected_edit = None;
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::NotCollectedCancel => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.not_collected_edit = None;
            }
            Task::none()
        }

        MappingMessage::EditNotCollectedReason {
            variable,
            current_reason,
        } => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.not_collected_edit = Some(NotCollectedEdit {
                    variable,
                    reason: current_reason,
                });
            }
            Task::none()
        }

        MappingMessage::ClearNotCollected(variable) => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                domain.mapping.clear_assignment(&variable);
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::MarkOmitted(variable) => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                let _ = domain.mapping.mark_omit(&variable);
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::ClearOmitted(variable) => {
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                domain.mapping.clear_assignment(&variable);
                domain.invalidate_validation();
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { preview_cache, .. } = &mut state.view {
                *preview_cache = None;
            }
            Task::none()
        }

        MappingMessage::FilterUnmappedToggled(enabled) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.filter_unmapped = enabled;
            }
            Task::none()
        }

        MappingMessage::FilterRequiredToggled(enabled) => {
            if let ViewState::DomainEditor { mapping_ui, .. } = &mut state.view {
                mapping_ui.filter_required = enabled;
            }
            Task::none()
        }
    }
}

// =============================================================================
// NORMALIZATION HANDLER
// =============================================================================

#[allow(clippy::needless_pass_by_value)]
fn handle_normalization_message(state: &mut AppState, msg: NormalizationMessage) -> Task<Message> {
    match msg {
        NormalizationMessage::RuleSelected(idx) => {
            if let ViewState::DomainEditor {
                normalization_ui, ..
            } = &mut state.view
            {
                normalization_ui.selected_rule = Some(idx);
            }
            Task::none()
        }
    }
}

// =============================================================================
// VALIDATION HANDLER
// =============================================================================

fn handle_validation_message(state: &mut AppState, msg: ValidationMessage) -> Task<Message> {
    let domain_code = match &state.view {
        ViewState::DomainEditor { domain, .. } => domain.clone(),
        _ => return Task::none(),
    };

    match msg {
        ValidationMessage::RefreshValidation => {
            let domain = match state.study.as_ref().and_then(|s| s.domain(&domain_code)) {
                Some(d) => d,
                None => return Task::none(),
            };

            let df = match &state.view {
                ViewState::DomainEditor {
                    preview_cache: Some(df),
                    ..
                } => df.clone(),
                _ => domain.source.data.clone(),
            };

            let sdtm_domain = domain.mapping.domain().clone();
            let not_collected: std::collections::BTreeSet<String> =
                domain.mapping.all_not_collected().keys().cloned().collect();

            let input = ValidationInput {
                domain: sdtm_domain,
                df,
                ct_registry: state.terminology.clone(),
                not_collected,
            };

            let domain_for_result = domain_code.clone();

            Task::perform(compute_validation(input), move |report| {
                Message::ValidationComplete {
                    domain: domain_for_result,
                    report,
                }
            })
        }

        ValidationMessage::IssueSelected(idx) => {
            if let ViewState::DomainEditor { validation_ui, .. } = &mut state.view {
                validation_ui.selected_issue = Some(idx);
            }
            Task::none()
        }

        ValidationMessage::SeverityFilterChanged(filter) => {
            if let ViewState::DomainEditor { validation_ui, .. } = &mut state.view {
                validation_ui.severity_filter = match filter {
                    crate::message::domain_editor::SeverityFilter::All => {
                        crate::state::SeverityFilter::All
                    }
                    crate::message::domain_editor::SeverityFilter::Errors => {
                        crate::state::SeverityFilter::Errors
                    }
                    crate::message::domain_editor::SeverityFilter::Warnings => {
                        crate::state::SeverityFilter::Warnings
                    }
                    crate::message::domain_editor::SeverityFilter::Info => {
                        crate::state::SeverityFilter::Info
                    }
                };
            }
            Task::none()
        }

        ValidationMessage::GoToIssueSource { variable } => {
            if let ViewState::DomainEditor {
                tab, mapping_ui, ..
            } = &mut state.view
            {
                *tab = EditorTab::Mapping;
                if let Some(domain) = state.study.as_ref().and_then(|s| s.domain(&domain_code)) {
                    let sdtm_domain = domain.mapping.domain();
                    if let Some(idx) = sdtm_domain
                        .variables
                        .iter()
                        .position(|v| v.name == variable)
                    {
                        mapping_ui.selected_variable = Some(idx);
                    }
                }
            }
            Task::none()
        }
    }
}

// =============================================================================
// PREVIEW HANDLER
// =============================================================================

#[allow(clippy::needless_pass_by_value)]
fn handle_preview_message(state: &mut AppState, msg: PreviewMessage) -> Task<Message> {
    let domain_code = match &state.view {
        ViewState::DomainEditor { domain, .. } => domain.clone(),
        _ => return Task::none(),
    };

    match msg {
        PreviewMessage::GoToPage(page) => {
            if let ViewState::DomainEditor { preview_ui, .. } = &mut state.view {
                preview_ui.current_page = page;
            }
            Task::none()
        }

        PreviewMessage::NextPage => {
            if let ViewState::DomainEditor { preview_ui, .. } = &mut state.view {
                preview_ui.current_page = preview_ui.current_page.saturating_add(1);
            }
            Task::none()
        }

        PreviewMessage::PreviousPage => {
            if let ViewState::DomainEditor { preview_ui, .. } = &mut state.view {
                preview_ui.current_page = preview_ui.current_page.saturating_sub(1);
            }
            Task::none()
        }

        PreviewMessage::RowsPerPageChanged(rows) => {
            if let ViewState::DomainEditor { preview_ui, .. } = &mut state.view {
                preview_ui.rows_per_page = rows;
                preview_ui.current_page = 0;
            }
            state.settings.display.preview_rows_per_page = rows;
            // Best-effort: preference saving is non-critical (#273)
            crate::util::best_effort!(state.settings.save(), "saving display preference");
            Task::none()
        }

        PreviewMessage::RebuildPreview => {
            let domain = match state.study.as_ref().and_then(|s| s.domain(&domain_code)) {
                Some(d) => d,
                None => return Task::none(),
            };

            let input = PreviewInput {
                source_df: domain.source.data.clone(),
                mapping: domain.mapping.clone(),
                ct_registry: state.terminology.clone(),
            };

            let domain_for_result = domain_code.clone();

            Task::perform(compute_preview(input), move |result| {
                Message::PreviewReady {
                    domain: domain_for_result,
                    result: result.map_err(|e| e.to_string()),
                }
            })
        }
    }
}

// =============================================================================
// SUPP HANDLER
// =============================================================================

fn handle_supp_message(state: &mut AppState, msg: SuppMessage) -> Task<Message> {
    let domain_code = match &state.view {
        ViewState::DomainEditor { domain, .. } => domain.clone(),
        _ => return Task::none(),
    };

    match msg {
        SuppMessage::ColumnSelected(col_name) => {
            // Clear any edit draft when changing selection
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.selected_column = Some(col_name.clone());
                supp_ui.edit_draft = None;
            }
            // Initialize config if not exists
            if let Some(domain) = state
                .study
                .as_mut()
                .and_then(|s| s.domain_mut(&domain_code))
            {
                domain
                    .supp_config
                    .entry(col_name.clone())
                    .or_insert_with(|| SuppColumnConfig::from_column(&col_name));
            }
            Task::none()
        }

        SuppMessage::SearchChanged(text) => {
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.search_filter = text;
            }
            Task::none()
        }

        SuppMessage::FilterModeChanged(mode) => {
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.filter_mode = mode;
            }
            Task::none()
        }

        SuppMessage::QnamChanged(value) => {
            let value = value.chars().take(8).collect::<String>().to_uppercase();
            update_supp_field(state, &domain_code, |config, draft| {
                if let Some(d) = draft {
                    d.qnam = value;
                } else {
                    config.qnam = value;
                }
            });
            Task::none()
        }

        SuppMessage::QlabelChanged(value) => {
            let value: String = value.chars().take(40).collect();
            update_supp_field(state, &domain_code, |config, draft| {
                if let Some(d) = draft {
                    d.qlabel = value;
                } else {
                    config.qlabel = value;
                }
            });
            Task::none()
        }

        SuppMessage::QorigChanged(value) => {
            update_supp_field(state, &domain_code, |config, draft| {
                if let Some(d) = draft {
                    d.qorig = value;
                } else {
                    config.qorig = value;
                }
            });
            Task::none()
        }

        SuppMessage::QevalChanged(value) => {
            update_supp_field(state, &domain_code, |config, draft| {
                if let Some(d) = draft {
                    d.qeval = value.clone();
                } else {
                    config.qeval = if value.is_empty() { None } else { Some(value) };
                }
            });
            Task::none()
        }

        SuppMessage::AddToSupp => {
            let col = match &state.view {
                ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                _ => None,
            };

            if let Some(col_name) = col
                && let Some(domain) = state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                && let Some(config) = domain.supp_config.get_mut(&col_name)
            {
                if config.qnam.trim().is_empty() || config.qlabel.trim().is_empty() {
                    return Task::none();
                }
                config.action = SuppAction::Include;
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.edit_draft = None;
            }
            Task::none()
        }

        SuppMessage::Skip => {
            let col = match &state.view {
                ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                _ => None,
            };

            if let Some(col_name) = col
                && let Some(domain) = state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                && let Some(config) = domain.supp_config.get_mut(&col_name)
            {
                config.action = SuppAction::Skip;
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.edit_draft = None;
            }
            Task::none()
        }

        SuppMessage::UndoAction => {
            let col = match &state.view {
                ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                _ => None,
            };

            if let Some(col_name) = col
                && let Some(domain) = state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                && let Some(config) = domain.supp_config.get_mut(&col_name)
            {
                config.action = SuppAction::Pending;
                state.dirty_tracker.mark_dirty();
            }
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.edit_draft = None;
            }
            Task::none()
        }

        SuppMessage::StartEdit => {
            let col = match &state.view {
                ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
                _ => None,
            };

            if let Some(col_name) = &col
                && let Some(domain) = state.study.as_ref().and_then(|s| s.domain(&domain_code))
                && let Some(config) = domain.supp_config.get(col_name)
            {
                let draft = SuppEditDraft::from_config(config);
                if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                    supp_ui.edit_draft = Some(draft);
                }
            }
            Task::none()
        }

        SuppMessage::SaveEdit => {
            let (col, draft) = match &state.view {
                ViewState::DomainEditor { supp_ui, .. } => {
                    (supp_ui.selected_column.clone(), supp_ui.edit_draft.clone())
                }
                _ => (None, None),
            };

            if let (Some(col_name), Some(draft)) = (col, draft) {
                if draft.qnam.trim().is_empty() || draft.qlabel.trim().is_empty() {
                    return Task::none();
                }

                if let Some(domain) = state
                    .study
                    .as_mut()
                    .and_then(|s| s.domain_mut(&domain_code))
                    && let Some(config) = domain.supp_config.get_mut(&col_name)
                {
                    config.qnam = draft.qnam;
                    config.qlabel = draft.qlabel;
                    config.qorig = draft.qorig;
                    config.qeval = if draft.qeval.is_empty() {
                        None
                    } else {
                        Some(draft.qeval)
                    };
                    state.dirty_tracker.mark_dirty();
                }
            }
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.edit_draft = None;
            }
            Task::none()
        }

        SuppMessage::CancelEdit => {
            if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view {
                supp_ui.edit_draft = None;
            }
            Task::none()
        }
    }
}

/// Helper to update a SUPP field, routing to draft or config as appropriate.
fn update_supp_field<F>(state: &mut AppState, domain_code: &str, update: F)
where
    F: FnOnce(&mut SuppColumnConfig, Option<&mut SuppEditDraft>),
{
    let col = match &state.view {
        ViewState::DomainEditor { supp_ui, .. } => supp_ui.selected_column.clone(),
        _ => return,
    };

    let Some(col_name) = col else { return };

    let is_editing = match &state.view {
        ViewState::DomainEditor { supp_ui, .. } => supp_ui.edit_draft.is_some(),
        _ => false,
    };

    if is_editing {
        if let ViewState::DomainEditor { supp_ui, .. } = &mut state.view
            && let Some(draft) = &mut supp_ui.edit_draft
        {
            let mut dummy = SuppColumnConfig::from_column("");
            update(&mut dummy, Some(draft));
        }
    } else if let Some(domain) = state.study.as_mut().and_then(|s| s.domain_mut(domain_code))
        && let Some(config) = domain.supp_config.get_mut(&col_name)
    {
        update(config, None);
    }
}
