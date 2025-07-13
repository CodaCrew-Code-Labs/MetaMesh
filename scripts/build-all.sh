#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔨 MetaMesh Cross-Platform Build Script${NC}"
echo "=========================================="

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

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

print_status "Installing cross-compilation targets..."

# Define targets and descriptions as parallel arrays
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
    "armv7-unknown-linux-gnueabihf"
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "xtensa-esp32-espidf"
    "thumbv7em-none-eabihf"
)

DESCRIPTIONS=(
    "Linux x64"
    "Linux ARM64"
    "macOS x64 (Intel)"
    "macOS ARM64 (Apple Silicon)"
    "Windows x64"
    "Raspberry Pi ARMv7"
    "Android ARM64"
    "Android ARMv7"
    "ESP32"
    "Arduino ARM Cortex-M4"
)

# Install cross tool for problematic targets
print_status "Installing cross tool for cross-compilation..."
if ! command -v cross &> /dev/null; then
    cargo install cross --git https://github.com/cross-rs/cross
    print_success "Cross tool installed"
else
    print_success "Cross tool already available"
fi

# Install all targets
for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    desc="${DESCRIPTIONS[$i]}"
    print_status "Installing target: $target ($desc)"
    if rustup target add "$target" 2>/dev/null; then
        print_success "Target $target installed"
    else
        print_warning "Target $target already installed or failed to install"
    fi
done

echo ""
print_status "Starting cross-compilation builds..."
echo ""

# Create output directory
mkdir -p dist/

# Build counter
total_targets=${#TARGETS[@]}
current=0
successful=0
failed=0

# Build for each target
for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    desc="${DESCRIPTIONS[$i]}"
    current=$((current + 1))
    echo -e "${BLUE}[$current/$total_targets]${NC} Building for ${YELLOW}$target${NC} ($desc)"
    echo "----------------------------------------"
    
    start_time=$(date +%s)
    
    # Determine build method based on target
    build_success=false
    
    case "$target" in
        "x86_64-apple-darwin"|"aarch64-apple-darwin")
            # Native macOS builds (only work on macOS)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                print_status "Using cargo for native macOS target: $target"
                if cargo build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                    echo "  $line"
                done; then
                    build_success=true
                fi
            else
                print_warning "Skipping macOS target $target (not running on macOS)"
                build_success=false
            fi
            ;;
        "x86_64-unknown-linux-gnu")
            # Native Linux build (prefer cargo on Linux, cross elsewhere)
            if [[ "$OSTYPE" == "linux-gnu"* ]]; then
                print_status "Using cargo for native Linux target: $target"
                if cargo build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                    echo "  $line"
                done; then
                    build_success=true
                fi
            else
                print_status "Using cross for Linux target: $target"
                if cross build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                    echo "  $line"
                done; then
                    build_success=true
                fi
            fi
            ;;
        "x86_64-pc-windows-gnu")
            # Windows build (prefer cargo on Windows, cross elsewhere)
            if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
                print_status "Using cargo for native Windows target: $target"
                if cargo build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                    echo "  $line"
                done; then
                    build_success=true
                fi
            else
                print_status "Using cross for Windows target: $target"
                if cross build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                    echo "  $line"
                done; then
                    build_success=true
                fi
            fi
            ;;
        *)
            # Use cross for all other cross-compilation targets
            print_status "Using cross tool for $target"
            if cross build --release --target "$target" --bin metamesh-daemon --bin metamesh-client 2>&1 | while IFS= read -r line; do
                echo "  $line"
            done; then
                build_success=true
            fi
            ;;
    esac
    
    if $build_success; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "Build completed for $target in ${duration}s"
        
        # Copy binaries to dist folder
        target_dir="dist/$target"
        mkdir -p "$target_dir"
        
        if [[ "$target" == *"windows"* ]]; then
            cp "target/$target/release/metamesh-daemon.exe" "$target_dir/" 2>/dev/null || true
            cp "target/$target/release/metamesh-client.exe" "$target_dir/" 2>/dev/null || true
        else
            cp "target/$target/release/metamesh-daemon" "$target_dir/" 2>/dev/null || true
            cp "target/$target/release/metamesh-client" "$target_dir/" 2>/dev/null || true
        fi
        
        successful=$((successful + 1))
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "Build failed for $target after ${duration}s"
        failed=$((failed + 1))
    fi
    
    echo ""
done

# Build summary
echo "=========================================="
echo -e "${BLUE}📊 Build Summary${NC}"
echo "=========================================="
echo -e "Total targets: $total_targets"
echo -e "${GREEN}Successful: $successful${NC}"
echo -e "${RED}Failed: $failed${NC}"

if [ $successful -gt 0 ]; then
    echo ""
    print_success "Binaries available in dist/ directory:"
    find dist/ -name "metamesh-*" -type f | sort
fi

if [ $failed -eq 0 ]; then
    echo ""
    print_success "🎉 All builds completed successfully!"
    exit 0
else
    echo ""
    print_warning "⚠️  Some builds failed. Check logs above for details."
    exit 1
fi