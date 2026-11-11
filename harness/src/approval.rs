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
    /// Claimed by an approval that is being carried out right now.
    Executing,
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
        let tmp = self.path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4().simple()));
        std::fs::write(&tmp, serde_json::to_vec_pretty(items)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    /// Load, change and save as one step, exclusive across processes: the
    /// app and each `mcp-serve` child all write this file.
    fn update<T>(&self, change: impl FnOnce(&mut Vec<PendingAction>) -> Result<T>) -> Result<T> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let lock = self.path.with_extension("lock");
        let started = std::time::Instant::now();
        loop {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // A holder that died leaves the lock behind; an update takes milliseconds.
                    let stale = std::fs::metadata(&lock)
                        .and_then(|m| m.modified())
                        .is_ok_and(|t| t.elapsed().is_ok_and(|age| age > std::time::Duration::from_secs(10)));
                    if stale || started.elapsed() > std::time::Duration::from_secs(15) {
                        let _ = std::fs::remove_file(&lock);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => return Err(e).context("Không khóa được hàng chờ duyệt"),
            }
        }
        let result = (|| {
            let mut items = self.all()?;
            let value = change(&mut items)?;
            self.save(&items)?;
            Ok(value)
        })();
        let _ = std::fs::remove_file(&lock);
        result
    }

    pub fn add(&self, agent: &str, tool: &str, input: &Value) -> Result<String> {
        let id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
        self.update(|items| {
            items.push(PendingAction {
                id: id.clone(),
                agent: agent.into(),
                tool: tool.into(),
                input: input.clone(),
                created_at: chrono::Local::now(),
                status: Status::Pending,
                result: None,
            });
            Ok(())
        })?;
        Ok(id)
    }

    /// Move a pending action to `to` and return it. Fails if it is not
    /// pending any more, so two approvals of one action cannot both proceed.
    pub fn claim(&self, id: &str, to: Status) -> Result<PendingAction> {
        self.update(|items| {
            let item = items
                .iter_mut()
                .find(|a| a.id == id)
                .with_context(|| format!("Không có hành động nào có id '{id}'"))?;
            if item.status != Status::Pending {
                bail!("Hành động '{id}' đã được xử lý rồi hoặc đang được thực hiện");
            }
            item.status = to;
            Ok(item.clone())
        })
    }

    pub fn resolve(&self, id: &str, status: Status, result: Option<String>) -> Result<()> {
        self.update(|items| {
            let item = items
                .iter_mut()
                .find(|a| a.id == id)
                .with_context(|| format!("Không có hành động nào có id '{id}'"))?;
            item.status = status;
            item.result = result;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_can_be_claimed_only_once() {
        let dir = tempfile::tempdir().unwrap();
        let store = ApprovalStore::at(dir.path().join("approvals.json"));
        let id = store.add("a", "gmail_send", &serde_json::json!({"to": "x@y.z"})).unwrap();

        assert_eq!(store.claim(&id, Status::Executing).unwrap().tool, "gmail_send");
        // A second approval (double click, second tab, the CLI) must not get it.
        assert!(store.claim(&id, Status::Executing).is_err());
        assert!(store.claim(&id, Status::Rejected).is_err());
        assert_eq!(store.pending_count(), 0);

        // A failed attempt hands it back, and it can be claimed again.
        store.resolve(&id, Status::Pending, None).unwrap();
        assert!(store.claim(&id, Status::Executing).is_ok());
        assert!(!dir.path().join("approvals.lock").exists());
    }

    #[test]
    fn concurrent_writers_do_not_lose_actions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("approvals.json");
        let threads: Vec<_> = (0..8)
            .map(|i| {
                let path = path.clone();
                std::thread::spawn(move || {
                    let store = ApprovalStore::at(path);
                    for j in 0..5 {
                        store.add("a", "t", &serde_json::json!({"n": i * 10 + j})).unwrap();
                    }
                })
            })
            .collect();
        threads.into_iter().for_each(|t| t.join().unwrap());
        assert_eq!(ApprovalStore::at(path).all().unwrap().len(), 40);
    }
}
