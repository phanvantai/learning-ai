//! Local web app: the UI and the JSON API behind it.
//!
//! Listens on 127.0.0.1 only. Every API call must carry a random token that
//! is handed to the UI in the URL fragment, and must be addressed to
//! 127.0.0.1/localhost, so other websites open in the user's browser cannot
//! drive the agent (no CSRF, no DNS rebinding).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use axum::extract::{Path, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::approval::{ApprovalStore, Status};
use crate::config::{Config, JobConfig, ProviderConfig};
use crate::{gmail, scheduler, secrets, Host, Session};

struct App {
    token: String,
    host: Arc<dyn Host>,
    sessions: Mutex<HashMap<String, SharedSession>>,
    /// Serializes read-modify-write of config.toml.
    config_lock: Mutex<()>,
    running_jobs: Mutex<HashSet<String>>,
}

type SharedSession = Arc<Mutex<Box<dyn Session>>>;
type AppState = Arc<App>;

/// Start the UI server and the job scheduler. Returns the URL to open.
pub async fn start(host: Arc<dyn Host>) -> Result<String> {
    Config::load_or_init()?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let app = Arc::new(App {
        token: token.clone(),
        host: host.clone(),
        sessions: Mutex::default(),
        config_lock: Mutex::default(),
        running_jobs: Mutex::default(),
    });
    let router = router(app);
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("Máy chủ giao diện dừng: {e}");
        }
    });
    tokio::spawn(scheduler::run_loop(host));
    Ok(format!("http://127.0.0.1:{port}/#token={token}"))
}

fn router(app: AppState) -> Router {
    let api = Router::new()
        .route("/state", get(state))
        .route("/chat", post(chat))
        .route("/chat/{session}", delete(end_chat))
        .route("/approvals/{id}/approve", post(approve))
        .route("/approvals/{id}/reject", post(reject))
        .route("/jobs/{name}", put(put_job).delete(delete_job))
        .route("/jobs/{name}/run", post(run_job))
        .route("/reports/{job}/{file}", get(report))
        .route("/providers/{name}", put(put_provider).delete(delete_provider))
        .route("/providers/{name}/key", post(set_key))
        .route("/providers/{name}/test", post(test_provider))
        .route("/onboarding", post(set_onboarded))
        .route("/agents/{name}", put(put_agent))
        .route("/gmail/login", post(gmail_login))
        .route("/autostart", post(set_autostart))
        .route("/open", post(open_link))
        .layer(axum::middleware::from_fn_with_state(app.clone(), guard));
    Router::new()
        .route("/", get(|| async { asset("text/html; charset=utf-8", include_str!("ui/index.html")) }))
        .route("/app.js", get(|| async { asset("text/javascript; charset=utf-8", include_str!("ui/app.js")) }))
        .route("/app.css", get(|| async { asset("text/css; charset=utf-8", include_str!("ui/app.css")) }))
        .nest("/api", api)
        .with_state(app)
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-store"),
            // The UI loads nothing from elsewhere and may not be framed.
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
            ),
        ],
        body,
    )
        .into_response()
}

async fn guard(State(app): State<AppState>, req: Request, next: Next) -> Response {
    if !allowed(&app.token, req.headers()) {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "Không được phép"}))).into_response();
    }
    next.run(req).await
}

fn allowed(token: &str, headers: &axum::http::HeaderMap) -> bool {
    let value = |name: &str| headers.get(name).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let local = |host: &str| host.starts_with("127.0.0.1:") || host.starts_with("localhost:");
    let origin = value("origin");
    local(value("host"))
        && (origin.is_empty() || origin.strip_prefix("http://").is_some_and(local))
        && constant_time_eq(value("x-lai-token").as_bytes(), token.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Any failure becomes `{ "error": "..." }` for the UI to show.
struct ApiError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{:#}", self.0)}))).into_response()
    }
}

type Api = Result<Json<Value>, ApiError>;

fn ok() -> Api {
    Ok(Json(json!({"ok": true})))
}

fn check_name(what: &str, name: &str) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        bail!("Tên {what} chỉ được gồm chữ không dấu, số, '-' và '_'");
    }
    Ok(())
}

/// Load, change and save the config as one step.
async fn edit_config(app: &App, change: impl FnOnce(&mut Config) -> Result<()>) -> Result<()> {
    let _guard = app.config_lock.lock().await;
    let mut cfg = Config::load()?;
    change(&mut cfg)?;
    cfg.save()
}

// ----------------------------------------------------------------- state ---

fn onboarded_marker() -> std::path::PathBuf {
    crate::config::home().join("onboarded")
}

async fn state(State(app): State<AppState>) -> Api {
    // Only the UI holds the token, so the first call here means the window
    // really loaded and ran. Leave a trace that an install check can look for.
    static UI_SEEN: std::sync::Once = std::sync::Once::new();
    UI_SEEN.call_once(|| {
        let run_dir = crate::config::home().join("run");
        let _ = std::fs::create_dir_all(&run_dir);
        let _ = std::fs::write(run_dir.join("ui-loaded"), "");
    });
    let cfg = Config::load()?;
    let now = chrono::Local::now();

    let mut providers = vec![];
    for (name, provider) in &cfg.providers {
        let (ready, status) = provider_status(name, provider)?;
        let mut entry = serde_json::to_value(provider)?;
        entry["name"] = json!(name);
        entry["ready"] = json!(ready);
        entry["status"] = json!(status);
        providers.push(entry);
    }
    let agents: Vec<Value> = cfg
        .agents
        .iter()
        .map(|(name, a)| json!({"name": name, "provider": a.provider, "instructions": a.instructions, "tools": a.tools}))
        .collect();
    let running = app.running_jobs.lock().await.clone();
    let jobs: Vec<Value> = cfg
        .jobs
        .iter()
        .map(|job| {
            json!({
                "name": job.name,
                "agent": job.agent,
                "schedule": job.schedule,
                "prompt": job.prompt,
                "next_run": scheduler::next_run(job, now).ok().map(|t| t.format("%Y-%m-%d %H:%M").to_string()),
                "reports": scheduler::list_reports(&job.name).into_iter().take(30).collect::<Vec<_>>(),
                "running": running.contains(&job.name),
            })
        })
        .collect();
    let pending: Vec<_> =
        ApprovalStore::default().all()?.into_iter().filter(|a| a.status == Status::Pending).collect();

    Ok(Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "onboarded": onboarded_marker().exists(),
        "config_path": crate::config::config_path(),
        "default_agent": cfg.agent_name(None).ok(),
        "agents": agents,
        "providers": providers,
        "gmail": {
            "connected": secrets::get(secrets::GMAIL_REFRESH_TOKEN)?.is_some(),
            "address": secrets::get(secrets::GMAIL_ADDRESS)?,
            "builtin_client": gmail::builtin_client().is_some(),
        },
        "jobs": jobs,
        "pending": pending,
        "autostart": app.host.autostart(),
    })))
}

fn provider_status(name: &str, provider: &ProviderConfig) -> Result<(bool, String)> {
    if let Some(command) = provider.cli_command() {
        return Ok(match crate::proc::resolve(command) {
            Some(path) => (true, format!("Đã tìm thấy lệnh: {}", path.display())),
            None => (false, format!("Chưa tìm thấy lệnh `{command}` trên máy")),
        });
    }
    let has_key = secrets::provider_key(name, provider.key_env().unwrap_or_default())?.is_some();
    Ok(match provider {
        _ if has_key => (true, "Đã có API key".into()),
        // Local endpoints (Ollama and the like) need no key.
        ProviderConfig::OpenAi(p) if !p.base_url.contains("api.openai.com") => {
            (true, "Chưa có API key (không sao nếu máy chủ này không cần)".into())
        }
        _ => (false, "Chưa có API key".into()),
    })
}

// ------------------------------------------------------------------ chat ---

#[derive(Deserialize)]
struct ChatBody {
    session_id: Option<String>,
    agent: Option<String>,
    message: String,
}

async fn chat(State(app): State<AppState>, Json(body): Json<ChatBody>) -> Api {
    let existing = match &body.session_id {
        Some(id) => app.sessions.lock().await.get(id).cloned().map(|s| (id.clone(), s)),
        None => None,
    };
    let (id, session, first) = match existing {
        Some((id, session)) => (id, session, false),
        None => {
            let cfg = Config::load()?;
            let agent = cfg.agent_name(body.agent.as_deref())?;
            // Nobody can answer a terminal prompt here: writes go to the queue the UI shows.
            let session = crate::open_session(&cfg, &agent, crate::approval::queue()).await?;
            let session = Arc::new(Mutex::new(session));
            let id = uuid::Uuid::new_v4().simple().to_string();
            app.sessions.lock().await.insert(id.clone(), session.clone());
            (id, session, true)
        }
    };
    let message = if first { crate::with_timestamp(&body.message) } else { body.message };
    let answer = session.lock().await.send(&message).await?;
    Ok(Json(json!({"session_id": id, "answer": answer})))
}

async fn end_chat(State(app): State<AppState>, Path(session): Path<String>) -> Api {
    app.sessions.lock().await.remove(&session);
    ok()
}

// ------------------------------------------------------------- approvals ---

async fn approve(Path(id): Path<String>) -> Api {
    let result = crate::approve_action(&id).await?;
    Ok(Json(json!({"result": result})))
}

async fn reject(Path(id): Path<String>) -> Api {
    crate::reject_action(&id)?;
    ok()
}

// ------------------------------------------------------------------ jobs ---

#[derive(Deserialize)]
struct JobBody {
    agent: Option<String>,
    schedule: String,
    prompt: String,
}

async fn put_job(State(app): State<AppState>, Path(name): Path<String>, Json(body): Json<JobBody>) -> Api {
    check_name("việc", &name)?;
    if body.prompt.trim().is_empty() {
        return Err(anyhow!("Chưa ghi nội dung việc cần làm").into());
    }
    edit_config(&app, |cfg| {
        let job = JobConfig {
            name: name.clone(),
            agent: body.agent.filter(|a| !a.is_empty()),
            schedule: body.schedule.trim().to_string(),
            prompt: body.prompt,
        };
        match cfg.jobs.iter_mut().find(|j| j.name == name) {
            Some(existing) => *existing = job,
            None => cfg.jobs.push(job),
        }
        Ok(())
    })
    .await?;
    ok()
}

async fn delete_job(State(app): State<AppState>, Path(name): Path<String>) -> Api {
    edit_config(&app, |cfg| {
        cfg.jobs.retain(|j| j.name != name);
        Ok(())
    })
    .await?;
    ok()
}

async fn run_job(State(app): State<AppState>, Path(name): Path<String>) -> Api {
    let cfg = Config::load()?;
    let job = cfg.job(&name)?;
    if !app.running_jobs.lock().await.insert(name.clone()) {
        return Err(anyhow!("Việc '{name}' đang chạy rồi").into());
    }
    let result = scheduler::run_job(&cfg, job).await;
    app.running_jobs.lock().await.remove(&name);
    let (report, path) = result?;
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned());
    Ok(Json(json!({"report": report, "file": file})))
}

async fn report(Path((job, file)): Path<(String, String)>) -> Api {
    check_name("việc", &job)?;
    // Report names are timestamps; refusing anything else keeps reads inside the reports folder.
    let stem = file.strip_suffix(".md").context("Tên báo cáo không hợp lệ")?;
    if stem.is_empty() || !stem.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(anyhow!("Tên báo cáo không hợp lệ").into());
    }
    let text = std::fs::read_to_string(scheduler::reports_dir(&job).join(&file)).context("Không đọc được báo cáo")?;
    Ok(Json(json!({"text": text})))
}

// ------------------------------------------------- providers and agents ---

async fn put_provider(State(app): State<AppState>, Path(name): Path<String>, Json(mut body): Json<Value>) -> Api {
    check_name("bộ não", &name)?;
    let key = body.as_object_mut().and_then(|o| o.remove("api_key"));
    let provider: ProviderConfig = serde_json::from_value(body).context("Thông tin bộ não không hợp lệ")?;
    edit_config(&app, |cfg| {
        cfg.providers.insert(name.clone(), provider);
        Ok(())
    })
    .await?;
    if let Some(key) = key.as_ref().and_then(Value::as_str).map(str::trim).filter(|k| !k.is_empty()) {
        secrets::set_provider_key(&name, key)?;
    }
    ok()
}

async fn delete_provider(State(app): State<AppState>, Path(name): Path<String>) -> Api {
    edit_config(&app, |cfg| {
        if let Some((agent, _)) = cfg.agents.iter().find(|(_, a)| a.provider == name) {
            bail!("Agent '{agent}' đang dùng bộ não này; hãy chọn bộ não khác cho agent trước");
        }
        cfg.providers.remove(&name);
        Ok(())
    })
    .await?;
    ok()
}

#[derive(Deserialize)]
struct KeyBody {
    key: String,
}

async fn set_key(Path(name): Path<String>, Json(body): Json<KeyBody>) -> Api {
    let cfg = Config::load()?;
    if !cfg.providers.contains_key(&name) {
        return Err(anyhow!("Không có bộ não tên '{name}'").into());
    }
    let key = body.key.trim();
    if key.is_empty() {
        return Err(anyhow!("Key trống").into());
    }
    secrets::set_provider_key(&name, key)?;
    ok()
}

async fn test_provider(Path(name): Path<String>) -> Api {
    let cfg = Config::load()?;
    let answer = crate::test_provider(&cfg, &name).await?;
    Ok(Json(json!({"answer": answer})))
}

#[derive(Deserialize)]
struct OnboardedBody {
    done: bool,
}

async fn set_onboarded(Json(body): Json<OnboardedBody>) -> Api {
    let marker = onboarded_marker();
    if body.done {
        std::fs::write(marker, "")?;
    } else if marker.exists() {
        std::fs::remove_file(marker)?;
    }
    ok()
}

#[derive(Deserialize)]
struct AgentBody {
    provider: String,
    instructions: String,
}

async fn put_agent(State(app): State<AppState>, Path(name): Path<String>, Json(body): Json<AgentBody>) -> Api {
    edit_config(&app, |cfg| {
        let agent = cfg.agents.get_mut(&name).with_context(|| format!("Không có agent tên '{name}'"))?;
        agent.provider = body.provider;
        agent.instructions = body.instructions;
        Ok(())
    })
    .await?;
    // Open chats keep the brain they started with; drop them so the change is visible at once.
    app.sessions.lock().await.clear();
    ok()
}

// ------------------------------------------------------- gmail, autostart ---

#[derive(Deserialize)]
struct GmailBody {
    client_id: Option<String>,
    client_secret: Option<String>,
}

async fn gmail_login(Json(body): Json<GmailBody>) -> Api {
    let given = |v: Option<String>| v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let (id, secret) = match (given(body.client_id), given(body.client_secret), gmail::builtin_client()) {
        (Some(id), Some(secret), _) => (id, secret),
        (_, _, Some((id, secret))) => (id.to_string(), secret.to_string()),
        _ => match (secrets::get(secrets::GMAIL_CLIENT_ID)?, secrets::get(secrets::GMAIL_CLIENT_SECRET)?) {
            (Some(id), Some(secret)) => (id, secret),
            _ => return Err(anyhow!("Cần Client ID và Client secret của Google").into()),
        },
    };
    gmail::login(&id, &secret).await?;
    // Reading the address doubles as proof that the new login really works.
    let address = gmail::GmailClient::from_secrets()
        .profile_email()
        .await
        .context("Đã đăng nhập nhưng chưa đọc được hộp thư")?;
    secrets::set(secrets::GMAIL_ADDRESS, &address)?;
    Ok(Json(json!({"address": address})))
}

#[derive(Deserialize)]
struct AutostartBody {
    enabled: bool,
}

async fn set_autostart(State(app): State<AppState>, Json(body): Json<AutostartBody>) -> Api {
    app.host.set_autostart(body.enabled)?;
    ok()
}

#[derive(Deserialize)]
struct OpenBody {
    url: String,
}

/// Open a web link from an answer in the system browser.
async fn open_link(Json(body): Json<OpenBody>) -> Api {
    let url = url::Url::parse(&body.url).context("Liên kết không hợp lệ")?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(anyhow!("Chỉ mở được liên kết web").into());
    }
    open::that(url.as_str()).context("Không mở được trình duyệt")?;
    ok()
}
