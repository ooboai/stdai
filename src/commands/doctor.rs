use crate::error::Result;
use crate::storage::{db, Workspace};

pub fn run() -> Result<()> {
    print_check("stdai workspace", check_workspace());
    print_check("object store", check_objects());
    print_check("database", check_db());
    print_check("database schema", check_schema());
    print_check("git available", check_git());

    if let Ok(ws) = Workspace::find() {
        if let Ok(conn) = db::open(&ws.db_path()) {
            if let Ok(count) = db::artifact_count(&conn) {
                println!("  {:<20} {}", "artifacts", count);
            }
        }
    }

    Ok(())
}

fn print_check(label: &str, ok: bool) {
    let status = if ok { "ok" } else { "FAIL" };
    println!("  {:<20} {}", label, status);
}

fn check_workspace() -> bool {
    Workspace::find().is_ok()
}

fn check_objects() -> bool {
    Workspace::find()
        .map(|ws| ws.objects_dir().is_dir())
        .unwrap_or(false)
}

fn check_db() -> bool {
    Workspace::find()
        .map(|ws| ws.db_path().exists())
        .unwrap_or(false)
}

fn check_schema() -> bool {
    Workspace::find()
        .and_then(|ws| {
            let conn = db::open(&ws.db_path())?;
            db::check_schema(&conn)
        })
        .unwrap_or(false)
}

fn check_git() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
