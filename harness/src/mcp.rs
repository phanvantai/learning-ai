//! Minimal MCP over stdio (newline-delimited JSON-RPC), tools only.
//!
//! - `serve`: exposes an agent's tool registry, so the Claude CLI can use our tools.
//! - `connect`: uses tools from an external MCP server.

use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

use crate::approval::Approver;
use crate::config::McpServerConfig;
use crate::tools::{Risk, Tool, ToolRegistry};

const PROTOCOL_VERSION: &str = "2025-06-18";

// ---------------------------------------------------------------- server ---

/// Serve `registry` on stdin/stdout until the client closes the pipe.
pub async fn serve(registry: ToolRegistry, approver: Arc<dyn Approver>, agent: &str) -> Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else { continue };
        // Notifications carry no id and get no reply.
        let Some(id) = msg.get("id").filter(|id| !id.is_null()).cloned() else { continue };
        let reply = match handle(&registry, approver.as_ref(), agent, &msg).await {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err((code, message)) => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
            }
        };
        stdout.write_all(format!("{reply}\n").as_bytes()).await?;
        stdout.flush().await?;
    }
    Ok(())
}

async fn handle(
    registry: &ToolRegistry,
    approver: &dyn Approver,
    agent: &str,
    msg: &Value,
) -> Result<Value, (i64, String)> {
    match msg["method"].as_str().unwrap_or_default() {
        "initialize" => Ok(json!({
            "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or(PROTOCOL_VERSION),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "lai-harness", "version": env!("CARGO_PKG_VERSION")},
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": registry.iter().map(|t| json!({
            "name": t.name(),
            "description": t.description(),
            "inputSchema": t.input_schema(),
        })).collect::<Vec<_>>()})),
        "tools/call" => {
            let name = msg["params"]["name"].as_str().unwrap_or_default();
            let input = msg["params"].get("arguments").cloned().unwrap_or_else(|| json!({}));
            let (text, is_error) = crate::tools::execute(registry, approver, agent, name, input).await;
            Ok(json!({"content": [{"type": "text", "text": text}], "isError": is_error}))
        }
        other => Err((-32601, format!("method not found: {other}"))),
    }
}

// ---------------------------------------------------------------- client ---

struct Client {
    name: String,
    /// One request at a time: holding this lock covers the write and the matching read.
    io: Mutex<(ChildStdin, Lines<BufReader<ChildStdout>>)>,
    next_id: AtomicU64,
    _child: Child,
}

impl Client {
    async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let exchange = async {
            let mut io = self.io.lock().await;
            io.0.write_all(format!("{msg}\n").as_bytes()).await?;
            io.0.flush().await?;
            loop {
                let Some(line) = io.1.next_line().await? else {
                    bail!("MCP server '{}' đã thoát", self.name);
                };
                // Skip log lines, notifications and replies to other ids.
                let Ok(reply) = serde_json::from_str::<Value>(&line) else { continue };
                if reply["id"].as_u64() != Some(id) {
                    continue;
                }
                if let Some(err) = reply.get("error") {
                    bail!("MCP server '{}' báo lỗi: {}", self.name, err["message"].as_str().unwrap_or("?"));
                }
                return Ok(reply["result"].clone());
            }
        };
        tokio::time::timeout(Duration::from_secs(120), exchange)
            .await
            .with_context(|| format!("MCP server '{}' không trả lời '{method}' sau 120 giây", self.name))?
    }

    async fn notify(&self, method: &str) -> Result<()> {
        let msg = json!({"jsonrpc": "2.0", "method": method});
        let mut io = self.io.lock().await;
        io.0.write_all(format!("{msg}\n").as_bytes()).await?;
        io.0.flush().await?;
        Ok(())
    }
}

struct McpTool {
    client: Arc<Client>,
    remote_name: String,
    name: String,
    description: String,
    schema: Value,
    risk: Risk,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn input_schema(&self) -> Value {
        self.schema.clone()
    }
    fn risk(&self) -> Risk {
        self.risk
    }
    async fn call(&self, input: Value) -> Result<String> {
        let result = self
            .client
            .request("tools/call", json!({"name": self.remote_name, "arguments": input}))
            .await?;
        let text = result["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|block| block["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if result["isError"].as_bool().unwrap_or(false) {
            bail!("{text}");
        }
        Ok(text)
    }
}

/// Start an MCP server and return its tools, named `<server>__<tool>`.
/// A tool is treated as needing approval unless the config lists it as read-only.
pub async fn connect(server_name: &str, cfg: &McpServerConfig) -> Result<Vec<Arc<dyn Tool>>> {
    let mut child = tokio::process::Command::new(&cfg.command)
        .args(&cfg.args)
        .envs(&cfg.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("Không chạy được MCP server '{server_name}' ({})", cfg.command))?;
    let stdin = child.stdin.take().expect("stdin was piped");
    let stdout = BufReader::new(child.stdout.take().expect("stdout was piped")).lines();
    let client = Arc::new(Client {
        name: server_name.to_string(),
        io: Mutex::new((stdin, stdout)),
        next_id: AtomicU64::new(1),
        _child: child,
    });

    client
        .request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "lai-harness", "version": env!("CARGO_PKG_VERSION")},
            }),
        )
        .await?;
    client.notify("notifications/initialized").await?;

    let listed = client.request("tools/list", json!({})).await?;
    let mut tools: Vec<Arc<dyn Tool>> = vec![];
    for tool in listed["tools"].as_array().into_iter().flatten() {
        let Some(remote_name) = tool["name"].as_str() else { continue };
        let read_only = cfg.read_only_tools.iter().any(|t| t == remote_name);
        tools.push(Arc::new(McpTool {
            client: client.clone(),
            remote_name: remote_name.to_string(),
            name: format!("{server_name}__{remote_name}"),
            description: tool["description"].as_str().unwrap_or_default().to_string(),
            schema: tool.get("inputSchema").cloned().unwrap_or_else(|| json!({"type": "object"})),
            risk: if read_only { Risk::ReadOnly } else { Risk::Write },
        }));
    }
    Ok(tools)
}
