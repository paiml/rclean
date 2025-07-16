[![Quality Gate](https://github.com/paiml/rclean/actions/workflows/quality-gate.yml/badge.svg)](https://github.com/paiml/rclean/actions/workflows/quality-gate.yml)
[![CI](https://github.com/paiml/rclean/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/rclean/actions/workflows/ci.yml)
[![Security Audit](https://github.com/paiml/rclean/actions/workflows/security.yml/badge.svg)](https://github.com/paiml/rclean/actions/workflows/security.yml)
[![MCP Server](https://img.shields.io/badge/MCP-Server-blue.svg)](https://modelcontextprotocol.io/)
[![PMAT Certified](https://img.shields.io/badge/PMAT-Certified-green.svg)](https://github.com/paiml/rclean/actions/workflows/quality-gate.yml)
[![Crates.io](https://img.shields.io/crates/v/rclean.svg)](https://crates.io/crates/rclean)
[![Downloads](https://img.shields.io/crates/d/rclean.svg)](https://crates.io/crates/rclean)
[![License](https://img.shields.io/crates/l/rclean.svg)](LICENSE)
[![Coverage](https://img.shields.io/badge/Coverage-80%25+-brightgreen.svg)](#testing)
[![TDG](https://img.shields.io/badge/TDG-%E2%89%A4%201.0-green.svg)](#quality-standards)

## 🎓 Pragmatic AI Labs | Join 1M+ ML Engineers

### 🔥 Hot Course Offers:
* 🤖 [Master GenAI Engineering](https://ds500.paiml.com/learn/course/0bbb5/) - Build Production AI Systems
* 🦀 [Learn Professional Rust](https://ds500.paiml.com/learn/course/g6u1k/) - Industry-Grade Development
* 📊 [AWS AI & Analytics](https://ds500.paiml.com/learn/course/31si1/) - Scale Your ML in Cloud
* ⚡ [Production GenAI on AWS](https://ds500.paiml.com/learn/course/ehks1/) - Deploy at Enterprise Scale
* 🛠️ [Rust DevOps Mastery](https://ds500.paiml.com/learn/course/ex8eu/) - Automate Everything

### 🚀 Level Up Your Career:
* 💼 [Production ML Program](https://paiml.com) - Complete MLOps & Cloud Mastery
* 🎯 [Start Learning Now](https://ds500.paiml.com) - Fast-Track Your ML Career
* 🏢 Trusted by Fortune 500 Teams

Learn end-to-end ML engineering from industry veterans at [PAIML.COM](https://paiml.com)


## RClean

A high-performance Rust-based disk cleanup tool that finds duplicate files and storage outliers.

### Features

* **Duplicate Detection**: Find duplicate files using MD5 hashing with parallel processing
* **Similar File Detection**: Identify similar files using fuzzy matching algorithms
* **Storage Outliers**: Detect large files, hidden space consumers, and file patterns
* **Cluster Analysis**: Find groups of similar large files using DBSCAN clustering
* **Fast Performance**: Leverages Rust's parallelization with Rayon
* **Multiple Output Formats**: Table, JSON, CSV reports
* **MCP Support**: Can be used as an MCP (Model Context Protocol) server

![hpc-threaded-data-engineering](https://user-images.githubusercontent.com/58792/215359439-243cf62a-e8b1-41fd-b83e-697d7e612657.png)

## Quality Standards

**PMAT-Certified Quality Gates** - Following Production Manufacturing and Assembly Technology principles:

| Metric | Target | Status | Verification |
|--------|--------|--------|--------------|
| Technical Debt Gauge (TDG) | ≤ 1.0 | ✅ | `make quality-gate` |
| Cyclomatic Complexity | ≤ 20 | ✅ | `make lint` |
| SATD Comments | 0 | ✅ | `make lint` |
| Test Coverage | ≥ 80% | ✅ | `make coverage` |
| Lint Violations | 0 | ✅ | `make lint` |
| Security Vulnerabilities | 0 | ✅ | `make security-audit` |
| Documentation Coverage | 100% | ✅ | `make test-doc` |

### Testing Strategy
* 🧪 **126+ Total Tests**: Comprehensive multi-layer testing approach
* 🔗 **Integration Tests**: End-to-end workflow validation
* 🎲 **Property Tests**: Mathematical invariant verification (proptest)
* 📚 **Documentation Tests**: 6+ executable examples in docs
* 📋 **Example Tests**: Real-world usage demonstrations
* 🚀 **Performance Tests**: Efficiency and scalability validation

### Quality Automation
* 🔄 **Continuous Integration**: Cross-platform testing (Linux, macOS, Windows)
* 🛡️ **Security Scanning**: Automated vulnerability detection
* 📊 **Coverage Monitoring**: Comprehensive test coverage analysis
* 🔍 **Code Quality**: Zero-tolerance linting with clippy
* 🏗️ **Release Pipeline**: Automated binary builds and publishing
* ✅ **Documentation**: All public APIs documented with examples
* ✅ **Coverage**: High test coverage with comprehensive edge case testing

## Installation

### Requirements
- **Rust**: 1.70+ (MSRV - Minimum Supported Rust Version)
- **Platform**: Linux, macOS, Windows (x86_64)
- **Memory**: 512MB+ RAM recommended for large datasets

### Method 1: From crates.io (Recommended)
```bash
# Install latest stable release
cargo install rclean

# Verify installation
rclean --version
```

### Method 2: From GitHub Releases
Download pre-built binaries from [Releases](https://github.com/paiml/rclean/releases):

```bash
# Linux/macOS
curl -L https://github.com/paiml/rclean/releases/latest/download/rclean-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv rclean /usr/local/bin/

# Or using wget
wget https://github.com/paiml/rclean/releases/latest/download/rclean-x86_64-unknown-linux-gnu.tar.gz
tar xf rclean-x86_64-unknown-linux-gnu.tar.gz
sudo mv rclean /usr/local/bin/
```

### Method 3: From Source (Development)
```bash
# Clone repository
git clone https://github.com/paiml/rclean.git
cd rclean

# Build and install (debug)
cargo install --path .

# Or build release version
make build-release
sudo cp target/release/rclean /usr/local/bin/
```

### Method 4: Development Setup
```bash
# Clone and setup development environment
git clone https://github.com/paiml/rclean.git
cd rclean

# Install development dependencies
rustup component add rustfmt clippy
cargo install cargo-audit cargo-tarpaulin

# Verify development setup
make quality-gate
```

### Verification
```bash
# Check installation
rclean --version
# Expected: rclean 0.1.2

# Run basic test
rclean --help

# Test with current directory
rclean
```

### Quick Start

```bash
# Scan current directory for duplicates
rclean

# Scan specific directory
rclean /path/to/directory

# Filter by pattern
rclean ~/Documents --pattern "*.pdf" --pattern-type glob

# Generate CSV report
rclean . --csv duplicate_report.csv

# Find similar files (fuzzy matching) with 70% similarity threshold
rclean ~/Documents --similarity 70
```

### Storage Outliers Detection (NEW!)

Find files that are consuming disproportionate disk space:

```bash
# Find large file outliers
rclean outliers /path --min-size 100MB

# Find hidden space consumers (node_modules, .git, etc.)
rclean outliers ~ --check-hidden --format json

# Find file patterns (backups, logs, etc.)
rclean outliers . --check-patterns

# Export outliers report
rclean outliers . --csv outliers_report.csv

# Combine all features
rclean outliers ~ --min-size 50MB --check-hidden --check-patterns --top 50

# Enable clustering to find groups of similar large files
rclean outliers /path --cluster --cluster-similarity 80 --min-cluster-size 3
```

**Outliers Detection Features:**
- **Statistical Analysis**: Files that are X standard deviations larger than the mean
- **Hidden Consumers**: Detects node_modules, .git, .cache, and other known space hogs
- **Pattern Detection**: Finds groups of similar files (backup-001, backup-002, etc.)
- **Cluster Analysis**: Uses DBSCAN to find clusters of similar large files (e.g., different versions of the same document)
- **Smart Recommendations**: Provides cleanup suggestions for each type of outlier

### Fuzzy Matching (Similarity Detection)

Find files that are similar but not identical:

```bash
# Find files with 70% or higher similarity
rclean ~/Documents --similarity 70

# Find similar Python files
rclean ~/code --pattern "*.py" --pattern-type glob --similarity 80

# Generate CSV report including similar files
rclean . --similarity 60 --csv similarity_report.csv
```

**Use Cases:**
- Different versions of documents (v1, v2, draft, final)
- Slightly modified code files
- Images with minor edits
- Reports with small updates

### Advanced Pattern Matching

RClean supports ripgrep-style pattern matching:

#### Pattern Types

* **Literal** (default): Simple string contains matching
  ```bash
  rclean search --path . --pattern ".txt"
  ```

* **Glob**: Shell-style patterns
  ```bash
  rclean search --path . --pattern "*.txt" --pattern-type glob
  rclean search --path . --pattern "**/*.rs" --pattern-type glob
  ```

* **Regex**: Full regular expression support
  ```bash
  rclean search --path . --pattern "test_.*\.rs$" --pattern-type regex
  ```

#### Additional Options

* `--hidden`: Include hidden files
* `--no-ignore`: Ignore .gitignore rules
* `--max-depth <N>`: Maximum directory depth to traverse

### MCP Server Mode

RClean can run as an MCP server for integration with AI assistants:

```bash
# Run as MCP server
rclean  # Will auto-detect MCP mode when piped
```

### Building and Development

#### Quality Standards ✅
All lint checks now pass! The project follows PMAT (Production Manufacturing and Assembly Technology) quality standards with zero tolerance for warnings.

```bash
# Build and test
make all

# Development commands
make format        # Format code
make lint          # Run clippy linting (FIXED - passes cleanly!)
make lint-extreme  # Run extreme linting with PMAT standards
make test          # Run all tests
make test-examples # Run example tests (NEW!)

# Build variants
make build-release # Release build for production

# Quality assurance
make quality-gate  # Run all quality checks
make format-check  # Verify formatting
```

#### Recent Improvements (v0.1.1)
- ✅ **Fixed all clippy warnings** - `make lint` now passes without errors
- ✅ **Added example test coverage** - New `make test-examples` target
- ✅ **Improved code quality** - Reduced function complexity with better abstractions
- ✅ **Enhanced CI/CD readiness** - All quality gates pass consistently

### OS X Install

* Install rust via [rustup](https://rustup.rs/)
* Add to `~/.cargo/config`

```bash
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```
* run `make all` in rclean directory

### License

MIT