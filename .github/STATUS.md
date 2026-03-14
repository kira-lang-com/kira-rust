# Build Status

This page shows the current status of all CI/CD workflows for the Kira project.

## Main Workflows

| Workflow | Status | Description |
|----------|--------|-------------|
| CI | [![CI](https://github.com/kira-lang-com/kira/workflows/CI/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/ci.yml) | Code quality checks and tests |
| Build | [![Build](https://github.com/kira-lang-com/kira/workflows/Build/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/build.yml) | Multi-platform binary builds |
| Release | [![Release](https://github.com/kira-lang-com/kira/workflows/Release/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/release.yml) | Automated releases |

## Workflow Triggers

### CI Workflow
- ✅ Runs on **every push** to **any branch**
- ✅ Runs on **every pull request** to **any branch**
- Includes: Format check, Clippy, Tests (Linux/macOS/Windows)

### Build Workflow
- ✅ Runs on **every push** to **any branch**
- ✅ Runs on **every pull request** to **any branch**
- ✅ Runs on **release creation**
- Builds for: Linux x86_64, macOS x86_64, macOS aarch64, Windows x86_64

### Release Workflow
- ✅ Runs on **version tag push** (e.g., `v0.1.0`)
- Creates GitHub release with binaries and checksums

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 | ✅ Supported |
| macOS | x86_64 (Intel) | ✅ Supported |
| macOS | aarch64 (Apple Silicon) | ✅ Supported |
| Windows | x86_64 | ✅ Supported |

## Viewing Build Results

### For Your Branch
1. Push your commits to any branch
2. Go to the [Actions tab](https://github.com/kira-lang-com/kira/actions)
3. Find your branch in the workflow runs
4. Click to see detailed logs

### For Pull Requests
- Build status appears automatically on your PR
- All checks must pass before merging
- Click "Details" next to any check to see logs

## Build Artifacts

After each successful build:
- Binaries are available as downloadable artifacts
- Artifacts are kept for 90 days
- Release builds are permanently attached to releases

## Troubleshooting

### My Build Failed
1. Check the Actions tab for error logs
2. Common issues:
   - Formatting: Run `cargo fmt`
   - Clippy warnings: Run `cargo clippy --fix`
   - Test failures: Run `cargo test` locally
   - Platform-specific: Check the specific platform logs

### Build Takes Too Long
- First builds take longer (no cache)
- Subsequent builds are faster (cached dependencies)
- Typical times:
  - CI checks: 5-10 minutes
  - Full builds: 15-25 minutes per platform

### Need Help?
- Check [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
- Check [WORKFLOWS.md](WORKFLOWS.md) for detailed workflow info
- Open an issue if you need assistance

## Recent Activity

View all recent workflow runs: [Actions Tab](https://github.com/kira-lang-com/kira/actions)
