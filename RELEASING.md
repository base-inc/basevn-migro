# Release Process

This document describes the process for creating a new release of basevn-migro.

## Prerequisites

- Write access to the repository
- All tests passing on main branch
- Updated CHANGELOG.md

## Release Steps

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
name = "basevn-migro"
version = "0.1.0"  # Update this
```

### 2. Update CHANGELOG

Move entries from `[Unreleased]` to a new version section in `CHANGELOG.md`:

```markdown
## [0.2.0] - 2024-12-27

### Added
- New feature 1
- New feature 2

### Changed
- Change 1

### Fixed
- Bug fix 1
```

Update the links at the bottom:

```markdown
[Unreleased]: https://github.com/basevn/basevn-migro/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/basevn/basevn-migro/releases/tag/v0.2.0
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

### 4. Create and Push Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag to trigger release workflow
git push origin v0.2.0
```

### 5. Monitor Release

The GitHub Actions workflow will automatically:
1. Create a GitHub Release
2. Build binaries for Linux, macOS (Intel + ARM), and Windows
3. Run tests on all platforms
4. Upload binaries as release assets
5. Generate SHA256 checksums

Monitor the workflow at: https://github.com/basevn/basevn-migro/actions

### 6. Verify Release

1. Go to https://github.com/basevn/basevn-migro/releases
2. Verify the release was created
3. Download and test binaries from each platform
4. Verify checksums match

### 7. Publish Announcement (Optional)

- Update README.md badges with latest version
- Announce on social media, blog, etc.
- Update Base.vn documentation

## Hotfix Releases

For critical bug fixes:

1. Create a hotfix branch from the tag:
   ```bash
   git checkout -b hotfix/0.1.1 v0.1.0
   ```

2. Fix the bug and commit

3. Follow steps 1-6 above with the hotfix version (e.g., v0.1.1)

4. Merge hotfix back to main:
   ```bash
   git checkout main
   git merge hotfix/0.1.1
   git push origin main
   ```

## Troubleshooting

### Build fails on macOS

- Ensure you have the latest Xcode Command Line Tools
- Check that the target is added: `rustup target add <target>`

### Build fails on Windows

- Ensure Visual Studio Build Tools are installed
- Check that Windows SDK is available

### Release workflow doesn't trigger

- Verify the tag matches the pattern `v*`
- Check GitHub Actions permissions
- Ensure GITHUB_TOKEN has required permissions

## Rollback

If a release has critical issues:

1. Delete the GitHub release (do not delete the tag yet)
2. Fix the issue on main
3. Create a new patch release (e.g., v0.1.1)
4. Optionally delete the problematic tag:
   ```bash
   git tag -d v0.1.0
   git push origin :refs/tags/v0.1.0
   ```
