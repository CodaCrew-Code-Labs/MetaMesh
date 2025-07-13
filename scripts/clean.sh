#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧹 MetaMesh Cleanup Script${NC}"
echo "=========================="

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Clean Rust build artifacts
print_status "Cleaning Rust build artifacts..."
if [ -d "target" ]; then
    rm -rf target/
    print_success "Removed target/ directory"
else
    print_warning "target/ directory not found"
fi

# Clean distribution files
print_status "Cleaning distribution files..."
if [ -d "dist" ]; then
    rm -rf dist/
    print_success "Removed dist/ directory"
else
    print_warning "dist/ directory not found"
fi

# Clean temporary files
print_status "Cleaning temporary files..."
find . -name "*.tmp" -delete 2>/dev/null || true
find . -name "*.log" -delete 2>/dev/null || true
find . -name "*~" -delete 2>/dev/null || true
find . -name "*.swp" -delete 2>/dev/null || true
find . -name "*.swo" -delete 2>/dev/null || true
find . -name ".DS_Store" -delete 2>/dev/null || true
print_success "Removed temporary files"

# Clean storage files (optional)
if [ "$1" = "--all" ]; then
    print_status "Cleaning storage files..."
    find . -name "*.db" -delete 2>/dev/null || true
    find . -name "*.sqlite" -delete 2>/dev/null || true
    if [ -d "storage" ]; then
        rm -rf storage/
        print_success "Removed storage/ directory"
    fi
    if [ -d "data" ]; then
        rm -rf data/
        print_success "Removed data/ directory"
    fi
fi

# Clean cargo cache (optional)
if [ "$1" = "--deep" ]; then
    print_status "Deep cleaning cargo cache..."
    cargo clean
    print_success "Cargo cache cleaned"
fi

echo ""
print_success "🎉 Cleanup completed!"

if [ "$1" != "--all" ] && [ "$1" != "--deep" ]; then
    echo ""
    echo "Options:"
    echo "  --all   : Also remove storage and data files"
    echo "  --deep  : Also clean cargo cache"
fi