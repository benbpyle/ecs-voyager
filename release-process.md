# Release Process

This document describes the GitFlow-based release process for ecs-voyager.

## GitFlow Overview

We use a simplified GitFlow model:

- **main** - Production-ready code, tagged with version numbers
- **develop** - Integration branch for features
- **release/vX.Y.Z** - Release preparation branches

## Prerequisites

1. **Git Flow installed** (optional but recommended):
   ```bash
   # macOS
   brew install git-flow

   # Ubuntu/Debian
   sudo apt-get install git-flow
   ```

2. **GitHub repository setup**:
   - Repository created at `github.com/benbpyle/ecs-voyager`
   - `main` and `develop` branches exist
   - GitHub Actions enabled

3. **Workflow enabled** (when ready):
   ```bash
   # Rename the disabled workflow
   mv .github/workflows/gitflow-release.yml.disabled .github/workflows/gitflow-release.yml
   git add .github/workflows/gitflow-release.yml
   git commit -m "Enable GitFlow release workflow"
   git push
   ```

## Release Steps

### 1. Start a Release (from develop)

```bash
# Ensure you're on develop and up to date
git checkout develop
git pull origin develop

# Start release (choose version number)
git flow release start v0.2.0

# Or manually:
git checkout -b release/v0.2.0 develop
```

### 2. Prepare the Release

Make any final changes:

```bash
# Update CHANGELOG.md
cat << 'EOF' >> CHANGELOG.md
## [0.2.0] - $(date +%Y-%m-%d)

### Added
- New feature X
- Enhancement Y

### Fixed
- Bug Z

EOF

# The workflow will automatically update Cargo.toml version
# But you can do it manually if needed:
sed -i '' 's/version = ".*"/version = "0.2.0"/' Cargo.toml

# Commit changes
git add .
git commit -m "chore: prepare release v0.2.0"
```

### 3. Push Release Branch

```bash
# Push to trigger the automated workflow
git push -u origin release/v0.2.0
```

### 4. Workflow Automation

The GitHub Actions workflow will automatically:

1. ✅ **Extract version** from branch name
2. ✅ **Update Cargo.toml** with version
3. ✅ **Run tests** (all tests, clippy, formatting, security audit)
4. ✅ **Build binaries** for all platforms:
   - macOS x86_64 (Intel)
   - macOS aarch64 (Apple Silicon)
   - Linux x86_64
   - Linux aarch64
   - Windows x86_64
5. ✅ **Create GitHub Release** with all binaries
6. ✅ **Update Homebrew formula** with new version and checksums
7. ✅ **Merge to main** and tag the release
8. ✅ **Merge back to develop**
9. ✅ **Delete release branch**

### 5. Verify Release

After workflow completes:

1. Check GitHub Releases page
2. Verify Homebrew formula is updated
3. Test installation:
   ```bash
   brew upgrade ecs-voyager
   ecs-voyager --version
   ```

## Manual Release (If Workflow Fails)

If you need to complete the release manually:

### Finish Release with Git Flow

```bash
# Finish the release
git flow release finish v0.2.0

# Or manually:
git checkout main
git merge --no-ff release/v0.2.0
git tag -a v0.2.0 -m "Release v0.2.0"
git checkout develop
git merge --no-ff main
git branch -d release/v0.2.0

# Push everything
git push origin main develop --tags
git push origin --delete release/v0.2.0
```

### Build and Upload Binaries

```bash
# Build for macOS Intel
cargo build --release --target x86_64-apple-darwin
strip target/x86_64-apple-darwin/release/ecs-voyager
tar czf ecs-voyager-v0.2.0-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release ecs-voyager

# Build for macOS ARM
cargo build --release --target aarch64-apple-darwin
strip target/aarch64-apple-darwin/release/ecs-voyager
tar czf ecs-voyager-v0.2.0-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release ecs-voyager

# Create GitHub release and upload assets
gh release create v0.2.0 \
  ecs-voyager-v0.2.0-x86_64-apple-darwin.tar.gz \
  ecs-voyager-v0.2.0-aarch64-apple-darwin.tar.gz \
  --title "Release v0.2.0" \
  --generate-notes
```

### Update Homebrew Formula

```bash
# Calculate checksums
X86_SHA=$(shasum -a 256 ecs-voyager-v0.2.0-x86_64-apple-darwin.tar.gz | awk '{print $1}')
ARM_SHA=$(shasum -a 256 ecs-voyager-v0.2.0-aarch64-apple-darwin.tar.gz | awk '{print $1}')

# Update formula
sed -i '' "s/version \".*\"/version \"0.2.0\"/" ecs-voyager.rb
sed -i '' "s|download/v[^/]*/|download/v0.2.0/|g" ecs-voyager.rb
sed -i '' "s/PLACEHOLDER_X86_64_SHA256/$X86_SHA/" ecs-voyager.rb
sed -i '' "s/PLACEHOLDER_ARM64_SHA256/$ARM_SHA/" ecs-voyager.rb

# Commit and push
git add ecs-voyager.rb
git commit -m "chore: update Homebrew formula to v0.2.0"
git push origin main develop
```

## Hotfix Process

For urgent fixes to production:

```bash
# Start hotfix from main
git flow hotfix start v0.1.1

# Or manually:
git checkout -b hotfix/v0.1.1 main

# Make fixes, update version, commit
sed -i '' 's/version = ".*"/version = "0.1.1"/' Cargo.toml
git add .
git commit -m "fix: critical bug fix"

# Push to trigger workflow (if enabled for hotfix branches)
git push -u origin hotfix/v0.1.1

# Or finish manually
git flow hotfix finish v0.1.1
git push origin main develop --tags
```

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (v1.0.0): Breaking changes
- **MINOR** (v0.2.0): New features, backward compatible
- **PATCH** (v0.1.1): Bug fixes, backward compatible

## Troubleshooting

### Workflow Fails at Build Step

Check the build logs in GitHub Actions. Common issues:
- Rust compilation errors
- Test failures
- Clippy warnings

Fix issues on the release branch and push again:
```bash
# Fix issues
git add .
git commit -m "fix: resolve build issues"
git push
```

### Workflow Fails at Homebrew Update

The formula might have manual changes. Update manually:
```bash
git checkout release/v0.2.0
# Update ecs-voyager.rb manually
git add ecs-voyager.rb
git commit -m "chore: update Homebrew formula"
git push
```

### Release Tag Already Exists

Delete the tag and recreate:
```bash
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0
gh release delete v0.2.0 --yes
```

## Post-Release Tasks

1. **Announce the release**:
   - Update documentation site (if any)
   - Post on social media
   - Update community channels

2. **Monitor issues**:
   - Watch for bug reports
   - Prepare hotfix if critical issues arise

3. **Plan next release**:
   - Create milestone for next version
   - Organize issues and PRs

## Workflow Status

- ⚠️ **Currently DISABLED** - Workflow file is `.github/workflows/gitflow-release.yml.disabled`
- To enable: Rename to `.github/workflows/gitflow-release.yml`

## Checklist

Before enabling the automated workflow:

- [ ] GitHub repository created and configured
- [ ] `main` and `develop` branches exist
- [ ] GitHub Actions enabled
- [ ] Homebrew tap repository created (optional)
- [ ] First manual release completed and tested
- [ ] Workflow file renamed to enable
