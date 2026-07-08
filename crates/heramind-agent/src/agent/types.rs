//! Agent types - Events, Messages, Responses, and Configuration.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use heramind_core::{
    config::{agent_env_vars, endpoints, models},
    Message,
};

/// Agent event emitted during streaming processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Thinking/reasoning content chunk
    Thinking {
        /// Content chunk
        content: String,
    },
    /// Actual response content chunk
    Content {
        /// Content chunk
        content: String,
    },
    /// Tool call is starting
    ToolCallStart {
        /// Tool name
        tool: String,
        /// Tool arguments
        arguments: Value,
        /// Round number (1-based) for multi-round tool calling
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
    },
    /// Tool call completed with result
    ToolCallEnd {
        /// Tool name
        tool: String,
        /// Result (success or error)
        result: String,
        /// Whether it succeeded
        success: bool,
        /// Round number (1-based) for multi-round tool calling
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
    },
    /// Error occurred
    Error {
        /// Error message
        message: String,
    },
    /// Warning message (non-fatal)
    Warning {
        /// Warning message
        message: String,
    },
    /// Stream ended
    End {
        /// Prompt tokens used in this request (from LLM backend)
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt_tokens: Option<u32>,
    },
    /// Intermediate end (for multi-round tool calling)
    /// Indicates the current round is complete but more processing is coming
    IntermediateEnd,
    /// Intent classification result
    Intent {
        /// Intent category (e.g., "Device", "Rule", "Data")
        category: String,
        /// Display name (e.g., "设备管理", "自动化规则")
        display_name: String,
        /// Confidence score
        #[serde(skip_serializing_if = "Option::is_none")]
        confidence: Option<f32>,
        /// Keywords that matched
        #[serde(skip_serializing_if = "Option::is_none")]
        keywords: Option<Vec<String>>,
    },
    /// Execution plan step
    Plan {
        /// Step description
        step: String,
        /// Stage name
        stage: String,
    },
    /// Heartbeat to keep connection alive
    Heartbeat {
        /// Timestamp when heartbeat was sent
        timestamp: i64,
    },
    /// Progress update
    Progress {
        /// Progress message
        message: String,
        /// Stage name
        #[serde(skip_serializing_if = "Option::is_none")]
        stage: Option<String>,
        /// Elapsed time in ms
        #[serde(rename = "elapsedMs", skip_serializing_if = "Option::is_none")]
        elapsed_ms: Option<u64>,
    },
}

impl AgentEvent {
    /// Create a thinking chunk event.
    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    /// Create a content chunk event.
    pub fn content(content: impl AsRef<str>) -> Self {
        Self::Content {
            content: content.as_ref().to_string(),
        }
    }

    /// Create a tool call start event.
    pub fn tool_call_start(tool: impl Into<String>, arguments: Value) -> Self {
        Self::ToolCallStart {
            tool: tool.into(),
            arguments,
            round: None,
        }
    }

    /// Create a tool call end event.
    pub fn tool_call_end(
        tool: impl Into<String>,
        result: impl Into<String>,
        success: bool,
    ) -> Self {
        Self::ToolCallEnd {
            tool: tool.into(),
            result: result.into(),
            success,
            round: None,
        }
    }

    /// Create a tool call start event with round number.
    pub fn tool_call_start_round(tool: impl Into<String>, arguments: Value, round: usize) -> Self {
        Self::ToolCallStart {
            tool: tool.into(),
            arguments,
            round: Some(round),
        }
    }

    /// Create a tool call end event with round number.
    pub fn tool_call_end_round(
        tool: impl Into<String>,
        result: impl Into<String>,
        success: bool,
        round: usize,
    ) -> Self {
        Self::ToolCallEnd {
            tool: tool.into(),
            result: result.into(),
            success,
            round: Some(round),
        }
    }

    /// Create an error event.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    /// Create a warning event.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::Warning {
            message: message.into(),
        }
    }

    /// Create an end event.
    pub fn end() -> Self {
        Self::End {
            prompt_tokens: None,
        }
    }

    /// Create an end event with token usage data.
    pub fn end_with_tokens(prompt_tokens: u32) -> Self {
        Self::End {
            prompt_tokens: Some(prompt_tokens),
        }
    }

    /// Create an intent event.
    pub fn intent(
        category: impl Into<String>,
        display_name: impl Into<String>,
        confidence: impl Into<Option<f32>>,
        keywords: impl Into<Option<Vec<String>>>,
    ) -> Self {
        Self::Intent {
            category: category.into(),
            display_name: display_name.into(),
            confidence: confidence.into(),
            keywords: keywords.into(),
        }
    }

    /// Create a plan event.
    pub fn plan(step: impl Into<String>, stage: impl Into<String>) -> Self {
        Self::Plan {
            step: step.into(),
            stage: stage.into(),
        }
    }

    /// Create a heartbeat event.
    pub fn heartbeat() -> Self {
        Self::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a progress event.
    pub fn progress(message: impl Into<String>, stage: impl Into<String>, elapsed: u64) -> Self {
        Self::Progress {
            message: message.into(),
            stage: Some(stage.into()),
            elapsed_ms: Some(elapsed),
        }
    }

    /// Check if this event ends the stream.
    pub fn is_end(&self) -> bool {
        matches!(self, Self::End { .. })
    }

    /// Check if this event is any kind of end (End or IntermediateEnd).
    pub fn is_any_end(&self) -> bool {
        matches!(self, Self::End { .. } | Self::IntermediateEnd)
    }

    /// Convert to JSON for WebSocket transmission.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// System prompt
    pub system_prompt: String,
    /// Maximum tokens in context
    pub max_context_tokens: usize,
    /// Temperature for LLM
    pub temperature: f32,
    /// Enable tool calling
    pub enable_tools: bool,
    /// Enable memory
    pub enable_memory: bool,
    /// Model to use
    pub model: String,
    /// API endpoint for cloud LLM
    pub api_endpoint: Option<String>,
    /// API key for cloud LLM
    pub api_key: Option<String>,
    /// Maximum tool calls per request (default: 5)
    #[serde(default = "default_max_tool_calls")]
    pub max_tool_calls: usize,
    /// Number of recent tool results to keep intact (default: 2)
    #[serde(default = "default_keep_tool_results")]
    pub keep_recent_tool_results: usize,
}

/// Default value for max tool calls per request.
fn default_max_tool_calls() -> usize {
    100
}

/// Default value for keep recent tool results.
fn default_keep_tool_results() -> usize {
    2
}

/// Generate the default system prompt with language adaptation support.
/// The prompt instructs the LLM to respond in the same language as the user.
fn default_system_prompt() -> String {
    use crate::prompts::PromptBuilder;
    PromptBuilder::new()
        .with_thinking(true)
        .build_system_prompt()
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "HeraMind Agent".to_string(),
            system_prompt: default_system_prompt(),
            // Load from environment variables with fallback to defaults
            max_context_tokens: agent_env_vars::max_context_tokens(),
            temperature: agent_env_vars::temperature(),
            enable_tools: true,
            enable_memory: true,
            model: models::OLLAMA_DEFAULT.to_string(),
            api_endpoint: std::env::var("OLLAMA_ENDPOINT")
                .ok()
                .or_else(|| std::env::var("OPENAI_ENDPOINT").ok())
                .or_else(|| Some(endpoints::OLLAMA.to_string())),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            max_tool_calls: default_max_tool_calls(),
            keep_recent_tool_results: default_keep_tool_results(),
        }
    }
}

/// User message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Role (user, assistant, system, tool)
    pub role: String,
    /// Content — Arc<str> for cheap cloning across context windows
    pub content: std::sync::Arc<str>,
    /// Tool calls (if any)
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool call ID (for tool responses)
    pub tool_call_id: Option<String>,
    /// Tool call name (for tracking which tool was called)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    /// Thinking content (for AI reasoning process)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// Images attached to the message (base64 data URLs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<AgentMessageImage>>,
    /// Per-round intermediate text for multi-round tool calling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_contents: Option<Value>,
    /// Per-round thinking content for grouped rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_thinking: Option<Value>,
    /// Timestamp
    pub timestamp: i64,
}

/// An image attached to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageImage {
    /// Base64 data URL (e.g., "data:image/png;base64,...")
    pub data: String,
    /// MIME type (e.g., "image/png")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl AgentMessage {
    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a user message with images.
    pub fn user_with_images(content: impl Into<String>, images: Vec<AgentMessageImage>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: Some(images),
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create an assistant message with thinking.
    pub fn assistant_with_thinking(
        content: impl Into<String>,
        thinking: impl Into<String>,
    ) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: Some(thinking.into()),
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: Some(tool_name.into()),
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into().into(),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create an assistant message with tool calls and thinking.
    pub fn assistant_with_tools_and_thinking(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
        thinking: impl Into<String>,
    ) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into().into(),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            tool_call_name: None,
            thinking: Some(thinking.into()),
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Convert to core Message.
    /// IMPORTANT: When tool_calls exist, include them in content for LLM context.
    /// This ensures the model knows what tools were called in previous turns.
    /// IMPORTANT: Images are preserved for multimodal context in follow-up requests.
    pub fn to_core(&self) -> Message {
        match self.role.as_str() {
            "user" => {
                // Check if this message has images - create multimodal message
                if let Some(ref images) = self.images {
                    if !images.is_empty() {
                        return self.to_core_multimodal();
                    }
                }
                Message::user(self.content.as_ref())
            }
            "assistant" => {
                let mut content = self.content.to_string();
                // When tool_calls exist and content is empty, inject a concise summary
                // so the LLM knows what tools were called in previous turns.
                if let Some(ref tool_calls) = self.tool_calls {
                    if !tool_calls.is_empty() && content.is_empty() {
                        let summaries: Vec<String> = tool_calls
                            .iter()
                            .map(|tc| {
                                let args_summary = summarize_tool_args(&tc.name, &tc.arguments);
                                let result_preview = tc
                                    .result
                                    .as_ref()
                                    .map(|r| {
                                        if let Some(s) = r.as_str() {
                                            s.chars().take(120).collect::<String>()
                                        } else {
                                            let s = r.to_string();
                                            s.chars().take(120).collect::<String>()
                                        }
                                    })
                                    .unwrap_or_default();
                                if result_preview.is_empty() {
                                    format!("the {} tool with {}", tc.name, args_summary)
                                } else {
                                    format!(
                                        "the {} tool with {} and received: {}",
                                        tc.name, args_summary, result_preview
                                    )
                                }
                            })
                            .collect();
                        content = format!("In a previous turn I called {}. These results are from earlier and should not be repeated.", summaries.join(", then "));
                    }
                }
                Message::assistant(&content)
            }
            "system" => Message::system(self.content.as_ref()),
            "tool" => {
                // Tool result messages - include which tool was called
                if let Some(ref tool_name) = self.tool_call_name {
                    let tool_content = format!("[Tool: {} returned]\n{}", tool_name, self.content);
                    Message::user(&tool_content)
                } else {
                    Message::user(self.content.as_ref())
                }
            }
            _ => Message::user(self.content.as_ref()),
        }
    }

    /// Convert to core Message with multimodal content (text + images).
    /// This preserves image context for follow-up LLM requests.
    fn to_core_multimodal(&self) -> Message {
        use heramind_core::{Content, ContentPart, MessageRole};

        let images = match &self.images {
            Some(imgs) if !imgs.is_empty() => imgs,
            _ => return Message::user(self.content.as_ref()),
        };

        // Build content parts: text + images
        let mut parts = vec![ContentPart::text(self.content.to_string())];

        for image in images {
            // Prefer the mime type stored with the message (set at upload time).
            // If missing, parse from the data URL header or infer from magic bytes.
            let parsed = crate::image_utils::parse_image_data(&image.data);
            let mime_type = image
                .mime_type
                .clone()
                .or_else(|| parsed.map(|p| p.mime_type.to_string()))
                .unwrap_or_else(|| "image/png".to_string());
            let base64_data = parsed.map(|p| p.base64).unwrap_or(image.data.as_str());

            parts.push(ContentPart::image_base64(base64_data, mime_type));
        }

        Message::new(MessageRole::User, Content::Parts(parts))
    }

    /// Convert from core Message.
    pub fn from_core(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.as_text().into(),
            tool_calls: None,
            tool_call_id: None,
            tool_call_name: None,
            thinking: None,
            images: None,
            round_contents: None,
            round_thinking: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Tool call from LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Call ID
    pub id: String,
    /// Arguments
    pub arguments: Value,
    /// Execution result (populated after tool execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Round number for multi-round tool calling (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<usize>,
}

/// Agent response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Response message
    pub message: AgentMessage,
    /// Tool calls made (if any)
    pub tool_calls: Vec<ToolCall>,
    /// Memory used
    pub memory_context_used: bool,
    /// Tools used
    pub tools_used: Vec<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub id: String,
    /// Started at
    pub started_at: i64,
    /// Last activity
    pub last_activity: i64,
    /// Message count
    pub message_count: usize,
    /// Metadata
    pub metadata: Value,
}

impl SessionState {
    /// Create a new session state.
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            started_at: now,
            last_activity: now,
            message_count: 0,
            metadata: Value::Object(serde_json::Map::new()),
        }
    }

    /// Update activity.
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now().timestamp();
    }

    /// Increment message count.
    pub fn increment_messages(&mut self) {
        self.message_count += 1;
        self.touch();
    }
}

/// Find the array of items from a tool response JSON.
/// Tries: `val[list_key]` → `val["data"][list_key]` → `val["data"]` (if array) → `val` (if array)
fn find_json_list<'a>(
    val: &'a serde_json::Value,
    list_key: &str,
) -> Option<&'a Vec<serde_json::Value>> {
    val.get(list_key)
        .and_then(|v| v.as_array())
        .or_else(|| {
            val.get("data")
                .and_then(|d| d.get(list_key))
                .and_then(|v| v.as_array())
        })
        .or_else(|| val.get("data").and_then(|v| v.as_array()))
        .or_else(|| val.as_array())
}

/// Summarize tool call arguments into a concise string.
/// E.g., `{"action":"list"}` → `"list"`,
///       `{"action":"history","device_id":"b626d0","metric":"battery"}` → `"history b626d0/battery"`
pub fn summarize_tool_args(tool_name: &str, args: &Value) -> String {
    let obj = match args.as_object() {
        Some(o) => o,
        None => return args.to_string().chars().take(80).collect(),
    };

    // Extract the action field if present
    let action = obj.get("action").and_then(|v| v.as_str()).unwrap_or("");

    match tool_name {
        "device" => match action {
            "list" => "list".to_string(),
            "get" | "query" | "history" => {
                let device_id = obj.get("device_id").and_then(|v| v.as_str()).unwrap_or("?");
                let metric = obj.get("metric").and_then(|v| v.as_str()).unwrap_or("");
                if metric.is_empty() {
                    format!("{} {}", action, device_id)
                } else {
                    format!("{} {}/{}", action, device_id, metric)
                }
            }
            _ => action.to_string(),
        },
        "agent" => match action {
            "list" => "list".to_string(),
            _ => action.to_string(),
        },
        _ => {
            // Generic: show action + up to 2 key params
            let mut parts = vec![];
            if !action.is_empty() {
                parts.push(action.to_string());
            }
            for (k, v) in obj.iter() {
                if k == "action" {
                    continue;
                }
                if parts.len() >= 3 {
                    break;
                }
                if let Some(s) = v.as_str() {
                    parts.push(format!("{}={}", k, s.chars().take(20).collect::<String>()));
                }
            }
            if parts.is_empty() {
                "?".to_string()
            } else {
                parts.join(",")
            }
        }
    }
}

/// Cached large tool result data (images, big base64 blobs, etc.).
/// This data is stored outside LLM message history to prevent token bloat.
#[derive(Debug, Clone)]
pub struct CachedLargeResult {
    /// MIME type or content description (e.g., "image/jpeg", "application/json")
    pub content_type: String,
    /// Full data (base64 for images, raw string for JSON/text)
    pub data: String,
    /// Size of the data in bytes
    pub size_bytes: usize,
    /// Timestamp when cached
    pub cached_at: i64,
}

/// Cache for large tool results that must not enter LLM context.
/// Stores full data keyed by tool_name. Data is retrieved only when
/// needed — for tool calls that accept base64 or multimodal LLM calls.
#[derive(Debug, Clone, Default)]
pub struct LargeDataCache {
    entries: std::collections::HashMap<String, CachedLargeResult>,
}

/// Threshold below which we don't cache — just pass through.
/// Set to 32KB: compact time-series format (2000 points ≈ 14KB) should reach LLM intact.
/// Only truly large payloads (images, huge JSON) get cached with a summary reference.
const CACHE_THRESHOLD_BYTES: usize = 32 * 1024;

/// Maximum number of cache entries. Oldest entries are evicted when exceeded.
const MAX_CACHE_ENTRIES: usize = 20;

/// Maximum total cache size in bytes (50MB). Oldest entries are evicted when exceeded.
const MAX_CACHE_TOTAL_BYTES: usize = 50 * 1024 * 1024;

impl LargeDataCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evict oldest entries if cache exceeds limits.
    fn evict_if_needed(&mut self) {
        // Evict by count
        while self.entries.len() > MAX_CACHE_ENTRIES {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                tracing::debug!("Evicting cache entry '{}' (count limit)", oldest_key);
                self.entries.remove(&oldest_key);
            } else {
                break;
            }
        }

        // Evict by total size
        let total_size: usize = self.entries.values().map(|e| e.size_bytes).sum();
        if total_size > MAX_CACHE_TOTAL_BYTES {
            // Sort by age (oldest first) and evict until under budget
            let mut sorted: Vec<(String, i64)> = self
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.cached_at))
                .collect();
            sorted.sort_by_key(|(_, ts)| *ts);

            let mut current_size = total_size;
            for (key, _) in sorted {
                if current_size <= MAX_CACHE_TOTAL_BYTES {
                    break;
                }
                if let Some(entry) = self.entries.remove(&key) {
                    tracing::debug!(
                        "Evicting cache entry '{}' (size limit, freed {} bytes)",
                        key,
                        entry.size_bytes
                    );
                    current_size -= entry.size_bytes;
                }
            }
        }
    }

    /// Store a tool result. Returns a summary string to use in message history.
    /// - Small results (<32KB): not cached, returned as-is
    /// - Large/image/base64 results: cached, returns summary with reference key
    pub fn store(&mut self, tool_name: &str, result: &str) -> String {
        let size_bytes = result.len();

        // Small results: pass through unchanged
        if size_bytes < CACHE_THRESHOLD_BYTES {
            return result.to_string();
        }

        // Detect content type
        let content_type = Self::detect_content_type(result);

        let cached = CachedLargeResult {
            content_type: content_type.clone(),
            data: result.to_string(),
            size_bytes,
            cached_at: chrono::Utc::now().timestamp(),
        };
        self.entries.insert(tool_name.to_string(), cached);

        // Evict oldest entries if cache exceeds size/count limits
        self.evict_if_needed();

        // Return a concise summary for message history — NO raw base64 preview
        let human_size = Self::humanize_bytes(size_bytes);
        let cached_ref = format!("$cached:{}", tool_name);
        if content_type.starts_with("image/") || content_type == "application/json+base64" {
            // Show JSON structure without base64 content
            let structure_preview = Self::describe_structure(result);
            format!(
                "[Image data, {}. Use \"{}\" to reference this data in subsequent tool calls. Structure: {}]",
                human_size, cached_ref, structure_preview
            )
        } else if content_type == "application/json" {
            // JSON data: extract key summary values for LLM context
            let summary = Self::smart_json_summary(tool_name, result, 500);
            format!(
                "[Data: {}, {}. Use \"{}\" to reference. {}]",
                content_type, human_size, cached_ref, summary
            )
        } else {
            let preview: String = result.chars().take(300).collect();
            format!(
                "[Data: {}, {}. Use \"{}\" to reference. Preview: {}]",
                content_type, human_size, cached_ref, preview
            )
        }
    }

    /// Get the most recently cached image-like data entry.
    /// Used for auto-injection when the LLM fails to pass valid image arguments.
    /// Returns the extracted image data (base64) and the cache key.
    pub fn get_latest_image(&self) -> Option<(String, String)> {
        // Priority 1: user-uploaded images
        if let Some(user_img) = self.entries.get("user_image") {
            return Some((
                Self::extract_image_data(&user_img.data),
                "user_image".to_string(),
            ));
        }
        // Priority 2: most recent image-type cached entry by timestamp
        let mut best: Option<(&String, &CachedLargeResult)> = None;
        for (key, entry) in &self.entries {
            let is_image = entry.content_type.starts_with("image/")
                || entry.content_type == "application/json+base64";
            if is_image {
                let is_better = match best {
                    None => true,
                    Some((_, prev)) => entry.cached_at > prev.cached_at,
                };
                if is_better {
                    best = Some((key, entry));
                }
            }
        }
        best.map(|(key, entry)| (Self::extract_image_data(&entry.data), key.clone()))
    }

    /// Detect content type from content heuristics.
    fn detect_content_type(content: &str) -> String {
        // Check for data URL images (data:image/png;base64,...)
        if content.starts_with("data:image/") {
            // Extract MIME from data URL
            let end = content.find(';').unwrap_or(15);
            return content[..end].trim_start_matches("data:").to_string();
        }
        // Check for raw base64 image data (long string of base64 chars)
        if content.len() > 10_000 && Self::looks_like_base64(content) {
            return "image/base64".to_string();
        }
        // Check for JSON with embedded base64 image
        if content.starts_with('{') || content.starts_with('[') {
            if content.contains("base64,")
                || content.contains("\"data:image/")
                || content.contains("\"base64_data\"")
                || content.contains("\"data_type\":\"image\"")
            {
                return "application/json+base64".to_string();
            }
            return "application/json".to_string();
        }
        "text/plain".to_string()
    }

    /// Quick heuristic: does this look like base64 data?
    fn looks_like_base64(s: &str) -> bool {
        // Sample first 200 chars (NOT bytes — byte slicing panics on
        // multi-byte UTF-8, see sanitize.rs fix for the same bug).
        let sample: String = s.chars().take(200).collect();
        sample.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '\n' || c == '\r'
        })
    }

    /// Resolve a $cached reference to the actual data.
    /// Format: "$cached:tool_name" — extracts the relevant image/base64 data
    /// from the cached tool result. Returns the full cached data as fallback.
    pub fn resolve_reference(&self, reference: &str) -> Option<String> {
        let tool_name = reference.strip_prefix("$cached:")?;
        let cached = self.entries.get(tool_name)?;
        Some(Self::extract_image_data(&cached.data))
    }

    /// Extract image/base64 data from a cached result string.
    /// Recursively walks the JSON tree to find base64_data regardless of nesting depth,
    /// so it works for any tool response structure (device get, query, extensions, etc.).
    fn extract_image_data(data: &str) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(found) = Self::find_base64_in_value(&json) {
                return found;
            }
        }
        // Fallback: return raw data (works for pure base64)
        data.to_string()
    }

    /// Recursively search a JSON value for base64 image data.
    /// Priority: base64_data fields > data:image URLs > large string values.
    fn find_base64_in_value(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::Object(map) => {
                // Direct base64_data field (any depth)
                if let Some(b64) = map.get("base64_data").and_then(|v| v.as_str()) {
                    return Some(b64.to_string());
                }
                // data:image URL (extract base64 portion)
                for key in &["data", "image", "content", "url"] {
                    if let Some(s) = map.get(*key).and_then(|v| v.as_str()) {
                        if let Some(b64) = s.strip_prefix("data:image/") {
                            if let Some(after_comma) = b64.split(',').nth(1) {
                                return Some(after_comma.to_string());
                            }
                        }
                        // Large string likely to be raw base64 image.
                        // Require substantial size (>10KB) and valid base64 alphabet to avoid
                        // false positives on non-image base64 data (certificates, tokens, etc.).
                        if s.len() > 10000
                            && s.len() % 4 <= 1 // base64 length is always 4n, 4n+2, 4n+3
                            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
                        {
                            return Some(s.to_string());
                        }
                    }
                }
                // Recurse into child values
                for v in map.values() {
                    if let Some(found) = Self::find_base64_in_value(v) {
                        return Some(found);
                    }
                }
                None
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    if let Some(found) = Self::find_base64_in_value(v) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Describe JSON structure without exposing raw base64 content.
    /// Shows field names, types, and sizes instead of actual data.
    fn describe_structure(data: &str) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
            Self::describe_value(&json, 0)
        } else {
            format!("<{} bytes>", data.len())
        }
    }

    /// Generate a smart summary of JSON data preserving key identifiers.
    /// Falls back to structure description if data shape is unrecognized.
    fn smart_json_summary(tool_name: &str, data: &str, max_len: usize) -> String {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
            let summary = match tool_name {
                "device" => Self::extract_device_summary(&val),
                "agent" => Self::extract_agent_summary(&val),
                "rule" => Self::extract_rule_summary(&val),
                "message" | "alert" => Self::extract_message_summary(&val),
                "extension" => Self::extract_extension_summary(&val),
                _ => Self::generic_json_summary(&val),
            };
            // Truncate to max_len
            let chars: Vec<char> = summary.chars().collect();
            if chars.len() > max_len {
                format!("{}...", chars[..max_len].iter().collect::<String>())
            } else {
                summary
            }
        } else {
            // Not valid JSON, just truncate
            let s: String = data.chars().take(max_len).collect();
            format!("Preview: {}", s)
        }
    }

    /// Extract device names and IDs from device tool results.
    /// E.g., "6 devices: NE101-Shelf(b62730), NE101-Refrigerator(b65020), ..."
    fn extract_device_summary(val: &serde_json::Value) -> String {
        Self::extract_list_summary(
            val,
            "devices",
            &["name", "device_name"],
            &["id", "device_id"],
            "devices",
        )
    }

    /// Extract agent names and IDs from agent tool results.
    fn extract_agent_summary(val: &serde_json::Value) -> String {
        Self::extract_list_summary(
            val,
            "agents",
            &["name", "agent_name"],
            &["id", "agent_id"],
            "agents",
        )
    }

    /// Extract rule names and IDs from rule tool results.
    fn extract_rule_summary(val: &serde_json::Value) -> String {
        Self::extract_list_summary(val, "rules", &["name"], &["id", "rule_id"], "rules")
    }

    /// Generic list summary extractor.
    /// Looks for an array under `list_key` (or "data"), extracts name+id pairs.
    fn extract_list_summary(
        val: &serde_json::Value,
        list_key: &str,
        name_keys: &[&str],
        id_keys: &[&str],
        label: &str,
    ) -> String {
        let arr = find_json_list(val, list_key);
        if let Some(arr) = arr {
            let count = arr.len();
            let items: Vec<String> = arr
                .iter()
                .take(10)
                .map(|item: &serde_json::Value| {
                    let name = name_keys
                        .iter()
                        .find_map(|k| item.get(*k).and_then(|v| v.as_str()))
                        .unwrap_or("?");
                    let id = id_keys
                        .iter()
                        .find_map(|k| item.get(*k).and_then(|v| v.as_str()))
                        .unwrap_or("?");
                    format!("{}({})", name, id)
                })
                .collect();
            if items.is_empty() {
                return format!("{} {}", count, label);
            }
            let suffix = if count > 10 { ", ..." } else { "" };
            format!("{} {}: {}{}", count, label, items.join(", "), suffix)
        } else {
            Self::describe_value(val, 0)
        }
    }

    /// Extract message/alert titles and IDs from message tool results.
    fn extract_message_summary(val: &serde_json::Value) -> String {
        let arr = find_json_list(val, "messages");
        if let Some(arr) = arr {
            let count = arr.len();
            let items: Vec<String> = arr
                .iter()
                .take(10)
                .map(|item: &serde_json::Value| {
                    let title = item
                        .get("title")
                        .or_else(|| item.get("subject"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let level = item
                        .get("level")
                        .and_then(|v| v.as_str())
                        .map(|l| format!("[{}]", l))
                        .unwrap_or_default();
                    format!("{}{}", title, level)
                })
                .collect();
            if items.is_empty() {
                return format!("{} messages", count);
            }
            let suffix = if count > 10 { ", ..." } else { "" };
            format!("{} messages: {}{}", count, items.join(", "), suffix)
        } else {
            Self::describe_value(val, 0)
        }
    }

    /// Extract extension names, IDs, and state from extension tool results.
    fn extract_extension_summary(val: &serde_json::Value) -> String {
        let arr = find_json_list(val, "extensions");
        if let Some(arr) = arr {
            let count = arr.len();
            let items: Vec<String> = arr
                .iter()
                .take(10)
                .map(|item: &serde_json::Value| {
                    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let state = item
                        .get("state")
                        .and_then(|v| v.as_str())
                        .map(|s| format!(":{}", s))
                        .unwrap_or_default();
                    format!("{}({}){}", name, id, state)
                })
                .collect();
            if items.is_empty() {
                return format!("{} extensions", count);
            }
            let suffix = if count > 10 { ", ..." } else { "" };
            format!("{} extensions: {}{}", count, items.join(", "), suffix)
        } else {
            Self::describe_value(val, 0)
        }
    }

    /// Generic JSON summary: show first N items' key fields.
    fn generic_json_summary(val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Array(arr) => {
                let count = arr.len();
                if count == 0 {
                    return "[]".to_string();
                }
                // Show first 3 items concisely
                let previews: Vec<String> = arr
                    .iter()
                    .take(3)
                    .map(|item| match item {
                        serde_json::Value::Object(map) => {
                            let fields: Vec<String> = map
                                .iter()
                                .take(3)
                                .map(|(k, v)| {
                                    let vs = match v {
                                        serde_json::Value::String(s) => {
                                            s.chars().take(20).collect::<String>()
                                        }
                                        other => {
                                            other.to_string().chars().take(20).collect::<String>()
                                        }
                                    };
                                    format!("{}={}", k, vs)
                                })
                                .collect();
                            format!("{{{}}}", fields.join(", "))
                        }
                        other => other.to_string().chars().take(60).collect(),
                    })
                    .collect();
                let suffix = if count > 3 {
                    format!(", ... ({} items)", count)
                } else {
                    "".to_string()
                };
                format!("[{}{}]", previews.join(", "), suffix)
            }
            serde_json::Value::Object(map) => {
                let fields: Vec<String> = map
                    .iter()
                    .take(5)
                    .map(|(k, v)| {
                        let vs = match v {
                            serde_json::Value::String(s) if s.len() > 40 => {
                                let end = s.floor_char_boundary(40);
                                format!("{}...", &s[..end])
                            }
                            serde_json::Value::Array(a) => format!("Array({})", a.len()),
                            serde_json::Value::Object(_) => "{...}".to_string(),
                            other => other.to_string(),
                        };
                        format!("{}: {}", k, vs)
                    })
                    .collect();
                format!("{{{}}}", fields.join(", "))
            }
            other => {
                let s = other.to_string();
                s.chars().take(200).collect()
            }
        }
    }

    /// Recursively describe a JSON value's structure.
    fn describe_value(value: &serde_json::Value, depth: usize) -> String {
        if depth > 3 {
            return "{...}".to_string();
        }
        match value {
            serde_json::Value::Object(map) => {
                let fields: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        let val_desc = match v {
                            serde_json::Value::String(s) if s.len() > 100 => {
                                format!("String({} bytes)", s.len())
                            }
                            serde_json::Value::Array(arr) => {
                                format!("Array({} items)", arr.len())
                            }
                            serde_json::Value::Object(_) => Self::describe_value(v, depth + 1),
                            other => {
                                let s = other.to_string();
                                if s.len() > 50 {
                                    let end = s.floor_char_boundary(50);
                                    format!("{}...", &s[..end])
                                } else {
                                    s
                                }
                            }
                        };
                        format!("\"{}\": {}", k, val_desc)
                    })
                    .collect();
                format!("{{{}}}", fields.join(", "))
            }
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else {
                    let first = Self::describe_value(&arr[0], depth + 1);
                    format!("[{}... ({} items)]", first, arr.len())
                }
            }
            serde_json::Value::String(s) if s.len() > 100 => {
                format!("String({} bytes)", s.len())
            }
            other => other.to_string(),
        }
    }

    /// Format bytes as human-readable string.
    fn humanize_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = 1024 * KB;
        if bytes >= MB {
            format!("{:.1}MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1}KB", bytes as f64 / KB as f64)
        } else {
            format!("{}B", bytes)
        }
    }
}

/// Unified internal state for the agent.
/// Combines session state with LLM readiness and other runtime state.
#[derive(Debug, Clone)]
pub struct AgentInternalState {
    /// Session ID
    pub session_id: String,
    /// Whether LLM is ready
    pub llm_ready: bool,
    /// Session state
    pub session: SessionState,
    /// Message history for this session
    pub memory: Vec<AgentMessage>,
    /// Recent assistant response hashes (for cross-turn repetition detection)
    pub recent_response_hashes: Vec<u64>,
    /// Cache for large tool results (images, base64) that must not enter LLM context
    pub large_data_cache: LargeDataCache,
    /// Session-level tool result cache for deduplication across turns.
    /// Shared via Arc<RwLock<>> so streaming code can access it concurrently.
    pub tool_result_cache: std::sync::Arc<tokio::sync::RwLock<super::streaming::ToolResultCache>>,
    /// Cached compaction: (message_count_at_cache_time, max_tokens, compacted_messages)
    /// Invalidated when messages change or max_tokens differs.
    pub compaction_cache: Option<(usize, usize, Vec<AgentMessage>)>,
}

impl AgentInternalState {
    /// Create a new internal state.
    pub fn new(session_id: String) -> Self {
        let session = SessionState::new(session_id.clone());
        Self {
            session_id,
            llm_ready: false,
            session,
            memory: Vec::new(),
            recent_response_hashes: Vec::new(),
            large_data_cache: LargeDataCache::new(),
            tool_result_cache: std::sync::Arc::new(tokio::sync::RwLock::new(
                super::streaming::ToolResultCache::new(std::time::Duration::from_secs(300)),
            )),
            compaction_cache: None,
        }
    }

    /// Calculate similarity hash for a response content.
    fn hash_response(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Normalize content for hashing:
        // - Remove extra whitespace
        // - Convert to lowercase for case-insensitive comparison
        // - Remove common filler phrases
        let normalized = content
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let mut h = DefaultHasher::new();
        normalized.hash(&mut h);
        h.finish()
    }

    /// Register a new assistant response for repetition detection.
    pub fn register_response(&mut self, content: &str) {
        let hash = Self::hash_response(content);
        self.recent_response_hashes.push(hash);

        // Keep only the last 10 response hashes
        if self.recent_response_hashes.len() > 10 {
            self.recent_response_hashes.remove(0);
        }
    }

    /// Update LLM readiness.
    pub fn set_llm_ready(&mut self, ready: bool) {
        self.llm_ready = ready;
    }

    /// Touch the session (update activity).
    pub fn touch(&mut self) {
        self.session.touch();
    }

    /// Push a message to the memory.
    pub fn push_message(&mut self, message: AgentMessage) {
        self.memory.push(message);
        self.session.increment_messages();
    }

    /// Restore memory from a list of messages.
    pub fn restore_memory(&mut self, messages: Vec<AgentMessage>) {
        self.memory = messages;
    }

    /// Clear the memory.
    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }
}

/// LLM backend type with configuration.
#[derive(Debug, Clone)]
pub enum LlmBackend {
    /// Ollama (local)
    Ollama {
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// OpenAI-compatible API
    OpenAi {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// Anthropic API
    Anthropic {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// Google AI API
    Google {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// xAI (Grok) API
    XAi {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// Qwen (Alibaba DashScope)
    Qwen {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// DeepSeek API
    DeepSeek {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// Zhipu GLM API
    GLM {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// MiniMax API
    MiniMax {
        api_key: String,
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
    /// llama.cpp standalone server (local)
    LlamaCpp {
        endpoint: String,
        model: String,
        capabilities: Option<heramind_core::BackendCapabilities>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_event_creation() {
        let thinking = AgentEvent::thinking("test thinking");
        assert!(matches!(thinking, AgentEvent::Thinking { .. }));

        let content = AgentEvent::content("test content");
        assert!(matches!(content, AgentEvent::Content { .. }));

        let end = AgentEvent::end();
        assert!(end.is_end());
    }

    #[test]
    fn test_agent_message_creation() {
        let user_msg = AgentMessage::user("Hello");
        assert_eq!(user_msg.role, "user");
        assert_eq!(&*user_msg.content, "Hello");

        let assistant_msg = AgentMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.role, "assistant");

        let sys_msg = AgentMessage::system("You are helpful");
        assert_eq!(sys_msg.role, "system");

        let tool_msg = AgentMessage::tool_result("list_devices", "Success");
        assert_eq!(tool_msg.role, "tool");
        assert_eq!(tool_msg.tool_call_name, Some("list_devices".to_string()));
    }

    #[test]
    fn test_session_state() {
        let mut state = SessionState::new("session_1".to_string());
        assert_eq!(state.message_count, 0);

        state.increment_messages();
        assert_eq!(state.message_count, 1);

        state.touch();
        assert!(state.last_activity > 0);
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.name, "HeraMind Agent");
        assert_eq!(config.model, models::OLLAMA_DEFAULT);
        assert!(config.enable_tools);
        assert!(config.enable_memory);
    }
}
