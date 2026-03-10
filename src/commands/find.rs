use crate::error::{Error, Result};
use crate::storage::{db, Workspace};

pub struct FindArgs {
    pub query: Option<String>,
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub task: Option<String>,
    pub limit: u32,
    pub json: bool,
}

pub fn run(args: &FindArgs) -> Result<()> {
    if args.query.is_none() && args.kind.is_none() && args.tag.is_none() && args.task.is_none() {
        return Err(Error::Other(
            "provide a search query or at least one filter (--kind, --tag, --task)".to_string(),
        ));
    }

    let ws = Workspace::find_or_init()?;
    let conn = db::open(&ws.db_path())?;

    let artifacts = if let Some(ref q) = args.query {
        db::find_artifacts(
            &conn,
            q,
            args.kind.as_deref(),
            args.tag.as_deref(),
            args.task.as_deref(),
            args.limit,
        )?
    } else {
        db::find_by_filters(
            &conn,
            args.kind.as_deref(),
            args.tag.as_deref(),
            args.task.as_deref(),
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
