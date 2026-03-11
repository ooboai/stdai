use crate::error::Result;
use crate::storage::{db, global_store_path, Workspace};

pub struct ContextArgs {
    pub json: bool,
}

pub fn run(args: &ContextArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;
    let store_path = global_store_path();

    let project = ws.project().map(|p| p.to_string());
    let project_count = project
        .as_deref()
        .and_then(|p| db::artifact_count_for_project(&conn, p).ok())
        .unwrap_or(0);
    let total_count = db::artifact_count(&conn).unwrap_or(0);

    let repo_root = crate::metadata::project_root()
        .map(|p| p.display().to_string());

    if args.json {
        let val = serde_json::json!({
            "project": project,
            "repo_root": repo_root,
            "global_store": store_path.display().to_string(),
            "project_artifacts": project_count,
            "total_artifacts": total_count,
        });
        println!("{}", serde_json::to_string_pretty(&val)?);
    } else {
        println!(
            "  {:<20} {}",
            "project",
            project.as_deref().unwrap_or("(none)")
        );
        if let Some(ref root) = repo_root {
            println!("  {:<20} {}", "repo root", root);
        }
        println!("  {:<20} {}", "global store", store_path.display());
        println!(
            "  {:<20} {} (in current project)",
            "artifacts", project_count
        );
        println!("  {:<20} {} (total)", "artifacts", total_count);
    }

    Ok(())
}
