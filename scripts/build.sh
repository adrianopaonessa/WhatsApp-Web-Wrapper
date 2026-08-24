#!/usr/bin/env bash
#
# Build Script for WhatsApp Web Wrapper (Tauri v2)
# Generates release binaries and installer packages (.AppImage, .deb, .rpm, etc.)
#

set -e

# Workaround for linuxdeploy on rolling-release distros (Arch, CachyOS, Fedora)
export NO_STRIP=true
export APPIMAGE_EXTRACT_AND_RUN=1

# Color formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

info() {
    printf "${BLUE}${BOLD}[INFO]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}${BOLD}[SUCCESS]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}${BOLD}[WARNING]${NC} %s\n" "$1"
}

error() {
    printf "${RED}${BOLD}[ERROR]${NC} %s\n" "$1" >&2
}

show_help() {
    cat << EOF
Usage: ./scripts/build.sh [OPTIONS]

Options:
    -d, --debug             Build in debug mode instead of release
    -b, --bundles <TYPES>   Specify bundle targets (e.g. deb,appimage,rpm)
    -t, --target <TRIPLE>   Specify compilation target triple
    -h, --help              Show this help message

Examples:
    ./scripts/build.sh                    # Full release build (all available bundles)
    ./scripts/build.sh -b appimage        # Build AppImage only
    ./scripts/build.sh -b deb             # Build Debian package (.deb) only
    ./scripts/build.sh --debug            # Build debug binary
EOF
}

BUILD_MODE="release"
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--debug)
            BUILD_MODE="debug"
            shift
            ;;
        -b|--bundles)
            EXTRA_ARGS+=(--bundles "$2")
            shift 2
            ;;
        -t|--target)
            EXTRA_ARGS+=(--target "$2")
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            EXTRA_ARGS+=("$1")
            shift
            ;;
    esac
done

cd "$ROOT_DIR"

# Check for Tauri CLI
TAURI_CMD=""
if command -v cargo-tauri >/dev/null 2>&1 || cargo tauri --version >/dev/null 2>&1; then
    TAURI_CMD="cargo tauri"
elif command -v tauri >/dev/null 2>&1; then
    TAURI_CMD="tauri"
elif [ -f "$ROOT_DIR/node_modules/.bin/tauri" ]; then
    TAURI_CMD="npx tauri"
else
    error "Tauri CLI not found."
    printf "Run ${BOLD}./scripts/setup.sh${NC} to install all necessary development tools.\n"
    exit 1
fi

info "Starting $BUILD_MODE build..."
if [ "$BUILD_MODE" == "release" ]; then
    $TAURI_CMD build "${EXTRA_ARGS[@]}"
else
    $TAURI_CMD build --debug "${EXTRA_ARGS[@]}"
fi

printf "\n"
success "Build completed successfully!"
printf "\nOutput artifacts:\n"

BUNDLE_DIR="$ROOT_DIR/src-tauri/target/$BUILD_MODE/bundle"
BIN_PATH="$ROOT_DIR/src-tauri/target/$BUILD_MODE/whatsapp-web-wrapper"

if [ -f "$BIN_PATH" ]; then
    printf "  - Binary: %s\n" "$BIN_PATH"
fi

if [ -d "$BUNDLE_DIR" ]; then
    printf "  - Packages directory: %s\n" "$BUNDLE_DIR"
    find "$BUNDLE_DIR" -type f \( -name "*.AppImage" -o -name "*.deb" -o -name "*.rpm" -o -name "*.dmg" -o -name "*.msi" \) 2>/dev/null | while read -r file; do
        printf "    --> %s\n" "$file"
    done
fi

printf "\n"
