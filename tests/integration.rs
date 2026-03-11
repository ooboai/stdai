mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn stdai() -> Command {
    common::stdai()
}

fn cmd(home: &Path, identity: &str) -> Command {
    common::stdai_cmd(home, identity)
}

// ─── init (deprecated) ─────────────────────────────────────────────────────

#[test]
fn init_prints_deprecation_message() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("no longer needed"));
}

// ─── write (direct mode) ───────────────────────────────────────────────────

#[test]
fn write_direct_returns_artifact_id() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "hello world");
    assert!(!aid.is_empty());
    assert!(aid.len() >= 20, "expected ULID-length ID, got: {}", aid);
}

#[test]
fn write_direct_json_returns_full_artifact() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "hello world");
    assert_eq!(val["kind"], "note");
    assert!(val["id"].as_str().unwrap().len() >= 20);
    assert!(val["content_hash"].as_str().is_some());
    assert_eq!(val["source_mode"], "direct");
    assert!(val["signature"].as_str().is_some());
    assert!(val["signer_address"].as_str().is_some());
    assert!(val["signer_pubkey"].as_str().is_some());
}

#[test]
fn write_records_project_on_artifact() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "project test");
    assert_eq!(val["project"], "test-project");
}

#[test]
fn write_creates_object_file_in_global_store() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "test object storage");
    let hash = val["content_hash"].as_str().unwrap();
    let prefix = &hash[..2];
    let suffix = &hash[2..];
    let obj_path = env.path().join("objects").join(prefix).join(suffix);
    assert!(
        obj_path.exists(),
        "object file should exist at {:?}",
        obj_path
    );

    let stored = std::fs::read_to_string(&obj_path).unwrap();
    assert_eq!(stored, "test object storage");
}

#[test]
fn write_deduplicates_content() {
    let (env, id) = common::create_test_env();
    let v1 = common::write_artifact_json(env.path(), &id, "note", "duplicate content");
    let v2 = common::write_artifact_json(env.path(), &id, "note", "duplicate content");
    assert_eq!(v1["content_hash"], v2["content_hash"]);
    assert_ne!(v1["id"], v2["id"], "artifacts should have distinct IDs");
}

#[test]
fn write_with_tags() {
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "note", "--content", "tagged content",
            "--tag", "security", "--tag", "auth", "--json",
        ])
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
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "research", "--content", "findings",
            "--name", "Auth Flow Analysis", "--agent", "cursor", "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["name"], "Auth Flow Analysis");
    assert_eq!(val["agent_id"], "cursor");
}

#[test]
fn write_empty_content_fails() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args(["write", "--kind", "note", "--content", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no content"));
}

#[test]
fn write_auto_creates_global_store() {
    let (env, _id) = common::create_test_env();
    assert!(env.path().join("stdai.db").exists());
    assert!(env.path().join("objects").is_dir());
}

// ─── write (pipe mode) ─────────────────────────────────────────────────────

#[test]
fn pipe_passthrough_preserves_content() {
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args(["write", "--kind", "research"])
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
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args(["write", "--kind", "note", "--no-forward"])
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
    let (env, id) = common::create_test_env();
    let multiline = "line 1\nline 2\nline 3\n";
    let output = cmd(env.path(), &id)
        .args(["write", "--kind", "note"])
        .write_stdin(multiline)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), multiline);
}

// ─── based_on lineage ──────────────────────────────────────────────────────

#[test]
fn write_with_based_on() {
    let (env, id) = common::create_test_env();
    let id1 = common::write_artifact(env.path(), &id, "research", "research findings");
    let output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "fact_check", "--content", "validated",
            "--based-on", &id1, "--json",
        ])
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
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args([
            "write", "--kind", "note", "--content", "orphan",
            "--based-on", "NONEXISTENT_ID",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn write_multiple_based_on() {
    let (env, id) = common::create_test_env();
    let id1 = common::write_artifact(env.path(), &id, "research", "first");
    let id2 = common::write_artifact(env.path(), &id, "research", "second");
    let output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "summary", "--content", "combined",
            "--based-on", &id1, "--based-on", &id2, "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let based_on = val["based_on"].as_array().unwrap();
    assert_eq!(based_on.len(), 2);
}

// ─── cross-project lineage ─────────────────────────────────────────────────

#[test]
fn cross_project_based_on() {
    let (env, id) = common::create_test_env();
    let id_a = common::write_artifact_in_project(env.path(), &id, "project-a", "research", "from project A");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "project-b")
        .env("STDAI_IDENTITY", &id)
        .args([
            "write", "--kind", "summary", "--content", "references project A",
            "--based-on", &id_a, "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["project"], "project-b");
    assert_eq!(val["based_on"][0], id_a);
}

#[test]
fn upstream_crosses_project_boundaries() {
    let (env, id) = common::create_test_env();
    let parent = common::write_artifact_in_project(env.path(), &id, "project-a", "research", "parent in A");

    let child_output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "project-b")
        .env("STDAI_IDENTITY", &id)
        .args([
            "write", "--kind", "summary", "--content", "child in B",
            "--based-on", &parent,
        ])
        .output()
        .unwrap();
    let child_id = String::from_utf8(child_output.stdout).unwrap().trim().to_string();

    let output = cmd(env.path(), &id)
        .args(["upstream", &child_id, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], parent);
    assert_eq!(arr[0]["project"], "project-a");
}

// ─── show ───────────────────────────────────────────────────────────────────

#[test]
fn show_displays_artifact() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "show me this");
    cmd(env.path(), &id)
        .args(["show", &aid])
        .assert()
        .success()
        .stdout(predicate::str::contains("show me this"))
        .stdout(predicate::str::contains(&aid));
}

#[test]
fn show_json_output() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "json show test");
    let output = cmd(env.path(), &id)
        .args(["show", &aid, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["id"], aid);
    assert_eq!(val["kind"], "note");
}

#[test]
fn show_content_only() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "raw content only");
    let output = cmd(env.path(), &id)
        .args(["show", &aid, "--content-only"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "raw content only");
}

#[test]
fn show_prefix_match() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "prefix test");
    let prefix = &aid[..8];
    cmd(env.path(), &id)
        .args(["show", prefix])
        .assert()
        .success()
        .stdout(predicate::str::contains("prefix test"));
}

#[test]
fn show_not_found() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args(["show", "NONEXISTENT"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn show_displays_signer_address() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "signed artifact");
    let output = cmd(env.path(), &id)
        .args(["show", &aid])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Signer"), "should show signer line");
    assert!(stdout.contains("stdai:"), "should contain stdai: address");
    assert!(stdout.contains("Signed    yes"), "should show signed yes");
}

#[test]
fn show_json_includes_signature_fields() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "signature fields");
    let aid = val["id"].as_str().unwrap();
    let output = cmd(env.path(), &id)
        .args(["show", aid, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(val["signature"].as_str().is_some());
    assert!(val["signer_address"].as_str().is_some());
    assert!(val["signer_pubkey"].as_str().is_some());
}

// ─── find ───────────────────────────────────────────────────────────────────

#[test]
fn find_by_text() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "research", "oauth vulnerability analysis");
    common::write_artifact(env.path(), &id, "note", "unrelated stuff");

    cmd(env.path(), &id)
        .args(["find", "oauth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("oauth"));
}

#[test]
fn find_by_kind_filter() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "research", "some research");
    common::write_artifact(env.path(), &id, "note", "some note");

    let output = cmd(env.path(), &id)
        .args(["find", "--kind", "research", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert!(arr.iter().all(|a| a["kind"] == "research"));
}

#[test]
fn find_by_tag_filter() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args([
            "write", "--kind", "note", "--content", "security related",
            "--tag", "security",
        ])
        .assert()
        .success();
    common::write_artifact(env.path(), &id, "note", "untagged");

    let output = cmd(env.path(), &id)
        .args(["find", "--tag", "security", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
}

#[test]
fn find_no_results() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "note", "hello world");
    cmd(env.path(), &id)
        .args(["find", "xyznonexistent"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no artifacts"));
}

#[test]
fn find_scoped_to_current_project() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "shared keyword stuff");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "shared keyword stuff");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["find", "shared", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["project"], "alpha");
}

#[test]
fn find_all_returns_all_projects() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "shared keyword stuff");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "shared keyword stuff");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["find", "shared", "--all", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn find_with_project_flag() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "unique alpha content");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "unique beta content");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["find", "unique", "--project", "beta", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["project"], "beta");
}

// ─── list ───────────────────────────────────────────────────────────────────

#[test]
fn list_shows_recent_artifacts() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "note", "first");
    common::write_artifact(env.path(), &id, "note", "second");
    common::write_artifact(env.path(), &id, "research", "third");

    let output = cmd(env.path(), &id)
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 3);
}

#[test]
fn list_filter_by_kind() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "note", "a note");
    common::write_artifact(env.path(), &id, "research", "a research");

    let output = cmd(env.path(), &id)
        .args(["list", "--kind", "research", "--json"])
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
    let (env, id) = common::create_test_env();
    for i in 0..5 {
        common::write_artifact(env.path(), &id, "note", &format!("item {}", i));
    }

    let output = cmd(env.path(), &id)
        .args(["list", "--limit", "3", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 3);
}

#[test]
fn list_scoped_to_current_project() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "alpha item");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "beta item");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["project"], "alpha");
}

#[test]
fn list_all_returns_all_projects() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "alpha item");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "beta item");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["list", "--all", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 2);
}

#[test]
fn list_with_project_flag() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "alpha item");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "beta item");

    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "alpha")
        .env("STDAI_IDENTITY", &id)
        .args(["list", "--project", "beta", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["project"], "beta");
}

// ─── upstream / downstream ─────────────────────────────────────────────────

#[test]
fn upstream_shows_parents() {
    let (env, id) = common::create_test_env();
    let parent = common::write_artifact(env.path(), &id, "research", "parent research");
    let child_output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "fact_check", "--content", "child fact check",
            "--based-on", &parent, "--json",
        ])
        .output()
        .unwrap();
    let child: serde_json::Value = serde_json::from_slice(&child_output.stdout).unwrap();
    let child_id = child["id"].as_str().unwrap();

    let output = cmd(env.path(), &id)
        .args(["upstream", child_id, "--json"])
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
    let (env, id) = common::create_test_env();
    let parent = common::write_artifact(env.path(), &id, "research", "parent");
    let child_output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "summary", "--content", "child",
            "--based-on", &parent,
        ])
        .output()
        .unwrap();
    let child_id = String::from_utf8(child_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let output = cmd(env.path(), &id)
        .args(["downstream", &parent, "--json"])
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
    let (env, id) = common::create_test_env();
    let a = common::write_artifact(env.path(), &id, "research", "grandparent");
    let b_output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "fact_check", "--content", "parent",
            "--based-on", &a,
        ])
        .output()
        .unwrap();
    let b = String::from_utf8(b_output.stdout)
        .unwrap()
        .trim()
        .to_string();
    let c_output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "summary", "--content", "child",
            "--based-on", &b,
        ])
        .output()
        .unwrap();
    let c = String::from_utf8(c_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let output = cmd(env.path(), &id)
        .args(["upstream", &c, "--json"])
        .output()
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 1);

    let output = cmd(env.path(), &id)
        .args(["upstream", &c, "--recursive", "--json"])
        .output()
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 2);
}

// ─── hash behavior ─────────────────────────────────────────────────────────

#[test]
fn same_content_same_hash() {
    let (env, id) = common::create_test_env();
    let v1 = common::write_artifact_json(env.path(), &id, "note", "identical");
    let v2 = common::write_artifact_json(env.path(), &id, "note", "identical");
    assert_eq!(v1["content_hash"], v2["content_hash"]);
}

#[test]
fn different_content_different_hash() {
    let (env, id) = common::create_test_env();
    let v1 = common::write_artifact_json(env.path(), &id, "note", "alpha");
    let v2 = common::write_artifact_json(env.path(), &id, "note", "beta");
    assert_ne!(v1["content_hash"], v2["content_hash"]);
}

// ─── format detection ───────────────────────────────────────────────────────

#[test]
fn detects_json_format() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", r#"{"key": "value"}"#);
    assert_eq!(val["content_format"], "json");
}

#[test]
fn detects_markdown_format() {
    let (env, id) = common::create_test_env();
    let val =
        common::write_artifact_json(env.path(), &id, "note", "# Heading\n\nSome **bold** text");
    assert_eq!(val["content_format"], "md");
}

#[test]
fn detects_text_format() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "plain text content here");
    assert_eq!(val["content_format"], "text");
}

#[test]
fn explicit_format_override() {
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args([
            "write", "--kind", "note", "--content", "not really json",
            "--format", "json", "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["content_format"], "json");
}

// ─── metadata capture ───────────────────────────────────────────────────────

#[test]
fn captures_cwd_metadata() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "metadata test");
    assert!(val["cwd"].as_str().is_some());
    assert!(!val["cwd"].as_str().unwrap().is_empty());
}

#[test]
fn captures_session_id() {
    let (env, id) = common::create_test_env();
    let val = common::write_artifact_json(env.path(), &id, "note", "session test");
    assert!(val["session_id"].as_str().is_some());
}

// ─── doctor ─────────────────────────────────────────────────────────────────

#[test]
fn doctor_on_global_store() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "note", "seed");

    cmd(env.path(), &id)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"))
        .stdout(predicate::str::contains("global store"));
}

// ─── projects ───────────────────────────────────────────────────────────────

#[test]
fn projects_lists_known_projects() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "from alpha");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "from beta");

    let output = cmd(env.path(), &id)
        .args(["projects", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    let names: Vec<&str> = arr
        .iter()
        .map(|p| p["project"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn projects_shows_artifact_counts() {
    let (env, id) = common::create_test_env();
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "one");
    common::write_artifact_in_project(env.path(), &id, "alpha", "note", "two");
    common::write_artifact_in_project(env.path(), &id, "beta", "note", "one");

    let output = cmd(env.path(), &id)
        .args(["projects", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    let alpha = arr.iter().find(|p| p["project"] == "alpha").unwrap();
    assert_eq!(alpha["artifacts"], 2);
}

// ─── context ────────────────────────────────────────────────────────────────

#[test]
fn context_shows_current_project() {
    let (env, id) = common::create_test_env();
    common::write_artifact(env.path(), &id, "note", "seed");

    let output = cmd(env.path(), &id)
        .args(["context", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["project"], "test-project");
    assert!(val["global_store"].as_str().is_some());
    assert_eq!(val["project_artifacts"], 1);
}

// ─── edge cases ─────────────────────────────────────────────────────────────

#[test]
fn large_content() {
    let (env, id) = common::create_test_env();
    let large = "x".repeat(100_000);
    let aid = common::write_artifact(env.path(), &id, "note", &large);
    let output = cmd(env.path(), &id)
        .args(["show", &aid, "--content-only"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 100_000);
}

#[test]
fn find_with_task_filter() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args([
            "write", "--kind", "note", "--content", "bug investigation",
            "--task", "auth-bug",
        ])
        .assert()
        .success();
    common::write_artifact(env.path(), &id, "note", "other work");

    let output = cmd(env.path(), &id)
        .args(["find", "--task", "auth-bug", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["task_id"], "auth-bug");
}

// ─── full pipeline scenario ─────────────────────────────────────────────────

#[test]
fn prd_scenario_research_to_fact_check_to_summary() {
    let (env, id) = common::create_test_env();

    let research_id = common::write_artifact(
        env.path(),
        &id,
        "research",
        "oauth flow has vulnerability in token refresh",
    );

    let fc_output = cmd(env.path(), &id)
        .args([
            "write",
            "--kind",
            "fact_check",
            "--content",
            "confirmed: token refresh lacks PKCE",
            "--based-on",
            &research_id,
        ])
        .output()
        .unwrap();
    let fc_id = String::from_utf8(fc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let sum_output = cmd(env.path(), &id)
        .args([
            "write",
            "--kind",
            "summary",
            "--content",
            "critical: add PKCE to oauth refresh flow",
            "--based-on",
            &fc_id,
        ])
        .output()
        .unwrap();
    let sum_id = String::from_utf8(sum_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let up = cmd(env.path(), &id)
        .args(["upstream", &sum_id, "--recursive", "--json"])
        .output()
        .unwrap();
    let ancestors: serde_json::Value = serde_json::from_slice(&up.stdout).unwrap();
    assert_eq!(ancestors.as_array().unwrap().len(), 2);

    let down = cmd(env.path(), &id)
        .args(["downstream", &research_id, "--recursive", "--json"])
        .output()
        .unwrap();
    let descendants: serde_json::Value = serde_json::from_slice(&down.stdout).unwrap();
    assert_eq!(descendants.as_array().unwrap().len(), 2);

    cmd(env.path(), &id)
        .args(["find", "oauth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("research"));

    cmd(env.path(), &id)
        .args(["show", &research_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("oauth flow has vulnerability"));
}

// ─── env var resolution ─────────────────────────────────────────────────────

#[test]
fn stdai_home_overrides_store_location() {
    let custom = tempfile::tempdir().unwrap();
    let identity = common::create_test_identity(custom.path());
    stdai()
        .env("STDAI_HOME", custom.path())
        .env("STDAI_PROJECT", "env-test")
        .env("STDAI_IDENTITY", &identity)
        .args(["write", "--kind", "note", "--content", "custom home"])
        .assert()
        .success();
    assert!(custom.path().join("stdai.db").exists());
}

#[test]
fn xdg_data_home_fallback() {
    let xdg = tempfile::tempdir().unwrap();
    let xdg_stdai = xdg.path().join("stdai");
    std::fs::create_dir_all(&xdg_stdai).unwrap();
    let identity = common::create_test_identity(&xdg_stdai);
    stdai()
        .env("XDG_DATA_HOME", xdg.path())
        .env_remove("STDAI_HOME")
        .env("STDAI_PROJECT", "xdg-test")
        .env("STDAI_IDENTITY", &identity)
        .args(["write", "--kind", "note", "--content", "xdg path"])
        .assert()
        .success();
    assert!(xdg.path().join("stdai").join("stdai.db").exists());
}

#[test]
fn stdai_project_env_overrides_detection() {
    let (env, id) = common::create_test_env();
    let val = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "custom-project-name")
        .env("STDAI_IDENTITY", &id)
        .args(["write", "--kind", "note", "--content", "project override", "--json"])
        .output()
        .unwrap();
    assert!(val.status.success());
    let artifact: serde_json::Value = serde_json::from_slice(&val.stdout).unwrap();
    assert_eq!(artifact["project"], "custom-project-name");
}

// ─── identity management ────────────────────────────────────────────────────

#[test]
fn identity_new_creates_keypair() {
    let dir = tempfile::tempdir().unwrap();
    let output = stdai()
        .env("STDAI_HOME", dir.path())
        .args(["identity", "new"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stdai:"), "should print address");
    assert!(stdout.contains("Created identity"), "should confirm creation");
}

#[test]
fn identity_new_with_label() {
    let dir = tempfile::tempdir().unwrap();
    let output = stdai()
        .env("STDAI_HOME", dir.path())
        .args(["identity", "new", "--label", "my-bot"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("my-bot"), "should show label");
}

#[test]
fn identity_list_shows_identities() {
    let (env, _id) = common::create_test_env();
    let output = stdai()
        .env("STDAI_HOME", env.path())
        .args(["identity", "list", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = val.as_array().unwrap();
    assert!(!arr.is_empty(), "should have at least one identity");
    assert!(arr[0]["address"].as_str().unwrap().starts_with("stdai:"));
}

#[test]
fn identity_show_displays_detail() {
    let (env, id) = common::create_test_env();
    let output = stdai()
        .env("STDAI_HOME", env.path())
        .args(["identity", "show", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Address"), "should show address");
    assert!(stdout.contains("Pubkey"), "should show pubkey");
    assert!(stdout.contains("Secret    yes"), "should show has secret");
}

#[test]
fn identity_export_prints_pubkey() {
    let (env, id) = common::create_test_env();
    let output = stdai()
        .env("STDAI_HOME", env.path())
        .args(["identity", "export", &id])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert_eq!(stdout.len(), 64, "pubkey hex should be 64 chars (32 bytes)");
}

#[test]
fn identity_import_stores_pubkey() {
    let (env, id) = common::create_test_env();
    // Export the pubkey first
    let export_output = stdai()
        .env("STDAI_HOME", env.path())
        .args(["identity", "export", &id])
        .output()
        .unwrap();
    let pubkey = String::from_utf8(export_output.stdout).unwrap().trim().to_string();

    // Import into a different store
    let other = tempfile::tempdir().unwrap();
    let output = stdai()
        .env("STDAI_HOME", other.path())
        .args(["identity", "import", "--pubkey", &pubkey, "--label", "remote-agent"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Imported identity"), "should confirm import");
    assert!(stdout.contains("verification-only"), "should note no secret");
}

// ─── mandatory signing ──────────────────────────────────────────────────────

#[test]
fn write_without_identity_fails_with_instructions() {
    let dir = tempfile::tempdir().unwrap();
    // Initialize the store without creating an identity
    let output = common::stdai_cmd_no_identity(dir.path())
        .args(["write", "--kind", "note", "--content", "should fail"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("identity required"),
        "should mention identity required, got: {}",
        stderr
    );
    assert!(
        stderr.contains("stdai identity new"),
        "should include creation instructions, got: {}",
        stderr
    );
}

#[test]
fn write_with_identity_flag_succeeds() {
    let (env, id) = common::create_test_env();
    let output = common::stdai_cmd_no_identity(env.path())
        .args(["write", "--kind", "note", "--content", "with flag", "--identity", &id])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "write with --identity should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn write_with_env_identity_succeeds() {
    let (env, id) = common::create_test_env();
    let output = cmd(env.path(), &id)
        .args(["write", "--kind", "note", "--content", "with env"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn write_identity_flag_overrides_env() {
    let (env, id) = common::create_test_env();
    // Create a second identity
    let id2 = common::create_test_identity(env.path());
    let output = stdai()
        .env("STDAI_HOME", env.path())
        .env("STDAI_PROJECT", "test-project")
        .env("STDAI_IDENTITY", &id)
        .args(["write", "--kind", "note", "--content", "override test", "--identity", &id2, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let signer = val["signer_address"].as_str().unwrap();
    assert_eq!(signer, id2, "flag should override env");
}

#[test]
fn write_with_invalid_identity_fails() {
    let (env, _id) = common::create_test_env();
    let output = common::stdai_cmd_no_identity(env.path())
        .args(["write", "--kind", "note", "--content", "bad id", "--identity", "stdai:0000000000000000000000000000000000000000"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"), "should report identity not found, got: {}", stderr);
}

// ─── signature verification ─────────────────────────────────────────────────

#[test]
fn verify_signed_artifact_succeeds() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "verify this");
    let output = cmd(env.path(), &id)
        .args(["verify", &aid])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("verified"), "should say verified");
}

#[test]
fn verify_signed_artifact_json() {
    let (env, id) = common::create_test_env();
    let aid = common::write_artifact(env.path(), &id, "note", "verify json");
    let output = cmd(env.path(), &id)
        .args(["verify", &aid, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(val["signed"], true);
    assert_eq!(val["verified"], true);
}

#[test]
fn verify_nonexistent_artifact_fails() {
    let (env, id) = common::create_test_env();
    cmd(env.path(), &id)
        .args(["verify", "NONEXISTENT"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
