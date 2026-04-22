#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
#  Yoru Installer
#  Installs all dependencies, builds the project, and places the `yoru`
#  binary on your PATH so you can launch the TUI from anywhere.
#
#  Usage:
#    chmod +x install.sh
#    ./install.sh
#
#  Supported platforms:
#    Linux  (Debian/Ubuntu, Arch, Fedora/RHEL, openSUSE)
#    macOS  (via Homebrew)
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}  ▶  $*${RESET}"; }
success() { echo -e "${GREEN}${BOLD}  ✓  $*${RESET}"; }
warn()    { echo -e "${YELLOW}${BOLD}  ⚠  $*${RESET}"; }
error()   { echo -e "${RED}${BOLD}  ✗  $*${RESET}"; exit 1; }
divider() { echo -e "${CYAN}────────────────────────────────────────────────────────────${RESET}"; }

# ── Banner ────────────────────────────────────────────────────────────────────
clear
echo ""
echo -e "${CYAN}${BOLD}"
echo "  ██╗   ██╗ ██████╗ ██████╗ ██╗   ██╗"
echo "  ╚██╗ ██╔╝██╔═══██╗██╔══██╗██║   ██║"
echo "   ╚████╔╝ ██║   ██║██████╔╝██║   ██║"
echo "    ╚██╔╝  ██║   ██║██╔══██╗██║   ██║"
echo "     ██║    ╚██████╔╝██║  ██║╚██████╔╝"
echo "     ╚═╝     ╚═════╝ ╚═╝  ╚═╝ ╚═════╝"
echo -e "${RESET}"
echo -e "  ${BOLD}Terminal API Client${RESET}  ·  ${CYAN}Postman for the Shell${RESET}"
echo ""
divider
echo ""

# ── Platform detection ────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"
info "Detected OS: ${OS} / ${ARCH}"

case "$OS" in
  Linux)  PLATFORM="linux"  ;;
  Darwin) PLATFORM="macos"  ;;
  *)      error "Unsupported OS: $OS. Only Linux and macOS are supported." ;;
esac

# ── Helper: check if a command exists ────────────────────────────────────────
has() { command -v "$1" &>/dev/null; }

# ── Step 1: System build tools ───────────────────────────────────────────────
echo ""
info "Step 1/5 — Checking system build tools..."
echo ""

install_build_tools_linux() {
  if has apt-get; then
    info "Detected Debian/Ubuntu — installing build-essential, pkg-config, libssl-dev..."
    info "Refreshing apt cache (broken PPAs in your sources will be skipped)..."
    sudo apt-get update -qq --ignore-missing 2>/dev/null ||\
      sudo apt-get update -oAcquire::AllowInsecureRepositories=true \
                          -oAcquire::AllowDowngradeToInsecureRepositories=true \
                          --allow-unauthenticated -qq 2>/dev/null ||\
      warn "apt-get update had errors — attempting install anyway..."
    sudo apt-get install -y build-essential pkg-config libssl-dev curl
  elif has pacman; then
    info "Detected Arch Linux — installing base-devel, pkg-config, openssl..."
    sudo pacman -Syu --noconfirm base-devel pkg-config openssl
  elif has dnf; then
    info "Detected Fedora/RHEL — installing gcc, make, pkgconfig, openssl-devel..."
    sudo dnf install -y gcc make pkgconfig openssl-devel curl
  elif has yum; then
    info "Detected CentOS/RHEL (yum) — installing gcc, make, pkgconfig, openssl-devel..."
    sudo yum install -y gcc make pkgconfig openssl-devel curl
  elif has zypper; then
    info "Detected openSUSE — installing gcc, make, pkg-config, libopenssl-devel..."
    sudo zypper install -y gcc make pkg-config libopenssl-devel curl
  else
    warn "Unknown Linux distro — assuming build tools are already present."
    warn "If the build fails, install: gcc, make, pkg-config, openssl-dev, curl"
  fi
}

install_build_tools_macos() {
  if ! has xcode-select || ! xcode-select -p &>/dev/null; then
    info "Installing Xcode Command Line Tools..."
    xcode-select --install 2>/dev/null || true
    echo ""
    warn "A dialog may have appeared asking you to install Xcode CLT."
    warn "Please complete that installation, then re-run this script."
    exit 0
  fi

  if ! has brew; then
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    # Add brew to PATH for this session
    if [[ "$ARCH" == "arm64" ]]; then
      eval "$(/opt/homebrew/bin/brew shellenv)"
    else
      eval "$(/usr/local/bin/brew shellenv)"
    fi
  fi

  info "Installing openssl via Homebrew (needed for TLS)..."
  brew install openssl@3 pkg-config 2>/dev/null || true
}

if [[ "$PLATFORM" == "linux"  ]]; then install_build_tools_linux; fi
if [[ "$PLATFORM" == "macos"  ]]; then install_build_tools_macos; fi
success "Build tools OK"

# ── Step 2: Rust toolchain ────────────────────────────────────────────────────
echo ""
info "Step 2/5 — Checking Rust toolchain (stable ≥ 1.85)..."
echo ""

install_rust() {
  info "Rust not found — installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # Source the cargo env for the rest of this script
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
  success "Rust installed via rustup"
}

if ! has rustup; then
  install_rust
else
  info "rustup found — updating to latest stable..."
  rustup update stable
fi

# Ensure cargo is on PATH even if it was just installed
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

if ! has cargo; then
  error "cargo not found even after Rust install. Please restart your shell and re-run."
fi

RUST_VERSION=$(rustc --version)
success "Rust ready: ${RUST_VERSION}"

# ── Step 3: Build ─────────────────────────────────────────────────────────────
echo ""
info "Step 3/5 — Compiling Yoru (release build — this may take a minute)..."
echo ""

# Resolve the project root: the directory containing this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  error "Cargo.toml not found in $SCRIPT_DIR — make sure you run install.sh from the Yoru project root."
fi

cd "$SCRIPT_DIR"

# macOS: help pkg-config locate openssl if Homebrew installed it
if [[ "$PLATFORM" == "macos" ]]; then
  if has brew; then
    OPENSSL_PREFIX="$(brew --prefix openssl@3 2>/dev/null || brew --prefix openssl 2>/dev/null || true)"
    if [[ -n "$OPENSSL_PREFIX" ]]; then
      export PKG_CONFIG_PATH="${OPENSSL_PREFIX}/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
    fi
  fi
fi

cargo build --release 2>&1

BINARY="$SCRIPT_DIR/target/release/yoru"
if [[ ! -x "$BINARY" ]]; then
  error "Build succeeded but binary not found at $BINARY"
fi

success "Build complete: $(du -sh "$BINARY" | cut -f1) binary at $BINARY"

# ── Step 4: Install binary ────────────────────────────────────────────────────
echo ""
info "Step 4/5 — Installing the yoru binary to your PATH..."
echo ""

# Preferred install locations (in order)
CANDIDATE_DIRS=(
  "$HOME/.local/bin"
  "$HOME/.cargo/bin"
  "/usr/local/bin"
)

INSTALL_DIR=""
for dir in "${CANDIDATE_DIRS[@]}"; do
  if echo ":${PATH}:" | grep -q ":${dir}:"; then
    INSTALL_DIR="$dir"
    break
  fi
done

# If none of the preferred dirs are on PATH, use ~/.local/bin and add it
if [[ -z "$INSTALL_DIR" ]]; then
  INSTALL_DIR="$HOME/.local/bin"
  warn "$HOME/.local/bin is not on your PATH — it will be added automatically below."
fi

mkdir -p "$INSTALL_DIR"

# If target is a system dir we may need sudo
if [[ "$INSTALL_DIR" == /usr/* ]]; then
  info "Copying to $INSTALL_DIR (may prompt for sudo password)..."
  sudo cp "$BINARY" "$INSTALL_DIR/yoru"
  sudo chmod 755 "$INSTALL_DIR/yoru"
else
  cp "$BINARY" "$INSTALL_DIR/yoru"
  chmod 755 "$INSTALL_DIR/yoru"
fi

success "Binary installed → $INSTALL_DIR/yoru"

# ── Step 5: Shell PATH setup ──────────────────────────────────────────────────
echo ""
info "Step 5/5 — Ensuring $INSTALL_DIR is on your PATH..."
echo ""

SHELL_NAME="$(basename "${SHELL:-/bin/sh}")"

add_to_path() {
  local rc_file="$1"
  local export_line='export PATH="$HOME/.local/bin:$PATH"'
  if [[ -f "$rc_file" ]] && grep -qF '.local/bin' "$rc_file"; then
    info "$rc_file already contains .local/bin — skipping"
  else
    echo "" >> "$rc_file"
    echo "# Added by Yoru installer" >> "$rc_file"
    echo "$export_line" >> "$rc_file"
    success "Added PATH entry to $rc_file"
  fi
}

if [[ "$INSTALL_DIR" == "$HOME/.local/bin" ]]; then
  case "$SHELL_NAME" in
    bash)
      add_to_path "$HOME/.bashrc"
      [[ -f "$HOME/.bash_profile" ]] && add_to_path "$HOME/.bash_profile"
      ;;
    zsh)
      add_to_path "$HOME/.zshrc"
      ;;
    fish)
      FISH_CONFIG="$HOME/.config/fish/config.fish"
      mkdir -p "$(dirname "$FISH_CONFIG")"
      if ! grep -q '.local/bin' "$FISH_CONFIG" 2>/dev/null; then
        echo "" >> "$FISH_CONFIG"
        echo "# Added by Yoru installer" >> "$FISH_CONFIG"
        echo 'fish_add_path $HOME/.local/bin' >> "$FISH_CONFIG"
        success "Added PATH entry to $FISH_CONFIG"
      else
        info "$FISH_CONFIG already contains .local/bin — skipping"
      fi
      ;;
    *)
      warn "Unknown shell ($SHELL_NAME) — please add the following to your shell config manually:"
      echo ""
      echo '    export PATH="$HOME/.local/bin:$PATH"'
      echo ""
      ;;
  esac

  # Make it available for the current session too
  export PATH="$HOME/.local/bin:$PATH"
fi

# ── Verify ────────────────────────────────────────────────────────────────────
echo ""
divider
echo ""

if has yoru; then
  INSTALLED_PATH="$(command -v yoru)"
  success "yoru is on your PATH: $INSTALLED_PATH"
else
  warn "yoru was installed to $INSTALL_DIR but that directory is not yet active"
  warn "in the current shell session. Run one of:"
  echo ""
  echo "    source ~/.bashrc          # bash"
  echo "    source ~/.zshrc           # zsh"
  echo "    exec \$SHELL              # reload any shell"
  echo ""
fi

echo ""
echo -e "${CYAN}${BOLD}  Installation complete!${RESET}"
echo ""
echo -e "  Start Yoru by running:  ${CYAN}${BOLD}yoru${RESET}"
echo ""
echo -e "  Other commands:"
echo -e "    ${CYAN}yoru send --method GET --url https://httpbin.org/get${RESET}   one-shot request"
echo -e "    ${CYAN}yoru init --name \"My Project\"${RESET}                          fresh workspace"
echo -e "    ${CYAN}yoru --help${RESET}                                              full CLI help"
echo ""
divider
echo ""
