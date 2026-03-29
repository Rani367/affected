use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

/// Helper: create a Cargo workspace in a temp dir with git.
fn setup_cargo_workspace(dir: &Path) {
    std::fs::write(
        dir.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = ["crates/core", "crates/app"]
"#,
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("crates/core/src")).unwrap();
    std::fs::write(
        dir.join("crates/core/Cargo.toml"),
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("crates/core/src/lib.rs"), "pub fn hello() {}\n").unwrap();

    std::fs::create_dir_all(dir.join("crates/app/src")).unwrap();
    std::fs::write(
        dir.join("crates/app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ncore = { path = \"../core\" }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("crates/app/src/main.rs"),
        "fn main() { println!(\"hi\"); }\n",
    )
    .unwrap();

    // Git init + commit
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@test.com",
            "commit",
            "-m",
            "init",
        ])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn affected_cmd() -> Command {
    Command::cargo_bin("affected").unwrap()
}

// ─── detect ─────────────────────────────────────────────────

#[test]
fn test_cli_detect() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .arg("detect")
        .arg("--root")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo"))
        .stdout(predicate::str::contains("core"))
        .stdout(predicate::str::contains("app"));
}

#[test]
fn test_cli_detect_no_project() {
    let dir = tempfile::tempdir().unwrap();

    affected_cmd()
        .arg("detect")
        .arg("--root")
        .arg(dir.path())
        .assert()
        .failure();
}

// ─── graph ──────────────────────────────────────────────────

#[test]
fn test_cli_graph() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .arg("graph")
        .arg("--root")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("→"));
}

#[test]
fn test_cli_graph_dot() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .arg("graph")
        .arg("--dot")
        .arg("--root")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph"))
        .stdout(predicate::str::contains("->"));
}

// ─── list ───────────────────────────────────────────────────

#[test]
fn test_cli_list_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .args(["list", "--base", "HEAD", "--root"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No packages affected"));
}

#[test]
fn test_cli_list_with_changes() {
    let dir = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(dir.path()).unwrap();
    setup_cargo_workspace(&root);

    // Make a change
    std::fs::write(
        root.join("crates/core/src/lib.rs"),
        "pub fn hello() { /* v2 */ }\n",
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(&root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@test.com",
            "commit",
            "-m",
            "change core",
        ])
        .current_dir(&root)
        .output()
        .unwrap();

    affected_cmd()
        .args(["list", "--base", "HEAD~1", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("core"))
        .stdout(predicate::str::contains("app"));
}

#[test]
fn test_cli_list_json() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    // Make a change
    std::fs::write(
        dir.path().join("crates/core/src/lib.rs"),
        "pub fn hello() { /* changed */ }\n",
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@test.com",
            "commit",
            "-m",
            "change",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let output = affected_cmd()
        .args(["list", "--base", "HEAD~1", "--json", "--root"])
        .arg(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["affected"].is_array());
    assert!(json["total_packages"].as_u64().unwrap() >= 2);
    assert!(json["changed_files"].as_u64().unwrap() >= 1);
}

// ─── test --dry-run ─────────────────────────────────────────

#[test]
fn test_cli_test_dry_run() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    // Make a change
    std::fs::write(
        dir.path().join("crates/app/src/main.rs"),
        "fn main() { println!(\"v2\"); }\n",
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@test.com",
            "commit",
            "-m",
            "change app",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    affected_cmd()
        .args(["test", "--base", "HEAD~1", "--dry-run", "--root"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"))
        .stdout(predicate::str::contains("cargo test"));
}

#[test]
fn test_cli_test_dry_run_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .args(["test", "--base", "HEAD", "--dry-run", "--root"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No packages affected"));
}

// ─── Error cases ────────────────────────────────────────────

#[test]
fn test_cli_invalid_base_ref() {
    let dir = tempfile::tempdir().unwrap();
    setup_cargo_workspace(dir.path());

    affected_cmd()
        .args(["list", "--base", "nonexistent-ref", "--root"])
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_cli_no_subcommand() {
    affected_cmd().assert().failure();
}

#[test]
fn test_cli_version() {
    affected_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("affected"));
}

#[test]
fn test_cli_help() {
    affected_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run only the tests that matter"));
}
