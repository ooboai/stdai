use crate::error::Result;
use crate::storage::{db, global_store_path, Workspace};

pub fn run() -> Result<()> {
    let ws = Workspace::open()?;
    let store_path = global_store_path();

    print_check(
        "global store",
        ws.root().is_dir(),
        Some(&format!("({})", store_path.display())),
    );
    print_check("object store", ws.objects_dir().is_dir(), None);
    print_check("database", ws.db_path().exists(), None);

    let schema_ok = db::open(&ws.db_path())
        .and_then(|conn| db::check_schema(&conn))
        .unwrap_or(false);
    print_check("database schema", schema_ok, None);
    print_check("git available", check_git(), None);

    if let Ok(conn) = db::open(&ws.db_path()) {
        if let Ok(count) = db::artifact_count(&conn) {
            let detail = format!("({} artifacts)", count);
            print_check("database", true, Some(&detail));
        }

        if let Some(project) = ws.project() {
            if let Ok(count) = db::artifact_count_for_project(&conn, project) {
                let detail = format!("{} ({} artifacts)", project, count);
                println!("  {:<20} {}", "current project", detail);
            }
        } else {
            println!("  {:<20} (none detected)", "current project");
        }
    }

    Ok(())
}

fn print_check(label: &str, ok: bool, detail: Option<&str>) {
    let status = if ok { "ok" } else { "FAIL" };
    if let Some(d) = detail {
        println!("  {:<20} {}    {}", label, status, d);
    } else {
        println!("  {:<20} {}", label, status);
    }
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
