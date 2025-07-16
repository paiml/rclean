# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## RDedupe - Rust Deduplication Tool

A high-performance, multi-threaded file deduplication tool built with Rust, leveraging parallel processing for efficient duplicate detection across large filesystems.

## The Toyota Way: Our Guiding Philosophy

Following PMAT principles:
- **Kaizen (改善)**: Continuous incremental improvement - enhance one component at a time
- **Genchi Genbutsu (現地現物)**: Use profiling and benchmarks to identify actual bottlenecks
- **Jidoka (自働化)**: Automate with quality checks - ensure all changes pass CI/CD

## Absolute Rules

1. **NEVER Leave Stub Implementations**: All features must be fully functional. No "TODO" or placeholder code.
2. **NEVER Add SATD Comments**: Zero tolerance for self-admitted technical debt (TODO, FIXME, etc.)
3. **ALWAYS Run From Project Root**: All commands must be executed from `/home/noah/src/rdedupe`
4. **ALWAYS Use Make Commands**: The Makefile is the single source of truth for all operations
5. **Binary Location**: The release binary is at `./target/release/rdedupe`

## Build and Development Commands

```bash
# Development workflow
make format        # Format all code with rustfmt
make lint          # Run clippy linting (FIXED - now passes without errors)
make lint-extreme  # Run EXTREME linting (PMAT style)
make test          # Run all tests
make test-examples # Run example tests (NEW)
make run           # Run with example: dedupe --path tests --pattern .txt
make all           # Run complete workflow: format, lint, test, run, build-release

# Build and installation
cargo build                # Debug build
make build-release         # Release build for production
make install               # Install using cargo install
make install-local         # Install to ~/.local/bin (for CI/CD)

# Coverage analysis
make coverage              # Generate HTML coverage report
make coverage-stdout       # Show coverage summary in terminal
make coverage-report       # Display existing coverage report

# Quality checks
make quality-gate          # Run all quality checks
make format-check          # Check formatting without changes

# Running the tool (dedupe is the default command)
rdedupe                    # Scan current directory
rdedupe /path/to/dir      # Scan specific directory
rdedupe ~/Documents --pattern "*.pdf" --pattern-type glob
rdedupe search . --pattern "*.rs" --pattern-type glob
rdedupe count ~/Downloads --pattern "*.pdf" --pattern-type glob
```

## Testing Strategy

RDedupe employs a comprehensive testing approach with multiple layers of testing to ensure code quality and reliability:

### Test Types

1. **Unit Tests** (in lib.rs)
   - Tests for individual functions
   - Located alongside the code in the same file
   - Run with `cargo test --lib`

2. **Documentation Tests** (doc-tests)
   - Examples in function documentation that are executable tests
   - Ensures documentation stays accurate and up-to-date
   - Run with `make test-doc` or `cargo test --doc`
   - Examples:
     - `display_thread_info()` - verifies output format
     - `find()` - demonstrates pattern matching behavior
     - `find_advanced()` - shows glob and regex patterns
     - `calculate_similarity()` - tests fuzzy hash comparison

3. **Property-Based Tests** (tests/property_tests.rs)
   - Uses `proptest` for generative testing
   - Tests invariants that should hold for all inputs
   - Run with `make test-property`
   - Key properties tested:
     - `find()` always returns a subset of input files
     - Empty patterns return all files
     - Pattern matching is consistent across types
     - File size conversions are accurate
     - Similarity scores are bounded (0-100)

4. **Integration Tests** (tests/cli.rs)
   - Tests the CLI interface end-to-end
   - Verifies command-line argument parsing
   - Run with `cargo test --test cli`

### Running Tests

```bash
# Run all tests
make test
cargo test

# Run specific test categories
make test-doc        # Documentation tests only
make test-property   # Property-based tests only
make test-examples   # Example tests only (NEW)
cargo test --lib     # Unit tests only
cargo test --test cli # Integration tests only

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel with more threads
cargo test -- --test-threads=8
```

### Known Issues and Workarounds

1. **SSDEEP Library Crashes**: The ssdeep library has bugs that cause crashes with certain inputs during property testing. We work around this by using pre-generated known-good hashes in the `similarity_score_bounds_safe` test instead of generating hashes from random content.

### Test Coverage

```bash
# Generate HTML coverage report
make coverage

# Show coverage summary in terminal
make coverage-stdout

# View existing coverage report
make coverage-report
```

### Best Practices

1. **Doc-tests**: Add examples to all public functions that demonstrate typical usage
2. **Property Tests**: Focus on invariants and edge cases that are hard to catch with example-based tests
3. **Integration Tests**: Test complete workflows and CLI interactions
4. **Coverage**: Aim for high coverage but focus on meaningful tests over metrics

## Quality Standards

Following PMAT's extreme quality standards:
- **Complexity**: Keep cyclomatic complexity ≤20 per function
- **Coverage**: Maintain comprehensive test coverage
- **Linting**: Zero clippy warnings (enforced in CI)
- **Formatting**: Consistent rustfmt style (enforced in CI)

## Recent Quality Improvements (v0.1.1)

### Linting Fixes
All clippy warnings have been resolved:
- **Fixed unnecessary cast** in clustering similarity calculations
- **Fixed manual range contains** by using `!(50..=100).contains()`
- **Reduced function complexity** by introducing `DbscanContext` and `OutlierParams` structs
- **Fixed large error variants** by boxing `McpResponse` 
- **Improved pattern matching** by replacing `match` with `if let` where appropriate
- **Fixed default construction** warnings in unit tests
- **Updated test assertions** to use `!is_empty()` instead of `len() > 0`

### New Test Coverage
- Added `make test-examples` target for running example-specific tests
- Enhanced test categorization for better CI/CD workflows

## Mandatory Checks Before Committing

```bash
make format-check  # Verify formatting
make lint          # Check for clippy warnings (NOW PASSES!)
make test          # Run all tests
make test-examples # Run example tests
```

## Architecture Overview

### Core Components

1. **lib.rs**: Core deduplication logic
   - `FileInfo` struct: Represents file metadata with MD5 hashing
   - `walk()`: Filesystem traversal using walkdir
   - `find()`: Pattern matching for file filtering
   - `dedupe()`: Parallel duplicate detection with Rayon
   - `generate_statistics()`: Polars DataFrame generation for analysis

2. **main.rs**: CLI interface using Clap
   - `search`: Find files matching pattern
   - `dedupe`: Find and optionally remove duplicates
   - `count`: Count files matching pattern

### Key Dependencies

- **rayon**: Parallel processing for performance
- **indicatif**: Progress bars for user feedback
- **polars**: DataFrame operations for statistics
- **walkdir**: Basic filesystem traversal (legacy)
- **ignore**: Ripgrep's gitignore-aware file walker
- **globset**: Glob pattern matching
- **regex**: Regular expression support
- **clap**: Command-line argument parsing
- **md5**: File content hashing

### Performance Features

- Multi-threaded file processing using Rayon
- Progress indication for long operations
- Efficient MD5 hashing for content comparison
- DataFrame-based statistics with Polars

## CI/CD Pipeline

GitHub Actions workflows:
- **tests.yml**: Runs `make test` on every push/PR
- **lint.yml**: Runs `make lint` for code quality
- **rustfmt.yml**: Enforces code formatting
- **release.yml**: Builds release binary

## Release Process

Following PMAT-style binary releases:

```bash
# 1. Update version in Cargo.toml
# 2. Create and push tag
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0

# 3. GitHub Actions will automatically build release binaries
```

## Future Enhancements (from README)

- Add GUI interface
- Add web interface  
- Fix GitHub Actions build process silent failures
- Store operation logs across multiple runs

## Common Development Tasks

### Adding a New Command
1. Add variant to `Commands` enum in main.rs
2. Implement logic in lib.rs
3. Add command handling in main.rs match statement
4. Add tests in tests/cli.rs
5. Update help text and documentation

### Performance Profiling
```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/rdedupe dedupe --path /large/directory
perf report

# Memory profiling  
valgrind --tool=massif ./target/release/rdedupe dedupe --path /large/directory
ms_print massif.out.*
```

### Debugging Tips
- Use `RUST_LOG=debug` for verbose output
- Add `#[derive(Debug)]` to structs for easier debugging
- Use `.expect()` during development, handle errors properly for production

## Important Notes

- The tool reads entire file contents into memory for MD5 hashing - be mindful of large files
- Parallel processing scales with available CPU cores (see `display_thread_info()`)
- CSV export includes detailed file metadata and duplicate grouping information