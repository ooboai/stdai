# Contributing to stdai

Thank you for your interest in contributing to stdai. This document covers
everything you need to get started.

## Getting Started

### Prerequisites

- Rust 1.75 or later (stdai uses edition 2021)
- Git

### Building

```bash
git clone https://github.com/ooboai/stdai.git
cd stdai
cargo build
```

### Running Tests

```bash
cargo test
```

The test suite includes unit tests for core modules and integration tests that
exercise the full CLI binary. Integration tests create temporary workspaces so
they can run in parallel safely.

### Running the CLI locally

```bash
cargo run -- init
cargo run -- write --kind note --content "hello from dev build"
cargo run -- list
```

## Architecture

```
src/
  main.rs             Entry point and CLI dispatch
  cli.rs              clap derive definitions for all subcommands
  error.rs            Error types (thiserror)
  artifact.rs         Artifact data model, serialization, display formatting
  metadata.rs         Runtime metadata capture (cwd, git info, hostname, session)
  storage/
    mod.rs            Workspace discovery and creation
    objects.rs        Content-addressed blob store (SHA-256)
    db.rs             SQLite schema, migrations, and all query functions
  commands/
    init.rs           stdai init
    write.rs          stdai write (pipe + direct modes)
    find.rs           stdai find (FTS search)
    show.rs           stdai show
    list.rs           stdai list
    upstream.rs       stdai upstream (lineage traversal)
    downstream.rs     stdai downstream (lineage traversal)
    doctor.rs         stdai doctor (workspace diagnostics)
```

### Key Design Decisions

- **Content-addressed storage**: Raw artifact content is stored as immutable
  blobs keyed by SHA-256 hash, similar to Git's object model.
- **SQLite for metadata**: All artifact metadata, lineage links, tags, and the
  FTS index live in a single `stdai.db` file.
- **Pipe semantics**: In pipe mode, stdin is forwarded to stdout byte-for-byte.
  Artifact metadata is emitted to stderr.
- **ULID IDs**: Artifact IDs are ULIDs (sortable, unique, timestamp-embedded).

## Submitting Changes

1. Fork the repository.
2. Create a feature branch: `git checkout -b my-feature`.
3. Make your changes and add tests.
4. Run `cargo test` and `cargo clippy` to verify.
5. Commit with a descriptive message.
6. Open a pull request against `main`.

## Code Style

- Follow standard Rust conventions (`rustfmt` defaults).
- Run `cargo fmt` before committing.
- Run `cargo clippy` and address any warnings.
- Keep functions focused and modules cohesive.
- Write integration tests for new CLI commands or flags.

## Reporting Issues

Open an issue on GitHub with:

- What you expected to happen
- What actually happened
- Steps to reproduce
- Your OS and Rust version (`rustc --version`)

## License

By contributing to stdai, you agree that your contributions will be licensed
under the Apache License 2.0.
