# Testing Release Builds

This guide explains how to test Windows and Linux builds both locally and in GitHub Actions.

## What's in the GitHub Actions Workflow

The `.github/workflows/release.yml` workflow now includes:

### Build Matrix
- **macOS**: x86_64 (Intel) and aarch64 (Apple Silicon)
- **Linux**: x86_64 and aarch64 (ARM64)
- **Windows**: x86_64

### Package Types
- **Tarballs** (`.tar.gz`): macOS and Linux binaries
- **ZIP** (`.zip`): Windows binary
- **Debian** (`.deb`): Ubuntu/Debian package
- **RPM** (`.rpm`): RedHat/Fedora/CentOS package

### Additional Features
- Individual SHA256 checksums for each artifact
- Consolidated `SHA256SUMS.txt` with all checksums
- Automatic Homebrew formula updates (macOS)
- Binary stripping for smaller file sizes

## How the Workflow Works

1. **Trigger**: Runs automatically after CI passes on master/main branches
2. **Tag Creation**: Creates a git tag from `Cargo.toml` version
3. **Parallel Builds**: Builds all 5 binary targets in parallel
4. **Package Building**: Creates .deb and .rpm packages from Linux x86_64 binary
5. **Checksum Generation**: Creates SHA256 checksums for all artifacts
6. **Release Upload**: Uploads all artifacts to GitHub release
7. **Homebrew Update**: Automatically updates the Homebrew tap

## Local Testing

### Prerequisites

Install cross-compilation tools:

```bash
# Rust targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc

# For Linux cross-compilation (on Linux)
sudo apt-get install gcc-aarch64-linux-gnu

# For easier cross-compilation
cargo install cross
```

### Test Building All Targets

```bash
# macOS (if on macOS)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Linux (use cross for easier cross-compilation)
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu

# Windows (use cross)
cross build --release --target x86_64-pc-windows-msvc
```

### Test Creating Archives

```bash
VERSION="0.2.7"

# macOS tarball (Intel)
cd target/x86_64-apple-darwin/release
tar -czf ../../ecs-voyager-v${VERSION}-x86_64-apple-darwin.tar.gz ecs-voyager
cd ../../..

# Linux tarball
cd target/x86_64-unknown-linux-gnu/release
tar -czf ../../ecs-voyager-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz ecs-voyager
cd ../../..

# Windows ZIP (requires PowerShell or zip tool)
cd target/x86_64-pc-windows-msvc/release
zip ../../ecs-voyager-v${VERSION}-x86_64-pc-windows-msvc.zip ecs-voyager.exe
cd ../../..
```

### Test .deb Package

```bash
# Extract Linux binary to packaging structure
cd target/x86_64-unknown-linux-gnu/release
tar -xzf ecs-voyager-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz
cd ../../..

# Prepare package
mkdir -p packaging/debian/usr/bin
cp target/x86_64-unknown-linux-gnu/release/ecs-voyager packaging/debian/usr/bin/
chmod 755 packaging/debian/usr/bin/ecs-voyager

# Build package
dpkg-deb --build packaging/debian ecs-voyager_${VERSION}_amd64.deb

# Test installation (requires sudo)
sudo dpkg -i ecs-voyager_${VERSION}_amd64.deb
ecs-voyager --version

# Uninstall
sudo dpkg -r ecs-voyager
```

### Test .rpm Package

```bash
# Requires RPM tools
sudo apt-get install rpm  # Debian/Ubuntu
# or
sudo dnf install rpm-build  # Fedora/RHEL

# Set up RPM build environment
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy tarball to SOURCES
cp target/x86_64-unknown-linux-gnu/release/ecs-voyager-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz \
   ~/rpmbuild/SOURCES/

# Copy spec file
cp packaging/rpm/ecs-voyager.spec ~/rpmbuild/SPECS/

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/ecs-voyager.spec

# Test installation (on Fedora/RHEL)
sudo dnf install ~/rpmbuild/RPMS/x86_64/ecs-voyager-${VERSION}-1.x86_64.rpm
ecs-voyager --version
```

## Testing GitHub Actions Workflow

### Option 1: Test on a Branch

Create a test branch and trigger the workflow:

```bash
# Create test branch
git checkout -b test-release-workflow

# Make a small change to trigger CI
# (e.g., update a comment or version in Cargo.toml)

# Commit and push
git commit -am "test: trigger release workflow"
git push origin test-release-workflow

# Monitor the workflow
gh run watch
```

**Note**: The release workflow only runs on master/main branches, so you may need to temporarily modify the workflow trigger for testing:

```yaml
on:
  push:
    branches:
      - test-release-workflow  # Add your test branch
```

### Option 2: Manual Workflow Dispatch

Add a manual trigger to the workflow for testing:

```yaml
on:
  workflow_dispatch:  # Add this to allow manual runs
  workflow_run:
    workflows: ["CI"]
    # ... existing triggers
```

Then trigger manually:

```bash
gh workflow run release.yml
gh run watch
```

### Option 3: Test Individual Jobs

Test specific parts of the workflow by extracting the commands:

```bash
# Test Linux ARM64 cross-compilation setup
sudo apt-get update
sudo apt-get install -y gcc-aarch64-linux-gnu
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
cargo build --release --target aarch64-unknown-linux-gnu

# Test Windows checksum generation (on Windows or with PowerShell)
pwsh -Command "(Get-FileHash -Path ecs-voyager.exe -Algorithm SHA256).Hash.ToLower()"

# Test checksums file creation
shasum -a 256 *.tar.gz *.zip *.deb *.rpm > SHA256SUMS.txt
```

## Verifying a Release

After a release is created, verify:

### 1. GitHub Release Page

```bash
gh release view v0.2.7
```

Check that all these files are present:
- `ecs-voyager-v0.2.7-x86_64-apple-darwin.tar.gz`
- `ecs-voyager-v0.2.7-aarch64-apple-darwin.tar.gz`
- `ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz`
- `ecs-voyager-v0.2.7-aarch64-unknown-linux-gnu.tar.gz`
- `ecs-voyager-v0.2.7-x86_64-pc-windows-msvc.zip`
- `ecs-voyager_0.2.7_amd64.deb`
- `ecs-voyager-0.2.7-1.x86_64.rpm`
- `SHA256SUMS.txt`
- Individual `.sha256` files for each archive

### 2. Test Installation

**macOS (Homebrew)**:
```bash
brew tap benbpyle/ecs-voyager
brew install ecs-voyager
ecs-voyager --version
```

**Linux (Debian/Ubuntu)**:
```bash
wget https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager_0.2.7_amd64.deb
sudo dpkg -i ecs-voyager_0.2.7_amd64.deb
ecs-voyager --version
```

**Linux (RedHat/Fedora)**:
```bash
wget https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-0.2.7-1.x86_64.rpm
sudo dnf install ecs-voyager-0.2.7-1.x86_64.rpm
ecs-voyager --version
```

**Windows**:
```powershell
# Download from GitHub releases
Invoke-WebRequest -Uri "https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-x86_64-pc-windows-msvc.zip" -OutFile ecs-voyager.zip
Expand-Archive -Path ecs-voyager.zip -DestinationPath .
.\ecs-voyager.exe --version
```

### 3. Verify Checksums

```bash
# Download SHA256SUMS.txt
wget https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/SHA256SUMS.txt

# Download an artifact
wget https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz

# Verify checksum
shasum -a 256 -c SHA256SUMS.txt --ignore-missing
```

Expected output: `ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz: OK`

## Common Issues

### Issue: Linux ARM64 build fails

**Solution**: Ensure gcc-aarch64-linux-gnu is installed:
```bash
sudo apt-get install -y gcc-aarch64-linux-gnu
```

### Issue: Windows ZIP creation fails

**Solution**: Ensure PowerShell is available or use the `zip` command:
```bash
# Alternative with zip tool
zip ecs-voyager.zip ecs-voyager.exe
```

### Issue: .deb package fails to install

**Solution**: Check dependencies:
```bash
sudo apt-get install -f
```

### Issue: Workflow doesn't trigger

**Solution**: Check that:
1. CI workflow passed successfully
2. Commit message doesn't start with "chore: update Homebrew formula"
3. Tag doesn't already exist for this version

## Next Steps

After testing locally:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes
4. Push to master/main
5. CI runs automatically
6. Release workflow triggers after CI succeeds
7. Monitor with `gh run watch`

For manual releases, see `RELEASING.md`.
