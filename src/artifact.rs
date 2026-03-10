use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub content_hash: String,
    pub object_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub content_format: String,
    pub created_at: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub source_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub based_on: Vec<String>,
}

impl Artifact {
    pub fn make_preview(content: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(content);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        let max_len = 200;
        let chars: Vec<char> = trimmed.chars().take(max_len + 1).collect();
        let truncated = chars.len() > max_len;
        let preview: String = chars
            .into_iter()
            .take(max_len)
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .collect();
        if truncated {
            Some(format!("{}...", preview))
        } else {
            Some(preview)
        }
    }

    pub fn detect_format(content: &[u8], hint: Option<&str>) -> String {
        if let Some(h) = hint {
            return h.to_string();
        }
        let text = String::from_utf8_lossy(content);
        let trimmed = text.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
        {
            return "json".to_string();
        }
        if trimmed.contains("# ") || trimmed.contains("```") || trimmed.contains("**") {
            return "md".to_string();
        }
        "text".to_string()
    }

    pub fn display_short_id(&self) -> &str {
        if self.id.len() > 12 {
            &self.id[..12]
        } else {
            &self.id
        }
    }

    pub fn display_row(&self) -> String {
        let kind = self.kind.as_deref().unwrap_or("-");
        let created = if self.created_at.len() >= 16 {
            &self.created_at[..16]
        } else {
            &self.created_at
        };
        let preview = self.preview.as_deref().unwrap_or("");
        let preview_short = if preview.len() > 60 {
            format!("{}...", &preview[..57])
        } else {
            preview.to_string()
        };
        format!(
            "{}  {:<12} {}  {}",
            self.display_short_id(),
            kind,
            created,
            preview_short,
        )
    }

    pub fn display_detail(&self, content: Option<&str>) -> String {
        let mut out = String::new();
        out.push_str(&format!("Artifact  {}\n", self.id));
        out.push_str(&format!("Hash      {}\n", self.content_hash));
        if let Some(ref k) = self.kind {
            out.push_str(&format!("Kind      {}\n", k));
        }
        if let Some(ref n) = self.name {
            out.push_str(&format!("Name      {}\n", n));
        }
        out.push_str(&format!("Format    {}\n", self.content_format));
        out.push_str(&format!("Created   {}\n", self.created_at));
        out.push_str(&format!("Size      {} bytes\n", self.size_bytes));
        if !self.tags.is_empty() {
            out.push_str(&format!("Tags      {}\n", self.tags.join(", ")));
        }
        if !self.based_on.is_empty() {
            out.push_str(&format!("Based on  {}\n", self.based_on.join(", ")));
        }
        if let Some(ref s) = self.session_id {
            out.push_str(&format!("Session   {}\n", s));
        }
        if let Some(ref a) = self.agent_id {
            out.push_str(&format!("Agent     {}\n", a));
        }
        if let Some(ref t) = self.task_id {
            out.push_str(&format!("Task      {}\n", t));
        }
        if let Some(ref c) = self.cwd {
            out.push_str(&format!("CWD       {}\n", c));
        }
        if let Some(ref rn) = self.repo_name {
            let branch = self.git_branch.as_deref().unwrap_or("?");
            let commit = self
                .git_commit
                .as_deref()
                .map(|c| if c.len() > 8 { &c[..8] } else { c })
                .unwrap_or("?");
            out.push_str(&format!("Repo      {} ({} @ {})\n", rn, branch, commit));
        }
        if let Some(ref h) = self.hostname {
            out.push_str(&format!("Host      {}\n", h));
        }
        out.push_str(&format!("Source    {}\n", self.source_mode));
        if let Some(text) = content {
            out.push_str("\n--- Content ---\n");
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }
}
