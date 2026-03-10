use std::io::{IsTerminal, Read, Write};

use crate::artifact::Artifact;
use crate::error::{Error, Result};
use crate::metadata::Metadata;
use crate::storage::{db, objects, Workspace};

pub struct WriteArgs {
    pub kind: Option<String>,
    pub content: Option<String>,
    pub based_on: Vec<String>,
    pub tags: Vec<String>,
    pub agent: Option<String>,
    pub task: Option<String>,
    pub name: Option<String>,
    pub format: Option<String>,
    pub json: bool,
    pub no_forward: bool,
}

pub fn run(args: &WriteArgs) -> Result<()> {
    let (content, source_mode) = read_content(args)?;

    if content.is_empty() {
        return Err(Error::Other("no content provided".to_string()));
    }

    let ws = Workspace::find_or_init()?;
    let conn = db::open(&ws.db_path())?;

    for parent_id in &args.based_on {
        db::get_artifact(&conn, parent_id).map_err(|_| {
            Error::Other(format!("based-on artifact not found: {}", parent_id))
        })?;
    }

    let (hash, object_path) = objects::store(&ws.objects_dir(), &content)?;
    let meta = Metadata::capture();
    let id = ulid::Ulid::new().to_string();
    let preview = Artifact::make_preview(&content);
    let content_format = Artifact::detect_format(&content, args.format.as_deref());

    let artifact = Artifact {
        id: id.clone(),
        content_hash: hash,
        object_path,
        kind: args.kind.clone(),
        name: args.name.clone(),
        content_format,
        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        size_bytes: content.len() as u64,
        session_id: Some(meta.session_id),
        agent_id: args.agent.clone(),
        task_id: args.task.clone(),
        cwd: meta.cwd,
        repo_root: meta.repo_root,
        repo_name: meta.repo_name,
        git_branch: meta.git_branch,
        git_commit: meta.git_commit,
        hostname: meta.hostname,
        source_mode: source_mode.clone(),
        preview,
        tags: args.tags.clone(),
        based_on: args.based_on.clone(),
    };

    db::insert_artifact(&conn, &artifact)?;

    if !args.tags.is_empty() {
        db::insert_tags(&conn, &id, &args.tags)?;
    }
    if !args.based_on.is_empty() {
        db::insert_lineage(&conn, &id, &args.based_on)?;
    }

    let content_text = String::from_utf8_lossy(&content);
    db::insert_fts(&conn, &artifact, &content_text)?;

    let should_forward = source_mode == "pipe" && !args.no_forward;

    if should_forward {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(&content)?;
        out.flush()?;
        eprintln!("stdai: stored artifact {}", id);
    } else if args.json {
        let full = db::get_artifact_full(&conn, &id)?;
        println!("{}", serde_json::to_string_pretty(&full)?);
    } else {
        println!("{}", id);
    }

    Ok(())
}

fn read_content(args: &WriteArgs) -> Result<(Vec<u8>, String)> {
    if let Some(ref c) = args.content {
        Ok((c.as_bytes().to_vec(), "direct".to_string()))
    } else {
        let stdin = std::io::stdin();
        if stdin.is_terminal() {
            return Err(Error::Other(
                "no content provided — use --content or pipe input via stdin".to_string(),
            ));
        }
        let mut buf = Vec::new();
        stdin.lock().read_to_end(&mut buf)?;
        Ok((buf, "pipe".to_string()))
    }
}
