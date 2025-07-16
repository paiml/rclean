# RClean Testing Strategy

**Status**: Active  
**Type**: Reference  
**Created**: 2025-01-16  
**Updated**: 2025-01-16  
**Author**: PAIML Team  

## Overview

RClean employs a comprehensive multi-layer testing strategy following PMAT quality standards, ensuring reliability, performance, and correctness across all components.

## Testing Layers

### 1. Unit Tests (`src/lib.rs` + modules)
**Purpose**: Test individual functions and components in isolation
**Location**: Inline with source code using `#[cfg(test)]`
**Coverage**: Core library functions, data structures, utilities

**Examples**:
- `FileInfo` construction and methods
- Pattern matching algorithms
- Hash calculation functions
- Error handling paths

### 2. Integration Tests (`tests/`)
**Purpose**: Test component interactions and system-level behavior
**Location**: `tests/` directory
**Coverage**: CLI interface, file system operations, end-to-end workflows

**Test Files**:
- `cli.rs`: Command-line interface testing
- `integration_tests.rs`: System integration scenarios
- `mcp_integration.rs`: MCP server functionality
- `outliers_tests.rs`: Outlier detection workflows

### 3. Property-Based Tests (`tests/property_tests.rs`)
**Purpose**: Test invariants and edge cases with generated inputs
**Framework**: `proptest` crate
**Coverage**: Mathematical properties, boundary conditions, data consistency

**Properties Tested**:
- `find()` always returns subset of input files
- Pattern matching consistency across types
- File size conversions maintain accuracy
- Similarity scores bounded (0-100)
- Clustering invariants (disjoint sets)

### 4. Documentation Tests (doc-tests)
**Purpose**: Ensure documentation examples remain accurate and executable
**Location**: Function documentation with `///` comments
**Coverage**: Public API examples, usage patterns

**Examples**:
```rust
/// Find files matching a pattern
/// 
/// # Examples
/// 
/// ```
/// use rclean::find;
/// let files = vec!["file1.txt".to_string(), "file2.rs".to_string()];
/// let matches = find(&files, ".txt");
/// assert_eq!(matches, vec!["file1.txt"]);
/// ```
pub fn find(files: &[String], pattern: &str) -> Vec<String>
```

### 5. Example Tests (`examples/`)
**Purpose**: Validate example code compiles and demonstrates features
**Location**: `examples/` directory
**Coverage**: Real-world usage scenarios, API demonstrations

**Example Files**:
- `basic_usage.rs`: Core functionality demo
- `cluster_analysis.rs`: Clustering feature showcase
- `outliers_detection.rs`: Outlier detection workflow
- `pattern_matching.rs`: Pattern matching examples

### 6. Performance Tests (Benchmarks)
**Purpose**: Ensure performance characteristics meet requirements
**Framework**: Criterion.rs (future enhancement)
**Coverage**: Critical path performance, memory usage

## Test Execution Commands

### Comprehensive Testing
```bash
make test              # Run all test types
make test-examples     # Run example tests only
make test-doc          # Run documentation tests
make test-property     # Run property-based tests
```

### Specific Test Categories
```bash
cargo test --lib                    # Unit tests only
cargo test --test cli               # CLI integration tests
cargo test --test property_tests    # Property tests only
cargo test --examples              # Example compilation tests
cargo test --doc                   # Documentation tests
```

### Coverage Analysis
```bash
make coverage          # Generate HTML coverage report
make coverage-stdout   # Terminal coverage summary
make coverage-report   # Open coverage report in browser
```

## Quality Metrics

### Coverage Targets
- **Overall Coverage**: ≥ 80%
- **Critical Path Coverage**: ≥ 95%
- **Public API Coverage**: 100%

### Test Count Statistics
- **Total Tests**: 100+ tests across all categories
- **Unit Tests**: ~50 tests
- **Integration Tests**: ~25 tests  
- **Property Tests**: ~15 tests
- **Doc Tests**: ~10 tests

### Performance Benchmarks
- **Large Directory Scan**: < 30 seconds for 100K files
- **Memory Usage**: < 1GB for typical workloads
- **Parallel Efficiency**: Near-linear scaling with CPU cores

## Testing Best Practices

### 1. Test Isolation
- Each test uses isolated temporary directories
- No shared state between tests
- Deterministic test execution order

### 2. Error Testing
- Test both success and failure paths
- Validate error messages and types
- Test edge cases and boundary conditions

### 3. Property-Based Testing
- Focus on invariants that must always hold
- Use shrinking to find minimal failing cases
- Test with realistic data distributions

### 4. Integration Testing
- Test complete workflows end-to-end
- Validate CLI output formatting
- Test with various file system conditions

### 5. Documentation Testing
- All public API examples must be tested
- Keep examples simple and focused
- Update examples when API changes

## Continuous Integration

### GitHub Actions Testing
- **Matrix Testing**: Multiple Rust versions (stable, beta)
- **Cross-Platform**: Linux, macOS, Windows
- **Quality Gates**: All tests must pass before merge

### Local Testing Workflow
```bash
# Pre-commit testing
make format-check      # Formatting verification
make lint             # Code quality checks
make test             # Full test suite
make coverage         # Coverage analysis
```

### Test Data Management
- Use `tempfile` for test isolation
- Generate test data programmatically
- Clean up resources automatically

## Known Issues and Workarounds

### SSDEEP Library Stability
**Issue**: SSDEEP library crashes with certain random inputs during property testing
**Workaround**: Use pre-generated known-good hashes in `similarity_score_bounds_safe` test
**Status**: Monitoring for upstream fixes

### Performance Test Variability
**Issue**: Performance tests can be affected by system load
**Workaround**: Use relative performance metrics rather than absolute times
**Status**: Future enhancement with dedicated benchmarking

## Future Enhancements

### Planned Additions
1. **Mutation Testing**: Verify test quality with `cargo-mutants`
2. **Fuzz Testing**: Discover edge cases with `cargo-fuzz`
3. **Stress Testing**: Long-running stability tests
4. **Performance Regression Testing**: Automated performance monitoring

### Testing Infrastructure
1. **Custom Test Harness**: Specialized testing utilities
2. **Test Data Generators**: Realistic file system simulation
3. **Coverage Dashboard**: Continuous coverage monitoring
4. **Test Result Analytics**: Historical test performance tracking

## Contributing

When adding new functionality:
1. Write tests first (TDD approach)
2. Include all test types (unit, integration, property, doc)
3. Maintain or improve coverage percentages
4. Update documentation with tested examples
5. Verify tests pass in CI environment

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed testing requirements.