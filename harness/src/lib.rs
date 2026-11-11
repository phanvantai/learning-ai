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
pub mod codex_cli;
pub mod config;
pub mod gmail;
pub mod mcp;
pub mod openai;
pub mod proc;
pub mod scheduler;
pub mod secrets;
pub mod server;
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
    let system = agent.system_prompt();
    match provider {
        ProviderConfig::Anthropic(p) => {
            let key = secrets::provider_key(&agent.provider, "ANTHROPIC_API_KEY")?.with_context(|| {
                format!("Chưa có API key cho '{}'. Nhập trong mục Kết nối (hoặc `lai auth key {}`).", agent.provider, agent.provider)
            })?;
            let registry = Arc::new(tools::build_registry(cfg, agent_name).await?);
            Ok(Box::new(anthropic::AnthropicSession::new(p, key, agent_name, &system, registry, approver)?))
        }
        ProviderConfig::OpenAi(p) => {
            let key = secrets::provider_key(&agent.provider, "OPENAI_API_KEY")?;
            let registry = Arc::new(tools::build_registry(cfg, agent_name).await?);
            Ok(Box::new(openai::OpenAiSession::new(p, &agent.provider, key, agent_name, &system, registry, approver)?))
        }
        ProviderConfig::ClaudeCli(p) => Ok(Box::new(claude_cli::ClaudeCliSession::new(p, agent_name, &system)?)),
        ProviderConfig::CodexCli(p) => Ok(Box::new(codex_cli::CodexCliSession::new(p, agent_name, &system)?)),
    }
}

/// How a vendor CLI should start our MCP server for `agent`: this same
/// executable, whether it is the `lai` command or the desktop app.
pub fn self_mcp_command(agent: &str) -> Result<(std::path::PathBuf, Vec<String>)> {
    let exe = std::env::current_exe().context("Không xác định được đường dẫn chương trình")?;
    Ok((exe, vec!["mcp-serve".into(), "--agent".into(), agent.into()]))
}

/// Entry point for `<exe> mcp-serve --agent <name>`.
pub async fn mcp_serve(agent: &str) -> Result<()> {
    let cfg = Config::load()?;
    let registry = tools::build_registry(&cfg, agent).await?;
    mcp::serve(registry, approval::queue(), agent).await
}

/// Carry out a queued action the user has just approved.
pub async fn approve_action(id: &str) -> Result<String> {
    let store = approval::ApprovalStore::default();
    let action = store.pending(id)?;
    let cfg = Config::load()?;
    let registry = tools::build_registry(&cfg, &action.agent).await?;
    let tool = registry
        .get(&action.tool)
        .with_context(|| format!("Agent '{}' không còn công cụ '{}'", action.agent, action.tool))?;
    // The human approved this exact input, so the tool is called directly.
    // On failure the action stays pending and can be approved again.
    let result = tool
        .call(action.input)
        .await
        .with_context(|| format!("Chưa thực hiện được; '{id}' vẫn nằm trong hàng chờ"))?;
    store.resolve(id, approval::Status::Done, Some(result.clone()))?;
    Ok(result)
}

pub fn reject_action(id: &str) -> Result<()> {
    let store = approval::ApprovalStore::default();
    store.pending(id)?;
    store.resolve(id, approval::Status::Rejected, None)
}

/// What the surrounding program (desktop app or plain CLI) can do for us.
pub trait Host: Send + Sync {
    /// Show a desktop notification.
    fn notify(&self, _title: &str, _body: &str) {}
    /// Whether we start with the computer; `None` if this host cannot do that.
    fn autostart(&self) -> Option<bool> {
        None
    }
    fn set_autostart(&self, _enabled: bool) -> Result<()> {
        anyhow::bail!("Chỉ bật được khi dùng app")
    }
}

/// Host for the command line: no notifications, no autostart.
pub struct NoHost;
impl Host for NoHost {}

/// Prefix put on the first message of a run so the agent knows "today".
/// It goes in the user turn, not the system prompt, to keep the prompt cache stable.
pub fn with_timestamp(prompt: &str) -> String {
    format!(
        "[Thời điểm hiện tại: {}]\n\n{prompt}",
        chrono::Local::now().format("%A %Y-%m-%d %H:%M %:z")
    )
}
