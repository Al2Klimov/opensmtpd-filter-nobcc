use std::io::Write;
use std::process::{Command, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_opensmtpd-filter-nobcc");

fn run_filter(input: &[u8]) -> (String, String) {
    let mut child = Command::new(BINARY)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn filter binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input)
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait for output");
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn config_ready_registers_all_events() {
    let (stdout, _) = run_filter(b"config|ready\n");
    assert!(stdout.contains("register|report|smtp-in|tx-begin\n"));
    assert!(stdout.contains("register|report|smtp-in|tx-rcpt\n"));
    assert!(stdout.contains("register|filter|smtp-in|data-line\n"));
    assert!(stdout.contains("register|filter|smtp-in|commit\n"));
    assert!(stdout.contains("register|report|smtp-in|link-disconnect\n"));
    assert!(stdout.contains("register|ready\n"));
}

#[test]
fn commit_proceeds_when_recipient_in_to() {
    let input = b"\
config|ready\n\
report|0.7|1|smtp-in|tx-begin|s1\n\
report|0.7|1|smtp-in|tx-rcpt|s1|m1|ok|rcpt@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|To: rcpt@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|\n\
filter|0.7|1|smtp-in|data-line|s1|t1|.\n\
filter|0.7|1|smtp-in|commit|s1|t1\n\
";
    let (stdout, _) = run_filter(input);
    assert!(stdout.contains("filter-result|s1|t1|proceed\n"));
}

#[test]
fn commit_proceeds_when_recipient_in_cc() {
    let input = b"\
config|ready\n\
report|0.7|1|smtp-in|tx-begin|s1\n\
report|0.7|1|smtp-in|tx-rcpt|s1|m1|ok|cc@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|Cc: cc@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|\n\
filter|0.7|1|smtp-in|data-line|s1|t1|.\n\
filter|0.7|1|smtp-in|commit|s1|t1\n\
";
    let (stdout, _) = run_filter(input);
    assert!(stdout.contains("filter-result|s1|t1|proceed\n"));
}

#[test]
fn commit_rejects_when_recipient_not_in_to_or_cc() {
    let input = b"\
config|ready\n\
report|0.7|1|smtp-in|tx-begin|s1\n\
report|0.7|1|smtp-in|tx-rcpt|s1|m1|ok|bcc@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|To: rcpt@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|\n\
filter|0.7|1|smtp-in|data-line|s1|t1|.\n\
filter|0.7|1|smtp-in|commit|s1|t1\n\
";
    let (stdout, _) = run_filter(input);
    assert!(stdout.contains("filter-result|s1|t1|reject|550 BCC not allowed\n"));
}

#[test]
fn data_line_is_echoed_to_stdout() {
    let input = b"\
config|ready\n\
report|0.7|1|smtp-in|tx-begin|s1\n\
filter|0.7|1|smtp-in|data-line|s1|t1|Subject: hello\n\
";
    let (stdout, _) = run_filter(input);
    assert!(stdout.contains("filter-dataline|s1|t1|Subject: hello\n"));
}

#[test]
fn link_disconnect_removes_session() {
    let input = b"\
config|ready\n\
report|0.7|1|smtp-in|tx-begin|s1\n\
report|0.7|1|smtp-in|tx-rcpt|s1|m1|ok|bcc@example.com\n\
report|0.7|1|smtp-in|link-disconnect|s1\n\
filter|0.7|1|smtp-in|data-line|s1|t1|To: rcpt@example.com\n\
filter|0.7|1|smtp-in|data-line|s1|t1|\n\
filter|0.7|1|smtp-in|data-line|s1|t1|.\n\
filter|0.7|1|smtp-in|commit|s1|t1\n\
";
    // After link-disconnect the session is gone, so the filter proceeds (unknown session = allow)
    let (stdout, _) = run_filter(input);
    assert!(stdout.contains("filter-result|s1|t1|proceed\n"));
}
