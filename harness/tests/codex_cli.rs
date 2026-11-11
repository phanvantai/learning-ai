//! The ChatGPT-subscription provider, against a stand-in for the `codex` command.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use lai_harness::codex_cli::CodexCliSession;
use lai_harness::config::CodexCliConfig;
use lai_harness::Session;

#[tokio::test]
async fn sends_rules_once_and_resumes_the_thread() {
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("LAI_HOME", home.path());
    let log = home.path().join("calls.log");
    let script = home.path().join("fake-codex");
    // Logs its arguments and stdin, writes the answer where `-o` points, prints the thread event.
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {log}\nprintf 'stdin=' >> {log}\ncat | tr '\\n' ' ' >> {log}\necho >> {log}\n\
             while [ $# -gt 0 ]; do if [ \"$1\" = \"-o\" ]; then out=\"$2\"; fi; shift; done\n\
             printf 'Chào bạn\\n' > \"$out\"\n\
             echo '{{\"type\":\"thread.started\",\"thread_id\":\"th-1\"}}'\n",
            log = log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let cfg = CodexCliConfig { command: script.to_string_lossy().into_owned(), model: Some("m1".into()) };
    let mut session = CodexCliSession::new(&cfg, "tro-ly", "QUY TẮC").unwrap();
    assert_eq!(session.send("việc một").await.unwrap(), "Chào bạn");
    assert_eq!(session.send("việc hai").await.unwrap(), "Chào bạn");

    let calls = std::fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = calls.lines().collect();
    assert!(lines[0].starts_with("exec --json --skip-git-repo-check --ignore-user-config -c sandbox_mode=\"read-only\""), "{}", lines[0]);
    assert!(lines[0].contains("mcp_servers.lai.args=[\"mcp-serve\", \"--agent\", \"tro-ly\"]"), "{}", lines[0]);
    assert!(lines[0].contains("-m m1 -"), "{}", lines[0]);
    assert!(lines[1].contains("QUY TẮC") && lines[1].contains("việc một"), "{}", lines[1]);
    assert!(lines[2].starts_with("exec resume th-1 --json"), "{}", lines[2]);
    assert_eq!(lines[3].trim(), "stdin=việc hai");
}
