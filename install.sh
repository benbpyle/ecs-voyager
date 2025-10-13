#!/usr/bin/env bash
# ECS Voyager Installation Script
# Automatically detects the system and installs the appropriate package

set -e

VERSION="0.2.7"
REPO="benbpyle/ecs-voyager"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"

    case "$os" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="macos"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    info "Detected platform: $OS ($ARCH)"
}

# Detect Linux distribution
detect_linux_distro() {
    if [ "$OS" != "linux" ]; then
        return
    fi

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO="$ID"
        DISTRO_VERSION="$VERSION_ID"
    elif [ -f /etc/redhat-release ]; then
        DISTRO="rhel"
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
    else
        warning "Could not detect Linux distribution, using generic installation"
        DISTRO="generic"
    fi

    info "Detected distribution: $DISTRO"
}

# Install on macOS using Homebrew
install_macos() {
    info "Installing ECS Voyager on macOS..."

    if ! command -v brew &> /dev/null; then
        error "Homebrew is not installed. Please install it from https://brew.sh"
    fi

    info "Adding tap benbpyle/ecs-voyager..."
    brew tap benbpyle/ecs-voyager

    info "Installing ecs-voyager..."
    brew install ecs-voyager

    success "ECS Voyager installed successfully!"
}

# Install on Debian/Ubuntu
install_debian() {
    info "Installing ECS Voyager on Debian/Ubuntu..."

    local download_url="https://github.com/$REPO/releases/download/v${VERSION}/ecs-voyager_${VERSION}_amd64.deb"
    local temp_file="/tmp/ecs-voyager_${VERSION}_amd64.deb"

    info "Downloading package..."
    curl -sL "$download_url" -o "$temp_file"

    info "Installing package (requires sudo)..."
    sudo dpkg -i "$temp_file"

    info "Installing dependencies..."
    sudo apt-get install -f -y

    rm "$temp_file"
    success "ECS Voyager installed successfully!"
}

# Install on RedHat/Fedora/CentOS
install_redhat() {
    info "Installing ECS Voyager on RedHat/Fedora/CentOS..."

    local download_url="https://github.com/$REPO/releases/download/v${VERSION}/ecs-voyager-${VERSION}-1.x86_64.rpm"
    local temp_file="/tmp/ecs-voyager-${VERSION}-1.x86_64.rpm"

    info "Downloading package..."
    curl -sL "$download_url" -o "$temp_file"

    if command -v dnf &> /dev/null; then
        info "Installing package with dnf (requires sudo)..."
        sudo dnf install -y "$temp_file"
    elif command -v yum &> /dev/null; then
        info "Installing package with yum (requires sudo)..."
        sudo yum install -y "$temp_file"
    else
        info "Installing package with rpm (requires sudo)..."
        sudo rpm -i "$temp_file"
    fi

    rm "$temp_file"
    success "ECS Voyager installed successfully!"
}

# Generic Linux installation (extract tarball)
install_generic() {
    info "Installing ECS Voyager (generic method)..."

    local download_url="https://github.com/$REPO/releases/download/v${VERSION}/ecs-voyager-v${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
    local temp_file="/tmp/ecs-voyager.tar.gz"
    local install_dir="/usr/local/bin"

    info "Downloading binary..."
    curl -sL "$download_url" -o "$temp_file"

    info "Extracting..."
    tar -xzf "$temp_file" -C /tmp

    info "Installing to $install_dir (requires sudo)..."
    sudo install -m 755 /tmp/ecs-voyager "$install_dir/ecs-voyager"

    rm "$temp_file"
    rm /tmp/ecs-voyager
    success "ECS Voyager installed successfully!"
}

# Main installation logic
main() {
    echo ""
    echo "ECS Voyager Installer v${VERSION}"
    echo "================================"
    echo ""

    detect_platform
    detect_linux_distro

    case "$OS" in
        macos)
            install_macos
            ;;
        linux)
            case "$DISTRO" in
                ubuntu|debian)
                    install_debian
                    ;;
                fedora|rhel|centos|rocky|alma)
                    install_redhat
                    ;;
                *)
                    install_generic
                    ;;
            esac
            ;;
    esac

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    info "Usage: ecs-voyager"
    info "       ecs-voyager --help"
    echo ""
    warning "For ECS Exec and Port Forwarding features, install session-manager-plugin:"
    info "  https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

main "$@"
