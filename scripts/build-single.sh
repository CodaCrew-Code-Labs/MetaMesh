#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check arguments
if [ $# -eq 0 ]; then
    echo -e "${BLUE}🔨 MetaMesh Single Target Build${NC}"
    echo "Usage: $0 <target>"
    echo ""
    echo "Available targets:"
    echo "  x86_64-unknown-linux-gnu     - Linux x64"
    echo "  aarch64-unknown-linux-gnu    - Linux ARM64"
    echo "  x86_64-apple-darwin          - macOS x64 (Intel)"
    echo "  aarch64-apple-darwin         - macOS ARM64 (Apple Silicon)"
    echo "  x86_64-pc-windows-gnu        - Windows x64"
    echo "  aarch64-linux-android        - Android ARM64"
    echo "  armv7-linux-androideabi      - Android ARMv7"
    echo "  thumbv7em-none-eabihf         - ARM Cortex-M4 (Embedded)"
    exit 1
fi

TARGET=$1

print_status "Building MetaMesh for target: $TARGET"
echo "=========================================="

# Install cross tool if needed for problematic targets
case "$TARGET" in
    "aarch64-unknown-linux-gnu"|"aarch64-linux-android"|"armv7-linux-androideabi"|"thumbv7em-none-eabihf")
        if ! command -v cross &> /dev/null; then
            print_status "Installing cross tool for cross-compilation..."
            cargo install cross --git https://github.com/cross-rs/cross
            print_success "Cross tool installed"
        fi
        ;;
esac

# Install target
print_status "Installing target: $TARGET"
if rustup target add "$TARGET" 2>/dev/null; then
    print_success "Target $TARGET installed"
else
    print_status "Target $TARGET already installed"
fi

# Start build
start_time=$(date +%s)
print_status "Starting build..."

# Determine build method based on target
build_success=false

case "$TARGET" in
    "aarch64-unknown-linux-gnu"|"aarch64-linux-android"|"armv7-linux-androideabi"|"thumbv7em-none-eabihf")
        # Use cross for problematic cross-compilation targets
        print_status "Using cross tool for $TARGET"
        if cross build --release --target "$TARGET" --bin metamesh-daemon --bin metamesh-client -v 2>&1 | while IFS= read -r line; do
            # Filter and format cross output
            if [[ "$line" == *"Compiling"* ]]; then
                echo -e "${YELLOW}📦${NC} $line"
            elif [[ "$line" == *"Finished"* ]]; then
                echo -e "${GREEN}✅${NC} $line"
            elif [[ "$line" == *"error"* ]]; then
                echo -e "${RED}❌${NC} $line"
            elif [[ "$line" == *"warning"* ]]; then
                echo -e "${YELLOW}⚠️${NC} $line"
            else
                echo "   $line"
            fi
        done; then
            build_success=true
        fi
        ;;
    *)
        # Use regular cargo for native and simple targets
        print_status "Using cargo for $TARGET"
        if cargo build --release --target "$TARGET" --bin metamesh-daemon --bin metamesh-client -v 2>&1 | while IFS= read -r line; do
            # Filter and format cargo output
            if [[ "$line" == *"Compiling"* ]]; then
                echo -e "${YELLOW}📦${NC} $line"
            elif [[ "$line" == *"Finished"* ]]; then
                echo -e "${GREEN}✅${NC} $line"
            elif [[ "$line" == *"error"* ]]; then
                echo -e "${RED}❌${NC} $line"
            elif [[ "$line" == *"warning"* ]]; then
                echo -e "${YELLOW}⚠️${NC} $line"
            else
                echo "   $line"
            fi
        done; then
            build_success=true
        fi
        ;;
esac

if $build_success; then
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    print_success "Build completed in ${duration}s"
    
    # Show binary locations
    echo ""
    print_status "Binaries created:"
    if [[ "$TARGET" == *"windows"* ]]; then
        ls -la "target/$TARGET/release/metamesh-daemon.exe" 2>/dev/null || print_error "metamesh-daemon.exe not found"
        ls -la "target/$TARGET/release/metamesh-client.exe" 2>/dev/null || print_error "metamesh-client.exe not found"
    else
        ls -la "target/$TARGET/release/metamesh-daemon" 2>/dev/null || print_error "metamesh-daemon not found"
        ls -la "target/$TARGET/release/metamesh-client" 2>/dev/null || print_error "metamesh-client not found"
    fi
    
    print_success "🎉 Build successful for $TARGET!"
else
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    print_error "Build failed for $TARGET after ${duration}s"
    exit 1
fi