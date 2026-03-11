use crate::error::Result;
use crate::storage::{db, Workspace};

pub struct ProjectsArgs {
    pub json: bool,
}

pub fn run(args: &ProjectsArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;
    let projects = db::list_projects(&conn)?;

    if args.json {
        let items: Vec<serde_json::Value> = projects
            .iter()
            .map(|(name, count, latest)| {
                serde_json::json!({
                    "project": name,
                    "artifacts": count,
                    "latest_activity": latest,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else if projects.is_empty() {
        eprintln!("no projects found");
    } else {
        for (name, count, latest) in &projects {
            let ts = if latest.len() >= 16 {
                &latest[..16]
            } else {
                latest
            };
            println!(
                "  {:<24} {:>4} artifacts    {}",
                name, count, ts,
            );
        }
    }

    Ok(())
}
