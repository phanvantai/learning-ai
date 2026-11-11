//! Provider: the official Claude Code CLI, for people on a subscription.
//!
//! We run `claude -p` as a child process. The CLI owns the login and the model
//! loop; we never read its credentials. Our tools reach it through an MCP
//! server that is this same binary (`lai mcp-serve`), so approvals still apply.
//! The CLI's own built-in tools (shell, file edits, web) are switched off.

use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use crate::config::ClaudeCliConfig;
use crate::Session;

const MCP_SERVER_NAME: &str = "lai";

pub struct ClaudeCliSession {
    command: String,
    model: Option<String>,
    system: String,
    mcp_config: PathBuf,
    /// Set after the first reply; lets follow-up messages continue the same conversation.
    session_id: Option<String>,
}

impl ClaudeCliSession {
    pub fn new(cfg: &ClaudeCliConfig, agent: &str, system: &str) -> Result<Self> {
        let exe = std::env::current_exe().context("Không xác định được đường dẫn của lai")?;
        let home = crate::config::home();
        let run_dir = home.join("run");
        std::fs::create_dir_all(&run_dir)?;
        let mcp_config = run_dir.join(format!("mcp-{agent}.json"));
        let config = json!({"mcpServers": {MCP_SERVER_NAME: {
            "command": exe,
            "args": ["mcp-serve", "--agent", agent],
            "env": {"LAI_HOME": home},
        }}});
        std::fs::write(&mcp_config, serde_json::to_vec_pretty(&config)?)?;
        Ok(Self {
            command: cfg.command.clone(),
            model: cfg.model.clone(),
            system: system.to_string(),
            mcp_config,
            session_id: None,
        })
    }
}

#[async_trait]
impl Session for ClaudeCliSession {
    async fn send(&mut self, user: &str) -> Result<String> {
        let mut cmd = tokio::process::Command::new(&self.command);
        cmd.arg("-p")
            .args(["--output-format", "json"])
            .arg("--mcp-config")
            .arg(&self.mcp_config)
            .arg("--strict-mcp-config")
            .args(["--tools", ""])
            .args(["--allowedTools", &format!("mcp__{MCP_SERVER_NAME}")])
            .args(["--append-system-prompt", &self.system]);
        if let Some(model) = &self.model {
            cmd.args(["--model", model]);
        }
        if let Some(id) = &self.session_id {
            cmd.args(["--resume", id]);
        }
        // Run from the harness home, not wherever the user happens to be, so
        // the CLI does not pick up an unrelated project's instructions.
        cmd.current_dir(crate::config::home())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "Không chạy được lệnh `{}`. Cài Claude Code và đăng nhập (`claude` rồi `/login`), \
                 hoặc sửa `command` của provider trong config.toml.",
                self.command
            )
        })?;
        // The prompt goes over stdin: no length limit, no shell quoting.
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(user.as_bytes()).await?;
        drop(stdin);

        let output = child.wait_with_output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let reply: Value = match serde_json::from_str(stdout.trim()) {
            Ok(v) => v,
            Err(_) => bail!(
                "Claude CLI không trả về kết quả hợp lệ (mã thoát {}).\n{}\n{}",
                output.status.code().unwrap_or(-1),
                stdout.trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        };
        if let Some(id) = reply["session_id"].as_str() {
            self.session_id = Some(id.to_string());
        }
        let text = reply["result"].as_str().unwrap_or_default().to_string();
        if reply["is_error"].as_bool().unwrap_or(false) || !output.status.success() {
            bail!("Claude CLI báo lỗi: {}", if text.is_empty() { stdout.trim() } else { &text });
        }
        Ok(text)
    }
}
