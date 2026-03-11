use crate::error::{Error, Result};
use crate::storage::{db, Workspace};

pub struct FindArgs {
    pub query: Option<String>,
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub task: Option<String>,
    pub limit: u32,
    pub json: bool,
    pub all: bool,
    pub project: Option<String>,
}

pub fn run(args: &FindArgs) -> Result<()> {
    if args.query.is_none() && args.kind.is_none() && args.tag.is_none() && args.task.is_none() {
        return Err(Error::Other(
            "provide a search query or at least one filter (--kind, --tag, --task)".to_string(),
        ));
    }

    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;

    let project_filter = resolve_project_filter(args.all, args.project.as_deref(), ws.project());

    let artifacts = if let Some(ref q) = args.query {
        db::find_artifacts(
            &conn,
            q,
            args.kind.as_deref(),
            args.tag.as_deref(),
            args.task.as_deref(),
            project_filter.as_deref(),
            args.limit,
        )?
    } else {
        db::find_by_filters(
            &conn,
            args.kind.as_deref(),
            args.tag.as_deref(),
            args.task.as_deref(),
            project_filter.as_deref(),
            args.limit,
        )?
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&artifacts)?);
    } else if artifacts.is_empty() {
        eprintln!("no artifacts found");
    } else {
        for a in &artifacts {
            println!("{}", a.display_row());
        }
    }

    Ok(())
}

/// Determine the effective project filter:
/// - `--all` → None (no filter)
/// - `--project X` → Some("X")
/// - otherwise → auto-detected project
fn resolve_project_filter(
    all: bool,
    explicit_project: Option<&str>,
    detected_project: Option<&str>,
) -> Option<String> {
    if all {
        None
    } else if let Some(p) = explicit_project {
        Some(p.to_string())
    } else {
        detected_project.map(|p| p.to_string())
    }
}
