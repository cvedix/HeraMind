//! Message compaction for the tool execution loop.
//!
//! Provides token-aware compaction (via heramind-core) with a legacy
//! count-based fallback for unknown context windows.

use heramind_core::message::{Content, Message, MessageRole};

/// Compact executor message history using token-aware compaction from heramind-core.
///
/// Falls back to the legacy count-based compaction when the context window is unknown.
pub(crate) fn compact_executor_messages(messages: &mut Vec<Message>, context_window: usize) {
    use heramind_core::llm::compaction::{compact_messages, CompactionConfig};

    // Unknown or unreasonably large context window → legacy fallback
    if context_window == 0 || context_window > 1_000_000 {
        compact_executor_messages_legacy(messages, 10);
        return;
    }

    let config = CompactionConfig::for_context_size(context_window);
    let result = compact_messages(messages, &config, context_window);

    if result.messages_removed > 0 || result.messages_truncated > 0 {
        tracing::debug!(
            original_tokens = result.original_tokens,
            compacted_tokens = result.compacted_tokens,
            removed = result.messages_removed,
            truncated = result.messages_truncated,
            "Compacted executor messages"
        );
    }

    *messages = result.messages;
}

/// Legacy count-based compaction fallback.
///
/// When the number of non-system messages exceeds `keep_recent * 2`, old tool result
/// messages are replaced with short summaries. The system prompt (first message) and
/// the most recent messages are always preserved.
pub(crate) fn compact_executor_messages_legacy(messages: &mut [Message], keep_recent: usize) {
    let non_system_count = messages
        .iter()
        .filter(|m| m.role != MessageRole::System)
        .count();

    let threshold = keep_recent * 2;
    if non_system_count <= threshold {
        return;
    }

    let to_compact = non_system_count.saturating_sub(keep_recent);
    if to_compact == 0 {
        return;
    }

    tracing::debug!(
        total_messages = messages.len(),
        non_system = non_system_count,
        to_compact,
        "Compacting executor messages (legacy)"
    );

    let mut compacted_count = 0usize;
    let mut i = 1; // Skip system prompt at index 0
    while i < messages.len() && compacted_count < to_compact {
        if messages[i].role == MessageRole::User {
            let text = messages[i].content.as_text();
            if text.starts_with("Skill guide retrieved") {
                let summary = if text.len() > 100 {
                    let preview = &text[..text.floor_char_boundary(80)];
                    format!("[Previous tool result: {}...]", preview)
                } else {
                    format!("[Previous tool result: {}]", text)
                };
                messages[i].content = Content::text(summary);
                compacted_count += 1;
            } else {
                compacted_count += 1;
            }
        } else if messages[i].role == MessageRole::Tool {
            let text = messages[i].content.as_text();
            let summary = if text.len() > 100 {
                let preview = &text[..text.floor_char_boundary(80)];
                format!("[Previous tool result: {}...]", preview)
            } else {
                format!("[Previous tool result: {}]", text)
            };
            messages[i].content = Content::text(summary);
            compacted_count += 1;
        } else if messages[i].role == MessageRole::Assistant {
            let text = messages[i].content.as_text();
            if text.len() > 200 {
                let preview = &text[..text.floor_char_boundary(100)];
                messages[i].content =
                    Content::text(format!("[Previous reasoning: {}...]", preview));
            }
            compacted_count += 1;
        }
        i += 1;
    }
}
