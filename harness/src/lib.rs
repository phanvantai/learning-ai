//! lai-harness: a personal AI agent harness.
//!
//! An *agent* is a system prompt + a set of tools + a *provider* (the model
//! behind it). Providers come in two shapes:
//!   - `anthropic`: we call the Messages API with an API key and run the tool
//!     loop ourselves (see `anthropic.rs`).
//!   - `claude-cli`: we drive the official `claude` CLI the user has logged
//!     into with their subscription, and hand it our tools over MCP (see
//!     `claude_cli.rs` and `mcp.rs`). We never touch the login token.
//!
//! Either way every tool call goes through `tools::execute`, which is where
//! the approval rule lives: read-only tools run, anything else needs a human.

pub mod anthropic;
pub mod approval;
pub mod claude_cli;
pub mod config;
pub mod gmail;
pub mod mcp;
pub mod scheduler;
pub mod secrets;
pub mod tools;

use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::approval::Approver;
use crate::config::{Config, ProviderConfig};

/// A running conversation with one agent.
#[async_trait]
pub trait Session: Send {
    /// Send one user message and run until the agent has a final answer.
    async fn send(&mut self, user: &str) -> Result<String>;
}

/// Open a session for `agent_name` using whichever provider it is configured with.
pub async fn open_session(
    cfg: &Config,
    agent_name: &str,
    approver: Arc<dyn Approver>,
) -> Result<Box<dyn Session>> {
    let agent = cfg.agent(agent_name)?;
    let provider = cfg.providers.get(&agent.provider).with_context(|| {
        format!(
            "Agent '{agent_name}' dùng provider '{}' nhưng provider này chưa khai báo trong config.toml",
            agent.provider
        )
    })?;
    match provider {
        ProviderConfig::Anthropic(p) => {
            let registry = Arc::new(tools::build_registry(cfg, agent_name).await?);
            Ok(Box::new(anthropic::AnthropicSession::new(
                p,
                agent_name,
                &agent.system_prompt(),
                registry,
                approver,
            )?))
        }
        ProviderConfig::ClaudeCli(p) => Ok(Box::new(claude_cli::ClaudeCliSession::new(
            p,
            agent_name,
            &agent.system_prompt(),
        )?)),
    }
}

/// Prefix put on the first message of a run so the agent knows "today".
/// It goes in the user turn, not the system prompt, to keep the prompt cache stable.
pub fn with_timestamp(prompt: &str) -> String {
    format!(
        "[Thời điểm hiện tại: {}]\n\n{prompt}",
        chrono::Local::now().format("%A %Y-%m-%d %H:%M %:z")
    )
}
