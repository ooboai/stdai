use crate::error::Result;
use crate::storage::{db, Workspace};

pub struct UpstreamArgs {
    pub id: String,
    pub recursive: bool,
    pub json: bool,
}

pub fn run(args: &UpstreamArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;

    let artifacts = db::get_upstream(&conn, &args.id, args.recursive)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&artifacts)?);
    } else if artifacts.is_empty() {
        eprintln!("no upstream artifacts");
    } else {
        for a in &artifacts {
            println!("{}", a.display_row());
        }
    }

    Ok(())
}
