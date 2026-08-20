//! Strongly-typed inference data contracts, messages, and configuration models.

use crate::metrics::TokenMetrics;
use crate::types::ProviderKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Author role of a conversation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// System instruction or steering prompt.
    System,
    /// Human end-user or client interaction.
    User,
    /// Assistant / AI generated response.
    Assistant,
    /// Tool observation or execution output.
    Tool,
}

/// Type alias for [`Role`].
pub type MessageRole = Role;

impl Role {
    /// Canonical string identifier for the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// Polymorphic content block within a multimodal message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plaintext or Markdown content.
    Text { text: String },
    /// Model reasoning / Chain-of-Thought thinking block.
    Thinking { reasoning: String },
    /// Base64-encoded image data.
    Image {
        media_type: String,
        data_base64: String,
    },
    /// Model invocation of an external tool.
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Result payload returned from tool execution.
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
}

impl ContentBlock {
    /// Constructs a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Constructs a thinking/reasoning content block.
    pub fn thinking(reasoning: impl Into<String>) -> Self {
        Self::Thinking {
            reasoning: reasoning.into(),
        }
    }

    /// Constructs an image content block.
    pub fn image(media_type: impl Into<String>, data_base64: impl Into<String>) -> Self {
        Self::Image {
            media_type: media_type.into(),
            data_base64: data_base64.into(),
        }
    }

    /// Constructs a tool use content block.
    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Constructs a tool result content block.
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error,
        }
    }

    /// Returns text slice if this block is [`ContentBlock::Text`].
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Returns reasoning slice if this block is [`ContentBlock::Thinking`].
    pub fn as_thinking(&self) -> Option<&str> {
        match self {
            Self::Thinking { reasoning } => Some(reasoning),
            _ => None,
        }
    }
}

/// Unified chat message representation across all providers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// Message author role.
    pub role: Role,
    /// Multimodal content blocks.
    pub content: Vec<ContentBlock>,
    /// Optional participant or tool identifier name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    /// Creates a new user message containing plain text.
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::text(text)],
            name: None,
        }
    }

    /// Creates a new assistant message containing plain text.
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::text(text)],
            name: None,
        }
    }

    /// Creates a system instruction message.
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentBlock::text(text)],
            name: None,
        }
    }

    /// Creates a tool result message.
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self {
            role: Role::Tool,
            content: vec![ContentBlock::tool_result(tool_use_id, content, is_error)],
            name: None,
        }
    }

    /// Sets an optional participant/tool name on the message.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Appends a content block to the message.
    pub fn add_content(&mut self, block: ContentBlock) {
        self.content.push(block);
    }

    /// Extracts all concatenated text content from text blocks.
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// JSON-Schema tool definition for model function calling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Unique name of the tool function.
    pub name: String,
    /// Human/model description explaining tool functionality.
    pub description: String,
    /// JSON Schema describing expected parameters.
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Constructs a new [`ToolDefinition`].
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Reason why inference generation terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Model completed its response turn normally.
    EndTurn,
    /// Model reached maximum token generation limit.
    MaxTokens,
    /// Generation halted upon matching a stop sequence.
    StopSequence,
    /// Model requested execution of a tool function.
    ToolUse,
    /// Output suppressed by safety / content filter.
    ContentFilter,
    /// Generation aborted due to an error.
    Error,
}

/// Generic inference request dispatched to any provider backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceRequest {
    /// Target model identifier (e.g. "claude-3-7-sonnet-20250219", "gpt-4o").
    pub model: String,
    /// Ordered conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Optional system steering instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Sampling temperature (0.0 to 2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Top-p nucleus sampling cutoff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Maximum output tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences that halt generation.
    #[serde(default)]
    pub stop_sequences: Vec<String>,
    /// Available tool definitions.
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    /// Whether to stream the response chunk-by-chunk.
    #[serde(default)]
    pub stream: bool,
    /// Reasoning effort level ("low", "medium", "high") for reasoning models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Optional provider specific configuration overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_config: Option<ProviderConfig>,
    /// Optional privacy and secret redaction filter settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_filter: Option<PrivacyFilter>,
}

fn default_temperature() -> f32 {
    0.7
}

impl InferenceRequest {
    /// Constructs a basic inference request.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            system_prompt: None,
            temperature: 0.7,
            top_p: None,
            max_tokens: None,
            stop_sequences: Vec::new(),
            tools: Vec::new(),
            stream: false,
            reasoning_effort: None,
            provider_config: None,
            privacy_filter: None,
        }
    }

    /// Sets system steering prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Sets sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Sets maximum output tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Enables or disables streaming.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Sets tool definitions.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    /// Sets reasoning effort.
    pub fn with_reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }
}

/// Unified response returned from non-streaming inference completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceResponse {
    /// Unique response identifier from provider.
    pub id: String,
    /// Concrete model name that generated the response.
    pub model: String,
    /// Generated content blocks (text, thinking, tool calls).
    pub content: Vec<ContentBlock>,
    /// Reason why generation stopped.
    pub stop_reason: StopReason,
    /// Token usage and timing metrics.
    pub metrics: TokenMetrics,
}

impl InferenceResponse {
    /// Extracts all concatenated text content from text blocks.
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extracts reasoning content from thinking blocks if present.
    pub fn thinking_content(&self) -> Option<String> {
        let thoughts = self
            .content
            .iter()
            .filter_map(|block| block.as_thinking())
            .collect::<Vec<_>>();
        if thoughts.is_empty() {
            None
        } else {
            Some(thoughts.join("\n"))
        }
    }

    /// Extracts all tool use invocations from content blocks.
    pub fn tool_uses(&self) -> Vec<(&str, &str, &serde_json::Value)> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.as_str(), name.as_str(), input))
                }
                _ => None,
            })
            .collect()
    }

    /// Returns true if execution completed successfully without errors.
    pub fn is_success(&self) -> bool {
        !matches!(self.stop_reason, StopReason::Error)
    }
}

/// Fine-grained delta payload within a streaming chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunkDelta {
    /// Text token delta.
    TextDelta { text: String },
    /// Thinking / CoT reasoning token delta.
    ThinkingDelta { reasoning: String },
    /// Partial tool call delta.
    ToolCallDelta {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        arguments_delta: String,
    },
}

/// Discrete chunk yielded during token streaming.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    /// Monotonically increasing chunk sequence index (0-indexed).
    pub chunk_index: u64,
    /// Incremental content delta.
    pub delta: StreamChunkDelta,
    /// Present on terminating chunk indicating stop reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    /// Partial or final token metrics if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_metrics: Option<TokenMetrics>,
}

impl StreamChunk {
    /// Constructs a text streaming chunk.
    pub fn text(chunk_index: u64, text: impl Into<String>) -> Self {
        Self {
            chunk_index,
            delta: StreamChunkDelta::TextDelta { text: text.into() },
            stop_reason: None,
            partial_metrics: None,
        }
    }

    /// Constructs a thinking streaming chunk.
    pub fn thinking(chunk_index: u64, reasoning: impl Into<String>) -> Self {
        Self {
            chunk_index,
            delta: StreamChunkDelta::ThinkingDelta {
                reasoning: reasoning.into(),
            },
            stop_reason: None,
            partial_metrics: None,
        }
    }

    /// Constructs a tool call delta chunk.
    pub fn tool_call(
        chunk_index: u64,
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: impl Into<String>,
    ) -> Self {
        Self {
            chunk_index,
            delta: StreamChunkDelta::ToolCallDelta {
                index,
                id,
                name,
                arguments_delta: arguments_delta.into(),
            },
            stop_reason: None,
            partial_metrics: None,
        }
    }
}

/// Provider connection and authentication configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// Provider family.
    pub provider: ProviderKind,
    /// Optional API key or bearer token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Custom base URL endpoint (e.g. "http://localhost:11434/v1" or custom proxy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Optional organization or project ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// Request timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Maximum retry attempts on transient errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Extra HTTP headers to include with requests.
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

impl ProviderConfig {
    /// Constructs a default configuration for a provider.
    pub fn new(provider: ProviderKind) -> Self {
        Self {
            provider,
            api_key: None,
            base_url: None,
            organization: None,
            timeout_ms: Some(30_000),
            max_retries: Some(3),
            extra_headers: HashMap::new(),
        }
    }

    /// Sets API key.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets custom base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets request timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Pre-flight privacy and air-gap filter rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyFilter {
    /// Whether privacy filtering is enabled.
    pub enabled: bool,
    /// Redact personal identifiable information (IPs, emails).
    pub redact_pii: bool,
    /// Redact API keys, AWS secrets, and JWT tokens.
    pub redact_secrets: bool,
    /// Force 100% air-gapped local model processing (reject cloud egress).
    pub air_gap_mode: bool,
    /// Custom regex patterns to redact.
    #[serde(default)]
    pub custom_redaction_patterns: Vec<String>,
}

impl Default for PrivacyFilter {
    fn default() -> Self {
        Self {
            enabled: true,
            redact_pii: true,
            redact_secrets: true,
            air_gap_mode: false,
            custom_redaction_patterns: Vec::new(),
        }
    }
}

impl PrivacyFilter {
    /// Strict privacy configuration with air-gap mode enabled.
    pub fn air_gapped() -> Self {
        Self {
            enabled: true,
            redact_pii: true,
            redact_secrets: true,
            air_gap_mode: true,
            custom_redaction_patterns: Vec::new(),
        }
    }

    /// Disabled privacy filter (pass-through mode).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            redact_pii: false,
            redact_secrets: false,
            air_gap_mode: false,
            custom_redaction_patterns: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::System.as_str(), "system");
        assert_eq!(Role::User.as_str(), "user");
        assert_eq!(Role::Assistant.as_str(), "assistant");
        assert_eq!(Role::Tool.as_str(), "tool");
    }

    #[test]
    fn test_content_block_helpers() {
        let img = ContentBlock::image("image/png", "aGVsbG8=");
        assert_eq!(img.as_text(), None);
        assert_eq!(img.as_thinking(), None);

        let res = ContentBlock::tool_result("tc_1", "out", true);
        match res {
            ContentBlock::ToolResult { is_error, .. } => assert!(is_error),
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_inference_response_empty_thinking() {
        let resp = InferenceResponse {
            id: "r1".into(),
            model: "m1".into(),
            content: vec![ContentBlock::text("answer")],
            stop_reason: StopReason::EndTurn,
            metrics: TokenMetrics::default(),
        };
        assert_eq!(resp.thinking_content(), None);
        assert_eq!(resp.tool_uses().len(), 0);
        assert!(resp.is_success());
    }
}

