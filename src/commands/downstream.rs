use crate::error::Result;
use crate::storage::{db, Workspace};

pub struct DownstreamArgs {
    pub id: String,
    pub recursive: bool,
    pub json: bool,
}

pub fn run(args: &DownstreamArgs) -> Result<()> {
    let ws = Workspace::open()?;
    let conn = db::open(&ws.db_path())?;

    let artifacts = db::get_downstream(&conn, &args.id, args.recursive)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&artifacts)?);
    } else if artifacts.is_empty() {
        eprintln!("no downstream artifacts");
    } else {
        for a in &artifacts {
            println!("{}", a.display_row());
        }
    }

    Ok(())
}
