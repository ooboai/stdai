#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/ooboai/stdai.git"
BIN_NAME="stdai"

main() {
    echo "Installing stdai..."

    if ! command -v cargo &>/dev/null; then
        echo "error: cargo (Rust toolchain) is required."
        echo "Install it from https://rustup.rs"
        exit 1
    fi

    if command -v "$BIN_NAME" &>/dev/null; then
        echo "Upgrading existing installation..."
    fi

    cargo install --git "$REPO" --locked 2>&1

    if command -v "$BIN_NAME" &>/dev/null; then
        echo ""
        echo "stdai $(stdai --version) installed successfully."
        echo ""
        echo "Get started:"
        echo "  stdai write --kind note --content \"hello world\""
        echo "  stdai list"
        echo "  stdai find hello"
        echo ""
        echo "No setup needed — workspace auto-initializes on first use."
    else
        echo ""
        echo "error: stdai was built but is not in PATH."
        echo "Make sure ~/.cargo/bin is in your PATH."
        exit 1
    fi
}

main "$@"
