use crate::error::Result;
use crate::storage::{db, objects, Workspace};

pub struct ShowArgs {
    pub id: String,
    pub json: bool,
    pub content_only: bool,
}

pub fn run(args: &ShowArgs) -> Result<()> {
    let ws = Workspace::find()?;
    let conn = db::open(&ws.db_path())?;
    let artifact = db::get_artifact_full(&conn, &args.id)?;

    if args.content_only {
        let content = objects::load(&ws.objects_dir(), &artifact.content_hash)?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        std::io::Write::write_all(&mut out, &content)?;
        return Ok(());
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&artifact)?);
    } else {
        let content = objects::load(&ws.objects_dir(), &artifact.content_hash)?;
        let content_text = String::from_utf8_lossy(&content);
        print!("{}", artifact.display_detail(Some(&content_text)));
    }

    Ok(())
}
