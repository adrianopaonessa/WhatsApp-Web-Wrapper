# WhatsApp Web Wrapper

A fast, native, and lightweight desktop wrapper for [WhatsApp Web](https://web.whatsapp.com), built with [Tauri v2](https://tauri.app) and Rust.

Designed for minimal resource usage, clean background system tray integration, and seamless Linux Wayland support (including KDE Plasma 6 / KWin).

---

## Features

- **Lightweight & Fast**: Pure Rust backend with native WebKitGTK / WebView2, using a fraction of the RAM of standard Electron apps.
- **Native Notifications**: Desktop notifications with sound, message previews, and direct chat opening on click via `org.freedesktop.Notifications`.
- **Media & Hardware Permissions**: Full native support for recording voice notes (microphone), taking photos/videos (camera), and sharing locations.
- **System Tray Integration**: Minimizes cleanly to the system tray when closing the window with the `X` button.
- **Customizable Tray Icons**: Includes dedicated light and dark tray icons (`icon-tray-white.png` & `icon-tray-dark.png`) with runtime theme switching.
- **Autostart Support**: Toggle "Start on Boot" directly from the system tray menu.
- **Granular Background Launching**:
  - **Start in Background on Boot**: Launch directly into the system tray when starting up with the system.
  - **Start in Background on Manual Launch**: Launch directly into the system tray on manual launches.
- **Wayland / KDE Plasma 6 Optimized**: Handles KWin Wayland surface re-mapping and window decoration state seamlessly.
- **State Persistence**: Automatically remembers and restores window dimensions and maximized state across sessions.
- **Zero Frontend Bloat**: Connects directly to WhatsApp Web without unnecessary local JavaScript bundle overhead.

---

## Installation & Prerequisites

### Option A: Automated Setup (Recommended)

Clone the repository and run the setup script. It automatically detects your operating system (Arch/CachyOS, Debian/Ubuntu, Fedora, openSUSE, macOS), installs all required system libraries (WebKitGTK, AppIndicator, build essentials), and sets up Rust and the Tauri v2 CLI:

```bash
git clone https://github.com/adrianopaonessa/whatsapp-web-wrapper.git
cd whatsapp-web-wrapper
./scripts/setup.sh
```

### Option B: Manual Setup

If you prefer to install dependencies manually:

**Arch Linux / CachyOS / Manjaro:**
```bash
sudo pacman -S --needed base-devel webkit2gtk-4.1 libayatana-appindicator openssl librsvg
```

**Ubuntu / Debian / Mint / Pop!_OS:**
```bash
sudo apt update && sudo apt install -y build-essential libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libgtk-3-dev
```

**Fedora / RHEL:**
```bash
sudo dnf install -y gcc gcc-c++ make openssl-devel gtk3-devel librsvg2-devel webkit2gtk4.1-devel libayatana-appindicator-devel
```

Ensure Rust and the Tauri v2 CLI are installed:
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri v2 CLI
cargo install tauri-cli --version "^2" --locked
```

---

## Development & Building

### Running in Development Mode

Run the app in development mode with live inspection:

```bash
cargo tauri dev
```

### Compiling & Packaging

#### Option A: Automated Build Script (Recommended)

Use the build script to compile and generate release packages:

```bash
# Full release build (all target packages)
./scripts/build.sh

# Or build a specific package format:
./scripts/build.sh -b appimage
./scripts/build.sh -b deb
./scripts/build.sh -b rpm
```

#### Option B: Manual Build

```bash
cargo tauri build
```

The compiled binaries and package installers (`.AppImage`, `.deb`, `.rpm`, etc.) will be available in:
`src-tauri/target/release/bundle/`

---

## Project Structure

```
├── ui/                     # Minimal fallback UI
│   └── index.html
├── src-tauri/              # Rust backend & Tauri configuration
│   ├── src/
│   │   ├── main.rs         # Application entry point & environment setup
│   │   ├── lib.rs          # Tauri initialization & lifecycle hooks
│   │   ├── window.rs       # Window management, permissions & Wayland fixes
│   │   ├── tray.rs         # System tray icon & context menu setup
│   │   └── state.rs        # Window state & settings persistence
│   ├── capabilities/       # Tauri v2 security capabilities
│   ├── icons/              # Application and tray icons
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/                # Helper scripts
│   ├── setup.sh            # Automated environment setup
│   └── build.sh            # Automated compilation and packaging
├── package.json            # Optional npm scripts
├── LICENSE                 # MIT License
└── README.md
```

---

## License

This project is licensed under the [MIT License](LICENSE).
