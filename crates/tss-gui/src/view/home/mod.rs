//! Home view for Trial Submission Studio.
//!
//! The home screen displays either:
//! - Welcome screen (no study loaded) - logo, workflow selector, recent studies
//! - Study dashboard (study loaded) - domain list with progress and validation

mod study;
mod welcome;

use iced::Element;

use crate::message::Message;
use crate::state::{AppState, ViewState, WorkflowMode};

pub use study::view_study;
pub use welcome::view_welcome;

/// Render the home view.
///
/// Routes to either the welcome screen or study dashboard based on
/// whether a study is currently loaded.
pub fn view_home(state: &AppState) -> Element<'_, Message> {
    // Get workflow mode from view state
    let workflow_mode = match &state.view {
        ViewState::Home { workflow_mode } => *workflow_mode,
        _ => WorkflowMode::default(),
    };

    if let Some(study) = &state.study {
        view_study(state, study, workflow_mode)
    } else {
        view_welcome(state, workflow_mode)
    }
}
