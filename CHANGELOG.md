# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and
this project adheres to [Semantic Versioning](https://semver.org/).

## [1.2.0] - 2026-03-11

### Added

- **Signing identities**: Ed25519 key pairs with Ethereum-style address derivation
- **Mandatory signing**: All new writes must be signed with a valid identity
- **`stdai identity new`**: Create a new signing identity with optional label
- **`stdai identity list`**: List all local identities
- **`stdai identity show`**: Show identity detail (address, label, public key)
- **`stdai identity export`**: Export public key for sharing
- **`stdai identity import`**: Import external public key for verification
- **`stdai verify`**: Cryptographically verify artifact signatures
- **`--identity` flag**: Specify signing identity on write
- **`$STDAI_IDENTITY` env var**: Set default identity for session
- **Self-service onboarding**: Clear error messages guide agents through identity creation

### Changed

- `stdai write` now requires a valid identity (no anonymous writes)
- `stdai show` displays signer address and verification status
- Artifact JSON output includes `signature`, `signer_address`, `signer_pubkey` fields

## [1.1.0] - 2026-03-10

### Added

- **Global storage**: Single store at `~/.stdai/` (or `$XDG_DATA_HOME/stdai/`, `$STDAI_HOME`)
- **Project context**: Artifacts auto-tagged with current project (git repo name, `$STDAI_PROJECT`, or cwd)
- **`--all` flag**: Search/list across all projects (`stdai find auth --all`, `stdai list --all`)
- **`--project` flag**: Query a specific project (`stdai find auth --project my-api`)
- **`stdai projects`**: List all known projects with artifact counts
- **`stdai context`**: Show current detected project, store path, and artifact counts
- **Cross-project lineage**: `--based-on` references work across projects naturally
- **Legacy migration**: Auto-migrates per-project `.stdai/` directories to global store on first use
- **Core skill file**: `skills/core/SKILL.md` for AI agent integration

### Changed

- `find` and `list` now default to current project scope (use `--all` for global)
- `show`, `upstream`, `downstream` always operate globally (IDs are unique)
- `doctor` reports global store health plus current project context
- Dynamic query building replaces combinatorial filter matching (cleaner internals)
- `init` deprecated — prints message that global store auto-creates

### Removed

- Per-project `.stdai/` storage (migrated automatically)
- `init` subcommand as a required step (hidden, prints deprecation notice)

## [0.1.0] - 2025-03-10

### Added

- Auto-initialization: workspace is created transparently on first use (at git repo root or cwd)
- `install.sh` one-liner installer
- `stdai init` — explicitly initialize a workspace (optional, all commands auto-init)
- `stdai write` — store artifacts from `--content` or stdin pipe with automatic metadata capture
- `stdai find` — full-text search across artifacts with kind, tag, and task filters
- `stdai show` — display full artifact detail, JSON output, or raw content
- `stdai list` — list recent artifacts with optional filters
- `stdai upstream` — show parent artifacts (based_on lineage) with optional recursive traversal
- `stdai downstream` — show child artifacts with optional recursive traversal
- `stdai doctor` — diagnostic checks for workspace health
- Pipe passthrough: stdin content forwarded to stdout unchanged while capturing artifact
- Content-addressed object storage (SHA-256) with deduplication
- SQLite FTS5 full-text search index
- Automatic git metadata capture (repo, branch, commit)
- Lineage DAG via `--based-on` links
- Tagging support via `--tag`
- ULID-based sortable artifact IDs
