# RClean Project Structure

**Status**: Active  
**Type**: Reference  
**Created**: 2025-01-16  
**Updated**: 2025-01-16  
**Author**: PAIML Team  

## Project Overview

RClean is a high-performance Rust-based disk cleanup tool with PMAT-certified quality standards.

## Directory Structure

```
rclean/
├── src/                        # Core source code
│   ├── lib.rs                  # Main library with deduplication logic
│   ├── main.rs                 # CLI application entry point
│   ├── clustering.rs           # DBSCAN clustering for similar files
│   ├── outliers.rs             # Storage outlier detection
│   └── mcp_server/             # Model Context Protocol server
│       ├── mod.rs              # MCP module exports
│       ├── server.rs           # MCP server implementation
│       ├── handlers.rs         # MCP request handlers
│       └── models.rs           # MCP data models
├── tests/                      # Comprehensive test suite
│   ├── cli.rs                  # CLI integration tests
│   ├── integration_tests.rs    # System integration tests
│   ├── property_tests.rs       # Property-based testing
│   ├── mcp_integration.rs      # MCP server tests
│   └── outliers_tests.rs       # Outlier detection tests
├── examples/                   # Usage examples and demos
│   ├── basic_usage.rs          # Simple API usage
│   ├── cluster_analysis.rs     # Clustering demonstration
│   ├── outliers_detection.rs   # Outlier detection demo
│   └── pattern_matching.rs     # Pattern matching examples
├── docs/                       # PMAT-style documentation
│   ├── README.md               # Documentation index
│   ├── architecture/           # Architecture documentation
│   ├── guides/                 # User guides
│   ├── testing/                # Testing documentation
│   └── api/                    # API reference
├── .github/workflows/          # GitHub Actions CI/CD
│   ├── ci.yml                  # Cross-platform testing
│   ├── quality-gate.yml        # PMAT quality gates
│   ├── security.yml            # Security auditing
│   └── release.yml             # Automated releases
├── .cargo/                     # Cargo configuration
│   ├── config.toml             # Build optimizations
│   └── audit.toml              # Security audit config
├── Cargo.toml                  # Project metadata and dependencies
├── Makefile                    # PMAT-style build automation
├── CLAUDE.md                   # AI assistant instructions
└── README.md                   # Project overview
```

## Core Components

### 1. Deduplication Engine (`lib.rs`)
- **FileInfo**: File metadata with MD5 hashing
- **walk()**: Filesystem traversal using walkdir/ignore
- **find()**: Pattern matching (literal, glob, regex)
- **dedupe()**: Parallel duplicate detection with Rayon
- **generate_statistics()**: Polars DataFrame analysis

### 2. Clustering Module (`clustering.rs`)
- **DBSCAN Algorithm**: Density-based clustering
- **Similarity Calculation**: SSDEEP fuzzy hashing
- **LargeFileCluster**: Cluster data structure
- **Performance Optimization**: Distance matrix caching

### 3. Outlier Detection (`outliers.rs`)
- **Statistical Analysis**: Z-score based outlier detection
- **Hidden Consumers**: Known space-wasting patterns
- **Pattern Groups**: File naming pattern detection
- **DataFrame Export**: Polars integration

### 4. MCP Server (`mcp_server/`)
- **Protocol Compliance**: Model Context Protocol v2024-11-05
- **Tool Interface**: dedupe, search, count, outliers, clustering
- **Async Handlers**: Tokio-based request processing
- **JSON-RPC**: Standard MCP communication

## Key Dependencies

### Core Libraries
- **rayon**: Parallel processing
- **polars**: DataFrame operations
- **walkdir/ignore**: Filesystem traversal
- **globset**: Pattern matching
- **md5**: Content hashing
- **ssdeep**: Fuzzy hashing

### Testing Framework
- **proptest**: Property-based testing
- **tempfile**: Test isolation
- **assert_cmd**: CLI testing
- **predicates**: Test assertions

### Quality Tools
- **clippy**: Linting
- **rustfmt**: Formatting
- **cargo-audit**: Security auditing
- **cargo-tarpaulin**: Coverage analysis

## Architecture Patterns

### 1. Error Handling
- Custom error types with thiserror
- Graceful degradation for file system errors
- Comprehensive error propagation

### 2. Performance Optimization
- Parallel processing with Rayon
- Efficient memory usage patterns
- LTO and stripping in release builds

### 3. Testing Strategy
- Multi-layer testing approach
- Property-based invariant testing
- Integration test isolation
- Comprehensive coverage

### 4. Quality Assurance
- Zero-tolerance linting
- Automated security auditing
- Continuous quality monitoring
- PMAT-certified standards

## Build System

The project uses a PMAT-style Makefile with comprehensive targets:
- Quality gates: `make quality-gate`
- Testing: `make test`, `make test-examples`
- Coverage: `make coverage`
- Security: `make security-audit`
- Release: `make build-release`

## CI/CD Pipeline

GitHub Actions workflows provide:
- Cross-platform testing (Linux, macOS, Windows)
- Quality gate enforcement
- Automated security scanning
- Binary releases and crates.io publishing