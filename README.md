# stdai

The standard stream for agent work.

Unix gave us `stdin`, `stdout`, and `stderr` — input, output, and diagnostics.
stdai adds the fourth stream: **durable work that should survive beyond the
immediate response.**

```bash
python3 research.py | stdai write --kind research --tag security | python3 fact_checker.py
```

stdai acts as a **semantic tee** — it reads work from stdin or direct input,
stores it as a signed, immutable artifact with metadata and lineage, makes it
searchable and inspectable later, and forwards the original content so pipelines
still work.

Every artifact is cryptographically signed with an Ed25519 identity, creating a
verifiable chain of provenance. Not just logs. Not just files. Not just stdout.
A standard local lane for agent work that should survive — and be attributable.

## Why

Agentic systems produce work that doesn't fit neatly into Unix's three streams:
research findings, plans, handoffs, fact checks, decisions, investigation notes.
This work should survive across sessions and agents, be searchable, and have
clear provenance.

stdai fills that gap:

- Every `write` creates an immutable, **signed artifact**
- Artifacts carry **metadata** captured automatically (cwd, git repo/branch/commit, hostname, timestamps)
- Every artifact is **cryptographically signed** with an Ed25519 identity
- Artifacts link to upstream work via **`based_on`** lineage, forming a provenance DAG
- Everything is **searchable** via full-text search
- Pipe semantics are preserved — stdout carries the original payload unchanged
- All artifacts live in a **single global store** scoped by project context

## Install

### One-liner

```bash
curl -sSf https://raw.githubusercontent.com/ooboai/stdai/main/install.sh | bash
```

### From source

```bash
cargo install --git https://github.com/ooboai/stdai.git --locked
```

### From a local checkout

```bash
cargo install --path .
```

No setup required. The global store (`~/.stdai/`) auto-creates on first use.

## Quick Start

```bash
# 1. Create a signing identity (required for all writes)
stdai identity new --label "my-agent"
# → Created identity:
# →   address:  stdai:a3b7c9d8e1f2...

# 2. Set it for your session
export STDAI_IDENTITY=stdai:a3b7c9d8e1f2...

# 3. Write artifacts (signed automatically)
stdai write --kind note --content "guest sessions break when middleware assumes user role"

# Pipe content through (output flows to next command unchanged)
echo "research output" | stdai write --kind research | next-step.sh

# Or pass identity per-command
stdai write --kind note --content "hello" --identity stdai:a3b7c9d8...

# Search for artifacts (current project by default)
stdai find "guest session"

# Search across all projects
stdai find "guest session" --all

# Show full detail (includes signer info)
stdai show 01HXYZ...

# Verify artifact signature
stdai verify 01HXYZ...

# Walk lineage (crosses project boundaries)
stdai upstream 01HXYZ... --recursive
```

## Storage

stdai uses a single global store:

```
~/.stdai/
  objects/          Content-addressed blob store (SHA-256)
    ab/
      cdef1234...   Raw artifact content
  stdai.db          SQLite database (metadata, lineage, FTS index)
  config.toml       Configuration
```

Override the location with environment variables:

| Variable | Purpose |
|----------|---------|
| `$STDAI_HOME` | Override global store location |
| `$XDG_DATA_HOME` | Standard XDG fallback (`$XDG_DATA_HOME/stdai/`) |
| `$STDAI_PROJECT` | Override project context detection |
| `$STDAI_IDENTITY` | Default signing identity address for the session |

Resolution order: `$STDAI_HOME` → `$XDG_DATA_HOME/stdai/` → `~/.stdai/`

### Identity and signing

Every artifact must be signed with an Ed25519 identity. An identity is a
cryptographic key pair with an Ethereum-style address (`stdai:` prefix + 40 hex
chars) derived from the public key.

```
~/.stdai/
  identities/
    <address-hex>/
      secret.key       Ed25519 secret (mode 0600)
      public.key       Ed25519 public key bytes
      identity.toml    { address, label, created_at }
```

Identity resolution on write:

1. `--identity <address>` flag (highest priority)
2. `$STDAI_IDENTITY` environment variable
3. If neither: command fails with instructions on how to create an identity

**Trust model:** Local honor system — any local process can use any identity's
private key. Signatures provide tamper detection: modifying content or metadata
after signing breaks verification. Legacy unsigned artifacts (from v1.1 and
earlier) remain accessible with NULL signature columns.

### Project context

Every artifact is automatically tagged with the current project, detected from:

1. `$STDAI_PROJECT` environment variable
2. Git repo name (`git rev-parse --show-toplevel` basename)
3. Current working directory basename

`find` and `list` default to showing artifacts from the current project.
Use `--all` to search globally, or `--project <name>` to query a specific project.

## Commands

### `stdai write`

Store an artifact. Supports direct content and pipe passthrough.

```bash
# Direct content
stdai write --kind note --content "middleware and session handling are the key files"

# Pipe mode — content passes through to stdout
python3 producer.py | stdai write --kind research | python3 consumer.py

# With lineage (works across projects)
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
| `--identity` | Signing identity address (or set `$STDAI_IDENTITY`) |

**Pipe behavior:**

- stdin is forwarded to stdout byte-for-byte
- Artifact ID is emitted to stderr
- `--content` overrides stdin (switches to direct mode)

### `stdai find`

Full-text search across artifacts.

```bash
stdai find auth                           # Current project
stdai find auth --all                     # All projects
stdai find auth --project my-api          # Specific project
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
stdai list                                # Current project
stdai list --all                          # All projects
stdai list --project payments-service     # Specific project
stdai list --kind research
stdai list --tag security --limit 50
stdai list --json
```

### `stdai upstream`

Show what an artifact is based on. Operates globally — lineage crosses projects.

```bash
stdai upstream 01HXYZ...              # Direct parents only
stdai upstream 01HXYZ... --recursive  # Full ancestor graph
stdai upstream 01HXYZ... --json
```

### `stdai downstream`

Show what was built from an artifact. Operates globally.

```bash
stdai downstream 01HXYZ...              # Direct children only
stdai downstream 01HXYZ... --recursive  # Full descendant graph
stdai downstream 01HXYZ... --json
```

### `stdai projects`

List all known projects with artifact counts.

```bash
stdai projects
stdai projects --json
```

### `stdai context`

Show the current detected context.

```bash
stdai context
stdai context --json
```

### `stdai identity`

Manage signing identities.

```bash
# Create a new identity
stdai identity new --label "my-agent"

# List all local identities
stdai identity list
stdai identity list --json

# Show identity detail
stdai identity show stdai:a3b7c9d8...

# Export public key for sharing
stdai identity export stdai:a3b7c9d8...

# Import an external public key (verification-only, no secret key)
stdai identity import --pubkey <64-hex-chars> --label "remote-agent"
```

### `stdai verify`

Cryptographically verify an artifact's signature.

```bash
stdai verify 01HXYZ...
stdai verify 01HXYZ... --json
```

Exit code 0 for verified (or unsigned/legacy), 1 for verification failure.

### `stdai doctor`

Run diagnostic checks on the global store.

```bash
stdai doctor
```

## Example: Research Pipeline

```bash
# Create an identity (one time)
stdai identity new --label "security-auditor"
export STDAI_IDENTITY=stdai:...

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

# Verify signatures
stdai verify "$id1"
stdai verify "$id2"

# Walk the full lineage
stdai upstream "$id2" --recursive
stdai downstream "$id1" --recursive
```

## Example: Cross-Project Work

```bash
# In project A: research
cd ~/projects/auth-service
id=$(stdai write --kind research --content "Session tokens need rotation")

# In project B: reference research from project A
cd ~/projects/api-gateway
stdai write --kind plan --content "Integrate token rotation from auth-service" \
  --based-on "$id"

# Search across all projects
stdai find "token rotation" --all

# See what you've worked on today
stdai list --all
```

## Example: Agent Handoff

```bash
# Session 1: Investigation
python3 inspect_repo.py | stdai write --kind investigation --tag payments

# Session 2 (later): Pick up where we left off
stdai find payments
stdai show <artifact_id>
```

## Migration from v0.x

If you have existing per-project `.stdai/` directories from v0.x, they are
automatically migrated to the global store on first use. The original directory
is renamed to `.stdai.migrated/` (not deleted) so you can verify the migration.

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
