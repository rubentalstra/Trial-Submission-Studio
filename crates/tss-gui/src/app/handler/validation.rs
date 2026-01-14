//! Validation tab message handlers.
//!
//! Handles:
//! - Validation refresh trigger
//! - Severity filter changes
//! - Validation computation callback

use iced::Task;

use crate::app::App;
use crate::message::Message;
use crate::message::domain_editor::ValidationMessage;
use crate::service::validation::{ValidationInput, compute_validation};
use crate::state::{EditorTab, ViewState};

impl App {
    /// Handle validation tab messages.
    pub fn handle_validation_message(&mut self, msg: ValidationMessage) -> Task<Message> {
        // Get current domain code
        let domain_code = match &self.state.view {
            ViewState::DomainEditor { domain, .. } => domain.clone(),
            _ => return Task::none(),
        };

        match msg {
            ValidationMessage::RefreshValidation => {
                // Get domain data
                let domain = match self
                    .state
                    .study
                    .as_ref()
                    .and_then(|s| s.domain(&domain_code))
                {
                    Some(d) => d,
                    None => return Task::none(),
                };

                // Get preview DataFrame (validation runs on transformed data)
                let df = match &self.state.view {
                    ViewState::DomainEditor {
                        preview_cache: Some(df),
                        ..
                    } => df.clone(),
                    _ => {
                        // Fall back to source data if no preview available
                        domain.source.data.clone()
                    }
                };

                // Get SDTM domain definition
                let sdtm_domain = domain.mapping.domain().clone();

                // Get not collected variables (convert from BTreeMap to BTreeSet)
                let not_collected: std::collections::BTreeSet<String> =
                    domain.mapping.all_not_collected().keys().cloned().collect();

                // Build input
                let input = ValidationInput {
                    domain: sdtm_domain,
                    df,
                    ct_registry: self.state.terminology.clone(),
                    not_collected,
                };

                let domain_for_result = domain_code.clone();

                // Start async validation
                Task::perform(compute_validation(input), move |report| {
                    Message::ValidationComplete {
                        domain: domain_for_result,
                        report,
                    }
                })
            }

            ValidationMessage::IssueSelected(idx) => {
                if let ViewState::DomainEditor { validation_ui, .. } = &mut self.state.view {
                    validation_ui.selected_issue = Some(idx);
                }
                Task::none()
            }

            ValidationMessage::SeverityFilterChanged(filter) => {
                if let ViewState::DomainEditor { validation_ui, .. } = &mut self.state.view {
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
                // Navigate to mapping tab and select the variable
                if let ViewState::DomainEditor {
                    tab, mapping_ui, ..
                } = &mut self.state.view
                {
                    *tab = EditorTab::Mapping;
                    // Try to find and select the variable by name
                    if let Some(domain) = self
                        .state
                        .study
                        .as_ref()
                        .and_then(|s| s.domain(&domain_code))
                    {
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
}
