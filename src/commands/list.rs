use crate::error::Result;
use crate::storage::{db, Workspace};

pub struct ListArgs {
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub limit: u32,
    pub json: bool,
    pub all: bool,
    pub project: Option<String>,
}

pub fn run(args: &ListArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;

    let project_filter = resolve_project_filter(args.all, args.project.as_deref(), ws.project());

    let artifacts = db::list_artifacts(
        &conn,
        args.kind.as_deref(),
        args.tag.as_deref(),
        project_filter.as_deref(),
        args.limit,
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&artifacts)?);
    } else if artifacts.is_empty() {
        eprintln!("no artifacts");
    } else {
        for a in &artifacts {
            println!("{}", a.display_row());
        }
    }

    Ok(())
}

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
