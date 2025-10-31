#!/usr/bin/env bash
# TONG Programming Language Setup Script (Unix)

set -euo pipefail

echo "🚀 Setting up TONG - The Ultimate Programming Language"
echo "=================================================="

# Check command line arguments early
WITH_SDL=0
if [[ ${1:-} == "--sdl" || ${2:-} == "--sdl" ]]; then
    WITH_SDL=1
fi

# Check for required tools
MISSING_TOOLS=()

if ! command -v cargo >/dev/null 2>&1; then
    MISSING_TOOLS+=("cargo (Rust toolchain)")
fi

if ! command -v git >/dev/null 2>&1; then
    MISSING_TOOLS+=("git")
fi

if ! command -v gh >/dev/null 2>&1; then
    MISSING_TOOLS+=("gh (GitHub CLI)")
fi

if ! command -v uv >/dev/null 2>&1; then
    MISSING_TOOLS+=("uv (Python package installer)")
fi

if [[ ${#MISSING_TOOLS[@]} -gt 0 ]]; then
    echo "❌ Missing required tools: ${MISSING_TOOLS[*]}"
    echo "Installing missing tools automatically..."
    echo ""
    
    if [[ " ${MISSING_TOOLS[*]} " =~ " cargo " ]]; then
        echo "Installing Rust toolchain..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --profile default --yes
        source ~/.cargo/env
        echo "✅ Rust installed."
    fi
    
    APT_PACKAGES=()
    if [[ " ${MISSING_TOOLS[*]} " =~ " git " ]]; then
        APT_PACKAGES+=("git")
    fi
    if [[ " ${MISSING_TOOLS[*]} " =~ " gh " ]]; then
        APT_PACKAGES+=("gh")
    fi
    
    # Add SDL3 runtime if --sdl flag is used
    if [[ $WITH_SDL -eq 1 ]]; then
        APT_PACKAGES+=("libsdl3-0")
        # Also add Xvfb for headless SDL testing
        APT_PACKAGES+=("xvfb")
    fi
    
    if [[ ${#APT_PACKAGES[@]} -gt 0 ]]; then
        echo "Installing apt packages: ${APT_PACKAGES[*]}"
        sudo apt update
        sudo apt install -y "${APT_PACKAGES[@]}"
        echo "✅ Apt packages installed."
    fi
    
    if [[ " ${MISSING_TOOLS[*]} " =~ " uv " ]]; then
        echo "Installing uv..."
        curl -LsSf https://astral.sh/uv/install.sh | sh
        export PATH="$HOME/.local/bin:$PATH"
        echo "✅ uv installed."
    fi
    
    echo "Re-checking tools..."
    # Re-check to ensure installation succeeded
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ Cargo still not found. Please check your PATH or install manually."
        exit 1
    fi
    if ! command -v git >/dev/null 2>&1; then
        echo "❌ Git still not found. Please install manually."
        exit 1
    fi
    if ! command -v gh >/dev/null 2>&1; then
        echo "❌ GitHub CLI still not found. Please install manually."
        exit 1
    fi
    if ! command -v uv >/dev/null 2>&1; then
        echo "❌ uv still not found. Please check your PATH or install manually."
        exit 1
    fi
    echo "✅ All tools are now available."
fi

# Install SDL dependencies if --sdl is used but tools were already available
if [[ $WITH_SDL -eq 1 && ${#MISSING_TOOLS[@]} -eq 0 ]]; then
    echo "Checking SDL dependencies..."
    SDL_PACKAGES=()
    
    if ! dpkg -l libsdl3-0 >/dev/null 2>&1; then
        SDL_PACKAGES+=("libsdl3-0")
    fi
    
    if ! dpkg -l xvfb >/dev/null 2>&1; then
        SDL_PACKAGES+=("xvfb")
    fi
    
    if [[ ${#SDL_PACKAGES[@]} -gt 0 ]]; then
        echo "Installing SDL packages: ${SDL_PACKAGES[*]}"
        sudo apt update
        sudo apt install -y "${SDL_PACKAGES[@]}"
        echo "✅ SDL packages installed."
    else
        echo "✅ SDL packages already installed."
    fi
fi

echo "Building tong (release)..."
pushd rust/tong >/dev/null
if [[ $WITH_SDL -eq 1 ]]; then
    echo "(enabling SDL3 feature)"
    cargo build --release --features sdl3
else
    cargo build --release
fi
popd >/dev/null

# Check display availability for SDL builds
if [[ $WITH_SDL -eq 1 ]]; then
    echo "Checking display availability for SDL..."
    if ! xset q >/dev/null 2>&1; then
        echo "⚠️  Warning: No X11 display available in this environment."
        echo "   SDL applications require a display server to run."
        echo ""
        echo "   Solutions:"
        echo "   1. X11 forwarding: docker run -e DISPLAY=\$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix <image>"
        echo "   2. Virtual display: xvfb-run tong <file>  (Xvfb was installed automatically)"
        echo "   3. GUI-enabled container with proper display setup"
        echo ""
        echo "   The binary was built successfully but SDL features won't work without a display."
    else
        echo "✅ Display available - SDL features should work"
    fi
fi

BIN="$(pwd)/rust/tong/target/release/tong"
if [[ "$1" = "--global" || "$2" = "--global" ]]; then
    echo "Creating global symlink..."
    sudo ln -sf "$BIN" /usr/local/bin/tong
    echo "✅ TONG is now available globally as 'tong'"
else
    echo "Built binary at: $BIN"
    echo "You can run: $BIN examples/hello.tong"
fi

echo ""
echo "🎯 Quick Start:"
echo "  cargo run -p tong -- ../../examples/hello.tong    # Run example"
echo "  cargo build -p tong --release                     # Build optimized binary"
echo "  tong ../../examples/hello.tong                    # After --global install"
echo ""
echo "🖼  SDL Pong example (needs feature):"
echo "  cargo run --features sdl3 -- ../../examples/modules/sdl/pong.tong"
echo "  ./setup.sh --sdl --global   # install global binary with SDL3 enabled"
echo "  xvfb-run tong examples/modules/sdl/pong.tong  # Run in headless containers"
echo ""
echo "📚 Examples available in examples/ directory"
echo "📖 See README.md for full documentation"
echo ""
echo "✨ TONG is ready for heterogeneous computing!"