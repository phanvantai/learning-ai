//! Starting other programs (the vendor CLIs, MCP servers) from a desktop app.
//!
//! Apps launched from Finder or the Start menu get a bare `PATH` that lacks
//! the places command-line tools install themselves, so we look there too.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn extra_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".claude/local"));
        dirs.push(home.join(".npm-global/bin"));
        dirs.push(home.join(".bun/bin"));
        dirs.push(home.join(".volta/bin"));
    }
    if cfg!(windows) {
        if let Some(data) = dirs::config_dir() {
            dirs.push(data.join("npm"));
        }
    } else {
        dirs.push("/opt/homebrew/bin".into());
        dirs.push("/usr/local/bin".into());
    }
    dirs
}

/// `PATH` plus the usual install locations.
pub fn search_path() -> OsString {
    let mut dirs: Vec<PathBuf> =
        std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    for dir in extra_dirs() {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    std::env::join_paths(dirs).unwrap_or_default()
}

/// Find a program by name. A name with a path separator is used as given.
pub fn resolve(name: &str) -> Option<PathBuf> {
    if Path::new(name).components().count() > 1 {
        return Path::new(name).is_file().then(|| name.into());
    }
    // On Windows npm installs `.cmd` shims, which are only found by full name.
    let extensions: &[&str] = if cfg!(windows) { &["", ".exe", ".cmd", ".bat"] } else { &[""] };
    std::env::split_paths(&search_path())
        .flat_map(|dir| extensions.iter().map(move |ext| dir.join(format!("{name}{ext}"))))
        .find(|candidate| candidate.is_file())
}

/// A command for `name` that can find its own helpers (e.g. `node`) and, on
/// Windows, does not flash a console window when started from the app.
pub fn command(name: &str) -> tokio::process::Command {
    let program = resolve(name).unwrap_or_else(|| name.into());
    let mut cmd = tokio::process::Command::new(program);
    cmd.env("PATH", search_path());
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
