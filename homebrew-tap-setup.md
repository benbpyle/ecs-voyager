# Homebrew Tap Setup Guide

This guide explains how to set up and maintain the Homebrew tap for ecs-voyager.

## Overview

There are two approaches to distributing via Homebrew:

1. **Simple Approach**: Keep the formula in the main repository (current setup)
2. **Tap Repository**: Create a separate `homebrew-ecs-voyager` repository (recommended for long-term)

## Current Setup (Simple Approach)

The formula `ecs-voyager.rb` is located in the main repository. Users can install directly:

```bash
brew install benbpyle/ecs-voyager/ecs-voyager
```

### Advantages
- Single repository to maintain
- Simpler workflow
- Formula updates automatically with releases

### Disadvantages
- Users must specify full path
- Not as discoverable

## Setting Up a Dedicated Tap (Recommended)

For a more professional setup, create a separate tap repository.

### Step 1: Create the Tap Repository

1. Create a new GitHub repository named `homebrew-ecs-voyager` (must start with "homebrew-")
2. The repository should be at: `https://github.com/benbpyle/homebrew-ecs-voyager`

### Step 2: Initialize the Tap Repository

```bash
# Clone the new repository
git clone https://github.com/benbpyle/homebrew-ecs-voyager.git
cd homebrew-ecs-voyager

# Copy the formula
cp /Users/benjamen/Development/github/ecs-voyager/ecs-voyager.rb ./ecs-voyager.rb

# Create README
cat > README.md << 'EOF'
# Homebrew Tap for ECS Voyager

Official Homebrew tap for [ecs-voyager](https://github.com/benbpyle/ecs-voyager).

## Installation

```bash
brew tap benbpyle/ecs-voyager
brew install ecs-voyager
```

## Upgrading

```bash
brew upgrade ecs-voyager
```

## About

ECS Voyager is a terminal user interface (TUI) for exploring and managing AWS ECS resources.

For more information, visit the [main repository](https://github.com/benbpyle/ecs-voyager).
EOF

# Commit and push
git add .
git commit -m "Initial tap setup with ecs-voyager formula"
git push origin main
```

### Step 3: Update GitHub Actions Workflow

Once you have a dedicated tap repository, update the workflow to push formula updates there instead:

```yaml
# Add this to .github/workflows/release.yml after build-and-release job

  update-homebrew-tap:
    name: Update Homebrew Tap
    needs: build-and-release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout tap repository
        uses: actions/checkout@v4
        with:
          repository: benbpyle/homebrew-ecs-voyager
          token: ${{ secrets.TAP_GITHUB_TOKEN }}
          path: homebrew-tap

      - name: Download release artifacts
        run: |
          VERSION="${{ github.ref_name }}"

          # Download both macOS archives
          curl -sL "https://github.com/benbpyle/ecs-voyager/releases/download/${VERSION}/ecs-voyager-${VERSION}-x86_64-apple-darwin.tar.gz" -o x86_64.tar.gz
          curl -sL "https://github.com/benbpyle/ecs-voyager/releases/download/${VERSION}/ecs-voyager-${VERSION}-aarch64-apple-darwin.tar.gz" -o aarch64.tar.gz

          # Calculate SHA256 checksums
          X86_64_SHA256=$(shasum -a 256 x86_64.tar.gz | awk '{print $1}')
          AARCH64_SHA256=$(shasum -a 256 aarch64.tar.gz | awk '{print $1}')

          echo "X86_64_SHA256=${X86_64_SHA256}" >> $GITHUB_ENV
          echo "AARCH64_SHA256=${AARCH64_SHA256}" >> $GITHUB_ENV
          echo "VERSION=${VERSION}" >> $GITHUB_ENV

      - name: Update formula in tap
        run: |
          cd homebrew-tap

          # Update version
          sed -i "s/version \".*\"/version \"${VERSION#v}\"/" ecs-voyager.rb

          # Update x86_64 SHA256
          sed -i "s/sha256 \".*\" # x86_64/sha256 \"${X86_64_SHA256}\" # x86_64/" ecs-voyager.rb

          # Update ARM64 SHA256
          sed -i "s/sha256 \".*\" # aarch64/sha256 \"${AARCH64_SHA256}\" # aarch64/" ecs-voyager.rb

          # Update URLs with new version
          sed -i "s|download/v[0-9]\+\.[0-9]\+\.[0-9]\+/|download/${VERSION}/|g" ecs-voyager.rb

      - name: Commit and push to tap
        run: |
          cd homebrew-tap
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add ecs-voyager.rb
          git commit -m "Update ecs-voyager to ${VERSION}"
          git push
```

**Note**: You'll need to create a GitHub Personal Access Token (PAT) with `repo` scope and add it as a secret named `TAP_GITHUB_TOKEN` in the main repository settings.

## Usage After Tap Setup

Once the tap repository is set up, users can install more easily:

```bash
# Add the tap (one time)
brew tap benbpyle/ecs-voyager

# Install
brew install ecs-voyager

# Upgrade
brew upgrade ecs-voyager

# Uninstall
brew uninstall ecs-voyager
```

## Testing the Formula

Before releasing, test the formula locally:

```bash
# Syntax check
brew audit --new-formula ecs-voyager.rb

# Test installation
brew install --build-from-source ./ecs-voyager.rb

# Test the binary
ecs-voyager --version

# Uninstall
brew uninstall ecs-voyager
```

## Release Process

1. **Update version** in `Cargo.toml`
2. **Commit changes**: `git commit -am "Bump version to vX.Y.Z"`
3. **Create and push tag**: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. **GitHub Actions** will automatically:
   - Build binaries for x86_64 and ARM64 macOS
   - Create a GitHub release
   - Upload archives and checksums
   - Update the Homebrew formula with new SHA256 checksums

5. **Verify release**:
   - Check GitHub Actions completed successfully
   - Verify release page has all artifacts
   - Test installation: `brew upgrade ecs-voyager` or `brew install benbpyle/ecs-voyager/ecs-voyager`

## Troubleshooting

### SHA256 Mismatch

If users report SHA256 mismatches:

```bash
# Download the release archive
curl -sL "https://github.com/benbpyle/ecs-voyager/releases/download/vX.Y.Z/ecs-voyager-vX.Y.Z-x86_64-apple-darwin.tar.gz" -o release.tar.gz

# Calculate the correct SHA256
shasum -a 256 release.tar.gz

# Update the formula with the correct checksum
```

### Formula Audit Failures

Run audit locally to debug:

```bash
brew audit --strict --online ecs-voyager.rb
```

Common issues:
- Incorrect URL format
- Missing license
- SHA256 mismatch
- Test block failures

### Binary Not Found After Install

Ensure the archive structure is correct:

```bash
tar -tzf ecs-voyager-vX.Y.Z-aarch64-apple-darwin.tar.gz
```

Should output:
```
ecs-voyager
```

The binary must be at the root of the archive, not in a subdirectory.

## Additional Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
