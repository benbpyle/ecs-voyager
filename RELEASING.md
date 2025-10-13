# Release Process Guide

This document provides step-by-step instructions for releasing a new version of ECS Voyager across all platforms.

## Overview

ECS Voyager is distributed through multiple channels:
- **macOS**: Homebrew tap (auto-updates)
- **Windows**: Chocolatey (auto-updates)
- **Linux**: .deb packages (Debian/Ubuntu), .rpm packages (RedHat/Fedora), generic tarballs
- **Source**: Cargo (crates.io), GitHub releases

## Prerequisites

Before starting a release, ensure you have:

1. **Development environment** set up:
   - Rust toolchain (1.70+)
   - Cross-compilation targets installed (see PACKAGING.md)
   - `gh` CLI tool installed for GitHub operations
   - `cargo` with publish permissions for crates.io

2. **Platform-specific tools** (if building locally):
   - macOS: Xcode Command Line Tools
   - Linux: `dpkg-deb`, `rpmbuild`
   - Windows: Chocolatey CLI, PowerShell

3. **Access credentials**:
   - GitHub personal access token with repo and packages permissions
   - Chocolatey API key (for pushing packages)
   - Cargo registry authentication (for crates.io)

## Release Workflow

### Step 1: Pre-Release Preparation

1. **Update version** in all necessary files:
   ```bash
   # Update Cargo.toml
   vim Cargo.toml  # Change version = "0.2.X"

   # Update install.sh
   vim install.sh  # Change VERSION="0.2.X"

   # Update PACKAGING.md examples
   vim PACKAGING.md  # Update version references

   # Update README.md if needed
   vim README.md
   ```

2. **Update CHANGELOG.md** with new version details:
   ```bash
   vim CHANGELOG.md
   ```
   Add a section like:
   ```markdown
   ## [0.2.X] - 2025-01-XX

   ### Added
   - New feature descriptions

   ### Changed
   - Modified functionality

   ### Fixed
   - Bug fixes
   ```

3. **Run quality checks**:
   ```bash
   # Format code
   cargo fmt

   # Run clippy
   cargo clippy --all-targets

   # Run all tests
   cargo test

   # Build release locally
   cargo build --release

   # Test the binary
   ./target/release/ecs-voyager --version
   ```

4. **Commit version bump**:
   ```bash
   git add Cargo.toml install.sh PACKAGING.md CHANGELOG.md README.md
   git commit -m "chore: bump version to v0.2.X"
   git push origin main
   ```

### Step 2: Create Git Tag

```bash
# Create annotated tag
git tag -a v0.2.X -m "Release v0.2.X"

# Push tag to trigger release workflows (if using GitHub Actions)
git push origin v0.2.X
```

### Step 3: Build Release Artifacts

#### Option A: Using GitHub Actions (Recommended)

If you have `.github/workflows/release.yml` configured:
- The push of the tag automatically triggers the workflow
- Binaries are built for all platforms
- GitHub release is created with artifacts

#### Option B: Manual Build

If building manually:

```bash
# Set version variable
VERSION="0.2.X"

# Build for all targets
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc

# Create tarballs for macOS and Linux
cd target/x86_64-apple-darwin/release
tar -czf ../../../ecs-voyager-v${VERSION}-x86_64-apple-darwin.tar.gz ecs-voyager
cd ../../..

cd target/aarch64-apple-darwin/release
tar -czf ../../../ecs-voyager-v${VERSION}-aarch64-apple-darwin.tar.gz ecs-voyager
cd ../../..

cd target/x86_64-unknown-linux-gnu/release
tar -czf ../../../ecs-voyager-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz ecs-voyager
cd ../../..

cd target/aarch64-unknown-linux-gnu/release
tar -czf ../../../ecs-voyager-v${VERSION}-aarch64-unknown-linux-gnu.tar.gz ecs-voyager
cd ../../..

# Create ZIP for Windows
cd target/x86_64-pc-windows-msvc/release
zip ../../../ecs-voyager-v${VERSION}-x86_64-pc-windows-msvc.zip ecs-voyager.exe
cd ../../..
```

### Step 4: Calculate Checksums

```bash
# Generate SHA256 checksums for all archives
shasum -a 256 ecs-voyager-v${VERSION}-*.tar.gz ecs-voyager-v${VERSION}-*.zip > SHA256SUMS.txt

# Display checksums for package manifests
cat SHA256SUMS.txt
```

### Step 5: Build Platform-Specific Packages

#### Debian Package (.deb)

```bash
# Update version in control file
vim packaging/debian/DEBIAN/control  # Set Version: 0.2.X

# Copy binary to package structure
mkdir -p packaging/debian/usr/bin
cp target/x86_64-unknown-linux-gnu/release/ecs-voyager packaging/debian/usr/bin/

# Set correct permissions
chmod 755 packaging/debian/usr/bin/ecs-voyager
chmod 755 packaging/debian/DEBIAN/postinst

# Build package
dpkg-deb --build packaging/debian ecs-voyager_${VERSION}_amd64.deb
```

#### RPM Package (.rpm)

```bash
# Update version in spec file
vim packaging/rpm/ecs-voyager.spec  # Set Version: 0.2.X

# Create RPM build environment if needed
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy spec file
cp packaging/rpm/ecs-voyager.spec ~/rpmbuild/SPECS/

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/ecs-voyager.spec

# Copy resulting RPM
cp ~/rpmbuild/RPMS/x86_64/ecs-voyager-${VERSION}-1.x86_64.rpm .
```

#### Chocolatey Package (.nupkg)

```bash
# Update version in nuspec
vim packaging/chocolatey/ecs-voyager.nuspec  # Set <version>0.2.X</version>

# Update download URL and checksum in install script
vim packaging/chocolatey/tools/chocolateyinstall.ps1

# Build package
cd packaging/chocolatey
choco pack
cd ../..

# Copy resulting package
cp packaging/chocolatey/ecs-voyager.${VERSION}.nupkg .
```

### Step 6: Create GitHub Release

```bash
# Create release with gh CLI
gh release create v${VERSION} \
  --title "v${VERSION} - Release Title" \
  --notes-file RELEASE_NOTES.md \
  ecs-voyager-v${VERSION}-*.tar.gz \
  ecs-voyager-v${VERSION}-*.zip \
  ecs-voyager_${VERSION}_amd64.deb \
  ecs-voyager-${VERSION}-1.x86_64.rpm \
  ecs-voyager.${VERSION}.nupkg \
  SHA256SUMS.txt

# Verify release was created
gh release view v${VERSION}
```

**RELEASE_NOTES.md Template**:
```markdown
# ECS Voyager v0.2.X - Release Name

## 🎉 Highlights

Brief description of major changes or new features.

## 📋 Changes

### Added
- Feature 1
- Feature 2

### Changed
- Enhancement 1
- Enhancement 2

### Fixed
- Bug fix 1
- Bug fix 2

## 📦 Installation

### macOS (Homebrew)
\`\`\`bash
brew tap benbpyle/ecs-voyager
brew install ecs-voyager
\`\`\`

### Windows (Chocolatey)
\`\`\`powershell
choco install ecs-voyager
\`\`\`

### Linux
See installation instructions in README.md for your distribution.

## 📝 Full Changelog
See CHANGELOG.md for complete details.

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

### Step 7: Update Package Managers

#### Update Homebrew Tap

```bash
# Clone the tap repository (if not already cloned)
git clone https://github.com/benbpyle/homebrew-ecs-voyager.git ../homebrew-ecs-voyager
cd ../homebrew-ecs-voyager

# Update formula version and checksums
vim Formula/ecs-voyager.rb

# Update the version number
# Update the URLs for both architectures
# Calculate and update SHA256 checksums:
#   shasum -a 256 ecs-voyager-v0.2.X-aarch64-apple-darwin.tar.gz
#   shasum -a 256 ecs-voyager-v0.2.X-x86_64-apple-darwin.tar.gz

# Example formula updates:
# version "0.2.X"
# url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.X/ecs-voyager-v0.2.X-aarch64-apple-darwin.tar.gz"
# sha256 "NEW_CHECKSUM_HERE"

# Commit and push
git add Formula/ecs-voyager.rb
git commit -m "chore: bump ecs-voyager to v0.2.X"
git push origin main

cd ../ecs-voyager
```

**Formula Update Example**:
```ruby
class EcsVoyager < Formula
  desc "Terminal User Interface for AWS ECS Management"
  homepage "https://github.com/benbpyle/ecs-voyager"
  version "0.2.X"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.X/ecs-voyager-v0.2.X-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_CHECKSUM"
    else
      url "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.X/ecs-voyager-v0.2.X-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_CHECKSUM"
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

#### Publish to Chocolatey

```bash
# Push package to Chocolatey (requires API key)
choco push ecs-voyager.${VERSION}.nupkg --source https://push.chocolatey.org/ --api-key YOUR_API_KEY

# Verify submission at https://community.chocolatey.org/packages/ecs-voyager
```

**Note**: Chocolatey packages go through moderation. It may take 24-48 hours for the package to be approved and available.

#### Publish to Crates.io

```bash
# Ensure you're on the correct commit/tag
git checkout v${VERSION}

# Publish to crates.io
cargo publish

# Verify publication
cargo search ecs-voyager
```

### Step 8: Post-Release Verification

1. **Test installation on each platform**:

   **macOS**:
   ```bash
   brew tap benbpyle/ecs-voyager
   brew install ecs-voyager
   ecs-voyager --version
   ```

   **Windows** (after Chocolatey approval):
   ```powershell
   choco install ecs-voyager
   ecs-voyager --version
   ```

   **Debian/Ubuntu**:
   ```bash
   wget https://github.com/benbpyle/ecs-voyager/releases/download/v${VERSION}/ecs-voyager_${VERSION}_amd64.deb
   sudo dpkg -i ecs-voyager_${VERSION}_amd64.deb
   ecs-voyager --version
   ```

   **RedHat/Fedora**:
   ```bash
   wget https://github.com/benbpyle/ecs-voyager/releases/download/v${VERSION}/ecs-voyager-${VERSION}-1.x86_64.rpm
   sudo dnf install ecs-voyager-${VERSION}-1.x86_64.rpm
   ecs-voyager --version
   ```

   **Universal Installer**:
   ```bash
   curl -sL https://raw.githubusercontent.com/benbpyle/ecs-voyager/main/install.sh | bash
   ```

2. **Verify download links** in README.md and GitHub release page

3. **Test basic functionality**:
   ```bash
   ecs-voyager --help
   ecs-voyager  # Should connect to AWS and display clusters
   ```

### Step 9: Announce Release

1. **Update project website** (if applicable)

2. **Social media announcement** (optional):
   - Twitter/X
   - Reddit (r/rust, r/aws)
   - Hacker News
   - Dev.to

3. **Community channels**:
   - Rust Discord
   - AWS Discord/Slack communities

## Troubleshooting

### Issue: Cross-compilation fails

**Solution**: Use the `cross` tool for more reliable cross-compilation:
```bash
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
```

### Issue: Homebrew checksum mismatch

**Solution**: Recalculate checksums after ensuring the tarball was created correctly:
```bash
shasum -a 256 ecs-voyager-v${VERSION}-aarch64-apple-darwin.tar.gz
shasum -a 256 ecs-voyager-v${VERSION}-x86_64-apple-darwin.tar.gz
```

### Issue: Chocolatey package rejected during moderation

**Solution**: Review the moderation comments and fix any issues (usually related to download URLs, checksums, or install script). Resubmit the package.

### Issue: GitHub Actions workflow fails

**Solution**: Check the workflow logs in GitHub Actions tab. Common issues:
- Missing secrets/credentials
- Compilation errors (ensure code compiles locally first)
- Network timeouts (retry the workflow)

## Automation Considerations

Consider automating the release process with GitHub Actions:

1. **On tag push**: Automatically build all binaries
2. **Create GitHub release**: With all artifacts attached
3. **Update Homebrew tap**: Using a bot or automated PR
4. **Publish to crates.io**: Using cargo publish in CI

See `.github/workflows/release.yml` template in PACKAGING.md for implementation details.

## Release Checklist

Use this checklist for each release:

- [ ] Update version in Cargo.toml
- [ ] Update version in install.sh
- [ ] Update CHANGELOG.md
- [ ] Update README.md (if needed)
- [ ] Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] Commit and push version bump
- [ ] Create and push git tag
- [ ] Build release binaries for all platforms
- [ ] Calculate SHA256 checksums
- [ ] Build .deb package
- [ ] Build .rpm package
- [ ] Build .nupkg package
- [ ] Create GitHub release with all artifacts
- [ ] Update Homebrew formula
- [ ] Push to Chocolatey
- [ ] Publish to crates.io
- [ ] Test installation on macOS
- [ ] Test installation on Windows
- [ ] Test installation on Linux (Debian)
- [ ] Test installation on Linux (RedHat)
- [ ] Test universal installer
- [ ] Verify all download links work
- [ ] Announce release

## Support

For issues with the release process, open an issue on GitHub or contact the maintainers.
