//! Our MCP client talking to our MCP server (the real `lai mcp-serve` binary).

use std::collections::BTreeMap;

use lai_harness::approval::{ApprovalStore, Status};
use lai_harness::config::{McpServerConfig, STARTER_CONFIG};
use lai_harness::mcp;
use lai_harness::tools::Risk;
use serde_json::json;

#[tokio::test]
async fn lists_tools_and_queues_write_calls() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("config.toml"), STARTER_CONFIG).unwrap();

    let server = McpServerConfig {
        command: env!("CARGO_BIN_EXE_lai").into(),
        args: vec!["mcp-serve".into(), "--agent".into(), "tro-ly".into()],
        env: BTreeMap::from([
            ("LAI_HOME".to_string(), home.path().to_string_lossy().into_owned()),
            ("LAI_SECRETS_FILE".to_string(), "1".to_string()),
        ]),
        read_only_tools: vec!["gmail_search".into()],
    };
    let tools = mcp::connect("lai", &server).await.unwrap();

    let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    assert_eq!(names, ["lai__gmail_search", "lai__gmail_read", "lai__gmail_send"]);
    // Only tools the config names as read-only skip approval on the client side.
    assert_eq!(tools[0].risk(), Risk::ReadOnly);
    assert_eq!(tools[1].risk(), Risk::Write);

    // The server side must park the send instead of sending.
    let input = json!({"to": "lan@example.com", "subject": "s", "body": "b"});
    let text = tools[2].call(input.clone()).await.unwrap();
    assert!(text.contains("CHƯA THỰC HIỆN"), "{text}");
    let queued = ApprovalStore::at(home.path().join("approvals.json")).all().unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!((queued[0].agent.as_str(), queued[0].tool.as_str()), ("tro-ly", "gmail_send"));
    assert_eq!(queued[0].status, Status::Pending);
    assert_eq!(queued[0].input, input);

    // A read tool with no Gmail login reports a usable error rather than hanging.
    let err = tools[0].call(json!({"query": "is:unread"})).await.unwrap_err();
    assert!(err.to_string().contains("lai auth gmail"), "{err}");
}
