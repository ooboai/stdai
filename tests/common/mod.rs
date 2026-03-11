use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

pub fn stdai() -> Command {
    Command::cargo_bin("stdai").unwrap()
}

/// Create an isolated global store for testing.
/// Returns a TempDir whose path should be passed as STDAI_HOME.
pub fn create_test_env() -> TempDir {
    TempDir::new().unwrap()
}

/// Build a command with STDAI_HOME and STDAI_PROJECT set for isolation.
pub fn stdai_cmd(home: &Path) -> Command {
    let mut cmd = stdai();
    cmd.env("STDAI_HOME", home);
    cmd.env("STDAI_PROJECT", "test-project");
    cmd
}

pub fn write_artifact(home: &Path, kind: &str, content: &str) -> String {
    let output = stdai_cmd(home)
        .args(["write", "--kind", kind, "--content", content])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "write failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}

pub fn write_artifact_json(home: &Path, kind: &str, content: &str) -> serde_json::Value {
    let output = stdai_cmd(home)
        .args(["write", "--kind", kind, "--content", content, "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "write failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Write an artifact under a specific project name.
pub fn write_artifact_in_project(home: &Path, project: &str, kind: &str, content: &str) -> String {
    let output = stdai()
        .env("STDAI_HOME", home)
        .env("STDAI_PROJECT", project)
        .args(["write", "--kind", kind, "--content", content])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "write failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}
