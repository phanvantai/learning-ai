use std::io::{BufRead, IsTerminal, Write};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use lai_harness::approval::{self, ApprovalStore, Approver, Status};
use lai_harness::config::{self, Config};
use lai_harness::{gmail, scheduler, secrets};

#[derive(Parser)]
#[command(name = "lai", version, about = "Trợ lý AI cá nhân: agent nhận việc, tự kiểm tra mail, báo cáo theo lịch")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Tạo file cấu hình mẫu
    Init,
    /// Kiểm tra cấu hình và các kết nối
    Doctor,
    /// Kết nối tài khoản
    Auth {
        #[command(subcommand)]
        target: AuthTarget,
    },
    /// Trò chuyện với một agent
    Chat {
        #[arg(long)]
        agent: Option<String>,
    },
    /// Giao một việc và nhận kết quả
    Run {
        #[arg(long)]
        agent: Option<String>,
        /// Nội dung việc cần làm
        #[arg(required = true)]
        prompt: Vec<String>,
    },
    /// Việc chạy theo lịch
    Jobs {
        #[command(subcommand)]
        action: JobsAction,
    },
    /// Mở giao diện trong trình duyệt (kèm chạy job đúng lịch)
    Ui {
        /// Chỉ in địa chỉ, không tự mở trình duyệt
        #[arg(long)]
        no_open: bool,
    },
    /// Chạy nền không giao diện, thực hiện các job đúng lịch
    Daemon,
    /// Xem báo cáo mới nhất của một job
    Report { job: String },
    /// Duyệt các hành động agent muốn thực hiện
    Approvals {
        #[command(subcommand)]
        action: ApprovalsAction,
    },
    /// (Nội bộ) Phục vụ công cụ của một agent qua MCP cho Claude CLI
    #[command(hide = true)]
    McpServe {
        #[arg(long)]
        agent: String,
    },
}

#[derive(Subcommand)]
enum AuthTarget {
    /// Nhập API key cho một provider dùng API key
    Key {
        /// Tên provider trong config.toml (bỏ trống nếu chỉ có một)
        provider: Option<String>,
    },
    /// Đăng nhập Gmail qua trình duyệt
    Gmail,
}

#[derive(Subcommand)]
enum JobsAction {
    /// Liệt kê job và lần chạy kế tiếp
    List,
    /// Chạy một job ngay bây giờ
    Run { name: String },
}

#[derive(Subcommand)]
enum ApprovalsAction {
    /// Liệt kê hành động đang chờ duyệt
    List,
    /// Đồng ý và thực hiện
    Approve { id: String },
    /// Từ chối
    Reject { id: String },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run(Cli::parse()).await {
        eprintln!("Lỗi: {e:#}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init => init(),
        Command::Doctor => doctor(),
        Command::Auth { target: AuthTarget::Key { provider } } => {
            let cfg = Config::load()?;
            let with_keys: Vec<&String> =
                cfg.providers.iter().filter(|(_, p)| p.cli_command().is_none()).map(|(name, _)| name).collect();
            let name = match provider {
                Some(name) if with_keys.iter().any(|n| **n == name) => name,
                Some(name) => bail!("'{name}' không phải provider dùng API key"),
                None if with_keys.len() == 1 => with_keys[0].clone(),
                None => bail!("Ghi rõ provider: {}", with_keys.iter().map(|n| n.as_str()).collect::<Vec<_>>().join(", ")),
            };
            let key = rpassword::prompt_password(format!("Dán API key cho '{name}' (không hiện khi gõ): "))?;
            if key.trim().is_empty() {
                bail!("Key trống");
            }
            secrets::set_provider_key(&name, key.trim())?;
            println!("Đã lưu API key.");
            Ok(())
        }
        Command::Auth { target: AuthTarget::Gmail } => {
            println!("Cần OAuth client loại \"Desktop app\" của Google Cloud (xem harness/README.md, mục Gmail).");
            let saved_id = secrets::get(secrets::GMAIL_CLIENT_ID)?;
            let saved_secret = secrets::get(secrets::GMAIL_CLIENT_SECRET)?;
            let client_id = ask("Client ID", saved_id)?;
            let client_secret = ask("Client secret", saved_secret)?;
            gmail::login(&client_id, &client_secret).await?;
            println!("Đã kết nối Gmail.");
            Ok(())
        }
        Command::Chat { agent } => chat(agent).await,
        Command::Run { agent, prompt } => {
            let cfg = Config::load()?;
            let agent = cfg.agent_name(agent.as_deref())?;
            let mut session = lai_harness::open_session(&cfg, &agent, approver_for_terminal()).await?;
            let answer = session.send(&lai_harness::with_timestamp(&prompt.join(" "))).await?;
            println!("{answer}");
            Ok(())
        }
        Command::Jobs { action: JobsAction::List } => {
            let cfg = Config::load()?;
            for job in &cfg.jobs {
                let next = scheduler::next_run(job, chrono::Local::now())?;
                println!("{:<28} {:<16} kế tiếp: {}", job.name, job.schedule, next.format("%Y-%m-%d %H:%M"));
            }
            Ok(())
        }
        Command::Jobs { action: JobsAction::Run { name } } => {
            let cfg = Config::load()?;
            let (report, path) = scheduler::run_job(&cfg, cfg.job(&name)?).await?;
            println!("{report}\n\n(Đã lưu: {})", path.display());
            Ok(())
        }
        Command::Ui { no_open } => {
            let url = lai_harness::server::start(Arc::new(lai_harness::NoHost)).await?;
            println!("Giao diện: {url}\nNhấn Ctrl+C để dừng.");
            if !no_open {
                let _ = open::that(&url);
            }
            std::future::pending::<()>().await;
            Ok(())
        }
        Command::Daemon => {
            Config::load()?;
            println!("Đang chạy các việc theo lịch. Nhấn Ctrl+C để dừng.");
            scheduler::run_loop(Arc::new(lai_harness::NoHost)).await;
            Ok(())
        }
        Command::Report { job } => {
            let latest = scheduler::list_reports(&job)
                .into_iter()
                .next()
                .with_context(|| format!("Job '{job}' chưa có báo cáo nào"))?;
            let path = scheduler::reports_dir(&job).join(latest);
            println!("{}\n\n({})", std::fs::read_to_string(&path)?, path.display());
            Ok(())
        }
        Command::Approvals { action } => approvals(action).await,
        Command::McpServe { agent } => lai_harness::mcp_serve(&agent).await,
    }
}

/// Ask on the terminal when someone is there; otherwise park write actions in the queue.
fn approver_for_terminal() -> Arc<dyn Approver> {
    if std::io::stdin().is_terminal() {
        Arc::new(approval::Interactive)
    } else {
        approval::queue()
    }
}

fn ask(label: &str, saved: Option<String>) -> Result<String> {
    match &saved {
        Some(_) => print!("{label} (Enter để giữ giá trị đã lưu): "),
        None => print!("{label}: "),
    }
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    match (line.trim(), saved) {
        ("", Some(saved)) => Ok(saved),
        ("", None) => bail!("{label} trống"),
        (value, _) => Ok(value.to_string()),
    }
}

fn init() -> Result<()> {
    let path = config::config_path();
    if path.exists() {
        println!("Đã có cấu hình: {}", path.display());
        return Ok(());
    }
    std::fs::create_dir_all(config::home())?;
    std::fs::write(&path, config::STARTER_CONFIG)?;
    println!(
        "Đã tạo {}\n\nBước tiếp theo:\n  1. lai ui               (mở giao diện; hoặc tiếp tục bằng dòng lệnh)\n  2. lai auth gmail\n  3. lai doctor\n  4. lai chat\n\nMặc định trợ lý dùng gói Claude qua lệnh `claude`. Muốn dùng API key: đổi `provider` của agent rồi `lai auth key`",
        path.display()
    );
    Ok(())
}

fn doctor() -> Result<()> {
    let cfg = Config::load()?;
    println!("✓ Cấu hình hợp lệ: {}", config::config_path().display());
    let mark = |ok: bool| if ok { "✓" } else { "✗" };

    for (name, provider) in &cfg.providers {
        if let Some(command) = provider.cli_command() {
            let found = lai_harness::proc::resolve(command).is_some();
            println!(
                "{} provider '{name}': lệnh `{command}`{}",
                mark(found),
                if found { " (chạy `lai ui` rồi bấm Thử kết nối để kiểm tra đăng nhập)" } else { " — chưa cài hoặc không tìm thấy" }
            );
            continue;
        }
        let ok = lai_harness::provider_api_key(name, provider)?.is_some();
        println!("{} provider '{name}': API key{}", mark(ok), if ok { String::new() } else { format!(" — chạy `lai auth key {name}`") });
    }
    let uses_gmail = cfg.agents.values().any(|a| a.tools.iter().any(|t| t.starts_with("gmail_")));
    if uses_gmail {
        let connected = secrets::get(secrets::GMAIL_REFRESH_TOKEN)?.is_some();
        println!("{} Gmail{}", mark(connected), if connected { "" } else { " — chạy `lai auth gmail`" });
    }
    println!("  {} agent, {} job", cfg.agents.len(), cfg.jobs.len());
    Ok(())
}

async fn chat(agent: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let agent = cfg.agent_name(agent.as_deref())?;
    let mut session = lai_harness::open_session(&cfg, &agent, approver_for_terminal()).await?;
    println!("Đang trò chuyện với '{agent}'. Gõ /exit để thoát.");
    let mut first = true;
    loop {
        print!("\n> ");
        std::io::stdout().flush()?;
        let line = tokio::task::spawn_blocking(|| {
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).map(|n| (n, line))
        })
        .await??;
        let text = line.1.trim();
        if line.0 == 0 || text == "/exit" {
            return Ok(());
        }
        if text.is_empty() {
            continue;
        }
        let message = if first { lai_harness::with_timestamp(text) } else { text.to_string() };
        match session.send(&message).await {
            Ok(answer) => {
                first = false;
                println!("\n{answer}");
            }
            Err(e) => eprintln!("\nLỗi: {e:#}"),
        }
    }
}

async fn approvals(action: ApprovalsAction) -> Result<()> {
    let store = ApprovalStore::default();
    match action {
        ApprovalsAction::List => {
            let pending: Vec<_> = store.all()?.into_iter().filter(|a| a.status == Status::Pending).collect();
            if pending.is_empty() {
                println!("Không có hành động nào chờ duyệt.");
            }
            for a in pending {
                println!(
                    "── {}  {}  (agent {}, {})\n{}\n",
                    a.id,
                    a.tool,
                    a.agent,
                    a.created_at.format("%Y-%m-%d %H:%M"),
                    serde_json::to_string_pretty(&a.input)?
                );
            }
            Ok(())
        }
        ApprovalsAction::Reject { id } => {
            lai_harness::reject_action(&id)?;
            println!("Đã từ chối {id}.");
            Ok(())
        }
        ApprovalsAction::Approve { id } => {
            println!("{}", lai_harness::approve_action(&id).await?);
            Ok(())
        }
    }
}
