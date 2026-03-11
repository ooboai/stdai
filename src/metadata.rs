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
    pub project: Option<String>,
}

impl Metadata {
    pub fn capture() -> Self {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string());
        let hostname = run_cmd("hostname", &[]);
        let session_id = ulid::Ulid::new().to_string();
        let git = GitInfo::capture();
        let project = detect_project_with_git(&git);

        Metadata {
            cwd,
            hostname,
            session_id,
            repo_root: git.root,
            repo_name: git.name,
            git_branch: git.branch,
            git_commit: git.commit,
            project,
        }
    }
}

/// Detect current project name.
/// Resolution: $STDAI_PROJECT > git repo basename > cwd basename
pub fn detect_project() -> Option<String> {
    let git = GitInfo::capture();
    detect_project_with_git(&git)
}

fn detect_project_with_git(git: &GitInfo) -> Option<String> {
    std::env::var("STDAI_PROJECT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| git.name.clone())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
}

/// Detect the project root directory (git root or cwd).
pub fn project_root() -> Option<PathBuf> {
    git_cmd(&["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
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
