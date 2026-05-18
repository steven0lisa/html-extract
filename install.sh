#!/usr/bin/env bash
set -euo pipefail

REPO="steven0lisa/html-extract"
BINARY="html-extract"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()    { printf "${GREEN}[INFO]${NC}  %s\n" "$1"; }
warn()    { printf "${YELLOW}[WARN]${NC}  %s\n" "$1"; }
error()   { printf "${RED}[ERROR]${NC} %s\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin) echo "macos" ;;
        Linux)  echo "linux" ;;
        *)      error "Unsupported OS: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "amd64" ;;
        arm64|aarch64) echo "arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest release tag from GitHub API
get_latest_version() {
    curl -sf "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name":\s*"([^"]+)".*/\1/'
}

# Find existing installation path
find_existing_binary() {
    command -v "$BINARY" 2>/dev/null || true
}

# Main
os=$(detect_os)
arch=$(detect_arch)

info "Detected: ${os}/${arch}"

# Check for existing installation
existing_path=$(find_existing_binary)
if [ -n "$existing_path" ]; then
    current_version=$("$existing_path" --version 2>/dev/null || echo "unknown")
    info "Found existing installation: ${existing_path} (${current_version})"
fi

# Get version
if [ -n "${1:-}" ]; then
    version="$1"
else
    info "Fetching latest version..."
    version=$(get_latest_version)
fi

if [ -z "$version" ]; then
    error "Failed to determine version. Check network or specify version manually: install.sh v0.1.0"
fi

# Skip if already up-to-date
if [ -n "$existing_path" ]; then
    current_version=$("$existing_path" --version 2>/dev/null || echo "unknown")
    if [ "$current_version" = "$version" ]; then
        info "Already up-to-date: ${version}"
        exit 0
    fi
    info "Updating ${current_version} -> ${version}"
else
    info "Installing ${BINARY} ${version}"
fi

# Build download URL
asset_name="${BINARY}-${os}-${arch}.tar.gz"
download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"

# Download
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

info "Downloading ${asset_name}..."
if ! curl -sfL -o "${tmp_dir}/${asset_name}" "$download_url"; then
    error "Download failed: ${download_url}\nCheck if the release exists at https://github.com/${REPO}/releases"
fi

# Extract
tar xzf "${tmp_dir}/${asset_name}" -C "$tmp_dir"

# Find the binary
binary_path="${tmp_dir}/${BINARY}"
if [ ! -f "$binary_path" ]; then
    error "Binary not found in archive"
fi
chmod +x "$binary_path"

# Determine install location
install_dir=""
if [ -n "$existing_path" ]; then
    # Update in-place: use the same directory as the existing installation
    install_dir=$(dirname "$existing_path")
    info "Updating at ${install_dir}/${BINARY}"
else
    # Fresh install: pick a location
    for candidate in "/usr/local/bin" "$HOME/.local/bin" "$HOME/bin"; do
        if [ -d "$candidate" ] || [ -w "$(dirname "$candidate")" ]; then
            install_dir="$candidate"
            break
        fi
    done
    if [ -z "$install_dir" ]; then
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
    fi
fi

# Install / Update
if cp "$binary_path" "${install_dir}/${BINARY}" 2>/dev/null; then
    info "Installed to ${install_dir}/${BINARY}"
else
    warn "Write permission denied for ${install_dir}, trying with sudo..."
    sudo cp "$binary_path" "${install_dir}/${BINARY}"
    info "Installed to ${install_dir}/${BINARY}"
fi

# Verify
if command -v "$BINARY" &>/dev/null; then
    installed_version=$("$BINARY" --version 2>/dev/null || echo "unknown")
    info "Successfully installed: ${installed_version}"
else
    warn "${BINARY} is not in your PATH."
    warn "Add ${install_dir} to your PATH:"
    warn "  echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.bashrc"
    warn "  source ~/.bashrc"
fi
