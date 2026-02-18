# Release Automation Guide

Automated release workflow for the Fax Compiler project using [release-plz](https://release-plz.ieni.dev/).

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Configuration Files](#configuration-files)
- [Workflow Triggers](#workflow-triggers)
- [Manual Release](#manual-release)
- [Automatic Release](#automatic-release)
- [Conventional Commits](#conventional-commits)
- [Publishing to crates.io](#publishing-to-cratesio)
- [Troubleshooting](#troubleshooting)

## Overview

This release automation system provides:

- ✅ **Automatic version bumping** based on conventional commits
- ✅ **Changelog generation** using git-cliff
- ✅ **GitHub release creation** with release notes
- ✅ **Multi-platform builds** (Linux, macOS, Windows)
- ✅ **Binary artifacts** uploaded to GitHub releases
- ✅ **Optional crates.io publishing**
- ✅ **Git tag creation**

## Quick Start

### Prerequisites

1. **GitHub Secrets** (required for publishing):
   - `CARGO_REGISTRY_TOKEN` - For publishing to crates.io (optional)

2. **Install release-plz** (for local testing):
   ```bash
   cargo install release-plz --locked
   ```

### First-Time Setup

1. **Add secrets to your repository**:
   - Go to `Settings > Secrets and variables > Actions`
   - Add `CARGO_REGISTRY_TOKEN` if publishing to crates.io

2. **Verify configuration**:
   ```bash
   cd faxc
   release-plz release-plan
   ```

## Configuration Files

### `/release-plz.toml`

Main configuration for release-plz:

```toml
[workspace]
# Enable GitHub releases
git_release_enable = true
git_tag_enable = true

# Changelog configuration
changelog_config = "cliff.toml"
changelog_update = true

# Version bumping (semver)
semver_check = true
```

### `/faxc/cliff.toml`

git-cliff configuration for changelog generation:

```toml
[git]
conventional_commits = true
filter_unconventional = true

[commit_parsers]
{ message = "^feat", group = "Features" }
{ message = "^fix", group = "Bug Fixes" }
```

### `/.github/workflows/release-automated.yml`

GitHub Actions workflow that:
- Detects pending releases
- Builds multi-platform binaries
- Creates GitHub releases
- Uploads release assets

## Workflow Triggers

The release workflow triggers on:

| Trigger | Description |
|---------|-------------|
| `workflow_dispatch` | Manual trigger via GitHub UI |
| `push` to `main` | Automatic detection of version changes |
| `schedule` | Weekly check (Mondays at 9:00 UTC) |

## Manual Release

### Via GitHub UI

1. Go to `Actions > Release - Automated`
2. Click **Run workflow**
3. Configure options:
   - **Dry run**: Test without creating release
   - **Publish crates**: Enable to publish to crates.io
   - **Release type**: Force specific version bump (auto-detect if empty)
4. Click **Run workflow**

### Via GitHub CLI

```bash
# Trigger release workflow
gh workflow run release-automated.yml

# With dry-run
gh workflow run release-automated.yml -f dry_run=true

# With crates.io publishing
gh workflow run release-automated.yml -f publish_crates=true
```

### Via API

```bash
curl -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/repos/fax-lang/faxc/actions/workflows/release-automated.yml/dispatches \
  -d '{"ref":"main","inputs":{"dry_run":"false","publish_crates":"false"}}'
```

## Automatic Release

The workflow automatically detects and creates releases when:

1. **Conventional commits** are pushed to `main`
2. **Version changes** are detected by release-plz
3. **No existing tag** for the detected version

### Automatic Flow

```
push to main
    ↓
detect changes (release-plz)
    ↓
build artifacts (all platforms)
    ↓
create GitHub release
    ↓
upload binary assets
    ↓
create git tag
```

## Conventional Commits

Version bumping follows [Semantic Versioning](https://semver.org/) based on commit types:

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat:` | Minor (0.x.0) | `feat: add pattern matching` |
| `fix:` | Patch (0.0.x) | `fix: resolve parser edge case` |
| `BREAKING CHANGE` | Major (x.0.0) | `feat: new AST format\n\nBREAKING CHANGE: ...` |
| `perf:` | Minor | `perf: optimize lexer` |
| `refactor:` | Patch | `refactor: simplify type inference` |
| `docs:` | Patch | `docs: update README` |
| `test:` | Patch | `test: add unit tests for parser` |
| `chore:` | Patch | `chore: update dependencies` |

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Examples

```bash
# Feature (minor version bump)
git commit -m "feat: add async/await support"

# Bug fix (patch version bump)
git commit -m "fix: handle empty input in lexer"

# Breaking change (major version bump)
git commit -m "feat: new type system

BREAKING CHANGE: removed implicit type coercion"

# With scope
git commit -m "feat(parser): add pattern matching"
```

## Publishing to crates.io

### Enable Publishing

1. **Add secret**:
   - Generate token at https://crates.io/settings/tokens
   - Add as `CARGO_REGISTRY_TOKEN` in GitHub Secrets

2. **Update `release-plz.toml`**:
   ```toml
   [workspace]
   publish = true
   ```

3. **Enable in workflow**:
   - Set `publish_crates: true` in manual trigger
   - Or modify workflow for automatic publishing

### Publish Specific Crates

Edit `release-plz.toml` to enable per-crate publishing:

```toml
[[package]]
name = "faxc-util"
publish = true

[[package]]
name = "faxc-lex"
publish = true
```

## Local Testing

### Check Release Plan

```bash
cd faxc
release-plz release-plan
```

### Dry Run Release

```bash
cd faxc
release-plz release --dry-run
```

### Generate Changelog

```bash
cd faxc
release-plz changelog
```

### Update Changelog

```bash
cd faxc
release-plz changelog --update
```

### Create Release PR

```bash
cd faxc
release-plz release-pr
```

## Release Artifacts

The workflow produces the following artifacts:

| Platform | Target | Archive |
|----------|--------|---------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `faxc-linux-x86_64.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `faxc-linux-aarch64.tar.gz` |
| macOS x86_64 | `x86_64-apple-darwin` | `faxc-macos-x86_64.tar.gz` |
| macOS ARM64 | `aarch64-apple-darwin` | `faxc-macos-aarch64.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `faxc-windows-x86_64.zip` |

Each archive contains:
- `faxc` or `faxc.exe` binary
- `README.md`
- `LICENSE`
- `CONTRIBUTING.md`
- `SHA256SUMS.txt`

## Troubleshooting

### Common Issues

#### "No releases to create"

**Cause**: No conventional commits since last release.

**Solution**: Ensure commits follow conventional commit format:
```bash
git log --oneline
# Should show commits like: feat: ..., fix: ..., etc.
```

#### "Cargo publish failed"

**Cause**: Missing or invalid `CARGO_REGISTRY_TOKEN`.

**Solution**:
1. Verify secret exists in GitHub Settings
2. Check token validity at crates.io
3. Ensure `publish = true` in release-plz.toml

#### "Build failed for specific target"

**Cause**: Missing dependencies for cross-compilation.

**Solution**: Check workflow logs for missing packages. Common fixes:
```bash
# Linux ARM64
sudo apt-get install gcc-aarch64-linux-gnu

# macOS
brew install llvm openssl
```

#### "Changelog not updating"

**Cause**: Missing cliff.toml or incorrect path.

**Solution**:
```bash
# Verify cliff.toml exists
ls faxc/cliff.toml

# Test changelog generation
release-plz changelog --update
```

### Debug Mode

Enable verbose logging in workflow:

```yaml
- name: Debug release-plz
  working-directory: faxc
  run: |
    release-plz release-plan --verbose
    release-plz release --dry-run --verbose
```

### Manual Recovery

If automatic release fails:

1. **Check release plan**:
   ```bash
   cd faxc
   release-plz release-plan
   ```

2. **Manually create tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **Manually create release**:
   - Go to GitHub Releases
   - Create new release with tag
   - Upload artifacts manually

## Version Management

### Force Version Bump

Use manual trigger with `release_type` input:

- `major` - Bump major version (1.0.0 → 2.0.0)
- `minor` - Bump minor version (1.0.0 → 1.1.0)
- `patch` - Bump patch version (1.0.0 → 1.0.1)
- `alpha` - Pre-release (1.0.0 → 1.0.1-alpha.1)
- `beta` - Pre-release (1.0.0 → 1.0.1-beta.1)
- `rc` - Release candidate (1.0.0 → 1.0.1-rc.1)

### Pre-releases

For pre-release versions, use conventional commits with pre-release markers:

```bash
git commit -m "feat: add new feature (alpha)"
```

Or force via workflow input.

## Security Considerations

- **Never commit secrets**: `CARGO_REGISTRY_TOKEN` must be in GitHub Secrets only
- **Use GITHUB_TOKEN**: Workflow uses automatic token for releases
- **Verify artifacts**: Check SHA256SUMS.txt for binary integrity
- **Review PRs**: Release PRs should be reviewed before merging

## References

- [release-plz Documentation](https://release-plz.ieni.dev/)
- [git-cliff Documentation](https://git-cliff.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
