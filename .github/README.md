# Kira GitHub Documentation

Welcome to the Kira project's GitHub documentation! This directory contains all information about contributing, CI/CD, and project workflows.

## 📚 Documentation Index

### Getting Started
- **[QUICKSTART.md](QUICKSTART.md)** - Fast-track guide for new contributors (5 minutes)
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Detailed contribution guidelines and development process

### CI/CD & Workflows
- **[WORKFLOWS.md](WORKFLOWS.md)** - Complete CI/CD workflow documentation
- **[BRANCH_BUILDS.md](BRANCH_BUILDS.md)** - How builds work on every branch
- **[STATUS.md](STATUS.md)** - Current build status and troubleshooting

### Templates
- **[pull_request_template.md](pull_request_template.md)** - PR template
- **[ISSUE_TEMPLATE/bug_report.md](ISSUE_TEMPLATE/bug_report.md)** - Bug report template
- **[ISSUE_TEMPLATE/feature_request.md](ISSUE_TEMPLATE/feature_request.md)** - Feature request template

### Scripts
- **[scripts/test-build.sh](scripts/test-build.sh)** - Local testing script

## 🚀 Quick Links

### For New Contributors
1. Read [QUICKSTART.md](QUICKSTART.md)
2. Clone and build the project
3. Make your changes
4. Run [scripts/test-build.sh](scripts/test-build.sh)
5. Submit a PR

### For Reviewers
- Check [STATUS.md](STATUS.md) for build status
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for standards
- Use PR template for consistency

### For Maintainers
- See [WORKFLOWS.md](WORKFLOWS.md) for CI/CD details
- See [BRANCH_BUILDS.md](BRANCH_BUILDS.md) for build policy
- Release process documented in [WORKFLOWS.md](WORKFLOWS.md)

## 🔧 Workflows

### CI Workflow
- **Trigger:** Every push to any branch
- **Purpose:** Code quality checks
- **Duration:** ~5-10 minutes
- **Details:** [WORKFLOWS.md](WORKFLOWS.md#1-ci-workflow-ciyml)

### Build Workflow
- **Trigger:** Every push to any branch
- **Purpose:** Multi-platform builds
- **Duration:** ~15-25 minutes per platform
- **Details:** [WORKFLOWS.md](WORKFLOWS.md#2-build-workflow-buildyml)

### Release Workflow
- **Trigger:** Version tag push (e.g., `v0.1.0`)
- **Purpose:** Automated releases
- **Duration:** ~20-30 minutes
- **Details:** [WORKFLOWS.md](WORKFLOWS.md#3-release-workflow-releaseyml)

## 📊 Build Status

| Workflow | Status |
|----------|--------|
| CI | [![CI](https://github.com/kira-lang-com/kira/workflows/CI/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/ci.yml) |
| Build | [![Build](https://github.com/kira-lang-com/kira/workflows/Build/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/build.yml) |
| Release | [![Release](https://github.com/kira-lang-com/kira/workflows/Release/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/release.yml) |

See [STATUS.md](STATUS.md) for detailed status information.

## 🎯 Key Policies

### Every Commit Builds
- ✅ CI runs on every push to any branch
- ✅ Builds run on every push to any branch
- ✅ All organization members get full CI/CD
- 📖 Details: [BRANCH_BUILDS.md](BRANCH_BUILDS.md)

### Code Quality Standards
- ✅ Must pass `cargo fmt` check
- ✅ Must pass `cargo clippy` with no warnings
- ✅ Must pass all tests on all platforms
- 📖 Details: [CONTRIBUTING.md](CONTRIBUTING.md)

### Release Process
- ✅ Tag with version number (e.g., `v0.1.0`)
- ✅ Automatic release creation
- ✅ Binaries for all platforms
- ✅ SHA256 checksums included
- 📖 Details: [WORKFLOWS.md](WORKFLOWS.md#creating-a-release)

## 🛠️ Local Development

### Quick Test
```bash
.github/scripts/test-build.sh
```

### Manual Testing
```bash
cd toolchain
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --release
cargo build --release
```

## 📞 Getting Help

- **Questions?** Open a [Discussion](https://github.com/kira-lang-com/kira/discussions)
- **Bug?** Use the [Bug Report template](ISSUE_TEMPLATE/bug_report.md)
- **Feature idea?** Use the [Feature Request template](ISSUE_TEMPLATE/feature_request.md)
- **Contributing?** Read [CONTRIBUTING.md](CONTRIBUTING.md)

## 📝 Document Maintenance

These documents are maintained by the Kira core team. If you find errors or have suggestions:

1. Open an issue describing the problem
2. Or submit a PR with the fix
3. Tag with `documentation` label

## 🔗 External Links

- [Main Repository](https://github.com/kira-lang-com/kira)
- [Actions Tab](https://github.com/kira-lang-com/kira/actions)
- [Releases](https://github.com/kira-lang-com/kira/releases)
- [Issues](https://github.com/kira-lang-com/kira/issues)
- [Pull Requests](https://github.com/kira-lang-com/kira/pulls)

---

**Last Updated:** 2026-03-08  
**Maintained By:** Kira Core Team