# Homebrew Setup for ecs-voyager

## Current Configuration

The Homebrew formula is stored in the main repository under `Formula/ecs-voyager.rb`. This allows users to tap the repository directly without needing a separate `homebrew-*` repository.

## For Users

### Installation

```bash
# Method 1: Tap and install (recommended)
brew tap benbpyle/ecs-voyager https://github.com/benbpyle/ecs-voyager
brew install ecs-voyager

# Method 2: Direct install (no tap needed)
brew install benbpyle/ecs-voyager/ecs-voyager
```

### Upgrade

```bash
brew upgrade ecs-voyager
```

### Uninstall

```bash
brew uninstall ecs-voyager
brew untap benbpyle/ecs-voyager
```

## For Maintainers

### Repository Structure

```
ecs-voyager/
├── Formula/
│   └── ecs-voyager.rb          # Homebrew formula
├── .github/
│   └── workflows/
│       ├── release.yml          # Auto-updates formula on tag push
│       └── gitflow-release.yml.disabled  # Complete GitFlow automation
└── ... (source code)
```

### How It Works

1. **On Release**: When a version tag (e.g., `v0.1.0`) is pushed:
   - GitHub Actions builds macOS binaries (x86_64 and ARM64)
   - Creates a GitHub release with the binaries
   - Downloads the binaries and calculates SHA256 checksums
   - Updates `Formula/ecs-voyager.rb` with new version and checksums
   - Commits the updated formula back to the repository

2. **Users Install**: When users run `brew install`:
   - Homebrew downloads the formula from `Formula/ecs-voyager.rb`
   - Downloads the appropriate binary for their architecture
   - Verifies the checksum
   - Installs to `/usr/local/bin` or `/opt/homebrew/bin`

### Testing the Formula Locally

```bash
# Install from local formula
brew install --build-from-source Formula/ecs-voyager.rb

# Test the formula
brew test ecs-voyager

# Audit the formula
brew audit --strict ecs-voyager
```

### Updating the Formula Manually

If you need to update manually (outside of the automated workflow):

```bash
# 1. Update version
sed -i '' 's/version ".*"/version "0.2.0"/' Formula/ecs-voyager.rb

# 2. Update URLs
sed -i '' 's|download/v[^/]*/|download/v0.2.0/|g' Formula/ecs-voyager.rb

# 3. Calculate and update checksums
X86_SHA=$(curl -sL https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.0/ecs-voyager-v0.2.0-x86_64-apple-darwin.tar.gz | shasum -a 256 | awk '{print $1}')
ARM_SHA=$(curl -sL https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.0/ecs-voyager-v0.2.0-aarch64-apple-darwin.tar.gz | shasum -a 256 | awk '{print $1}')

sed -i '' "s/PLACEHOLDER_X86_64_SHA256/$X86_SHA/" Formula/ecs-voyager.rb
sed -i '' "s/PLACEHOLDER_ARM64_SHA256/$ARM_SHA/" Formula/ecs-voyager.rb

# 4. Commit
git add Formula/ecs-voyager.rb
git commit -m "chore: update Homebrew formula to v0.2.0"
git push
```

### Troubleshooting

#### Error: "Repository not found"

If users see:
```
Error: Failure while executing; `git clone https://github.com/benbpyle/homebrew-ecs-voyager ...`
```

This means they used `brew tap benbpyle/ecs-voyager` without the URL. The correct command is:
```bash
brew tap benbpyle/ecs-voyager https://github.com/benbpyle/ecs-voyager
```

#### Checksum Mismatch

If checksums don't match:
1. Download the binary manually
2. Calculate: `shasum -a 256 <binary>.tar.gz`
3. Update `Formula/ecs-voyager.rb` with correct checksum
4. Commit and push

#### Formula Not Found After Tap

Check that:
- Formula is in `Formula/ecs-voyager.rb` (capital F)
- Formula class name matches filename: `class EcsVoyager < Formula`
- Repository is public
- Formula is pushed to main branch

## Alternative: Dedicated Homebrew Tap

If you want a cleaner URL (`brew tap benbpyle/ecs-voyager` without the URL), create a separate repository:

### Setup Steps

1. **Create Repository**: `homebrew-ecs-voyager`
   - Repository must be named `homebrew-*` for Homebrew to recognize it

2. **Move Formula**:
   ```bash
   # In homebrew-ecs-voyager repo
   mkdir Formula
   # Copy ecs-voyager.rb to Formula/
   ```

3. **Update Workflow**: Configure GitHub Actions to push formula to the tap repo

4. **Users Install**:
   ```bash
   brew tap benbpyle/ecs-voyager  # No URL needed!
   brew install ecs-voyager
   ```

### Pros and Cons

**Current Setup (Formula in main repo)**:
- ✅ Simple - one repository
- ✅ Formula updates with releases
- ✅ Easy to maintain
- ❌ Requires full URL when tapping

**Dedicated Tap (separate homebrew-* repo)**:
- ✅ Cleaner tap command
- ✅ Can host multiple formulas
- ✅ Professional appearance
- ❌ Two repositories to maintain
- ❌ More complex workflow

## Current Status

- ✅ Formula created in `Formula/ecs-voyager.rb`
- ✅ GitHub Actions workflow configured
- ✅ Supports both Intel and Apple Silicon Macs
- ✅ Auto-updates on release
- ✅ Version flag added to CLI (`--version`)
- ⏳ Waiting for first GitHub release

## Next Steps

1. Push your repository to GitHub
2. Create first release: `git tag v0.1.0 && git push --tags`
3. GitHub Actions will build binaries and update formula
4. Users can install with Homebrew!
