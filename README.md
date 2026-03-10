# stdai

A CLI primitive for retained agent work.

stdai acts as a **semantic tee** — it reads work from stdin or direct input,
stores it as a durable artifact with metadata and lineage, makes it searchable
and inspectable later, and forwards the original content so pipelines still work.

```bash
python3 research.py | stdai write --kind research --tag security | python3 fact_checker.py
```

It is not just logs. It is not just files. It is not just stdout.
It is a standard local lane for work that should survive.

## Why

Unix gives us `stdin`, `stdout`, and `stderr`. Those cover input, output, and
diagnostics. But agentic systems produce a fourth category: **work that should
survive beyond the immediate response** — research findings, plans, handoffs,
fact checks, notes worth preserving.

stdai fills that gap with a simple model:

- Every `write` creates an immutable **artifact**
- Artifacts carry **metadata** captured automatically (cwd, git repo/branch/commit, hostname, timestamps)
- Artifacts link to upstream work via **`based_on`** lineage, forming a provenance DAG
- Everything is **searchable** via full-text search
- Pipe semantics are preserved — stdout carries the original payload unchanged

## Install

### From source

```bash
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install stdai
```

## Quick Start

```bash
# Initialize a workspace
cd my-project
stdai init

# Write a note
stdai write --kind note --content "guest sessions break when middleware assumes user role"

# Pipe content through (output flows to next command unchanged)
echo "research output" | stdai write --kind research | next-step.sh

# Search for artifacts
stdai find "guest session"

# Show full detail
stdai show 01HXYZ...

# Walk lineage
stdai upstream 01HXYZ... --recursive
```

## Commands

### `stdai init`

Initialize a workspace in the current directory. Creates `.stdai/` with object
store, SQLite database, and config file.

```bash
stdai init
```

### `stdai write`

Store an artifact. Supports direct content and pipe passthrough.

```bash
# Direct content
stdai write --kind note --content "middleware and session handling are the key files"

# Pipe mode — content passes through to stdout
python3 producer.py | stdai write --kind research | python3 consumer.py

# With lineage
stdai write --kind fact_check --content "confirmed findings" --based-on 01HABC...

# With tags and metadata
stdai write --kind investigation --content "findings here" \
  --tag auth --tag security --task auth-bug --agent cursor --name "Auth Analysis"

# JSON output
stdai write --kind note --content "hello" --json

# Capture only (don't forward stdin to stdout)
echo "capture me" | stdai write --kind note --no-forward
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--kind` | Artifact kind (research, note, fact_check, summary, handoff, plan, decision, ...) |
| `--content` | Content string (if omitted, reads from stdin) |
| `--based-on` | Parent artifact ID (repeatable for multiple parents) |
| `--tag` | Tag (repeatable) |
| `--agent` | Agent identifier |
| `--task` | Task identifier |
| `--name` | Human-readable label |
| `--format` | Content format hint: text, json, md, auto (default: auto-detect) |
| `--json` | Output full artifact as JSON (direct mode) |
| `--no-forward` | Don't forward stdin to stdout |

**Pipe behavior:**

- stdin is forwarded to stdout byte-for-byte
- Artifact ID is emitted to stderr
- `--content` overrides stdin (switches to direct mode)

### `stdai find`

Full-text search across artifacts.

```bash
stdai find auth
stdai find "guest session"
stdai find --kind research auth
stdai find --tag security
stdai find --task auth-bug
stdai find --kind research --tag security --json
```

### `stdai show`

Display full artifact detail.

```bash
stdai show 01HXYZ...
stdai show 01HXYZ... --json
stdai show 01HXYZ... --content-only
```

Prefix matching is supported — `stdai show 01HX` works if the prefix is unique.

### `stdai list`

List recent artifacts.

```bash
stdai list
stdai list --kind research
stdai list --tag security --limit 50
stdai list --json
```

### `stdai upstream`

Show what an artifact is based on.

```bash
stdai upstream 01HXYZ...              # Direct parents only
stdai upstream 01HXYZ... --recursive  # Full ancestor graph
stdai upstream 01HXYZ... --json
```

### `stdai downstream`

Show what was built from an artifact.

```bash
stdai downstream 01HXYZ...              # Direct children only
stdai downstream 01HXYZ... --recursive  # Full descendant graph
stdai downstream 01HXYZ... --json
```

### `stdai doctor`

Run diagnostic checks on the workspace.

```bash
stdai doctor
```

## Storage Model

stdai uses a local hybrid storage model:

```
.stdai/
  objects/          Content-addressed blob store (SHA-256)
    ab/
      cdef1234...   Raw artifact content
  stdai.db          SQLite database (metadata, lineage, FTS index)
  config.toml       Workspace configuration
```

- **Object store**: Immutable, content-addressed. Same content = same hash = stored once.
- **SQLite database**: Artifact records, `based_on` lineage links, tags, and FTS5 full-text index.
- **Artifact IDs**: ULIDs — sortable, unique, timestamp-embedded.

## Example: Research Pipeline

```bash
# Step 1: Research
id1=$(stdai write --kind research \
  --content "OAuth flow has vulnerability in token refresh" \
  --tag security)

# Step 2: Fact check (linked to research)
id2=$(stdai write --kind fact_check \
  --content "Confirmed: token refresh lacks PKCE" \
  --based-on "$id1")

# Step 3: Decision (linked to fact check)
stdai write --kind decision \
  --content "Proceed with PKCE implementation in v1" \
  --based-on "$id2"

# Walk the full lineage
stdai upstream "$id2" --recursive
stdai downstream "$id1" --recursive
```

## Example: Agent Handoff

```bash
# Session 1: Investigation
python3 inspect_repo.py | stdai write --kind investigation --tag payments

# Session 2 (later): Pick up where we left off
stdai find payments
stdai show <artifact_id>
```

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
