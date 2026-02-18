# Release Checklist for Fax Compiler

This document outlines the steps required to release a new version of the Fax Compiler.

## Pre-Release Checklist

### Code Quality
- [ ] All tests passing (`./faxc/scripts/test.sh --release`)
- [ ] Code coverage above 80% (`./faxc/scripts/test.sh --coverage`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all -- --check`)
- [ ] MSRV verification passed (`./faxc/scripts/verify-msrv.sh`)

### Documentation
- [ ] CHANGELOG.md updated with all changes
- [ ] README.md version badge updated
- [ ] API documentation generated (`cargo doc --workspace`)
- [ ] Examples tested and working
- [ ] Release notes drafted

### Version Bump
- [ ] Version bumped in `faxc/Cargo.toml`
- [ ] Version bumped in `faxc/crates/*/Cargo.toml`
- [ ] Git tag created following SemVer
- [ ] Version references updated

### CI/CD
- [ ] All GitHub Actions workflows passing
- [ ] Docker build successful
- [ ] Cross-platform builds verified (Linux, macOS, Windows)
- [ ] Security scan passed (no vulnerabilities)

## Release Steps

### 1. Create Release Branch
```bash
git checkout -b release/v0.0.2
```

### 2. Update Version Numbers
```bash
# Update in all Cargo.toml files
# Use release-plz or manually update
release-plz release-plan
```

### 3. Update Changelog
```bash
# Generate changelog
git cliff --output CHANGELOG.md
```

### 4. Run Final Tests
```bash
./faxc/scripts/check.sh
./faxc/scripts/test.sh --release
```

### 5. Commit and Push
```bash
git commit -m "chore: release v0.0.2"
git push origin release/v0.0.2
```

### 6. Create Pull Request
- Create PR from `release/v0.0.2` to `main`
- Get approval from maintainers
- Merge PR

### 7. Tag Release
```bash
git tag -a v0.0.2 -m "Fax Compiler v0.0.2"
git push origin v0.0.2
```

## Post-Release

### GitHub Release
- [ ] GitHub release published with release notes
- [ ] Release artifacts uploaded (binaries for all platforms)
- [ ] Release marked as appropriate (pre-release/latest)

### Announcement
- [ ] Release announced on social media
- [ ] Release announced in Discord/community
- [ ] Blog post published (for major releases)

### Documentation
- [ ] Documentation website updated
- [ ] API docs deployed
- [ ] Examples updated

### Cleanup
- [ ] Release branch deleted
- [ ] Milestone closed on GitHub
- [ ] Next version planning started

## Emergency Rollback

If a critical issue is discovered after release:

1. **Assess severity**: Is rollback necessary?
2. **Create hotfix branch**: `git checkout -b hotfix/v0.0.2-1`
3. **Fix the issue**
4. **Test thoroughly**
5. **Release patch version**: `v0.0.2-1`
6. **Document the issue** in release notes

## Release Template

Use this template for release notes:

```markdown
## Fax Compiler v{VERSION}

**Release Date:** {DATE}

### 🎉 Highlights

- Major feature 1
- Major feature 2

### 🐛 Bug Fixes

- Fixed issue #123
- Fixed issue #456

### ⚡ Performance Improvements

- Improvement 1
- Improvement 2

### 📚 Documentation

- Updated guide for X
- Added examples for Y

### 🔄 Breaking Changes

If applicable, list breaking changes here with migration guide.

### 🙏 Thanks

Thanks to all contributors: @contributor1, @contributor2
```

---

**Last Updated:** 2026-02-18  
**Version:** 0.0.1 pre-alpha
