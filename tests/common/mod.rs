use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

pub fn stdai() -> Command {
    Command::cargo_bin("stdai").unwrap()
}

/// Create an isolated global store for testing with a pre-created identity.
/// Returns (TempDir, identity_address).
pub fn create_test_env() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let address = create_test_identity(dir.path());
    (dir, address)
}

/// Build a command with STDAI_HOME, STDAI_PROJECT, and STDAI_IDENTITY set.
pub fn stdai_cmd(home: &Path, identity: &str) -> Command {
    let mut cmd = stdai();
    cmd.env("STDAI_HOME", home);
    cmd.env("STDAI_PROJECT", "test-project");
    cmd.env("STDAI_IDENTITY", identity);
    cmd
}

/// Build a command with only STDAI_HOME and STDAI_PROJECT (no identity).
pub fn stdai_cmd_no_identity(home: &Path) -> Command {
    let mut cmd = stdai();
    cmd.env("STDAI_HOME", home);
    cmd.env("STDAI_PROJECT", "test-project");
    cmd.env_remove("STDAI_IDENTITY");
    cmd
}

pub fn create_test_identity(home: &Path) -> String {
    let output = stdai()
        .env("STDAI_HOME", home)
        .args(["identity", "new", "--label", "test-agent"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "identity new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(addr) = trimmed.strip_prefix("address:") {
            let addr = addr.trim();
            if addr.starts_with("stdai:") {
                return addr.to_string();
            }
        }
        if trimmed.starts_with("stdai:") && !trimmed.contains(' ') {
            return trimmed.to_string();
        }
    }
    panic!(
        "could not parse identity address from output:\n{}",
        stdout
    );
}

pub fn write_artifact(home: &Path, identity: &str, kind: &str, content: &str) -> String {
    let output = stdai_cmd(home, identity)
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

pub fn write_artifact_json(
    home: &Path,
    identity: &str,
    kind: &str,
    content: &str,
) -> serde_json::Value {
    let output = stdai_cmd(home, identity)
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
pub fn write_artifact_in_project(
    home: &Path,
    identity: &str,
    project: &str,
    kind: &str,
    content: &str,
) -> String {
    let output = stdai()
        .env("STDAI_HOME", home)
        .env("STDAI_PROJECT", project)
        .env("STDAI_IDENTITY", identity)
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
