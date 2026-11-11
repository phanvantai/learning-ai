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
    Ok(Cron::new(expr).parse()?)
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
    let report = session.send(&crate::with_timestamp(&job.prompt)).await?;

    let dir = reports_dir(&job.name);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", Local::now().format("%Y-%m-%d_%H%M%S")));
    std::fs::write(&path, &report).with_context(|| format!("Không ghi được {}", path.display()))?;
    Ok((report, path))
}

/// Run forever, firing each job at its scheduled times. Jobs run one at a time.
pub async fn run_daemon(cfg: &Config) -> Result<()> {
    if cfg.jobs.is_empty() {
        anyhow::bail!("Chưa có job nào trong config.toml");
    }
    loop {
        let now = Local::now();
        let (job, at) = cfg
            .jobs
            .iter()
            .map(|job| next_run(job, now).map(|at| (job, at)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .min_by_key(|(_, at)| *at)
            .expect("jobs is not empty");
        println!("Việc kế tiếp: '{}' lúc {}", job.name, at.format("%Y-%m-%d %H:%M"));
        // Sleep in short steps and re-check the wall clock, so the schedule
        // survives the laptop going to sleep.
        while Local::now() < at {
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        }
        println!("[{}] Chạy '{}'...", Local::now().format("%H:%M"), job.name);
        match run_job(cfg, job).await {
            Ok((_, path)) => println!("  Xong. Báo cáo: {}", path.display()),
            Err(e) => eprintln!("  Lỗi: {e:#}"),
        }
        let pending = crate::approval::ApprovalStore::default()
            .all()?
            .iter()
            .filter(|a| a.status == crate::approval::Status::Pending)
            .count();
        if pending > 0 {
            println!("  Có {pending} hành động chờ duyệt: `lai approvals list`");
        }
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
    fn rejects_six_field_expressions() {
        assert!(parse_schedule("0 0 7 * * 1").is_err());
    }
}
