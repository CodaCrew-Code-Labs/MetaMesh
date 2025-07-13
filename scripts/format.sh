#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎨 MetaMesh Code Formatting${NC}"
echo "=========================="

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Format Rust code
print_status "Formatting Rust code..."
cargo fmt --all
print_success "Rust code formatted"

# Run clippy for linting
print_status "Running clippy lints..."
cargo clippy --workspace --all-targets --all-features -- -D warnings
print_success "Clippy checks passed"

# Check for unused dependencies
print_status "Checking for unused dependencies..."
if command -v cargo-machete &> /dev/null; then
    cargo machete
    print_success "Dependency check completed"
else
    echo "  cargo-machete not installed, skipping unused dependency check"
    echo "  Install with: cargo install cargo-machete"
fi

print_success "🎉 Code formatting and linting completed!"