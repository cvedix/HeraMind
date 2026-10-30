//! Exercise gateway SSE failures through the real cloud parser and chat loop.
#![cfg(feature = "cloud")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use heramind_agent::agent::{
    process_stream_events_with_safeguards, AgentEvent, AgentInternalState, AgentMessage,
    StreamSafeguards,
};
use heramind_agent::llm::{ChatConfig, LlmInterface};
use heramind_agent::llm_backends::{CloudConfig, CloudRuntime};
use heramind_agent::toolkit::ToolRegistry;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

const CONTENT: &str = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
const ERROR: &str = "data: {\"error\":{\"message\":\"LLM request rejected: content-blocked (request id: test-id)\",\"type\":\"api_error\"}}\n\ndata: [DONE]\n\n";
const DONE: &str = "data: [DONE]\n\n";

async fn run_chat(responses: Vec<String>) -> (Vec<AgentEvent>, Vec<AgentMessage>, usize) {
    let (events, memory, requests) = run_chat_with_history(responses, &[], "Hello").await;
    (events, memory, requests.len())
}

async fn run_chat_with_history(
    responses: Vec<String>,
    history: &[AgentMessage],
    user_message: &str,
) -> (Vec<AgentEvent>, Vec<AgentMessage>, Vec<serde_json::Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let server_requests = requests.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 4096];
            loop {
                let n = socket.read(&mut chunk).await.unwrap();
                assert!(n > 0);
                request.extend_from_slice(&chunk[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]);
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let header_end = request.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
            let index = {
                let mut requests = server_requests.lock().unwrap();
                let index = requests.len();
                requests.push(serde_json::from_slice(&request[header_end..]).unwrap());
                index
            };
            let body = &responses[index.min(responses.len() - 1)];
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(headers.as_bytes()).await.unwrap();
            // Split SSE JSON across HTTP reads as gateways commonly do.
            for part in body.as_bytes().chunks(13) {
                if socket.write_all(part).await.is_err() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        }
    });

    let runtime = CloudRuntime::new(
        CloudConfig::custom("test-key", format!("http://{address}/v1")).with_model("test-model"),
    )
    .unwrap();
    let interface = Arc::new(LlmInterface::new(ChatConfig::default()));
    interface.set_use_instance_manager(false).await;
    interface.set_llm(Arc::new(runtime)).await;
    let state = Arc::new(RwLock::new(AgentInternalState::new("test".into())));
    for message in history {
        state.write().await.push_message(message.clone());
    }
    state
        .write()
        .await
        .push_message(AgentMessage::user(user_message));
    let events = process_stream_events_with_safeguards(
        interface,
        state.clone(),
        Arc::new(ToolRegistry::new()),
        user_message,
        StreamSafeguards::default(),
        None,
        None,
    )
    .await
    .unwrap();
    let events = tokio::time::timeout(Duration::from_secs(5), events.collect::<Vec<_>>()).await;
    server.abort();
    let memory = state.read().await.memory.clone();
    let requests = requests.lock().unwrap().clone();
    (
        events.expect("local chat should finish promptly"),
        memory,
        requests,
    )
}

#[tokio::test]
async fn gateway_error_is_surfaced_without_empty_content_retry() {
    for prefix in ["", CONTENT] {
        let (events, memory, requests) = run_chat(vec![format!("{prefix}{ERROR}")]).await;
        let errors: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                AgentEvent::Error { message } => Some(message),
                _ => None,
            })
            .collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert!(errors[0].contains("content-blocked"));
        assert!(errors[0].contains("test-id"));
        assert_eq!(requests, 1, "a failed stream must not trigger recovery");
        assert!(matches!(events.last(), Some(AgentEvent::End { .. })));
        assert!(!events.iter().any(|event| matches!(event,
            AgentEvent::Content { content } if content.contains("Sorry, the model"))));
        let assistant: Vec<_> = memory.iter().filter(|m| m.role == "assistant").collect();
        if prefix.is_empty() {
            assert!(assistant.is_empty(), "do not persist a fabricated response");
        } else {
            assert_eq!(assistant.len(), 1);
            assert_eq!(assistant[0].content.as_ref(), "Hello");
        }
    }
}

#[tokio::test]
async fn gateway_error_during_empty_content_retry_is_preserved() {
    let (events, _, requests) = run_chat(vec![DONE.into(), ERROR.into()]).await;
    assert_eq!(requests, 2);
    assert!(events.iter().any(|event| matches!(event,
        AgentEvent::Error { message } if message.contains("content-blocked"))));
    assert!(!events.iter().any(|event| matches!(event,
        AgentEvent::Content { content } if content.contains("Sorry, the model"))));
    assert!(matches!(events.last(), Some(AgentEvent::End { .. })));
}

#[tokio::test]
async fn healthy_gateway_stream_still_returns_content() {
    let (events, memory, requests) = run_chat(vec![format!("{CONTENT}{DONE}")]).await;
    assert_eq!(requests, 1);
    assert!(!events
        .iter()
        .any(|event| matches!(event, AgentEvent::Error { .. })));
    assert!(events.iter().any(|event| matches!(event,
        AgentEvent::Content { content } if content == "Hello")));
    assert_eq!(memory.last().unwrap().content.as_ref(), "Hello");
    assert!(matches!(events.last(), Some(AgentEvent::End { .. })));
}

#[tokio::test]
async fn empty_content_retry_targets_current_turn_instead_of_first_greeting() {
    let current_message = "Tell me about the second topic.";
    let history = [AgentMessage::user("Hello"), AgentMessage::assistant("Hi!")];
    let (events, _, requests) = run_chat_with_history(
        vec![DONE.into(), format!("{CONTENT}{DONE}")],
        &history,
        current_message,
    )
    .await;
    assert_eq!(requests.len(), 2);
    let retry_messages = requests[1]["messages"].as_array().unwrap();
    assert_eq!(
        retry_messages.last().unwrap()["content"],
        format!("Please respond to the user's message directly and concisely.\n\nUser: {current_message}")
    );
    assert!(events.iter().any(|event| matches!(event,
        AgentEvent::Content { content } if content == "Hello")));
    assert!(!events
        .iter()
        .any(|event| matches!(event, AgentEvent::Error { .. })));
}
