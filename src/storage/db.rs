use std::path::Path;

use rusqlite::{params, Connection};

use crate::artifact::Artifact;
use crate::error::{Error, Result};

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

pub fn initialize(path: &Path) -> Result<Connection> {
    let conn = open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS artifacts (
    id              TEXT PRIMARY KEY,
    content_hash    TEXT NOT NULL,
    object_path     TEXT NOT NULL,
    kind            TEXT,
    name            TEXT,
    content_format  TEXT NOT NULL DEFAULT 'text',
    created_at      TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL,
    session_id      TEXT,
    agent_id        TEXT,
    task_id         TEXT,
    cwd             TEXT,
    repo_root       TEXT,
    repo_name       TEXT,
    git_branch      TEXT,
    git_commit      TEXT,
    hostname        TEXT,
    source_mode     TEXT NOT NULL,
    preview         TEXT
);

CREATE TABLE IF NOT EXISTS artifact_links (
    child_id        TEXT NOT NULL REFERENCES artifacts(id),
    parent_id       TEXT NOT NULL,
    relation_type   TEXT NOT NULL DEFAULT 'based_on',
    PRIMARY KEY (child_id, parent_id)
);

CREATE TABLE IF NOT EXISTS artifact_tags (
    artifact_id     TEXT NOT NULL REFERENCES artifacts(id),
    tag             TEXT NOT NULL,
    PRIMARY KEY (artifact_id, tag)
);

CREATE VIRTUAL TABLE IF NOT EXISTS artifact_fts USING fts5(
    artifact_id UNINDEXED,
    kind,
    name,
    preview,
    content
);

CREATE INDEX IF NOT EXISTS idx_artifacts_kind ON artifacts(kind);
CREATE INDEX IF NOT EXISTS idx_artifacts_created ON artifacts(created_at);
CREATE INDEX IF NOT EXISTS idx_artifacts_task ON artifacts(task_id);
CREATE INDEX IF NOT EXISTS idx_links_parent ON artifact_links(parent_id);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON artifact_tags(tag);
";

pub fn insert_artifact(conn: &Connection, artifact: &Artifact) -> Result<()> {
    conn.execute(
        "INSERT INTO artifacts
         (id, content_hash, object_path, kind, name, content_format,
          created_at, size_bytes, session_id, agent_id, task_id,
          cwd, repo_root, repo_name, git_branch, git_commit,
          hostname, source_mode, preview)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        params![
            artifact.id,
            artifact.content_hash,
            artifact.object_path,
            artifact.kind,
            artifact.name,
            artifact.content_format,
            artifact.created_at,
            artifact.size_bytes as i64,
            artifact.session_id,
            artifact.agent_id,
            artifact.task_id,
            artifact.cwd,
            artifact.repo_root,
            artifact.repo_name,
            artifact.git_branch,
            artifact.git_commit,
            artifact.hostname,
            artifact.source_mode,
            artifact.preview,
        ],
    )?;
    Ok(())
}

pub fn insert_tags(conn: &Connection, artifact_id: &str, tags: &[String]) -> Result<()> {
    let mut stmt =
        conn.prepare("INSERT OR IGNORE INTO artifact_tags (artifact_id, tag) VALUES (?1, ?2)")?;
    for tag in tags {
        stmt.execute(params![artifact_id, tag])?;
    }
    Ok(())
}

pub fn insert_lineage(conn: &Connection, child_id: &str, parent_ids: &[String]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO artifact_links (child_id, parent_id, relation_type)
         VALUES (?1, ?2, 'based_on')",
    )?;
    for parent_id in parent_ids {
        stmt.execute(params![child_id, parent_id])?;
    }
    Ok(())
}

pub fn insert_fts(conn: &Connection, artifact: &Artifact, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO artifact_fts (artifact_id, kind, name, preview, content)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            artifact.id,
            artifact.kind.as_deref().unwrap_or(""),
            artifact.name.as_deref().unwrap_or(""),
            artifact.preview.as_deref().unwrap_or(""),
            content,
        ],
    )?;
    Ok(())
}

pub fn get_artifact(conn: &Connection, id: &str) -> Result<Artifact> {
    // Try exact match first, then prefix match
    let row = conn
        .query_row(
            "SELECT * FROM artifacts WHERE id = ?1",
            params![id],
            row_to_artifact,
        )
        .or_else(|_| {
            let pattern = format!("{}%", id);
            conn.query_row(
                "SELECT * FROM artifacts WHERE id LIKE ?1 ORDER BY created_at DESC LIMIT 1",
                params![pattern],
                row_to_artifact,
            )
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(id.to_string()),
            other => Error::Db(other),
        })?;
    Ok(row)
}

pub fn get_artifact_full(conn: &Connection, id: &str) -> Result<Artifact> {
    let mut artifact = get_artifact(conn, id)?;
    artifact.tags = get_tags(conn, &artifact.id)?;
    artifact.based_on = get_parents(conn, &artifact.id)?;
    Ok(artifact)
}

pub fn get_tags(conn: &Connection, artifact_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM artifact_tags WHERE artifact_id = ?1")?;
    let tags = stmt
        .query_map(params![artifact_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(tags)
}

pub fn get_parents(conn: &Connection, child_id: &str) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT parent_id FROM artifact_links WHERE child_id = ?1")?;
    let parents = stmt
        .query_map(params![child_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(parents)
}

pub fn find_artifacts(
    conn: &Connection,
    query: &str,
    kind: Option<&str>,
    tag: Option<&str>,
    task: Option<&str>,
    limit: u32,
) -> Result<Vec<Artifact>> {
    let fts_query = if query.contains('"')
        || query.contains('*')
        || query.contains("AND")
        || query.contains("OR")
        || query.contains("NOT")
    {
        query.to_string()
    } else {
        query
            .split_whitespace()
            .map(|w| format!("\"{}\"*", w))
            .collect::<Vec<_>>()
            .join(" OR ")
    };

    match (kind, tag, task) {
        (Some(k), Some(t), Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE artifact_fts MATCH ?1 AND a.kind = ?2 AND at.tag = ?3 AND a.task_id = ?4
                 ORDER BY a.created_at DESC LIMIT ?5",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, k, t, tk, limit])
        }
        (Some(k), Some(t), None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE artifact_fts MATCH ?1 AND a.kind = ?2 AND at.tag = ?3
                 ORDER BY a.created_at DESC LIMIT ?4",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, k, t, limit])
        }
        (Some(k), None, Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 WHERE artifact_fts MATCH ?1 AND a.kind = ?2 AND a.task_id = ?3
                 ORDER BY a.created_at DESC LIMIT ?4",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, k, tk, limit])
        }
        (None, Some(t), Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE artifact_fts MATCH ?1 AND at.tag = ?2 AND a.task_id = ?3
                 ORDER BY a.created_at DESC LIMIT ?4",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, t, tk, limit])
        }
        (Some(k), None, None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 WHERE artifact_fts MATCH ?1 AND a.kind = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, k, limit])
        }
        (None, Some(t), None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE artifact_fts MATCH ?1 AND at.tag = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, t, limit])
        }
        (None, None, Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 WHERE artifact_fts MATCH ?1 AND a.task_id = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, tk, limit])
        }
        (None, None, None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_fts fts ON a.id = fts.artifact_id
                 WHERE artifact_fts MATCH ?1
                 ORDER BY a.created_at DESC LIMIT ?2",
            )?;
            collect_artifacts(&mut stmt, params![fts_query, limit])
        }
    }
}

pub fn find_by_filters(
    conn: &Connection,
    kind: Option<&str>,
    tag: Option<&str>,
    task: Option<&str>,
    limit: u32,
) -> Result<Vec<Artifact>> {
    match (kind, tag, task) {
        (Some(k), Some(t), Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE a.kind = ?1 AND at.tag = ?2 AND a.task_id = ?3
                 ORDER BY a.created_at DESC LIMIT ?4",
            )?;
            collect_artifacts(&mut stmt, params![k, t, tk, limit])
        }
        (Some(k), Some(t), None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE a.kind = ?1 AND at.tag = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![k, t, limit])
        }
        (Some(k), None, Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 WHERE a.kind = ?1 AND a.task_id = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![k, tk, limit])
        }
        (None, Some(t), Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE at.tag = ?1 AND a.task_id = ?2
                 ORDER BY a.created_at DESC LIMIT ?3",
            )?;
            collect_artifacts(&mut stmt, params![t, tk, limit])
        }
        (Some(k), None, None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM artifacts WHERE kind = ?1
                 ORDER BY created_at DESC LIMIT ?2",
            )?;
            collect_artifacts(&mut stmt, params![k, limit])
        }
        (None, Some(t), None) => {
            let mut stmt = conn.prepare(
                "SELECT a.* FROM artifacts a
                 JOIN artifact_tags at ON a.id = at.artifact_id
                 WHERE at.tag = ?1
                 ORDER BY a.created_at DESC LIMIT ?2",
            )?;
            collect_artifacts(&mut stmt, params![t, limit])
        }
        (None, None, Some(tk)) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM artifacts WHERE task_id = ?1
                 ORDER BY created_at DESC LIMIT ?2",
            )?;
            collect_artifacts(&mut stmt, params![tk, limit])
        }
        (None, None, None) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM artifacts ORDER BY created_at DESC LIMIT ?1",
            )?;
            collect_artifacts(&mut stmt, params![limit])
        }
    }
}

pub fn list_artifacts(
    conn: &Connection,
    kind: Option<&str>,
    tag: Option<&str>,
    limit: u32,
) -> Result<Vec<Artifact>> {
    find_by_filters(conn, kind, tag, None, limit)
}

pub fn get_upstream(conn: &Connection, id: &str, recursive: bool) -> Result<Vec<Artifact>> {
    let artifact = get_artifact(conn, id)?;
    let real_id = &artifact.id;

    if recursive {
        let mut stmt = conn.prepare(
            "WITH RECURSIVE ancestors(aid) AS (
                 SELECT parent_id FROM artifact_links WHERE child_id = ?1
                 UNION
                 SELECT l.parent_id FROM artifact_links l
                 JOIN ancestors a ON l.child_id = a.aid
             )
             SELECT ar.* FROM artifacts ar
             JOIN ancestors anc ON ar.id = anc.aid
             ORDER BY ar.created_at",
        )?;
        collect_artifacts(&mut stmt, params![real_id])
    } else {
        let mut stmt = conn.prepare(
            "SELECT a.* FROM artifacts a
             JOIN artifact_links l ON a.id = l.parent_id
             WHERE l.child_id = ?1
             ORDER BY a.created_at",
        )?;
        collect_artifacts(&mut stmt, params![real_id])
    }
}

pub fn get_downstream(conn: &Connection, id: &str, recursive: bool) -> Result<Vec<Artifact>> {
    let artifact = get_artifact(conn, id)?;
    let real_id = &artifact.id;

    if recursive {
        let mut stmt = conn.prepare(
            "WITH RECURSIVE descendants(did) AS (
                 SELECT child_id FROM artifact_links WHERE parent_id = ?1
                 UNION
                 SELECT l.child_id FROM artifact_links l
                 JOIN descendants d ON l.parent_id = d.did
             )
             SELECT ar.* FROM artifacts ar
             JOIN descendants des ON ar.id = des.did
             ORDER BY ar.created_at",
        )?;
        collect_artifacts(&mut stmt, params![real_id])
    } else {
        let mut stmt = conn.prepare(
            "SELECT a.* FROM artifacts a
             JOIN artifact_links l ON a.id = l.child_id
             WHERE l.parent_id = ?1
             ORDER BY a.created_at",
        )?;
        collect_artifacts(&mut stmt, params![real_id])
    }
}

pub fn check_schema(conn: &Connection) -> Result<bool> {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    let expected = ["artifact_links", "artifact_tags", "artifacts"];
    Ok(expected.iter().all(|t| tables.contains(&t.to_string())))
}

pub fn artifact_count(conn: &Connection) -> Result<u64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))?;
    Ok(count as u64)
}

fn collect_artifacts(
    stmt: &mut rusqlite::Statement,
    params: impl rusqlite::Params,
) -> Result<Vec<Artifact>> {
    let artifacts = stmt
        .query_map(params, row_to_artifact)?
        .collect::<std::result::Result<Vec<Artifact>, _>>()?;
    Ok(artifacts)
}

fn row_to_artifact(row: &rusqlite::Row) -> rusqlite::Result<Artifact> {
    Ok(Artifact {
        id: row.get("id")?,
        content_hash: row.get("content_hash")?,
        object_path: row.get("object_path")?,
        kind: row.get("kind")?,
        name: row.get("name")?,
        content_format: row.get("content_format")?,
        created_at: row.get("created_at")?,
        size_bytes: row.get::<_, i64>("size_bytes")? as u64,
        session_id: row.get("session_id")?,
        agent_id: row.get("agent_id")?,
        task_id: row.get("task_id")?,
        cwd: row.get("cwd")?,
        repo_root: row.get("repo_root")?,
        repo_name: row.get("repo_name")?,
        git_branch: row.get("git_branch")?,
        git_commit: row.get("git_commit")?,
        hostname: row.get("hostname")?,
        source_mode: row.get("source_mode")?,
        preview: row.get("preview")?,
        tags: Vec::new(),
        based_on: Vec::new(),
    })
}
