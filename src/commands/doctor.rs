use crate::error::Result;
use crate::storage::{db, Workspace};

pub fn run() -> Result<()> {
    let ws = Workspace::find_or_init()?;

    print_check("stdai workspace", ws.root().is_dir());
    print_check("object store", ws.objects_dir().is_dir());
    print_check("database", ws.db_path().exists());

    let schema_ok = db::open(&ws.db_path())
        .and_then(|conn| db::check_schema(&conn))
        .unwrap_or(false);
    print_check("database schema", schema_ok);
    print_check("git available", check_git());

    if let Ok(conn) = db::open(&ws.db_path()) {
        if let Ok(count) = db::artifact_count(&conn) {
            println!("  {:<20} {}", "artifacts", count);
        }
    }

    Ok(())
}

fn print_check(label: &str, ok: bool) {
    let status = if ok { "ok" } else { "FAIL" };
    println!("  {:<20} {}", label, status);
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
