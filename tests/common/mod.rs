use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

pub fn stdai() -> Command {
    Command::cargo_bin("stdai").unwrap()
}

pub fn init_workspace(dir: &Path) {
    stdai()
        .arg("init")
        .current_dir(dir)
        .assert()
        .success();
}

pub fn create_workspace() -> TempDir {
    let dir = TempDir::new().unwrap();
    init_workspace(dir.path());
    dir
}

pub fn write_artifact(dir: &Path, kind: &str, content: &str) -> String {
    let output = stdai()
        .args(["write", "--kind", kind, "--content", content])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "write failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

pub fn write_artifact_json(dir: &Path, kind: &str, content: &str) -> serde_json::Value {
    let output = stdai()
        .args(["write", "--kind", kind, "--content", content, "--json"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "write failed: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}
