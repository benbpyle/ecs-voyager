# Packaging & Release Guide

This document describes how ECS Voyager is packaged and distributed across different platforms.

## Table of Contents

- [Overview](#overview)
- [Build Process](#build-process)
- [Platform Packages](#platform-packages)
  - [macOS (Homebrew)](#macos-homebrew)
  - [Windows (Chocolatey)](#windows-chocolatey)
  - [Debian/Ubuntu (.deb)](#debianubuntu-deb)
  - [RedHat/Fedora (.rpm)](#redhatfedora-rpm)
  - [Generic Linux (Tarball)](#generic-linux-tarball)
- [Release Process](#release-process)
- [Testing Packages](#testing-packages)
- [Troubleshooting](#troubleshooting)

## Overview

ECS Voyager supports multiple distribution channels to make installation easy across all major platforms:

| Platform | Package Manager | Format | Auto-Update |
|----------|----------------|--------|-------------|
| macOS | Homebrew | Formula | ✅ |
| Windows | Chocolatey | .nupkg | ✅ |
| Debian/Ubuntu | dpkg/apt | .deb | ❌ |
| RedHat/Fedora | rpm/dnf/yum | .rpm | ❌ |
| Generic Linux | Manual | .tar.gz | ❌ |
| All Platforms | Cargo | Source | ✅ |

## Build Process

### Prerequisites

1. **Rust Toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Cross-Compilation Targets**
   ```bash
   # macOS Intel
   rustup target add x86_64-apple-darwin

   # macOS Apple Silicon
   rustup target add aarch64-apple-darwin

   # Linux
   rustup target add x86_64-unknown-linux-gnu

   # Windows
   rustup target add x86_64-pc-windows-msvc
   ```

3. **Platform-Specific Tools**
   - **macOS**: Xcode Command Line Tools
   - **Linux**: `dpkg-deb`, `rpmbuild`
   - **Windows**: Visual Studio Build Tools

### Building Binaries

```bash
# Build for current platform
cargo build --release

# Cross-compile for specific target
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

The binaries will be in `target/release/` or `target/<target>/release/`.

## Platform Packages

### macOS (Homebrew)

**Location**: Separate tap repository at `benbpyle/homebrew-ecs-voyager`

**Files**:
- `Formula/ecs-voyager.rb` - Homebrew formula

**Formula Structure**:
```ruby
class EcsVoyager < Formula
  desc "Terminal User Interface for AWS ECS Management"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.2.7"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-aarch64-apple-darwin.tar.gz"
      sha256 "..." # Calculate with: shasum -a 256 file.tar.gz
    else
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  def install
    bin.install "ecs-voyager"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ecs-voyager --version")
  end
end
```

**Update Process**:
1. Build binaries for both architectures
2. Create tarballs
3. Calculate SHA256 checksums
4. Update formula with new version and checksums
5. Commit and push to tap repository

**Testing**:
```bash
# Install from local formula
brew install --build-from-source ./Formula/ecs-voyager.rb

# Test installation
ecs-voyager --version
```

### Windows (Chocolatey)

**Location**: `packaging/chocolatey/`

**Files**:
- `ecs-voyager.nuspec` - Package manifest
- `tools/chocolateyinstall.ps1` - Installation script
- `tools/chocolateyuninstall.ps1` - Uninstallation script

**Building Package**:
```powershell
# Navigate to packaging directory
cd packaging/chocolatey

# Build package
choco pack

# Output: ecs-voyager.0.2.7.nupkg
```

**Package Structure**:
```
ecs-voyager.0.2.7.nupkg
├── ecs-voyager.nuspec
├── tools/
│   ├── chocolateyinstall.ps1
│   └── chocolateyuninstall.ps1
```

**Update Process**:
1. Update version in `ecs-voyager.nuspec`
2. Update download URL in `chocolateyinstall.ps1`
3. Calculate SHA256 checksum: `Get-FileHash file.zip -Algorithm SHA256`
4. Update checksum in install script
5. Run `choco pack`
6. Test locally: `choco install ecs-voyager -s .`
7. Push to Chocolatey: `choco push ecs-voyager.0.2.7.nupkg --source https://push.chocolatey.org/`

**Testing**:
```powershell
# Install local package
choco install ecs-voyager -s . -y

# Test
ecs-voyager --version

# Uninstall
choco uninstall ecs-voyager -y
```

### Debian/Ubuntu (.deb)

**Location**: `packaging/debian/`

**Structure**:
```
packaging/debian/
├── DEBIAN/
│   ├── control        # Package metadata
│   └── postinst       # Post-installation script
├── usr/
│   ├── bin/
│   │   └── ecs-voyager    # Binary
│   └── share/
│       └── doc/
│           └── ecs-voyager/
│               └── copyright
```

**Building Package**:
```bash
# Create package directory
mkdir -p packaging/debian/usr/bin

# Copy binary
cp target/x86_64-unknown-linux-gnu/release/ecs-voyager packaging/debian/usr/bin/

# Set permissions
chmod 755 packaging/debian/usr/bin/ecs-voyager
chmod 755 packaging/debian/DEBIAN/postinst

# Build .deb package
dpkg-deb --build packaging/debian ecs-voyager_0.2.7_amd64.deb
```

**Update Process**:
1. Update version in `DEBIAN/control`
2. Build binary for `x86_64-unknown-linux-gnu`
3. Copy binary to package structure
4. Run `dpkg-deb --build`
5. Test: `sudo dpkg -i ecs-voyager_0.2.7_amd64.deb`
6. Upload to GitHub releases

**Testing**:
```bash
# Install
sudo dpkg -i ecs-voyager_0.2.7_amd64.deb

# Verify
ecs-voyager --version
dpkg -l | grep ecs-voyager

# Uninstall
sudo dpkg -r ecs-voyager
```

### RedHat/Fedora (.rpm)

**Location**: `packaging/rpm/`

**Files**:
- `ecs-voyager.spec` - RPM specification file

**Building Package**:
```bash
# Create RPM build environment
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy spec file
cp packaging/rpm/ecs-voyager.spec ~/rpmbuild/SPECS/

# Download source tarball to SOURCES
curl -L https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz \
  -o ~/rpmbuild/SOURCES/ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/ecs-voyager.spec

# Output: ~/rpmbuild/RPMS/x86_64/ecs-voyager-0.2.7-1.x86_64.rpm
```

**Update Process**:
1. Update version in `ecs-voyager.spec`
2. Update changelog in spec file
3. Build binary for `x86_64-unknown-linux-gnu`
4. Create tarball with proper directory structure
5. Run `rpmbuild -ba`
6. Test installation
7. Upload to GitHub releases

**Testing**:
```bash
# Install (Fedora/RHEL 8+)
sudo dnf install ~/rpmbuild/RPMS/x86_64/ecs-voyager-0.2.7-1.x86_64.rpm

# Install (RHEL/CentOS 7)
sudo yum install ~/rpmbuild/RPMS/x86_64/ecs-voyager-0.2.7-1.x86_64.rpm

# Verify
ecs-voyager --version
rpm -qi ecs-voyager

# Uninstall
sudo dnf remove ecs-voyager  # or: sudo yum remove ecs-voyager
```

### Generic Linux (Tarball)

**Building**:
```bash
# Build for Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Create tarball
cd target/x86_64-unknown-linux-gnu/release
tar -czf ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz ecs-voyager

# Also for ARM64
cargo build --release --target aarch64-unknown-linux-gnu
cd target/aarch64-unknown-linux-gnu/release
tar -czf ecs-voyager-v0.2.7-aarch64-unknown-linux-gnu.tar.gz ecs-voyager
```

**Testing**:
```bash
# Extract and test
tar -xzf ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz
./ecs-voyager --version

# Install system-wide
sudo install -m 755 ecs-voyager /usr/local/bin/
```

## Release Process

### 1. Pre-Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy --all-targets`
- [ ] Format code: `cargo fmt`
- [ ] Update `CHANGELOG.md`
- [ ] Update version in README.md
- [ ] Commit changes: `git commit -am "chore: bump version to v0.2.X"`
- [ ] Create git tag: `git tag -a v0.2.X -m "Release v0.2.X"`

### 2. Build Binaries

```bash
# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS ARM
cargo build --release --target aarch64-apple-darwin

# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### 3. Create Tarballs

```bash
# Script to create all tarballs
#!/bin/bash
VERSION="0.2.7"

for target in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  cd target/$target/release
  tar -czf ../../../ecs-voyager-v$VERSION-$target.tar.gz ecs-voyager
  cd ../../..
done

# Windows ZIP
cd target/x86_64-pc-windows-msvc/release
zip ../../../ecs-voyager-v$VERSION-x86_64-pc-windows-msvc.zip ecs-voyager.exe
cd ../../..
```

### 4. Calculate Checksums

```bash
# Create checksums file
shasum -a 256 ecs-voyager-v*.tar.gz ecs-voyager-v*.zip > SHA256SUMS.txt

# Or for individual files
shasum -a 256 ecs-voyager-v0.2.7-x86_64-apple-darwin.tar.gz
```

### 5. Build Platform Packages

```bash
# Debian package
dpkg-deb --build packaging/debian ecs-voyager_0.2.7_amd64.deb

# RPM package
rpmbuild -ba packaging/rpm/ecs-voyager.spec

# Chocolatey package
cd packaging/chocolatey && choco pack
```

### 6. Create GitHub Release

```bash
# Push tag
git push origin v0.2.7

# Create release using gh CLI
gh release create v0.2.7 \
  --title "v0.2.7 - Release Name" \
  --notes-file RELEASE_NOTES.md \
  ecs-voyager-v0.2.7-*.tar.gz \
  ecs-voyager-v0.2.7-*.zip \
  ecs-voyager_0.2.7_amd64.deb \
  ecs-voyager-0.2.7-1.x86_64.rpm \
  SHA256SUMS.txt
```

### 7. Update Package Managers

**Homebrew**:
```bash
cd ../homebrew-ecs-voyager
# Update Formula/ecs-voyager.rb with new version and checksums
git commit -am "chore: bump version to v0.2.7"
git push
```

**Chocolatey**:
```powershell
choco push ecs-voyager.0.2.7.nupkg --source https://push.chocolatey.org/ --api-key YOUR_API_KEY
```

**Cargo (crates.io)**:
```bash
cargo publish
```

### 8. Post-Release

- [ ] Verify all download links work
- [ ] Test installation on each platform
- [ ] Update website/documentation if applicable
- [ ] Announce release (Twitter, Reddit, HN, etc.)

## Testing Packages

### Local Testing Matrix

| Platform | Test Command | Expected Result |
|----------|-------------|-----------------|
| macOS (Homebrew) | `brew install --build-from-source ./Formula/ecs-voyager.rb` | Installation success |
| Windows (Chocolatey) | `choco install ecs-voyager -s . -y` | Installation success |
| Debian/Ubuntu | `sudo dpkg -i ecs-voyager_0.2.7_amd64.deb` | Installation success |
| RedHat/Fedora | `sudo dnf install ecs-voyager-0.2.7-1.x86_64.rpm` | Installation success |
| Generic Linux | `tar -xzf ... && sudo install -m 755 ecs-voyager /usr/local/bin/` | Manual install |

### Verification

After installation on each platform:

```bash
# Check version
ecs-voyager --version

# Check binary location
which ecs-voyager

# Run basic functionality test
ecs-voyager  # Should start and connect to AWS

# Check help
ecs-voyager --help
```

## Troubleshooting

### Common Issues

**Issue**: Binary not in PATH after installation
- **macOS**: Check `/usr/local/bin` or `$(brew --prefix)/bin`
- **Linux**: Check `/usr/bin` or `/usr/local/bin`
- **Windows**: Check `C:\ProgramData\chocolatey\bin`

**Issue**: Permission denied when running binary
```bash
chmod +x /path/to/ecs-voyager
```

**Issue**: Library version mismatch (Linux)
```bash
# Check required libraries
ldd /path/to/ecs-voyager

# Install missing dependencies
sudo apt-get install -f  # Debian/Ubuntu
sudo dnf install -y ...  # Fedora/RHEL
```

**Issue**: Windows Defender blocking executable
- This is normal for unsigned binaries
- Users may need to add an exception

### Build Troubleshooting

**Issue**: Cross-compilation failing
```bash
# Install cross-compilation tools
cargo install cross

# Use cross instead of cargo
cross build --release --target x86_64-unknown-linux-gnu
```

**Issue**: OpenSSL errors on Linux
```bash
# Install OpenSSL development libraries
sudo apt-get install libssl-dev pkg-config  # Debian/Ubuntu
sudo dnf install openssl-devel pkgconfig    # Fedora/RHEL
```

## Automated Release Workflow

Consider using GitHub Actions to automate the release process:

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v3
        with:
          name: ecs-voyager-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/ecs-voyager*
```

## Additional Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Chocolatey Package Creation](https://docs.chocolatey.org/en-us/create/create-packages)
- [Debian Package Building](https://www.debian.org/doc/manuals/debmake-doc/)
- [RPM Packaging Guide](https://rpm-packaging-guide.github.io/)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)

## Maintainer Notes

- Keep all package versions synchronized
- Test on clean VMs/containers before release
- Maintain changelog with user-facing changes
- Use semantic versioning (MAJOR.MINOR.PATCH)
- Sign releases when possible (GPG, notarization)

---

For questions or issues with packaging, please open an issue on GitHub.
