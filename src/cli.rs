use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "stdai",
    version,
    about = "A CLI primitive for retained agent work",
    long_about = "stdai captures work flowing through pipes or written directly,\n\
                   stores it as durable artifacts with metadata and lineage,\n\
                   and makes it searchable and inspectable later."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a stdai workspace in the current directory
    Init,

    /// Write an artifact (from --content or stdin pipe)
    Write {
        /// Artifact kind (e.g. research, note, fact_check, summary, handoff, plan, decision)
        #[arg(long)]
        kind: Option<String>,

        /// Content to store (if omitted, reads from stdin)
        #[arg(long)]
        content: Option<String>,

        /// Link to a parent artifact this work is based on (repeatable)
        #[arg(long = "based-on")]
        based_on: Vec<String>,

        /// Tag for this artifact (repeatable)
        #[arg(long)]
        tag: Vec<String>,

        /// Agent identifier
        #[arg(long)]
        agent: Option<String>,

        /// Task identifier
        #[arg(long)]
        task: Option<String>,

        /// Human-readable name for the artifact
        #[arg(long)]
        name: Option<String>,

        /// Content format hint (text, json, md, auto)
        #[arg(long)]
        format: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Capture only — do not forward stdin to stdout
        #[arg(long = "no-forward")]
        no_forward: bool,
    },

    /// Search artifacts by text query and/or filters
    Find {
        /// Full-text search query
        query: Option<String>,

        /// Filter by artifact kind
        #[arg(long)]
        kind: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by task ID
        #[arg(long)]
        task: Option<String>,

        /// Maximum results to return
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show full artifact detail
    Show {
        /// Artifact ID (prefix match supported)
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Print only the raw content
        #[arg(long = "content-only")]
        content_only: bool,
    },

    /// List recent artifacts
    List {
        /// Filter by artifact kind
        #[arg(long)]
        kind: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Maximum results to return
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show upstream artifacts (what this artifact is based on)
    Upstream {
        /// Artifact ID (prefix match supported)
        id: String,

        /// Walk the full ancestor graph
        #[arg(long)]
        recursive: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show downstream artifacts (what was built from this artifact)
    Downstream {
        /// Artifact ID (prefix match supported)
        id: String,

        /// Walk the full descendant graph
        #[arg(long)]
        recursive: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run diagnostic checks on the workspace
    Doctor,
}
