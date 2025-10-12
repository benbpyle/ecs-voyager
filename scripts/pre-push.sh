#!/bin/bash
set -e

echo "🔍 Running pre-push checks..."

echo "1️⃣  Formatting code..."
cargo fmt

echo "2️⃣  Running clippy..."
cargo clippy -- -D warnings

echo "3️⃣  Running tests..."
cargo test --quiet

echo "4️⃣  Building release..."
cargo build --release --quiet

echo "✅ All checks passed! Safe to push."
