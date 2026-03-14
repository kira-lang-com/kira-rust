# Branch Build Policy

## Overview

Every commit to any branch in the Kira repository triggers automated CI/CD workflows. This ensures code quality and cross-platform compatibility for all organization members' work.

## What Runs on Every Commit

### 1. CI Workflow (Fast - ~5-10 minutes)

Runs on every push to any branch (main, develop, feature branches, personal branches, etc.):

```yaml
on:
  push:
    branches: [ '**' ]  # All branches including main
```

**Checks:**
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Tests on Linux, macOS, and Windows

**Purpose:** Catch issues early before code review

### 2. Build Workflow (Slower - ~15-25 minutes per platform)

Runs on every push to any branch:

```yaml
on:
  push:
    branches: [ '**' ]  # All branches
```

**Builds:**
- ✅ Linux x86_64 binary
- ✅ macOS x86_64 binary (Intel)
- ✅ macOS aarch64 binary (Apple Silicon)
- ✅ Windows x86_64 binary

**Purpose:** Ensure code compiles on all target platforms

## Benefits

### For Developers
- **Immediate feedback** on code quality
- **Platform issues** caught before merge
- **Confidence** that code works everywhere
- **No surprises** during code review

### For Reviewers
- **Pre-validated** code quality
- **Build status** visible on PRs
- **Platform compatibility** verified
- **Focus on logic** not formatting

### For the Project
- **Consistent quality** across all branches
- **Early detection** of breaking changes
- **Cross-platform** reliability
- **Professional** development workflow

## Workflow Behavior

### Feature Branches
```bash
git checkout -b feature/my-feature
git commit -m "Add feature"
git push origin feature/my-feature
```
→ CI and Build workflows run automatically

### Personal Branches
```bash
git checkout -b username/experiment
git commit -m "Try something"
git push origin username/experiment
```
→ CI and Build workflows run automatically

### Any Branch
All branches are treated equally - every commit gets full CI/CD treatment.

## Viewing Your Build Status

### In GitHub UI
1. Go to [Actions tab](https://github.com/kira-lang-com/kira/actions)
2. Filter by your branch name
3. Click on any workflow run to see details

### On Pull Requests
- Status checks appear automatically
- Green checkmark = all passed
- Red X = something failed
- Click "Details" to see logs

### Via Badges
Branch-specific badges available:
```markdown
![CI](https://github.com/kira-lang-com/kira/workflows/CI/badge.svg?branch=your-branch)
```

## Performance Considerations

### Caching
- Dependencies are cached between runs
- First build: ~20-30 minutes
- Subsequent builds: ~5-15 minutes

### Parallel Execution
- All platform builds run in parallel
- CI checks run in parallel
- Total time = slowest job

### Resource Usage
- GitHub Actions provides generous free tier
- Private repos: 2,000 minutes/month
- Public repos: Unlimited

## Best Practices

### Before Pushing
Run locally to catch issues early:
```bash
cargo fmt --all
cargo clippy --all-features
cargo test
```

Or use the test script:
```bash
.github/scripts/test-build.sh
```

### During Development
- Push frequently to get feedback
- Don't wait for "perfect" code
- Fix issues as they're reported
- Learn from CI failures

### For Large Changes
- Break into smaller commits
- Each commit should pass CI
- Easier to identify issues
- Better git history

## Troubleshooting

### Build Failed on My Branch
1. Check the Actions tab for your branch
2. Click on the failed workflow
3. Expand the failed step
4. Read the error message
5. Fix locally and push again

### Too Many Builds Running
- GitHub queues builds automatically
- Older builds can be cancelled
- Latest commit is most important

### Need to Skip CI
Not recommended, but possible:
```bash
git commit -m "docs: update README [skip ci]"
```

Use sparingly - only for documentation-only changes.

## Questions?

- See [WORKFLOWS.md](WORKFLOWS.md) for technical details
- See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
- See [STATUS.md](STATUS.md) for current build status
- Open an issue for help

## Summary

✅ Every commit to any branch triggers CI/CD
✅ Ensures quality across all development work
✅ Provides immediate feedback to developers
✅ Maintains high standards for the project

This policy helps maintain Kira's quality and reliability while supporting rapid development by all organization members.
