//! Provider: Anthropic Messages API with an API key. We own the tool loop.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::approval::Approver;
use crate::config::AnthropicConfig;
use crate::tools::ToolRegistry;
use crate::{secrets, Session};

const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Upper bound on model round-trips for one user message, so a confused
/// agent cannot loop (and bill) forever.
const MAX_STEPS: usize = 30;

pub struct AnthropicSession {
    http: reqwest::Client,
    api_key: String,
    cfg: RequestConfig,
    agent: String,
    system: String,
    registry: Arc<ToolRegistry>,
    approver: Arc<dyn Approver>,
    /// Conversation so far, kept as raw JSON: assistant content (thinking,
    /// fallback blocks, ...) must be sent back exactly as received.
    messages: Vec<Value>,
}

struct RequestConfig {
    base_url: String,
    model: String,
    effort: String,
    max_tokens: u32,
    fallbacks: bool,
}

impl AnthropicSession {
    pub fn new(
        cfg: &AnthropicConfig,
        agent: &str,
        system: &str,
        registry: Arc<ToolRegistry>,
        approver: Arc<dyn Approver>,
    ) -> Result<Self> {
        let api_key = match std::env::var("ANTHROPIC_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => secrets::get(secrets::ANTHROPIC_API_KEY)?
                .context("Chưa có API key Anthropic. Chạy `lai auth anthropic`.")?,
        };
        Ok(Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(600)).build()?,
            api_key,
            cfg: RequestConfig {
                base_url: cfg.base_url.trim_end_matches('/').to_string(),
                model: cfg.model.clone(),
                effort: cfg.effort.clone(),
                max_tokens: cfg.max_tokens,
                fallbacks: cfg.fallbacks,
            },
            agent: agent.to_string(),
            system: system.to_string(),
            registry,
            approver,
            messages: vec![],
        })
    }

    fn request_body(&self) -> Value {
        let tools: Vec<Value> = self
            .registry
            .iter()
            .map(|t| json!({"name": t.name(), "description": t.description(), "input_schema": t.input_schema()}))
            .collect();
        let mut body = json!({
            "model": self.cfg.model,
            "max_tokens": self.cfg.max_tokens,
            "system": [{"type": "text", "text": self.system}],
            "output_config": {"effort": self.cfg.effort},
            // Auto-places a cache breakpoint on the last cacheable block, so
            // each step of the tool loop re-reads the previous prefix cheaply.
            "cache_control": {"type": "ephemeral"},
            "messages": self.messages,
        });
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
        }
        if self.cfg.fallbacks {
            body["fallbacks"] = json!("default");
        }
        body
    }

    async fn call_api(&self) -> Result<Value> {
        let body = self.request_body();
        let mut attempt = 0;
        loop {
            attempt += 1;
            let mut req = self
                .http
                .post(format!("{}/v1/messages", self.cfg.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION);
            if self.cfg.fallbacks {
                req = req.header("anthropic-beta", FALLBACK_BETA);
            }
            let (status, retry_after, text) = match req.json(&body).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let retry_after = resp
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok());
                    (status, retry_after, resp.text().await.unwrap_or_default())
                }
                Err(e) if attempt < 3 && (e.is_connect() || e.is_timeout()) => {
                    tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
                    continue;
                }
                Err(e) => return Err(e).context("Không gọi được API Anthropic"),
            };
            match status {
                200 => return serde_json::from_str(&text).context("API trả về JSON không hợp lệ"),
                429 | 500..=599 if attempt < 3 => {
                    let wait = retry_after.unwrap_or(2 * attempt).min(60);
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
                401 => bail!("API key Anthropic không hợp lệ. Chạy lại `lai auth anthropic`."),
                _ => {
                    let message = serde_json::from_str::<Value>(&text)
                        .ok()
                        .and_then(|v| v["error"]["message"].as_str().map(String::from))
                        .unwrap_or(text);
                    bail!("API Anthropic báo lỗi {status}: {message}")
                }
            }
        }
    }

    async fn run_turn(&mut self) -> Result<String> {
        for _ in 0..MAX_STEPS {
            let response = self.call_api().await?;
            let content = response["content"].as_array().cloned().unwrap_or_default();
            let text = content
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let stop_reason = response["stop_reason"].as_str().unwrap_or_default();
            if stop_reason == "refusal" {
                let category = response["stop_details"]["category"].as_str().unwrap_or("không rõ");
                bail!("Mô hình từ chối yêu cầu này (nhóm: {category}).");
            }
            self.messages.push(json!({"role": "assistant", "content": content}));
            match stop_reason {
                "tool_use" => {
                    let mut results = vec![];
                    for block in content.iter().filter(|b| b["type"] == "tool_use") {
                        let name = block["name"].as_str().unwrap_or_default();
                        let (output, is_error) = crate::tools::execute(
                            &self.registry,
                            self.approver.as_ref(),
                            &self.agent,
                            name,
                            block["input"].clone(),
                        )
                        .await;
                        results.push(json!({
                            "type": "tool_result",
                            "tool_use_id": block["id"],
                            "content": output,
                            "is_error": is_error,
                        }));
                    }
                    self.messages.push(json!({"role": "user", "content": results}));
                }
                // A server-side pause: send the conversation back to let it continue.
                "pause_turn" => {}
                "max_tokens" => return Ok(format!("{text}\n\n[Câu trả lời bị cắt vì chạm giới hạn max_tokens.]")),
                _ => return Ok(text),
            }
        }
        bail!("Agent chạy quá {MAX_STEPS} bước mà chưa xong; dừng lại để tránh lặp vô hạn.")
    }
}

#[async_trait]
impl Session for AnthropicSession {
    async fn send(&mut self, user: &str) -> Result<String> {
        let checkpoint = self.messages.len();
        self.messages.push(json!({"role": "user", "content": user}));
        let result = self.run_turn().await;
        if result.is_err() {
            // Drop the half-finished turn so the conversation stays valid for the next message.
            self.messages.truncate(checkpoint);
        }
        result
    }
}
