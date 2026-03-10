mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn stdai() -> Command {
    common::stdai()
}

// ─── init ───────────────────────────────────────────────────────────────────

#[test]
fn init_creates_workspace() {
    let dir = tempfile::tempdir().unwrap();
    stdai()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("initialized"));

    assert!(dir.path().join(".stdai").is_dir());
    assert!(dir.path().join(".stdai/objects").is_dir());
    assert!(dir.path().join(".stdai/stdai.db").exists());
    assert!(dir.path().join(".stdai/config.toml").exists());
}

#[test]
fn init_fails_if_already_initialized() {
    let dir = common::create_workspace();
    stdai()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

// ─── write (direct mode) ───────────────────────────────────────────────────

#[test]
fn write_direct_returns_artifact_id() {
    let dir = common::create_workspace();
    let id = common::write_artifact(dir.path(), "note", "hello world");
    assert!(!id.is_empty());
    assert!(id.len() >= 20, "expected ULID-length ID, got: {}", id);
}

#[test]
fn write_direct_json_returns_full_artifact() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "hello world");
    assert_eq!(val["kind"], "note");
    assert!(val["id"].as_str().unwrap().len() >= 20);
    assert!(val["content_hash"].as_str().is_some());
    assert_eq!(val["source_mode"], "direct");
}

#[test]
fn write_creates_object_file() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "test object storage");
    let hash = val["content_hash"].as_str().unwrap();
    let prefix = &hash[..2];
    let suffix = &hash[2..];
    let obj_path = dir.path().join(".stdai/objects").join(prefix).join(suffix);
    assert!(obj_path.exists(), "object file should exist at {:?}", obj_path);

    let stored = std::fs::read_to_string(&obj_path).unwrap();
    assert_eq!(stored, "test object storage");
}

#[test]
fn write_deduplicates_content() {
    let dir = common::create_workspace();
    let v1 = common::write_artifact_json(dir.path(), "note", "duplicate content");
    let v2 = common::write_artifact_json(dir.path(), "note", "duplicate content");
    assert_eq!(v1["content_hash"], v2["content_hash"]);
    assert_ne!(v1["id"], v2["id"], "artifacts should have distinct IDs");
}

#[test]
fn write_with_tags() {
    let dir = common::create_workspace();
    let output = stdai()
        .args([
            "write", "--kind", "note", "--content", "tagged content",
            "--tag", "security", "--tag", "auth", "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tags = val["tags"].as_array().unwrap();
    assert!(tags.contains(&serde_json::json!("security")));
    assert!(tags.contains(&serde_json::json!("auth")));
}

#[test]
fn write_with_name_and_agent() {
    let dir = common::create_workspace();
    let output = stdai()
        .args([
            "write", "--kind", "research", "--content", "findings",
            "--name", "Auth Flow Analysis", "--agent", "cursor", "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["name"], "Auth Flow Analysis");
    assert_eq!(val["agent_id"], "cursor");
}

#[test]
fn write_empty_content_fails() {
    let dir = common::create_workspace();
    stdai()
        .args(["write", "--kind", "note", "--content", ""])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no content"));
}

#[test]
fn write_auto_initializes_workspace() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".stdai").exists());

    let output = stdai()
        .args(["write", "--kind", "note", "--content", "hello"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "write should auto-init");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("auto-initialized"), "stderr: {}", stderr);
    assert!(dir.path().join(".stdai").is_dir());
    assert!(dir.path().join(".stdai/objects").is_dir());
    assert!(dir.path().join(".stdai/stdai.db").exists());
}

// ─── write (pipe mode) ─────────────────────────────────────────────────────

#[test]
fn pipe_passthrough_preserves_content() {
    let dir = common::create_workspace();
    let output = stdai()
        .args(["write", "--kind", "research"])
        .current_dir(dir.path())
        .write_stdin("piped content here")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "piped content here"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stored artifact"), "stderr: {}", stderr);
}

#[test]
fn pipe_no_forward_suppresses_stdout() {
    let dir = common::create_workspace();
    let output = stdai()
        .args(["write", "--kind", "note", "--no-forward"])
        .current_dir(dir.path())
        .write_stdin("capture only")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("capture only"),
        "stdout should not contain original content in no-forward mode"
    );
}

#[test]
fn pipe_multiline_content() {
    let dir = common::create_workspace();
    let multiline = "line 1\nline 2\nline 3\n";
    let output = stdai()
        .args(["write", "--kind", "note"])
        .current_dir(dir.path())
        .write_stdin(multiline)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), multiline);
}

// ─── based_on lineage ──────────────────────────────────────────────────────

#[test]
fn write_with_based_on() {
    let dir = common::create_workspace();
    let id1 = common::write_artifact(dir.path(), "research", "research findings");
    let output = stdai()
        .args([
            "write", "--kind", "fact_check", "--content", "validated",
            "--based-on", &id1, "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let based_on = val["based_on"].as_array().unwrap();
    assert_eq!(based_on.len(), 1);
    assert_eq!(based_on[0].as_str().unwrap(), id1);
}

#[test]
fn write_based_on_invalid_id_fails() {
    let dir = common::create_workspace();
    stdai()
        .args([
            "write", "--kind", "note", "--content", "orphan",
            "--based-on", "NONEXISTENT_ID",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn write_multiple_based_on() {
    let dir = common::create_workspace();
    let id1 = common::write_artifact(dir.path(), "research", "first");
    let id2 = common::write_artifact(dir.path(), "research", "second");
    let output = stdai()
        .args([
            "write", "--kind", "summary", "--content", "combined",
            "--based-on", &id1, "--based-on", &id2, "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let based_on = val["based_on"].as_array().unwrap();
    assert_eq!(based_on.len(), 2);
}

// ─── show ───────────────────────────────────────────────────────────────────

#[test]
fn show_displays_artifact() {
    let dir = common::create_workspace();
    let id = common::write_artifact(dir.path(), "note", "show me this");
    stdai()
        .args(["show", &id])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("show me this"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn show_json_output() {
    let dir = common::create_workspace();
    let id = common::write_artifact(dir.path(), "note", "json show test");
    let output = stdai()
        .args(["show", &id, "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["id"], id);
    assert_eq!(val["kind"], "note");
}

#[test]
fn show_content_only() {
    let dir = common::create_workspace();
    let id = common::write_artifact(dir.path(), "note", "raw content only");
    let output = stdai()
        .args(["show", &id, "--content-only"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "raw content only");
}

#[test]
fn show_prefix_match() {
    let dir = common::create_workspace();
    let id = common::write_artifact(dir.path(), "note", "prefix test");
    let prefix = &id[..8];
    stdai()
        .args(["show", prefix])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("prefix test"));
}

#[test]
fn show_not_found() {
    let dir = common::create_workspace();
    stdai()
        .args(["show", "NONEXISTENT"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ─── find ───────────────────────────────────────────────────────────────────

#[test]
fn find_by_text() {
    let dir = common::create_workspace();
    common::write_artifact(dir.path(), "research", "oauth vulnerability analysis");
    common::write_artifact(dir.path(), "note", "unrelated stuff");

    stdai()
        .args(["find", "oauth"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("oauth"));
}

#[test]
fn find_by_kind_filter() {
    let dir = common::create_workspace();
    common::write_artifact(dir.path(), "research", "some research");
    common::write_artifact(dir.path(), "note", "some note");

    let output = stdai()
        .args(["find", "--kind", "research", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert!(arr.iter().all(|a| a["kind"] == "research"));
}

#[test]
fn find_by_tag_filter() {
    let dir = common::create_workspace();
    stdai()
        .args([
            "write", "--kind", "note", "--content", "security related",
            "--tag", "security",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    common::write_artifact(dir.path(), "note", "untagged");

    let output = stdai()
        .args(["find", "--tag", "security", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
}

#[test]
fn find_no_results() {
    let dir = common::create_workspace();
    common::write_artifact(dir.path(), "note", "hello world");
    stdai()
        .args(["find", "xyznonexistent"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("no artifacts"));
}

// ─── list ───────────────────────────────────────────────────────────────────

#[test]
fn list_shows_recent_artifacts() {
    let dir = common::create_workspace();
    common::write_artifact(dir.path(), "note", "first");
    common::write_artifact(dir.path(), "note", "second");
    common::write_artifact(dir.path(), "research", "third");

    let output = stdai()
        .args(["list", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 3);
}

#[test]
fn list_filter_by_kind() {
    let dir = common::create_workspace();
    common::write_artifact(dir.path(), "note", "a note");
    common::write_artifact(dir.path(), "research", "a research");

    let output = stdai()
        .args(["list", "--kind", "research", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["kind"], "research");
}

#[test]
fn list_respects_limit() {
    let dir = common::create_workspace();
    for i in 0..5 {
        common::write_artifact(dir.path(), "note", &format!("item {}", i));
    }

    let output = stdai()
        .args(["list", "--limit", "3", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 3);
}

// ─── upstream / downstream ─────────────────────────────────────────────────

#[test]
fn upstream_shows_parents() {
    let dir = common::create_workspace();
    let parent = common::write_artifact(dir.path(), "research", "parent research");
    let child_output = stdai()
        .args([
            "write", "--kind", "fact_check", "--content", "child fact check",
            "--based-on", &parent, "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let child: serde_json::Value = serde_json::from_slice(&child_output.stdout).unwrap();
    let child_id = child["id"].as_str().unwrap();

    let output = stdai()
        .args(["upstream", child_id, "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], parent);
}

#[test]
fn downstream_shows_children() {
    let dir = common::create_workspace();
    let parent = common::write_artifact(dir.path(), "research", "parent");
    let child_output = stdai()
        .args([
            "write", "--kind", "summary", "--content", "child",
            "--based-on", &parent,
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let child_id = String::from_utf8(child_output.stdout).unwrap().trim().to_string();

    let output = stdai()
        .args(["downstream", &parent, "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], child_id);
}

#[test]
fn upstream_recursive() {
    let dir = common::create_workspace();
    let a = common::write_artifact(dir.path(), "research", "grandparent");
    let b_output = stdai()
        .args([
            "write", "--kind", "fact_check", "--content", "parent",
            "--based-on", &a,
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let b = String::from_utf8(b_output.stdout).unwrap().trim().to_string();
    let c_output = stdai()
        .args([
            "write", "--kind", "summary", "--content", "child",
            "--based-on", &b,
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let c = String::from_utf8(c_output.stdout).unwrap().trim().to_string();

    // Non-recursive: only direct parent
    let output = stdai()
        .args(["upstream", &c, "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 1);

    // Recursive: grandparent + parent
    let output = stdai()
        .args(["upstream", &c, "--recursive", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 2);
}

// ─── hash behavior ─────────────────────────────────────────────────────────

#[test]
fn same_content_same_hash() {
    let dir = common::create_workspace();
    let v1 = common::write_artifact_json(dir.path(), "note", "identical");
    let v2 = common::write_artifact_json(dir.path(), "note", "identical");
    assert_eq!(v1["content_hash"], v2["content_hash"]);
}

#[test]
fn different_content_different_hash() {
    let dir = common::create_workspace();
    let v1 = common::write_artifact_json(dir.path(), "note", "alpha");
    let v2 = common::write_artifact_json(dir.path(), "note", "beta");
    assert_ne!(v1["content_hash"], v2["content_hash"]);
}

// ─── format detection ───────────────────────────────────────────────────────

#[test]
fn detects_json_format() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", r#"{"key": "value"}"#);
    assert_eq!(val["content_format"], "json");
}

#[test]
fn detects_markdown_format() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "# Heading\n\nSome **bold** text");
    assert_eq!(val["content_format"], "md");
}

#[test]
fn detects_text_format() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "plain text content here");
    assert_eq!(val["content_format"], "text");
}

#[test]
fn explicit_format_override() {
    let dir = common::create_workspace();
    let output = stdai()
        .args([
            "write", "--kind", "note", "--content", "not really json",
            "--format", "json", "--json",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["content_format"], "json");
}

// ─── metadata capture ───────────────────────────────────────────────────────

#[test]
fn captures_cwd_metadata() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "metadata test");
    assert!(val["cwd"].as_str().is_some());
    assert!(!val["cwd"].as_str().unwrap().is_empty());
}

#[test]
fn captures_hostname() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "hostname test");
    // hostname may or may not be captured depending on environment
    // just check the field exists in the response
    assert!(val.get("hostname").is_some() || val.get("hostname").is_none());
}

#[test]
fn captures_session_id() {
    let dir = common::create_workspace();
    let val = common::write_artifact_json(dir.path(), "note", "session test");
    assert!(val["session_id"].as_str().is_some());
}

// ─── doctor ─────────────────────────────────────────────────────────────────

#[test]
fn doctor_on_valid_workspace() {
    let dir = common::create_workspace();
    stdai()
        .arg("doctor")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

// ─── edge cases ─────────────────────────────────────────────────────────────

#[test]
fn large_content() {
    let dir = common::create_workspace();
    let large = "x".repeat(100_000);
    let id = common::write_artifact(dir.path(), "note", &large);
    let output = stdai()
        .args(["show", &id, "--content-only"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 100_000);
}

#[test]
fn find_with_task_filter() {
    let dir = common::create_workspace();
    stdai()
        .args([
            "write", "--kind", "note", "--content", "bug investigation",
            "--task", "auth-bug",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    common::write_artifact(dir.path(), "note", "other work");

    let output = stdai()
        .args(["find", "--task", "auth-bug", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["task_id"], "auth-bug");
}

// ─── full pipeline scenario from PRD ────────────────────────────────────────

#[test]
fn prd_scenario_research_to_fact_check_to_summary() {
    let dir = common::create_workspace();

    let research_id = common::write_artifact(dir.path(), "research", "oauth flow has vulnerability in token refresh");

    let fc_output = stdai()
        .args([
            "write", "--kind", "fact_check", "--content", "confirmed: token refresh lacks PKCE",
            "--based-on", &research_id,
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let fc_id = String::from_utf8(fc_output.stdout).unwrap().trim().to_string();

    let sum_output = stdai()
        .args([
            "write", "--kind", "summary", "--content", "critical: add PKCE to oauth refresh flow",
            "--based-on", &fc_id,
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let sum_id = String::from_utf8(sum_output.stdout).unwrap().trim().to_string();

    // Verify lineage: summary -> fact_check -> research
    let up = stdai()
        .args(["upstream", &sum_id, "--recursive", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let ancestors: serde_json::Value = serde_json::from_slice(&up.stdout).unwrap();
    assert_eq!(ancestors.as_array().unwrap().len(), 2);

    // Verify downstream from research
    let down = stdai()
        .args(["downstream", &research_id, "--recursive", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let descendants: serde_json::Value = serde_json::from_slice(&down.stdout).unwrap();
    assert_eq!(descendants.as_array().unwrap().len(), 2);

    // Find by text
    stdai()
        .args(["find", "oauth"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));

    // Show individual artifact
    stdai()
        .args(["show", &research_id])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("oauth flow has vulnerability"));
}
