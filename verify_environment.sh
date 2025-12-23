#!/bin/bash
set -e  # Exit immediately if a command exits with a non-zero status

echo "🔍 Checking Environment..."

echo "------------------------------------------------"
echo "🛠  Checking Tools"
echo "------------------------------------------------"
echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo "UV: $(uv --version)"
echo "Python: $(uv run python --version)"

# Check if code compiles
echo "Running 'cargo check'..."
cargo check --workspace

# Run Rust unit tests
echo "Running 'cargo test'..."
cargo test --workspace

echo ""
echo "------------------------------------------------"
echo "📦 Verifying Packaging Configuration"
echo "------------------------------------------------"

# Verify unipred-core can be published to crates.io
echo "Checking 'unipred-core' publishability..."
cargo publish -p unipred-core --dry-run --allow-dirty

# Verify wheels can be built (writes to target/wheels, safe to ignore)
echo "Checking 'unipred' wheel build..."
uv run --with maturin maturin build --release

echo ""
echo "✅ Environment Verified! Project is healthy."
