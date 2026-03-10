use std::path::PathBuf;
use std::process::Command;

pub struct Metadata {
    pub cwd: Option<String>,
    pub hostname: Option<String>,
    pub session_id: String,
    pub repo_root: Option<String>,
    pub repo_name: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
}

impl Metadata {
    pub fn capture() -> Self {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string());
        let hostname = run_cmd("hostname", &[]);
        let session_id = ulid::Ulid::new().to_string();
        let git = GitInfo::capture();

        Metadata {
            cwd,
            hostname,
            session_id,
            repo_root: git.root,
            repo_name: git.name,
            git_branch: git.branch,
            git_commit: git.commit,
        }
    }
}

struct GitInfo {
    root: Option<String>,
    name: Option<String>,
    branch: Option<String>,
    commit: Option<String>,
}

impl GitInfo {
    fn capture() -> Self {
        let root = git_cmd(&["rev-parse", "--show-toplevel"]);
        let name = root.as_ref().and_then(|r| {
            PathBuf::from(r)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        });
        let branch = git_cmd(&["rev-parse", "--abbrev-ref", "HEAD"]);
        let commit = git_cmd(&["rev-parse", "HEAD"]);

        GitInfo {
            root,
            name,
            branch,
            commit,
        }
    }
}

fn git_cmd(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
