//! Provider: the official Codex CLI, for people on a ChatGPT subscription. Experimental.
//!
//! Same shape as the Claude CLI provider: `codex exec` owns the login and the
//! model loop, our tools reach it over MCP, and writes are queued for approval
//! by our MCP server. Codex keeps its own shell tool; we pin it to the
//! read-only sandbox and an empty working directory.

use std::process::Stdio;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::config::CodexCliConfig;
use crate::Session;

pub struct CodexCliSession {
    command: String,
    model: Option<String>,
    system: String,
    agent: String,
    thread_id: Option<String>,
}

impl CodexCliSession {
    pub fn new(cfg: &CodexCliConfig, agent: &str, system: &str) -> Result<Self> {
        Ok(Self {
            command: cfg.command.clone(),
            model: cfg.model.clone(),
            system: system.to_string(),
            agent: agent.to_string(),
            thread_id: None,
        })
    }
}

/// `-c key=value` where the value is TOML, so paths with backslashes or quotes survive.
fn config_override(key: &str, value: toml::Value) -> [String; 2] {
    ["-c".to_string(), format!("{key}={value}")]
}

#[async_trait]
impl Session for CodexCliSession {
    async fn send(&mut self, user: &str) -> Result<String> {
        let (exe, args) = crate::self_mcp_command(&self.agent)?;
        let home = crate::config::home();
        let work_dir = home.join("run").join("codex-work");
        std::fs::create_dir_all(&work_dir)?;
        let last_message = home.join("run").join(format!("codex-last-{}.txt", uuid::Uuid::new_v4().simple()));

        let mut cmd = crate::proc::command(&self.command);
        cmd.arg("exec");
        if let Some(id) = &self.thread_id {
            cmd.args(["resume", id]);
        }
        // Without the user's own config, their other MCP servers are not loaded:
        // the agent gets our tools and nothing that skips the approval queue.
        cmd.args(["--json", "--skip-git-repo-check", "--ignore-user-config"])
            .args(config_override("sandbox_mode", "read-only".into()))
            .args(config_override("mcp_servers.lai.command", exe.to_string_lossy().into_owned().into()))
            .args(config_override(
                "mcp_servers.lai.args",
                toml::Value::Array(args.into_iter().map(Into::into).collect()),
            ))
            .args(config_override(
                "mcp_servers.lai.env",
                toml::Value::Table(toml::map::Map::from_iter([(
                    "LAI_HOME".to_string(),
                    home.to_string_lossy().into_owned().into(),
                )])),
            ))
            .arg("-o")
            .arg(&last_message);
        if let Some(model) = &self.model {
            cmd.args(["-m", model]);
        }
        // "-" = read the prompt from stdin.
        cmd.arg("-")
            .current_dir(&work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "Không chạy được lệnh `{}`. Cài Codex CLI và đăng nhập (`codex login`), \
                 hoặc sửa `command` của provider trong config.toml.",
                self.command
            )
        })?;
        // Codex has no system-prompt flag, so our rules open the conversation.
        let prompt = match self.thread_id {
            None => format!("{}\n\n---\n\n{user}", self.system),
            Some(_) => user.to_string(),
        };
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(prompt.as_bytes()).await?;
        drop(stdin);
        // Dropping the child on timeout kills it (`kill_on_drop`).
        let output = tokio::time::timeout(std::time::Duration::from_secs(600), child.wait_with_output())
            .await
            .context("Codex CLI không trả lời sau 10 phút nên đã bị dừng")??;

        let mut last_error = None;
        for event in String::from_utf8_lossy(&output.stdout).lines().filter_map(|l| serde_json::from_str::<Value>(l).ok())
        {
            match event["type"].as_str() {
                Some("thread.started") => {
                    if let Some(id) = event["thread_id"].as_str() {
                        self.thread_id = Some(id.to_string());
                    }
                }
                Some("error") | Some("turn.failed") => {
                    let message = event["message"].as_str().or(event["error"]["message"].as_str());
                    last_error = message.map(String::from).or(last_error);
                }
                _ => {}
            }
        }
        let answer = std::fs::read_to_string(&last_message).unwrap_or_default();
        let _ = std::fs::remove_file(&last_message);
        if !output.status.success() || answer.trim().is_empty() {
            bail!(
                "Codex CLI không trả về kết quả: {}",
                last_error.unwrap_or_else(|| format!("mã thoát {}", output.status.code().unwrap_or(-1)))
            );
        }
        Ok(answer.trim().to_string())
    }
}
