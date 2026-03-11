---
name: stdai
description: Retain, search, and link agent work as durable artifacts with lineage tracking.
version: "1.2.0"
install:
  check: command -v stdai
  run: cargo install --git https://github.com/ooboai/stdai.git --locked
---

# stdai — Retained Agent Work

stdai is a local CLI that captures work as durable, searchable artifacts with
metadata and lineage. Use it to persist research, plans, decisions, fact checks,
investigation notes, and handoffs so they survive across sessions and agents.

All artifacts live in a single global store (`~/.stdai/`). Each artifact is
automatically tagged with the current project context, so searches default to
the current project while lineage crosses project boundaries naturally.

## Identity and signing

All writes must be signed with an Ed25519 identity. Before writing any artifacts,
create an identity and set it for your session:

```bash
# Create an identity (one time)
stdai identity new --label "agent-name"
# → address: stdai:a3b7c9d8e1f2...

# Set it for the session
export STDAI_IDENTITY=stdai:a3b7c9d8e1f2...
```

All writes are signed automatically once the identity is set. To verify a
signature later:

```bash
stdai verify <id>
```

If you attempt to write without an identity, the error message tells you exactly
what to do.

## When to write artifacts

Write an artifact whenever you produce work worth preserving beyond the current
response:

- **Research findings** — API docs, code analysis, dependency investigation
- **Fact checks** — verification of claims or assumptions
- **Plans and decisions** — architecture choices, implementation strategies
- **Investigation notes** — debugging steps, root cause analysis
- **Handoffs** — context for the next session or agent
- **Summaries** — distilled conclusions from multiple sources

## Recommended workflow

Before starting new work, **check for existing artifacts** on the topic:

```bash
stdai find <topic> --json
```

If relevant prior work exists, read it and link your new work to it:

```bash
stdai show <id> --content-only
stdai write --kind summary --content "..." --based-on <id> --json
```

This builds a lineage chain that makes provenance traceable.

## Commands

### Write an artifact

```bash
# Direct content
stdai write --kind research --content "OAuth refresh lacks PKCE" \
  --tag security --agent cursor --task auth-bug --json

# Pipe mode (content passes through to stdout unchanged)
python3 analyze.py | stdai write --kind research --tag api --json

# With lineage
stdai write --kind fact_check --content "Confirmed" --based-on <parent_id> --json
```

Flags: `--kind`, `--content`, `--based-on` (repeatable), `--tag` (repeatable),
`--agent`, `--task`, `--name`, `--format`, `--json`, `--no-forward`, `--identity`

### Search artifacts

```bash
stdai find auth --json                    # Current project
stdai find auth --all --json              # All projects
stdai find --kind research --tag security --json
stdai find auth --project other-repo --json
```

### List recent artifacts

```bash
stdai list --json                         # Current project
stdai list --all --json                   # All projects
stdai list --kind research --limit 10 --json
```

### Show artifact detail

```bash
stdai show <id> --json                    # Full metadata
stdai show <id> --content-only            # Raw content only
```

Prefix matching is supported — `stdai show 01HX` works if unique.

### Walk lineage

```bash
stdai upstream <id> --json                # Direct parents
stdai upstream <id> --recursive --json    # Full ancestor chain
stdai downstream <id> --json              # Direct children
stdai downstream <id> --recursive --json  # Full descendant tree
```

### Manage identities

```bash
stdai identity new --label "agent-name"   # Create identity
stdai identity list --json                # List all local identities
stdai identity show <address>             # Show detail
stdai identity export <address>           # Export pubkey hex
stdai identity import --pubkey <hex>      # Import external pubkey
```

### Verify signatures

```bash
stdai verify <id>                         # Verify artifact signature
stdai verify <id> --json                  # JSON output
```

### Context and projects

```bash
stdai context --json          # Current project, store path, artifact counts
stdai projects --json         # All known projects with counts
stdai doctor                  # Global store health check
```

## JSON output

Always use `--json` for machine-readable output. All commands that return
artifacts emit JSON arrays; `write` emits a single artifact object; `context`
and `projects` emit structured objects.

## Key concepts

- **Artifact ID**: ULID — globally unique, timestamp-sortable
- **Content hash**: SHA-256 — identical content produces the same hash
- **Identity**: Ed25519 key pair, address derived from public key (`stdai:` + 40 hex)
- **Signature**: every artifact is signed, verifiable with `stdai verify`
- **Project**: Auto-detected from git repo name, `$STDAI_PROJECT`, or cwd
- **Lineage**: `--based-on` creates parent→child links forming a DAG
- **Scope**: `find` and `list` default to current project; `--all` searches globally
- **Global store**: `~/.stdai/` (override with `$STDAI_HOME` or `$XDG_DATA_HOME`)

## Example: multi-step investigation

```bash
# Setup identity (one time)
stdai identity new --label "cursor"
export STDAI_IDENTITY=stdai:...

# Step 1: Research
id1=$(stdai write --kind research \
  --content "Token refresh endpoint allows reuse of expired refresh tokens" \
  --tag security --agent cursor)

# Step 2: Fact check (linked to research)
id2=$(stdai write --kind fact_check \
  --content "Confirmed: no token rotation on refresh" \
  --based-on "$id1" --agent cursor)

# Step 3: Decision (linked to fact check)
stdai write --kind decision \
  --content "Implement refresh token rotation in v2 auth service" \
  --based-on "$id2" --agent cursor

# Verify signature chain
stdai verify "$id1"
stdai verify "$id2"

# Later: trace the full lineage
stdai upstream "$id2" --recursive --json
```
