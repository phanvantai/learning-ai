//! The OpenAI-compatible provider's tool loop, against a fake Chat Completions API.

use std::sync::Arc;

use async_trait::async_trait;
use lai_harness::approval::{ApprovalStore, Queue};
use lai_harness::config::OpenAiConfig;
use lai_harness::openai::OpenAiSession;
use lai_harness::tools::{Risk, Tool, ToolRegistry};
use lai_harness::Session;
use serde_json::{json, Value};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Echo;

#[async_trait]
impl Tool for Echo {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "fake"
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object"})
    }
    fn risk(&self) -> Risk {
        Risk::ReadOnly
    }
    async fn call(&self, input: Value) -> anyhow::Result<String> {
        Ok(format!("đã nhận {}", input["q"]))
    }
}

#[tokio::test]
async fn runs_tool_calls_and_returns_the_final_answer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer k"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"choices": [{"message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "c1", "type": "function", "function": {"name": "echo", "arguments": "{\"q\":\"x\"}"}}],
        }, "finish_reason": "tool_calls"}]})))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"choices": [{"message": {"role": "assistant", "content": "Xong."}, "finish_reason": "stop"}]}),
        ))
        .mount(&server)
        .await;

    let mut registry = ToolRegistry::default();
    registry.add(Arc::new(Echo));
    let dir = tempfile::tempdir().unwrap();
    let cfg = OpenAiConfig { model: "m".into(), base_url: format!("{}/v1", server.uri()) };
    let mut session = OpenAiSession::new(
        &cfg,
        "p",
        Some("k".into()),
        "a",
        "quy tắc",
        Arc::new(registry),
        Arc::new(Queue(ApprovalStore::at(dir.path().join("q.json")))),
    )
    .unwrap();

    assert_eq!(session.send("chào").await.unwrap(), "Xong.");

    let requests = server.received_requests().await.unwrap();
    let first: Value = requests[0].body_json().unwrap();
    assert_eq!(first["messages"][0], json!({"role": "system", "content": "quy tắc"}));
    assert_eq!(first["tools"][0]["function"]["name"], "echo");
    let second: Value = requests[1].body_json().unwrap();
    let messages = second["messages"].as_array().unwrap();
    assert_eq!(messages[2]["tool_calls"][0]["id"], "c1");
    assert_eq!(messages[3], json!({"role": "tool", "tool_call_id": "c1", "content": "đã nhận \"x\""}));
}
