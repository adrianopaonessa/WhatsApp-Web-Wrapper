#!/usr/bin/env bash
#
# Development & System Setup Script for WhatsApp Web Wrapper (Tauri v2)
# Supports: Arch/CachyOS/Manjaro, Debian/Ubuntu/Mint, Fedora/RHEL, openSUSE, macOS
#

set -e

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

info "Detecting operating system and distribution..."

install_arch() {
    info "Detected Arch Linux / CachyOS / Manjaro"
    sudo pacman -S --needed --noconfirm \
        base-devel \
        curl \
        wget \
        openssl \
        webkit2gtk-4.1 \
        libayatana-appindicator \
        librsvg
}

install_debian() {
    info "Detected Debian / Ubuntu / Mint / Pop!_OS"
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        curl \
        wget \
        file \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libwebkit2gtk-4.1-dev || sudo apt-get install -y libwebkit2gtk-4.0-dev
}

install_fedora() {
    info "Detected Fedora / RHEL"
    sudo dnf install -y \
        gcc \
        gcc-c++ \
        make \
        openssl-devel \
        gtk3-devel \
        librsvg2-devel \
        webkit2gtk4.1-devel || sudo dnf install -y webkit2gtk3-devel
    sudo dnf install -y libayatana-appindicator-devel || sudo dnf install -y libappindicator-gtk3-devel || true
}

install_opensuse() {
    info "Detected openSUSE"
    sudo zypper install -y -t pattern devel_basis
    sudo zypper install -y \
        libopenssl-devel \
        gtk3-devel \
        librsvg-devel \
        libayatana-appindicator3-devel || true
}

install_macos() {
    info "Detected macOS"
    if ! command -v brew >/dev/null 2>&1; then
        warn "Homebrew not found. Please install Homebrew or Xcode Command Line Tools."
    fi
}

# Install system dependencies
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [ -f /etc/arch-release ] || [ -f /etc/cachyos-release ] || [ -f /etc/manjaro-release ]; then
        install_arch
    elif [ -f /etc/debian_version ]; then
        install_debian
    elif [ -f /etc/fedora-release ] || [ -f /etc/redhat-release ]; then
        install_fedora
    elif [ -f /etc/SuSE-release ] || [ -f /etc/zypp/zypp.conf ]; then
        install_opensuse
    else
        warn "Unsupported or unknown Linux distribution. Please install WebKitGTK and libayatana-appindicator manually."
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    install_macos
else
    warn "Unrecognized OS: $OSTYPE"
fi

# Check and install Rust if missing
info "Checking Rust installation..."
if ! command -v cargo >/dev/null 2>&1; then
    warn "Rust is not installed. Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # Source cargo env for current session
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    success "Rust installed successfully."
else
    RUST_VER=$(rustc --version)
    success "Found Rust: $RUST_VER"
fi

# Check and install Tauri CLI (Cargo)
info "Checking Tauri CLI (cargo-tauri)..."
if ! command -v cargo-tauri >/dev/null 2>&1 && ! cargo tauri --version >/dev/null 2>&1; then
    info "Installing cargo-tauri v2 CLI..."
    cargo install tauri-cli --version "^2" --locked
    success "cargo-tauri installed successfully."
else
    success "cargo-tauri is ready."
fi

# Register local icons and NoDisplay desktop entry for Wayland taskbar icon mapping in dev mode
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    info "Registering desktop icons for Wayland compositor..."
    mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps" \
             "$HOME/.local/share/icons/hicolor/512x512/apps" \
             "$HOME/.local/share/applications"

    cp "$ROOT_DIR/src-tauri/icons/128x128.png" "$HOME/.local/share/icons/hicolor/128x128/apps/whatsapp-web-wrapper.png" 2>/dev/null || true
    cp "$ROOT_DIR/src-tauri/icons/icon.png" "$HOME/.local/share/icons/hicolor/512x512/apps/whatsapp-web-wrapper.png" 2>/dev/null || true

    cat << 'EOF' > "$HOME/.local/share/applications/whatsapp-web-wrapper-dev.desktop"
[Desktop Entry]
Name=WhatsApp (Dev)
Exec=whatsapp-web-wrapper
Icon=whatsapp-web-wrapper
Type=Application
Terminal=false
NoDisplay=true
StartupWMClass=whatsapp-web-wrapper
EOF

    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

printf "\n"
success "All dependencies and development tools are installed and ready!"
printf "You can now run: ${BOLD}cargo tauri dev${NC} or ${BOLD}./scripts/build.sh${NC}\n\n"
