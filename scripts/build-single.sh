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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Ensure cargo bin is in PATH and set cross environment
export PATH="$HOME/.cargo/bin:$PATH"
export CROSS_CUSTOM_TOOLCHAIN=1

# Check arguments
if [ $# -eq 0 ]; then
    echo -e "${BLUE}🔨 MetaMesh Single Target Build${NC}"
    echo "Usage: $0 <target>"
    echo ""
    echo "Available targets:"
    echo "  x86_64-unknown-linux-gnu      - Linux x64"
    echo "  aarch64-unknown-linux-gnu     - Linux ARM64"
    echo "  x86_64-apple-darwin           - macOS x64 (Intel)"
    echo "  aarch64-apple-darwin          - macOS ARM64 (Apple Silicon)"
    echo "  x86_64-pc-windows-gnu         - Windows x64"
    echo "  armv7-unknown-linux-gnueabihf - Raspberry Pi ARMv7"
    echo "  aarch64-linux-android         - Android ARM64"
    echo "  armv7-linux-androideabi       - Android ARMv7"
    echo "  thumbv7em-none-eabihf          - Arduino ARM Cortex-M4"
    exit 1
fi

TARGET=$1

print_status "Building MetaMesh for target: $TARGET"
echo "=========================================="

# Determine build method and features
USE_CROSS=false
NO_BLE=false

case "$TARGET" in
    "x86_64-apple-darwin"|"aarch64-apple-darwin")
        # Native macOS builds - keep BLE
        if [[ "$OSTYPE" != "darwin"* ]]; then
            print_error "macOS targets can only be built on macOS"
            exit 1
        fi
        ;;
    "x86_64-unknown-linux-gnu")
        # Linux x64 - skip cross on Apple Silicon due to Docker issues
        if [[ "$OSTYPE" != "linux-gnu"* ]]; then
            if [[ $(uname -m) == "arm64" ]]; then
                print_warning "Skipping cross-compilation for x86_64 Linux on Apple Silicon (Docker compatibility issues)"
                print_warning "This target is better built in CI or on a Linux x64 machine"
                exit 0
            else
                USE_CROSS=true
            fi
        fi
        ;;
    "x86_64-pc-windows-gnu")
        # Windows - keep BLE, use cross if not on Windows
        if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
            USE_CROSS=true
        fi
        ;;
    "aarch64-unknown-linux-gnu"|"armv7-unknown-linux-gnueabihf"|"aarch64-linux-android"|"armv7-linux-androideabi"|"thumbv7em-none-eabihf")
        # Cross-compilation targets - disable BLE to avoid D-Bus issues
        USE_CROSS=true
        NO_BLE=true
        ;;
    *)
        # Other targets - use cross, disable BLE
        USE_CROSS=true
        NO_BLE=true
        ;;
esac

# Install cross tool if needed
if [ "$USE_CROSS" = true ]; then
    if ! command -v cross &> /dev/null; then
        print_status "Installing cross tool for cross-compilation..."
        cargo install cross --git https://github.com/cross-rs/cross
        print_success "Cross tool installed"
    fi
fi

# Install target
print_status "Installing target: $TARGET"
if rustup target add "$TARGET" 2>/dev/null; then
    print_success "Target $TARGET installed"
else
    print_status "Target $TARGET already installed"
fi

# Start build
start_time=$(date +%s)

# Build with appropriate method and features
build_success=false

if [ "$USE_CROSS" = true ]; then
    if [ "$NO_BLE" = true ]; then
        print_status "Using cross tool for $TARGET (without BLE)"
        if cross build --release --target "$TARGET" --bin metamesh-daemon --bin metamesh-client --no-default-features -v 2>&1 | while IFS= read -r line; do
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
    else
        print_status "Using cross tool for $TARGET (with BLE)"
        if cross build --release --target "$TARGET" --bin metamesh-daemon --bin metamesh-client -v 2>&1 | while IFS= read -r line; do
            if [[ "$line" == *"Compiling"* ]]; then
                echo -e "${YELLOW}📦${NC} $line"
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
    fi
else
    print_status "Using cargo for native $TARGET (with BLE)"
    if cargo build --release --target "$TARGET" --bin metamesh-daemon --bin metamesh-client -v 2>&1 | while IFS= read -r line; do
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
fi

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
    
    if [ "$NO_BLE" = true ]; then
        print_status "Note: Built without BLE support for cross-compilation compatibility"
    fi
    
    print_success "🎉 Build successful for $TARGET!"
else
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    print_error "Build failed for $TARGET after ${duration}s"
    exit 1
fi