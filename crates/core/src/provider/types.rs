//! Request and response types for AI provider completions.
//!
//! This module defines the core data types used for communicating with AI providers:
//! - [`Role`]: The role of a message sender (System, User, Assistant)
//! - [`Message`]: A single message in a conversation
//! - [`CompletionRequest`]: Configuration for a completion request

use serde::{Deserialize, Serialize};

/// The role of a message sender in a conversation.
///
/// Each message in a conversation has an associated role that indicates
/// who (or what) authored the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System messages provide instructions or context to the AI.
    /// These typically set the behavior, personality, or constraints.
    System,

    /// User messages are inputs from the human user.
    User,

    /// Assistant messages are responses from the AI.
    Assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

/// A single message in a conversation.
///
/// Messages are the fundamental unit of communication with AI providers.
/// Each message has a role indicating who sent it and content containing
/// the actual text.
///
/// # Examples
///
/// ```
/// use cherry2k_core::provider::{Message, Role};
///
/// let system_msg = Message::new(Role::System, "You are a helpful assistant.");
/// let user_msg = Message::new(Role::User, "Hello!");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,

    /// The content of the message.
    pub content: String,
}

impl Message {
    /// Creates a new message with the given role and content.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// A request for an AI completion.
///
/// This struct contains all the parameters needed to request a completion
/// from an AI provider. Use the builder pattern methods to configure the request.
///
/// # Examples
///
/// ```
/// use cherry2k_core::provider::{CompletionRequest, Message, Role};
///
/// let request = CompletionRequest::new()
///     .with_message(Message::user("What is Rust?"))
///     .with_model("gpt-4")
///     .with_temperature(0.7);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The messages in the conversation history.
    pub messages: Vec<Message>,

    /// The model to use for completion (provider-specific).
    /// If None, the provider's default model is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Sampling temperature (0.0 to 2.0).
    /// Higher values make output more random, lower values more deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum number of tokens to generate.
    /// If None, the provider's default limit is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl CompletionRequest {
    /// Creates a new empty completion request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a message to the conversation.
    pub fn with_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Adds multiple messages to the conversation.
    pub fn with_messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Sets the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the sampling temperature.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if temperature is not in range 0.0..=2.0.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        debug_assert!(
            (0.0..=2.0).contains(&temperature),
            "Temperature must be between 0.0 and 2.0, got {temperature}"
        );
        self.temperature = Some(temperature);
        self
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod role {
        use super::*;

        #[test]
        fn display_formats_lowercase() {
            assert_eq!(Role::System.to_string(), "system");
            assert_eq!(Role::User.to_string(), "user");
            assert_eq!(Role::Assistant.to_string(), "assistant");
        }

        #[test]
        fn serializes_lowercase() {
            assert_eq!(serde_json::to_string(&Role::System).unwrap(), "\"system\"");
            assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
            assert_eq!(
                serde_json::to_string(&Role::Assistant).unwrap(),
                "\"assistant\""
            );
        }

        #[test]
        fn deserializes_lowercase() {
            assert_eq!(
                serde_json::from_str::<Role>("\"system\"").unwrap(),
                Role::System
            );
            assert_eq!(
                serde_json::from_str::<Role>("\"user\"").unwrap(),
                Role::User
            );
            assert_eq!(
                serde_json::from_str::<Role>("\"assistant\"").unwrap(),
                Role::Assistant
            );
        }
    }

    mod message {
        use super::*;

        #[test]
        fn new_creates_message() {
            let msg = Message::new(Role::User, "Hello");
            assert_eq!(msg.role, Role::User);
            assert_eq!(msg.content, "Hello");
        }

        #[test]
        fn convenience_constructors_work() {
            let system = Message::system("Be helpful");
            let user = Message::user("Hello");
            let assistant = Message::assistant("Hi there!");

            assert_eq!(system.role, Role::System);
            assert_eq!(user.role, Role::User);
            assert_eq!(assistant.role, Role::Assistant);
        }

        #[test]
        fn serializes_to_json() {
            let msg = Message::new(Role::User, "Hello");
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("\"role\":\"user\""));
            assert!(json.contains("\"content\":\"Hello\""));
        }
    }

    mod completion_request {
        use super::*;

        #[test]
        fn default_is_empty() {
            let req = CompletionRequest::default();
            assert!(req.messages.is_empty());
            assert!(req.model.is_none());
            assert!(req.temperature.is_none());
            assert!(req.max_tokens.is_none());
        }

        #[test]
        fn builder_pattern_works() {
            let req = CompletionRequest::new()
                .with_message(Message::user("Hello"))
                .with_model("gpt-4")
                .with_temperature(0.7)
                .with_max_tokens(100);

            assert_eq!(req.messages.len(), 1);
            assert_eq!(req.model, Some("gpt-4".to_string()));
            assert_eq!(req.temperature, Some(0.7));
            assert_eq!(req.max_tokens, Some(100));
        }

        #[test]
        fn with_messages_adds_multiple() {
            let req = CompletionRequest::new().with_messages([
                Message::system("Be helpful"),
                Message::user("Hello"),
            ]);

            assert_eq!(req.messages.len(), 2);
            assert_eq!(req.messages[0].role, Role::System);
            assert_eq!(req.messages[1].role, Role::User);
        }

        #[test]
        #[should_panic(expected = "Temperature must be between")]
        #[cfg(debug_assertions)]
        fn temperature_panics_on_invalid_value() {
            let _ = CompletionRequest::new().with_temperature(3.0);
        }
    }
}
