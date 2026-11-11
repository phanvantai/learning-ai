//! Secret storage: OS keychain on macOS/Windows, a 0600 file elsewhere.
//!
//! `LAI_SECRETS_FILE=1` forces the file store (handy for tests and headless boxes).

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub const GMAIL_CLIENT_ID: &str = "gmail_client_id";
pub const GMAIL_CLIENT_SECRET: &str = "gmail_client_secret";
pub const GMAIL_REFRESH_TOKEN: &str = "gmail_refresh_token";

/// The API key for a provider: its environment variable if set, else the stored key.
pub fn provider_key(provider: &str, env_var: &str) -> Result<Option<String>> {
    match std::env::var(env_var) {
        Ok(key) if !key.is_empty() => Ok(Some(key)),
        _ => get(&format!("api_key:{provider}")),
    }
}

pub fn set_provider_key(provider: &str, key: &str) -> Result<()> {
    set(&format!("api_key:{provider}"), key)
}

fn use_file() -> bool {
    cfg!(not(any(target_os = "macos", target_os = "windows")))
        || std::env::var_os("LAI_SECRETS_FILE").is_some()
}

pub fn get(name: &str) -> Result<Option<String>> {
    if use_file() {
        return Ok(read_file()?.remove(name));
    }
    keychain::get(name)
}

pub fn set(name: &str, value: &str) -> Result<()> {
    if use_file() {
        let mut all = read_file()?;
        all.insert(name.to_string(), value.to_string());
        return write_file(&all);
    }
    keychain::set(name, value)
}

fn file_path() -> PathBuf {
    crate::config::home().join("secrets.json")
}

fn read_file() -> Result<BTreeMap<String, String>> {
    match std::fs::read_to_string(file_path()) {
        Ok(text) => serde_json::from_str(&text).context("secrets.json hỏng"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(e.into()),
    }
}

fn write_file(all: &BTreeMap<String, String>) -> Result<()> {
    let path = file_path();
    std::fs::create_dir_all(crate::config::home())?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    serde_json::to_writer_pretty(file, all)?;
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod keychain {
    use anyhow::Result;

    const SERVICE: &str = "lai-harness";

    pub fn get(name: &str) -> Result<Option<String>> {
        match keyring::Entry::new(SERVICE, name)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set(name: &str, value: &str) -> Result<()> {
        keyring::Entry::new(SERVICE, name)?.set_password(value)?;
        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod keychain {
    use anyhow::Result;

    pub fn get(_: &str) -> Result<Option<String>> {
        unreachable!("file store is always used on this platform")
    }
    pub fn set(_: &str, _: &str) -> Result<()> {
        unreachable!("file store is always used on this platform")
    }
}
