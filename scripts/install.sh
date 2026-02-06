#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# qwen-tts installer for macOS and Linux
#
# This script:
#   1. Checks prerequisites (Python 3.10+, git, cargo)
#   2. Builds and installs the qwen-tts binary via cargo
#   3. Creates the ~/.qwen-tts/ directory structure
#   4. Creates a Python virtual environment at ~/.qwen-tts/venv/
#   5. Installs the correct Python dependencies for the detected platform
#   6. Copies generate_compat.py for Linux/CPU/CUDA backends
#   7. Ensures ~/.cargo/bin is on the PATH
#   8. Runs `qwen-tts config init` to auto-detect platform settings
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/andreisuslov/qwen-tts/main/scripts/install.sh | bash
#   # or
#   bash scripts/install.sh
# =============================================================================

# ---------------------------------------------------------------------------
# Color helpers
# ---------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info()    { printf "${CYAN}${BOLD}[info]${NC}  %s\n" "$*"; }
success() { printf "${GREEN}${BOLD}[ok]${NC}    %s\n" "$*"; }
warn()    { printf "${YELLOW}${BOLD}[warn]${NC}  %s\n" "$*"; }
error()   { printf "${RED}${BOLD}[error]${NC} %s\n" "$*" >&2; }
die()     { error "$@"; exit 1; }

# ---------------------------------------------------------------------------
# Platform detection
# ---------------------------------------------------------------------------
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin) PLATFORM="macos" ;;
        Linux)  PLATFORM="linux" ;;
        *)      die "Unsupported operating system: $OS" ;;
    esac

    IS_APPLE_SILICON=false
    if [[ "$PLATFORM" == "macos" && "$ARCH" == "arm64" ]]; then
        IS_APPLE_SILICON=true
    fi

    info "Detected platform: $PLATFORM ($ARCH)"
    if $IS_APPLE_SILICON; then
        info "Apple Silicon detected — will use MLX backend"
    fi
}

# ---------------------------------------------------------------------------
# Prerequisite checks
# ---------------------------------------------------------------------------
check_command() {
    if ! command -v "$1" &>/dev/null; then
        return 1
    fi
    return 0
}

check_python() {
    # Try python3 first, then python
    PYTHON_CMD=""
    for cmd in python3 python; do
        if check_command "$cmd"; then
            PYTHON_CMD="$cmd"
            break
        fi
    done

    if [[ -z "$PYTHON_CMD" ]]; then
        die "Python not found. Please install Python 3.10 or later.
  macOS:  brew install python@3.12
  Linux:  sudo apt install python3 python3-venv python3-pip  (Debian/Ubuntu)
          sudo dnf install python3                           (Fedora)"
    fi

    # Check version (need 3.10+)
    PYTHON_VERSION=$("$PYTHON_CMD" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
    PYTHON_MAJOR=$("$PYTHON_CMD" -c 'import sys; print(sys.version_info.major)')
    PYTHON_MINOR=$("$PYTHON_CMD" -c 'import sys; print(sys.version_info.minor)')

    if [[ "$PYTHON_MAJOR" -lt 3 ]] || { [[ "$PYTHON_MAJOR" -eq 3 ]] && [[ "$PYTHON_MINOR" -lt 10 ]]; }; then
        die "Python 3.10+ required, but found $PYTHON_VERSION ($PYTHON_CMD).
Please upgrade Python and try again."
    fi

    success "Python $PYTHON_VERSION found ($PYTHON_CMD)"
}

check_git() {
    if ! check_command git; then
        die "git not found. Please install git.
  macOS:  xcode-select --install
  Linux:  sudo apt install git  (Debian/Ubuntu)"
    fi
    success "git found ($(git --version | head -1))"
}

check_cargo() {
    # Source cargo env if it exists but is not yet on PATH
    if ! check_command cargo; then
        if [[ -f "$HOME/.cargo/env" ]]; then
            # shellcheck disable=SC1091
            source "$HOME/.cargo/env"
        fi
    fi

    if ! check_command cargo; then
        die "cargo (Rust) not found. Please install Rust first:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Then restart your shell and re-run this script."
    fi
    success "cargo found ($(cargo --version))"
}

check_prerequisites() {
    info "Checking prerequisites..."
    check_python
    check_git
    check_cargo
    echo ""
}

# ---------------------------------------------------------------------------
# Install the Rust binary
# ---------------------------------------------------------------------------
install_binary() {
    info "Building and installing qwen-tts binary..."

    # If we are running inside the repo (Cargo.toml exists), build locally.
    # Otherwise install from the remote git repository.
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    REPO_ROOT="$(dirname "$SCRIPT_DIR")"

    if [[ -f "$REPO_ROOT/Cargo.toml" ]] && grep -q 'name = "qwen-tts"' "$REPO_ROOT/Cargo.toml" 2>/dev/null; then
        info "Local repository detected at $REPO_ROOT — building from source"
        cargo install --path "$REPO_ROOT"
    else
        info "Installing from remote repository..."
        cargo install --git https://github.com/andreisuslov/qwen-tts
    fi

    success "qwen-tts binary installed to ~/.cargo/bin/qwen-tts"
}

# ---------------------------------------------------------------------------
# Create directory structure
# ---------------------------------------------------------------------------
QWEN_HOME="$HOME/.qwen-tts"

create_directories() {
    info "Creating directory structure at $QWEN_HOME ..."

    mkdir -p "$QWEN_HOME/models"
    mkdir -p "$QWEN_HOME/voices"
    mkdir -p "$QWEN_HOME/outputs"

    success "Directory structure created"
}

# ---------------------------------------------------------------------------
# Python virtual environment and dependencies
# ---------------------------------------------------------------------------
setup_python_venv() {
    local venv_dir="$QWEN_HOME/venv"

    if [[ -d "$venv_dir" ]]; then
        info "Python venv already exists at $venv_dir — reusing"
    else
        info "Creating Python virtual environment at $venv_dir ..."
        "$PYTHON_CMD" -m venv "$venv_dir"
        success "Python venv created"
    fi

    # Activate the venv for dependency installation
    # shellcheck disable=SC1091
    source "$venv_dir/bin/activate"

    info "Upgrading pip..."
    pip install --upgrade pip --quiet

    info "Installing Python dependencies..."
    if [[ "$PLATFORM" == "macos" ]] && $IS_APPLE_SILICON; then
        # Apple Silicon: use MLX backend
        pip install mlx-audio huggingface_hub --quiet
        success "Installed mlx-audio and huggingface_hub (Apple Silicon / MLX)"
    else
        # Linux (or macOS x86): use transformers + torch
        pip install transformers torch torchaudio huggingface_hub soundfile --quiet
        success "Installed transformers, torch, torchaudio, huggingface_hub, soundfile"
    fi

    deactivate
}

# ---------------------------------------------------------------------------
# Copy generate_compat.py (Linux / CPU / CUDA backends)
# ---------------------------------------------------------------------------
copy_generate_compat() {
    # Only needed for non-MLX backends
    if [[ "$PLATFORM" == "macos" ]] && $IS_APPLE_SILICON; then
        info "MLX backend — generate_compat.py not needed, skipping"
        return 0
    fi

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    REPO_ROOT="$(dirname "$SCRIPT_DIR")"
    local dest="$QWEN_HOME/generate_compat.py"

    # Look for generate_compat.py in several likely locations
    local src=""
    for candidate in \
        "$REPO_ROOT/generate_compat.py" \
        "$REPO_ROOT/scripts/generate_compat.py" \
        "$SCRIPT_DIR/generate_compat.py"; do
        if [[ -f "$candidate" ]]; then
            src="$candidate"
            break
        fi
    done

    if [[ -n "$src" ]]; then
        cp "$src" "$dest"
        success "Copied generate_compat.py to $dest"
    elif [[ -f "$dest" ]]; then
        info "generate_compat.py already present at $dest"
    else
        warn "generate_compat.py not found in repository ($src)."
        warn "You may need to download it manually or run: qwen-tts models download"
        warn "The file should be placed at $dest"
    fi
}

# ---------------------------------------------------------------------------
# Ensure ~/.cargo/bin is on PATH
# ---------------------------------------------------------------------------
ensure_cargo_on_path() {
    local cargo_bin="$HOME/.cargo/bin"
    local path_export_line="export PATH=\"\$HOME/.cargo/bin:\$PATH\""

    # Check if already on PATH
    if echo "$PATH" | tr ':' '\n' | grep -qx "$cargo_bin"; then
        info "~/.cargo/bin is already on PATH"
        return 0
    fi

    info "Adding ~/.cargo/bin to PATH..."

    local shells_updated=()

    # Append to shell config files if the line is not already present
    for rc_file in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.profile"; do
        if [[ -f "$rc_file" ]]; then
            if ! grep -qF '.cargo/bin' "$rc_file"; then
                printf '\n# Added by qwen-tts installer\n%s\n' "$path_export_line" >> "$rc_file"
                shells_updated+=("$rc_file")
            fi
        fi
    done

    # If no RC file existed at all, create ~/.profile
    if [[ ${#shells_updated[@]} -eq 0 ]]; then
        # Check if any file already has it
        local already_set=false
        for rc_file in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.profile"; do
            if [[ -f "$rc_file" ]] && grep -qF '.cargo/bin' "$rc_file"; then
                already_set=true
                break
            fi
        done
        if ! $already_set; then
            printf '# Added by qwen-tts installer\n%s\n' "$path_export_line" >> "$HOME/.profile"
            shells_updated+=("$HOME/.profile")
        fi
    fi

    if [[ ${#shells_updated[@]} -gt 0 ]]; then
        success "PATH updated in: ${shells_updated[*]}"
    else
        info "~/.cargo/bin already configured in shell rc files"
    fi

    # Make cargo available for the rest of this script
    export PATH="$cargo_bin:$PATH"
}

# ---------------------------------------------------------------------------
# Run qwen-tts config init
# ---------------------------------------------------------------------------
run_config_init() {
    info "Initializing qwen-tts configuration..."

    if check_command qwen-tts; then
        qwen-tts config init
        success "Configuration initialized"
    else
        # Binary might not be on current PATH yet
        local binary="$HOME/.cargo/bin/qwen-tts"
        if [[ -x "$binary" ]]; then
            "$binary" config init
            success "Configuration initialized"
        else
            warn "qwen-tts binary not found — skipping config init"
            warn "Run 'qwen-tts config init' manually after restarting your shell"
        fi
    fi
}

# ---------------------------------------------------------------------------
# Print summary
# ---------------------------------------------------------------------------
print_success() {
    echo ""
    printf "${GREEN}${BOLD}============================================${NC}\n"
    printf "${GREEN}${BOLD}  qwen-tts installed successfully!${NC}\n"
    printf "${GREEN}${BOLD}============================================${NC}\n"
    echo ""
    echo "  Binary:    ~/.cargo/bin/qwen-tts"
    echo "  Data dir:  ~/.qwen-tts/"
    echo "  Venv:      ~/.qwen-tts/venv/"
    echo "  Config:    ~/.config/qwen-tts/config.toml"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Restart your shell (or run: source ~/.zshrc)"
    echo "  2. Download a model:"
    echo "       qwen-tts models download --variant pro"
    echo "  3. Generate speech:"
    echo "       qwen-tts speak \"Hello, world!\""
    echo ""
    if [[ "$PLATFORM" == "macos" ]] && $IS_APPLE_SILICON; then
        echo "  Backend: MLX (Apple Silicon)"
    elif check_command nvidia-smi 2>/dev/null; then
        echo "  Backend: CUDA (NVIDIA GPU detected)"
    else
        echo "  Backend: CPU"
    fi
    echo ""
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    echo ""
    printf "${BOLD}qwen-tts installer${NC}\n"
    printf "${BOLD}==================${NC}\n"
    echo ""

    detect_platform
    echo ""

    check_prerequisites

    install_binary
    echo ""

    create_directories
    echo ""

    setup_python_venv
    echo ""

    copy_generate_compat
    echo ""

    ensure_cargo_on_path
    echo ""

    run_config_init
    echo ""

    print_success
}

main "$@"
