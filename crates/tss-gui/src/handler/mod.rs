//! Message handler architecture for Iced-based GUI.
//!
//! This module provides a trait-based handler dispatch system that separates
//! message handling logic from the main App struct. This enables:
//!
//! - Better code organization (handlers grouped by feature)
//! - Easier testing (handlers can be tested independently)
//! - Clearer ownership boundaries
//!
//! # Architecture
//!
//! Each handler implements [`MessageHandler`] for a specific message type:
//!
//! ```ignore
//! pub struct HomeHandler;
//!
//! impl MessageHandler<HomeMessage> for HomeHandler {
//!     fn handle(&self, state: &mut AppState, msg: HomeMessage) -> Task<Message> {
//!         match msg {
//!             HomeMessage::SelectDomain(code) => { /* ... */ }
//!             // ...
//!         }
//!     }
//! }
//! ```
//!
//! The main `App::update()` dispatches to the appropriate handler:
//!
//! ```ignore
//! pub fn update(&mut self, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Home(msg) => HomeHandler.handle(&mut self.state, msg),
//!         Message::DomainEditor(msg) => DomainEditorHandler.handle(&mut self.state, msg),
//!         // ...
//!     }
//! }
//! ```

mod home;

use iced::Task;

use crate::message::Message;
use crate::state::AppState;

pub use home::HomeHandler;

/// Trait for handling messages in the Iced architecture.
///
/// This trait enables clean separation of message handling logic from the main
/// App struct. Each handler is responsible for a specific message type and can
/// access the full application state.
///
/// # Type Parameters
///
/// * `M` - The message type this handler processes
///
/// # Example
///
/// ```ignore
/// pub struct ValidationHandler;
///
/// impl MessageHandler<ValidationMessage> for ValidationHandler {
///     fn handle(&self, state: &mut AppState, msg: ValidationMessage) -> Task<Message> {
///         match msg {
///             ValidationMessage::RunValidation(domain) => {
///                 // Run validation logic
///                 Task::none()
///             }
///         }
///     }
/// }
/// ```
pub trait MessageHandler<M> {
    /// Handle a message, potentially mutating state and returning a follow-up task.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable reference to the application state
    /// * `msg` - The message to handle
    ///
    /// # Returns
    ///
    /// A `Task<Message>` for any async follow-up work, or `Task::none()` if complete.
    fn handle(&self, state: &mut AppState, msg: M) -> Task<Message>;
}

/// Extension trait for handlers that need additional context.
///
/// Some handlers may need access to resources beyond AppState, such as:
/// - Terminology registry
/// - Theme configuration
/// - Window management
///
/// This trait allows handlers to receive additional context.
#[allow(dead_code)]
pub trait MessageHandlerWithContext<M, C> {
    /// Handle a message with additional context.
    fn handle_with_context(&self, state: &mut AppState, msg: M, ctx: &C) -> Task<Message>;
}
