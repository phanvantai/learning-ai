//! The API-key provider's tool loop, against a fake Messages API.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use lai_harness::anthropic::AnthropicSession;
use lai_harness::approval::{ApprovalStore, Queue, Status};
use lai_harness::config::{Config, ProviderConfig};
use lai_harness::tools::{Risk, Tool, ToolRegistry};
use lai_harness::Session;
use serde_json::{json, Value};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Fake {
    name: &'static str,
    risk: Risk,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Tool for Fake {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "fake"
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object"})
    }
    fn risk(&self) -> Risk {
        self.risk
    }
    async fn call(&self, _input: Value) -> anyhow::Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok("42 thư chưa đọc".into())
    }
}

fn reply(stop_reason: &str, content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "id": "msg_1", "type": "message", "role": "assistant",
        "content": content, "stop_reason": stop_reason,
    }))
}

#[tokio::test]
async fn runs_read_tools_and_queues_write_tools() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .respond_with(reply(
            "tool_use",
            json!([
                {"type": "thinking", "thinking": "", "signature": "sig"},
                {"type": "tool_use", "id": "t1", "name": "lookup", "input": {}},
                {"type": "tool_use", "id": "t2", "name": "send", "input": {"to": "a@b.c"}},
            ]),
        ))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(reply("end_turn", json!([{"type": "text", "text": "Xong."}])))
        .mount(&server)
        .await;

    let read_calls = Arc::new(AtomicUsize::new(0));
    let write_calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::default();
    registry.add(Arc::new(Fake { name: "lookup", risk: Risk::ReadOnly, calls: read_calls.clone() }));
    registry.add(Arc::new(Fake { name: "send", risk: Risk::Write, calls: write_calls.clone() }));

    let dir = tempfile::tempdir().unwrap();
    let queue_path = dir.path().join("approvals.json");
    let cfg = Config::parse(&format!(
        "[providers.api]\nkind = \"anthropic\"\nbase_url = \"{}\"\n",
        server.uri()
    ))
    .unwrap();
    let ProviderConfig::Anthropic(provider) = &cfg.providers["api"] else { unreachable!() };
    let mut session = AnthropicSession::new(
        provider,
        "test-key".into(),
        "tro-ly",
        "system",
        Arc::new(registry),
        Arc::new(Queue(ApprovalStore::at(queue_path.clone()))),
    )
    .unwrap();

    assert_eq!(session.send("kiểm tra mail").await.unwrap(), "Xong.");

    // The read tool ran; the write tool did not, and is waiting in the queue.
    assert_eq!(read_calls.load(Ordering::SeqCst), 1);
    assert_eq!(write_calls.load(Ordering::SeqCst), 0);
    let queued = ApprovalStore::at(queue_path).all().unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!((queued[0].tool.as_str(), &queued[0].status), ("send", &Status::Pending));
    assert_eq!(queued[0].input, json!({"to": "a@b.c"}));

    // Second request: assistant content echoed unchanged, both results in one user message.
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let second: Value = requests[1].body_json().unwrap();
    let messages = second["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1]["content"][0]["signature"], "sig");
    let results = messages[2]["content"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["content"], "42 thư chưa đọc");
    assert!(results[1]["content"].as_str().unwrap().contains("CHƯA THỰC HIỆN"));
    assert_eq!(second["tools"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn failed_turn_leaves_conversation_usable() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({"error": {"message": "bad thing"}})))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(reply("end_turn", json!([{"type": "text", "text": "ok"}])))
        .mount(&server)
        .await;

    let cfg = Config::parse(&format!(
        "[providers.api]\nkind = \"anthropic\"\nbase_url = \"{}\"\nfallbacks = false\n",
        server.uri()
    ))
    .unwrap();
    let ProviderConfig::Anthropic(provider) = &cfg.providers["api"] else { unreachable!() };
    let dir = tempfile::tempdir().unwrap();
    let mut session = AnthropicSession::new(
        provider,
        "test-key".into(),
        "a",
        "system",
        Arc::new(ToolRegistry::default()),
        Arc::new(Queue(ApprovalStore::at(dir.path().join("q.json")))),
    )
    .unwrap();

    let err = session.send("một").await.unwrap_err();
    assert!(err.to_string().contains("bad thing"), "{err}");
    assert_eq!(session.send("hai").await.unwrap(), "ok");

    let requests = server.received_requests().await.unwrap();
    let second: Value = requests[1].body_json().unwrap();
    assert_eq!(second["messages"], json!([{"role": "user", "content": "hai"}]));
    assert!(second.get("fallbacks").is_none());
    assert!(second.get("tools").is_none());
}
