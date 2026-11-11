//! The subscription provider, against a stand-in for the `claude` command.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use lai_harness::claude_cli::ClaudeCliSession;
use lai_harness::config::ClaudeCliConfig;
use lai_harness::Session;

/// Both tests share one process, so they must agree on a single `LAI_HOME`.
fn home() -> &'static std::path::Path {
    static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = HOME.get_or_init(|| tempfile::tempdir().unwrap());
    std::env::set_var("LAI_HOME", dir.path());
    dir.path()
}

#[tokio::test]
async fn passes_prompt_on_stdin_and_resumes_the_session() {
    let home = home();
    let log = home.join("calls.log");
    let script = home.join("fake-claude");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {log}\nprintf 'stdin=' >> {log}\ncat >> {log}\necho >> {log}\n\
             echo '{{\"type\":\"result\",\"is_error\":false,\"result\":\"Chào bạn\",\"session_id\":\"sid-1\"}}'\n",
            log = log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let cfg = ClaudeCliConfig { command: script.to_string_lossy().into_owned(), model: None };
    let mut session = ClaudeCliSession::new(&cfg, "tro-ly", "quy tắc").unwrap();
    assert_eq!(session.send("việc một").await.unwrap(), "Chào bạn");
    assert_eq!(session.send("việc hai").await.unwrap(), "Chào bạn");

    let calls = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = calls.lines().collect();
    // Built-in tools off, only our MCP server, only its tools pre-approved.
    assert!(lines[0].contains("--strict-mcp-config --tools  --allowedTools mcp__lai"), "{}", lines[0]);
    assert!(!lines[0].contains("--resume"));
    assert_eq!(lines[1], "stdin=việc một");
    assert!(lines[2].contains("--resume sid-1"), "{}", lines[2]);
    assert_eq!(lines[3], "stdin=việc hai");

    // The MCP config points the CLI back at this harness for the same agent.
    let mcp: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("run/mcp-tro-ly.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["lai"]["args"], serde_json::json!(["mcp-serve", "--agent", "tro-ly"]));
}

#[tokio::test]
async fn missing_command_explains_how_to_fix() {
    home();
    let cfg = ClaudeCliConfig { command: "/nonexistent/claude".into(), model: None };
    let mut session = ClaudeCliSession::new(&cfg, "a", "s").unwrap();
    let err = session.send("x").await.unwrap_err();
    assert!(err.to_string().contains("Cài Claude Code"), "{err}");
}
