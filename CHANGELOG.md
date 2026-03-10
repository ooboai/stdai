# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and
this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2025-03-10

### Added

- `stdai init` — initialize a workspace with content-addressed object store and SQLite metadata database
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
