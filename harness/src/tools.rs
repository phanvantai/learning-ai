//! Tools an agent can call, and the single place tool calls are executed.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::approval::{Approver, Decision};
use crate::config::Config;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Risk {
    /// Only reads. Runs without asking.
    ReadOnly,
    /// Changes something outside the machine. Needs a human.
    Write,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn risk(&self) -> Risk;
    async fn call(&self, input: Value) -> Result<String>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn add(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Tool>> {
        self.tools.iter()
    }
}

/// Run one tool call for an agent. Returns the text for the model and whether it is an error.
///
/// This is the only path to `Tool::call` for model-initiated calls, so the
/// approval rule cannot be skipped by a provider.
pub async fn execute(
    registry: &ToolRegistry,
    approver: &dyn Approver,
    agent: &str,
    name: &str,
    input: Value,
) -> (String, bool) {
    let Some(tool) = registry.get(name) else {
        return (format!("Không có công cụ tên '{name}'."), true);
    };
    if tool.risk() == Risk::Write {
        match approver.decide(agent, name, &input).await {
            Decision::Allow => {}
            Decision::Deny(reason) => return (reason, true),
            Decision::Queued(id) => {
                return (
                    format!(
                        "CHƯA THỰC HIỆN. Hành động đã vào hàng chờ duyệt với id {id}. \
                         Người dùng sẽ duyệt trong mục Chờ duyệt của app (hoặc lệnh `lai approvals approve {id}`). \
                         Đừng gọi lại công cụ này."
                    ),
                    false,
                )
            }
        }
    }
    match tool.call(input).await {
        Ok(text) => (text, false),
        Err(e) => (format!("Lỗi: {e:#}"), true),
    }
}

/// Build the tool set an agent is allowed to use.
pub async fn build_registry(cfg: &Config, agent_name: &str) -> Result<ToolRegistry> {
    let agent = cfg.agent(agent_name)?;
    let mut registry = ToolRegistry::default();

    let gmail = Arc::new(crate::gmail::GmailClient::from_secrets());
    for tool in crate::gmail::tools(gmail) {
        if agent.tools.iter().any(|t| t == tool.name()) {
            registry.add(tool);
        }
    }
    for wanted in &agent.tools {
        if registry.get(wanted).is_none() {
            anyhow::bail!("agent '{agent_name}': không có công cụ dựng sẵn tên '{wanted}'");
        }
    }

    for server_name in &agent.mcp_servers {
        let server = &cfg.mcp_servers[server_name];
        for tool in crate::mcp::connect(server_name, server).await? {
            registry.add(tool);
        }
    }
    Ok(registry)
}
