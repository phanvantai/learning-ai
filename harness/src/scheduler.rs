//! Scheduled jobs: run an agent on a cron schedule and file its answer as a report.

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use croner::Cron;

use crate::config::{Config, JobConfig};

/// Parse a standard 5-field cron expression (minute hour day month weekday).
pub fn parse_schedule(expr: &str) -> Result<Cron> {
    if expr.split_whitespace().count() != 5 {
        anyhow::bail!("cần đúng 5 trường: phút giờ ngày tháng thứ");
    }
    let cron = Cron::new(expr).parse()?;
    // "31 February" parses but never comes.
    cron.find_next_occurrence(&Local::now(), false).map_err(|_| anyhow::anyhow!("lịch này không bao giờ tới"))?;
    Ok(cron)
}

pub fn next_run(job: &JobConfig, after: DateTime<Local>) -> Result<DateTime<Local>> {
    Ok(parse_schedule(&job.schedule)?.find_next_occurrence(&after, false)?)
}

pub fn reports_dir(job: &str) -> PathBuf {
    crate::config::home().join("reports").join(job)
}

/// Run one job now. Write actions are queued for approval, never executed.
/// Returns the report text and where it was saved.
pub async fn run_job(cfg: &Config, job: &JobConfig) -> Result<(String, PathBuf)> {
    let agent = cfg.agent_name(job.agent.as_deref())?;
    let mut session = crate::open_session(cfg, &agent, crate::approval::queue()).await?;
    // A stuck run must not hold up every later job.
    let report = tokio::time::timeout(std::time::Duration::from_secs(900), session.send(&crate::with_timestamp(&job.prompt)))
        .await
        .context("Việc chạy quá 15 phút nên đã bị dừng")??;

    let dir = reports_dir(&job.name);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", Local::now().format("%Y-%m-%d_%H%M%S")));
    std::fs::write(&path, &report).with_context(|| format!("Không ghi được {}", path.display()))?;
    Ok((report, path))
}

/// Reports of a job, newest first.
pub fn list_reports(job: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(reports_dir(job))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".md"))
        .collect();
    names.sort_by(|a, b| b.cmp(a));
    names
}

/// Run forever, firing each job at its scheduled times, one at a time.
///
/// The config is re-read on every tick, so jobs edited while we run take
/// effect without a restart, and the wall clock is re-checked rather than
/// slept through, so the schedule survives the laptop sleeping.
pub async fn run_loop(host: std::sync::Arc<dyn crate::Host>) {
    let mut checked_until = Local::now();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        let now = Local::now();
        let Ok(cfg) = Config::load() else {
            checked_until = now;
            continue;
        };
        for job in &cfg.jobs {
            // Due if a scheduled time fell in (checked_until, now].
            if !next_run(job, checked_until).is_ok_and(|at| at <= now) {
                continue;
            }
            println!("[{}] Chạy '{}'...", Local::now().format("%H:%M"), job.name);
            match run_job(&cfg, job).await {
                Ok((_, path)) => {
                    println!("  Xong. Báo cáo: {}", path.display());
                    host.notify("Có báo cáo mới", &format!("Việc '{}' đã chạy xong.", job.name));
                }
                Err(e) => {
                    eprintln!("  Lỗi: {e:#}");
                    host.notify("Việc theo lịch gặp lỗi", &format!("'{}': {e:#}", job.name));
                }
            }
            let pending = crate::approval::ApprovalStore::default().pending_count();
            if pending > 0 {
                host.notify("Có việc chờ bạn duyệt", &format!("{pending} hành động đang chờ duyệt."));
            }
        }
        checked_until = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike, Weekday};

    #[test]
    fn weekday_numbers_follow_standard_cron() {
        let job = JobConfig {
            name: "j".into(),
            agent: None,
            schedule: "0 7 * * 1-5".into(),
            prompt: String::new(),
        };
        // Saturday 2026-11-14 08:00 -> next weekday run is Monday 07:00.
        let saturday = Local.with_ymd_and_hms(2026, 11, 14, 8, 0, 0).unwrap();
        let next = next_run(&job, saturday).unwrap();
        assert_eq!(next.weekday(), Weekday::Mon);
        assert_eq!((next.day(), next.hour(), next.minute()), (16, 7, 0));
    }

    #[test]
    fn rejects_dates_that_never_come() {
        assert!(parse_schedule("0 7 31 2 *").is_err());
    }

    #[test]
    fn rejects_six_field_expressions() {
        assert!(parse_schedule("0 0 7 * * 1").is_err());
    }
}
