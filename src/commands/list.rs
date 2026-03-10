use crate::error::Result;
use crate::storage::{db, Workspace};

pub struct ListArgs {
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub limit: u32,
    pub json: bool,
}

pub fn run(args: &ListArgs) -> Result<()> {
    let ws = Workspace::find_or_init()?;
    let conn = db::open(&ws.db_path())?;

    let artifacts = db::list_artifacts(
        &conn,
        args.kind.as_deref(),
        args.tag.as_deref(),
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
