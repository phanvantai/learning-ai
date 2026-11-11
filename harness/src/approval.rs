//! Human approval for tool calls that change something outside the machine.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub enum Decision {
    Allow,
    Deny(String),
    /// Parked for later; carries the id to show the user.
    Queued(String),
}

#[async_trait]
pub trait Approver: Send + Sync {
    async fn decide(&self, agent: &str, tool: &str, input: &Value) -> Decision;
}

/// Asks on the terminal. Used when a person is sitting at `lai chat` / `lai run`.
pub struct Interactive;

#[async_trait]
impl Approver for Interactive {
    async fn decide(&self, _agent: &str, tool: &str, input: &Value) -> Decision {
        let pretty = serde_json::to_string_pretty(input).unwrap_or_default();
        eprintln!("\n⚠️  Agent muốn thực hiện: {tool}\n{pretty}");
        eprint!("Cho phép? [y/N] ");
        let answer = tokio::task::spawn_blocking(|| {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).map(|_| line)
        })
        .await;
        match answer {
            Ok(Ok(line)) if matches!(line.trim().to_lowercase().as_str(), "y" | "yes" | "c" | "co" | "có") => {
                Decision::Allow
            }
            _ => Decision::Deny("Người dùng từ chối hành động này.".into()),
        }
    }
}

/// Parks the action in the approval queue. Used when nobody is watching
/// (scheduled jobs, the MCP server behind the Claude CLI).
pub struct Queue(pub ApprovalStore);

#[async_trait]
impl Approver for Queue {
    async fn decide(&self, agent: &str, tool: &str, input: &Value) -> Decision {
        match self.0.add(agent, tool, input) {
            Ok(id) => Decision::Queued(id),
            Err(e) => Decision::Deny(format!("Không ghi được vào hàng chờ duyệt: {e}")),
        }
    }
}

pub fn queue() -> Arc<dyn Approver> {
    Arc::new(Queue(ApprovalStore::default()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    Done,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub id: String,
    pub agent: String,
    pub tool: String,
    pub input: Value,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub status: Status,
    #[serde(default)]
    pub result: Option<String>,
}

/// The approval queue, one JSON file. Small and human-inspectable on purpose.
pub struct ApprovalStore {
    path: PathBuf,
}

impl Default for ApprovalStore {
    fn default() -> Self {
        Self { path: crate::config::home().join("approvals.json") }
    }
}

impl ApprovalStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn all(&self) -> Result<Vec<PendingAction>> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text).context("approvals.json hỏng"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(e.into()),
        }
    }

    pub fn pending_count(&self) -> usize {
        self.all().map(|all| all.iter().filter(|a| a.status == Status::Pending).count()).unwrap_or(0)
    }

    fn save(&self, items: &[PendingAction]) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write-then-rename so a crash never leaves a half-written queue.
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(items)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn add(&self, agent: &str, tool: &str, input: &Value) -> Result<String> {
        let mut items = self.all()?;
        let id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
        items.push(PendingAction {
            id: id.clone(),
            agent: agent.into(),
            tool: tool.into(),
            input: input.clone(),
            created_at: chrono::Local::now(),
            status: Status::Pending,
            result: None,
        });
        self.save(&items)?;
        Ok(id)
    }

    pub fn pending(&self, id: &str) -> Result<PendingAction> {
        let item = self
            .all()?
            .into_iter()
            .find(|a| a.id == id)
            .with_context(|| format!("Không có hành động nào có id '{id}'"))?;
        if item.status != Status::Pending {
            bail!("Hành động '{id}' đã được xử lý rồi ({:?})", item.status);
        }
        Ok(item)
    }

    pub fn resolve(&self, id: &str, status: Status, result: Option<String>) -> Result<()> {
        let mut items = self.all()?;
        let item = items
            .iter_mut()
            .find(|a| a.id == id)
            .with_context(|| format!("Không có hành động nào có id '{id}'"))?;
        item.status = status;
        item.result = result;
        self.save(&items)
    }
}
