use std::fs;
use std::path::Path;

use rusqlite::params;

use crate::error::Result;
use crate::storage::db;

/// Migrate a legacy per-project `.stdai/` directory into the global store.
/// Idempotent: skips duplicate objects (content-addressed) and artifact IDs
/// (INSERT OR IGNORE). Renames the legacy directory to `.stdai.migrated/`.
pub fn migrate_legacy(legacy_root: &Path, global_root: &Path, project: Option<&str>) -> Result<()> {
    let legacy_db_path = legacy_root.join("stdai.db");
    if !legacy_db_path.exists() {
        return Ok(());
    }

    eprintln!(
        "stdai: migrating legacy workspace from {} ...",
        legacy_root.display()
    );

    copy_objects(&legacy_root.join("objects"), &global_root.join("objects"))?;

    let global_conn = db::open(&global_root.join("stdai.db"))?;

    global_conn.execute(
        "ATTACH DATABASE ?1 AS legacy",
        params![legacy_db_path.to_str().unwrap_or("")],
    )?;

    let has_project = legacy_has_project_column(&global_conn)?;

    let project_expr = if has_project {
        "COALESCE(la.project, ?1, la.repo_name)"
    } else {
        "COALESCE(?1, la.repo_name)"
    };

    let import_sql = format!(
        "INSERT OR IGNORE INTO main.artifacts
         (id, content_hash, object_path, kind, name, content_format,
          created_at, size_bytes, session_id, agent_id, task_id,
          cwd, repo_root, repo_name, git_branch, git_commit,
          hostname, source_mode, preview, project)
         SELECT la.id, la.content_hash, la.object_path, la.kind, la.name, la.content_format,
                la.created_at, la.size_bytes, la.session_id, la.agent_id, la.task_id,
                la.cwd, la.repo_root, la.repo_name, la.git_branch, la.git_commit,
                la.hostname, la.source_mode, la.preview, {}
         FROM legacy.artifacts la",
        project_expr,
    );

    let count = global_conn.execute(&import_sql, params![project])?;

    global_conn.execute(
        "INSERT OR IGNORE INTO main.artifact_tags (artifact_id, tag)
         SELECT artifact_id, tag FROM legacy.artifact_tags",
        [],
    )?;

    global_conn.execute(
        "INSERT OR IGNORE INTO main.artifact_links (child_id, parent_id, relation_type)
         SELECT child_id, parent_id, relation_type FROM legacy.artifact_links",
        [],
    )?;

    import_fts(&global_conn)?;

    global_conn.execute("DETACH DATABASE legacy", [])?;

    let parent = legacy_root.parent().unwrap();
    let migrated = parent.join(".stdai.migrated");
    if migrated.exists() {
        fs::remove_dir_all(&migrated)?;
    }
    fs::rename(legacy_root, &migrated)?;

    eprintln!("stdai: migrated {} artifact(s) to global store", count);
    eprintln!(
        "stdai: legacy workspace moved to {}",
        migrated.display(),
    );

    Ok(())
}

fn legacy_has_project_column(conn: &rusqlite::Connection) -> Result<bool> {
    let columns: Vec<String> = conn
        .prepare("PRAGMA legacy.table_info(artifacts)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(columns.contains(&"project".to_string()))
}

fn import_fts(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO main.artifact_fts (artifact_id, kind, name, preview, content)
         SELECT f.artifact_id, f.kind, f.name, f.preview, f.content
         FROM legacy.artifact_fts f
         INNER JOIN main.artifacts a ON f.artifact_id = a.id",
        [],
    )?;
    Ok(())
}

fn copy_objects(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let prefix = path.file_name().unwrap();
            let dest_dir = to.join(prefix);
            fs::create_dir_all(&dest_dir)?;
            for obj in fs::read_dir(&path)? {
                let obj = obj?;
                let dest = dest_dir.join(obj.file_name());
                if !dest.exists() {
                    fs::copy(obj.path(), &dest)?;
                }
            }
        }
    }
    Ok(())
}
