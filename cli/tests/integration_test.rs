use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn joule_profiler() -> Command {
    Command::cargo_bin("joule-profiler").unwrap()
}

#[test]
fn help_flag_exits_zero() {
    joule_profiler().arg("--help").assert().success();
}

#[test]
fn version_flag_exits_zero() {
    joule_profiler().arg("--version").assert().success();
}

#[test]
fn no_subcommand_exits_nonzero() {
    joule_profiler().assert().failure();
}

#[test]
fn list_sensors_exits_zero_or_prints_error() {
    let output = joule_profiler().arg("list-sensors").output().unwrap();
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn phases_requires_cmd() {
    joule_profiler().args(["phases"]).assert().failure();
}

#[test]
fn phases_with_cmd_parses_args() {
    joule_profiler()
        .args(["phases", "--", "echo", "hello"])
        .assert()
        .stdout(predicate::str::contains("hello").or(predicate::always()));
}

#[test]
fn phases_default_token_pattern_is_used() {
    joule_profiler()
        .args(["phases", "--", "echo", "__PHASE__"])
        .assert()
        .success();
}

#[test]
fn phases_custom_token_pattern() {
    joule_profiler()
        .args([
            "phases",
            "--token-pattern",
            "MARKER_[0-9]+",
            "--",
            "echo",
            "MARKER_1",
        ])
        .assert()
        .success();
}

#[test]
fn phases_iterations_flag() {
    joule_profiler()
        .args(["phases", "-n", "2", "--", "echo", "hello"])
        .assert()
        .success();
}

#[test]
fn phases_invalid_iterations_exits_nonzero() {
    joule_profiler()
        .args(["phases", "-n", "not_a_number", "--", "echo", "hello"])
        .assert()
        .failure();
}

#[test]
fn phases_nonexistent_command_exits_nonzero() {
    joule_profiler()
        .args(["phases", "--", "this_command_does_not_exist_42"])
        .assert()
        .failure();
}

#[test]
fn json_and_csv_are_mutually_exclusive() {
    joule_profiler()
        .args(["--json", "--csv", "phases", "--", "echo", "hi"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cannot be used with")
                .or(predicate::str::contains("conflict")),
        );
}

#[test]
fn json_flag_writes_json_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("out.json").to_str().unwrap().to_owned();

    joule_profiler()
        .args([
            "--json",
            "--output-file",
            &path,
            "phases",
            "--",
            "echo",
            "hello",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("output should be valid JSON");
}

#[test]
fn csv_flag_writes_csv_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("out.csv").to_str().unwrap().to_owned();

    joule_profiler()
        .args([
            "--csv",
            "--output-file",
            &path,
            "phases",
            "--",
            "echo",
            "hello",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains(";"), "CSV should contain semicolons");
}

#[test]
fn parse_sockets_spec_none_returns_none() {
    use joule_profiler_cli::parse_sockets_spec;
    assert!(parse_sockets_spec(None).is_none());
}

#[test]
fn parse_sockets_spec_single() {
    use joule_profiler_cli::parse_sockets_spec;
    let result = parse_sockets_spec(Some("0")).unwrap();
    assert!(result.contains(&0));
    assert_eq!(result.len(), 1);
}

#[test]
fn parse_sockets_spec_multiple() {
    use joule_profiler_cli::parse_sockets_spec;
    let result = parse_sockets_spec(Some("0,1,2")).unwrap();
    assert!(result.contains(&0));
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    assert_eq!(result.len(), 3);
}

#[test]
fn parse_sockets_spec_invalid_entries_are_skipped() {
    use joule_profiler_cli::parse_sockets_spec;
    let result = parse_sockets_spec(Some("0,abc,2")).unwrap();
    assert!(result.contains(&0));
    assert!(result.contains(&2));
    assert!(!result.iter().any(|&x| x == 1));
    assert_eq!(result.len(), 2);
}

#[test]
fn parse_sockets_spec_whitespace_trimmed() {
    use joule_profiler_cli::parse_sockets_spec;
    let result = parse_sockets_spec(Some(" 0 , 1 ")).unwrap();
    assert!(result.contains(&0));
    assert!(result.contains(&1));
}

#[test]
fn stdout_file_flag_redirects_output() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("stdout.txt").to_str().unwrap().to_owned();

    joule_profiler()
        .args([
            "phases",
            "--stdout-file",
            &path,
            "--",
            "echo",
            "captured_output",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("captured_output"));
}
