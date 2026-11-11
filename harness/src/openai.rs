//! Provider: any OpenAI-compatible Chat Completions API with an API key.
//! We own the tool loop, same as the Anthropic provider.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::approval::Approver;
use crate::config::OpenAiConfig;
use crate::tools::ToolRegistry;
use crate::Session;

const MAX_STEPS: usize = 30;

pub struct OpenAiSession {
    http: reqwest::Client,
    /// Local endpoints such as Ollama take no key.
    api_key: Option<String>,
    provider: String,
    base_url: String,
    model: String,
    agent: String,
    registry: Arc<ToolRegistry>,
    approver: Arc<dyn Approver>,
    messages: Vec<Value>,
}

impl OpenAiSession {
    pub fn new(
        cfg: &OpenAiConfig,
        provider: &str,
        api_key: Option<String>,
        agent: &str,
        system: &str,
        registry: Arc<ToolRegistry>,
        approver: Arc<dyn Approver>,
    ) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(600)).build()?,
            api_key,
            provider: provider.to_string(),
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            model: cfg.model.clone(),
            agent: agent.to_string(),
            registry,
            approver,
            messages: vec![json!({"role": "system", "content": system})],
        })
    }

    async fn call_api(&self) -> Result<Value> {
        let tools: Vec<Value> = self
            .registry
            .iter()
            .map(|t| {
                json!({"type": "function", "function": {
                    "name": t.name(), "description": t.description(), "parameters": t.input_schema(),
                }})
            })
            .collect();
        let mut body = json!({"model": self.model, "messages": self.messages});
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
        }
        let mut attempt = 0;
        loop {
            attempt += 1;
            let mut req = self.http.post(format!("{}/chat/completions", self.base_url)).json(&body);
            if let Some(key) = &self.api_key {
                req = req.bearer_auth(key);
            }
            let resp = match req.send().await {
                Ok(resp) => resp,
                Err(e) if attempt < 3 && (e.is_connect() || e.is_timeout()) => {
                    tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
                    continue;
                }
                Err(e) => return Err(e).with_context(|| format!("Không gọi được {}", self.base_url)),
            };
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            match status {
                200 => return serde_json::from_str(&text).context("API trả về JSON không hợp lệ"),
                429 | 500..=599 if attempt < 3 => tokio::time::sleep(Duration::from_secs(2 * attempt)).await,
                401 => bail!(
                    "API key của '{}' thiếu hoặc không hợp lệ. Nhập lại trong mục Kết nối (hoặc `lai auth key {}`).",
                    self.provider,
                    self.provider
                ),
                _ => {
                    let message = serde_json::from_str::<Value>(&text)
                        .ok()
                        .and_then(|v| v["error"]["message"].as_str().map(String::from))
                        .unwrap_or(text);
                    bail!("API báo lỗi {status}: {message}")
                }
            }
        }
    }

    async fn run_turn(&mut self) -> Result<String> {
        for _ in 0..MAX_STEPS {
            let response = self.call_api().await?;
            let message = response["choices"][0]["message"].clone();
            if !message.is_object() {
                bail!("API không trả về câu trả lời: {response}");
            }
            let text = message["content"].as_str().unwrap_or_default().to_string();
            let calls = message["tool_calls"].as_array().cloned().unwrap_or_default();
            self.messages.push(message);
            if calls.is_empty() {
                return Ok(text);
            }
            for call in calls {
                let name = call["function"]["name"].as_str().unwrap_or_default();
                // Arguments arrive as a JSON string.
                let args = call["function"]["arguments"].as_str().unwrap_or("{}");
                let output = match serde_json::from_str::<Value>(if args.trim().is_empty() { "{}" } else { args }) {
                    Ok(input) => {
                        crate::tools::execute(&self.registry, self.approver.as_ref(), &self.agent, name, input)
                            .await
                            .0
                    }
                    Err(e) => format!("Lỗi: tham số không phải JSON hợp lệ ({e})."),
                };
                self.messages.push(json!({"role": "tool", "tool_call_id": call["id"], "content": output}));
            }
        }
        bail!("Agent chạy quá {MAX_STEPS} bước mà chưa xong; dừng lại để tránh lặp vô hạn.")
    }
}

#[async_trait]
impl Session for OpenAiSession {
    async fn send(&mut self, user: &str) -> Result<String> {
        let checkpoint = self.messages.len();
        self.messages.push(json!({"role": "user", "content": user}));
        let result = self.run_turn().await;
        if result.is_err() {
            self.messages.truncate(checkpoint);
        }
        result
    }
}
