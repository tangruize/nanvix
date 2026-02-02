#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# A script to install Verus tools for Nanvix verification.
# Based on SVSM's vinstall.sh

set -e

VERISMO_REV=4f504b7
VERUS_RUST_VERSION=1.91.0
VERUSFMT_REV=0.5.7
PREBUILT_VERUSFMT=""

echo "=== Nanvix Verus Installation Script ==="
echo ""

for arg in "$@"; do
  case "$arg" in
    --prebuilt-verusfmt)
      PREBUILT_VERUSFMT="https://github.com/verus-lang/verusfmt/releases/download/v$VERUSFMT_REV/verusfmt-installer.sh"
      ;;
    --help|-h)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --prebuilt-verusfmt  Use prebuilt verusfmt binary instead of building from source"
      echo "  --help, -h           Show this help message"
      echo ""
      echo "This script installs:"
      echo "  1. Rust target x86_64-unknown-none for Verus"
      echo "  2. cargo-v (Verus cargo integration)"
      echo "  3. verus-rustc (Verus rustc wrapper)"
      echo "  4. Verus itself"
      echo "  5. verusfmt (Verus code formatter)"
      exit 0
      ;;
  esac
done

echo "Installing Verus with Rust version: $VERUS_RUST_VERSION"
echo "Using VeriSMo revision: $VERISMO_REV"
echo ""

# Install x86_64-unknown-none target for verus-compatible Rust version.
echo "Step 1/5: Installing x86_64-unknown-none target..."
export RUSTUP_TOOLCHAIN=$VERUS_RUST_VERSION
rustup target add x86_64-unknown-none --toolchain $RUSTUP_TOOLCHAIN

# Install the cargo-v tool (Verus cargo integration).
echo "Step 2/5: Installing cargo-v..."
cargo install --git https://github.com/microsoft/verismo/ --rev $VERISMO_REV cargo-v

# Install verus-rustc as a wrapper to call verus with proper rustc flags.
echo "Step 3/5: Installing verus-rustc..."
cargo install --git https://github.com/microsoft/verismo/ --rev $VERISMO_REV verus-rustc

# Install Verus itself.
echo "Step 4/5: Installing Verus..."
cargo v install-verus

# Install verusfmt for formatting Verus code.
echo "Step 5/5: Installing verusfmt..."
if [ -n "$PREBUILT_VERUSFMT" ]; then
    if ! verusfmt --version 2>/dev/null | grep -q "$VERUSFMT_REV$"; then
        curl --proto '=https' --tlsv1.2 -LsSf "$PREBUILT_VERUSFMT" | sh
    else
        echo "verusfmt is already at version $VERUSFMT_REV"
    fi
else
    cargo install --git https://github.com/verus-lang/verusfmt --rev v$VERUSFMT_REV
fi

echo ""
echo "=== Verus installation complete! ==="
echo ""
echo "To verify code, run:"
echo "  cd src/libs/raw-array && cargo verify"
echo ""
echo "To format Verus code, run:"
echo "  verusfmt src/libs/raw-array/src/lib.verus.rs"
