# Contributing to RClean

**Status**: Active  
**Type**: Guide  
**Created**: 2025-01-16  
**Updated**: 2025-01-16  
**Author**: PAIML Team  

## Code Quality Requirements

RClean follows PMAT (Production Manufacturing and Assembly Technology) quality standards with zero-tolerance policies.

| Metric | Requirement | Verification |
|--------|-------------|--------------|
| Technical Debt Gauge (TDG) | ≤ 1.0 | `make quality-gate` |
| Cyclomatic Complexity | ≤ 20 per function | `make lint` |
| SATD Comments | 0 (no TODO/FIXME) | `make lint` |
| Test Coverage | ≥ 80% overall | `make coverage` |
| Lint Violations | 0 warnings/errors | `make lint` |
| Security Vulnerabilities | 0 high/critical | `make security-audit` |
| Documentation Coverage | 100% public API | `make test-doc` |

## Development Workflow

### 1. Setup Development Environment
```bash
# Clone repository
git clone https://github.com/paiml/rclean.git
cd rclean

# Install Rust toolchain
rustup install stable
rustup component add rustfmt clippy

# Install additional tools
cargo install cargo-audit cargo-tarpaulin

# Verify setup
make quality-gate
```

### 2. Pre-Commit Requirements
**MANDATORY**: All checks must pass before committing
```bash
make format-check      # Code formatting verification
make lint             # Zero-tolerance linting
make test             # Comprehensive test suite
make test-examples    # Example compilation tests
make security-audit   # Security vulnerability scan
```

### 3. Commit Message Format
Follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code changes that neither fix bugs nor add features
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(clustering): add DBSCAN algorithm for file similarity detection

fix(outliers): handle edge case for empty directory scanning

docs(api): add comprehensive examples for MCP server usage

test(property): add invariant tests for clustering algorithms
```

## Testing Requirements

### Mandatory Test Types
1. **Unit Tests**: For all new functions/methods
2. **Integration Tests**: For new commands/workflows
3. **Property Tests**: For algorithms with invariants
4. **Documentation Tests**: For all public API examples

### Test Coverage Standards
- **New Code**: 100% coverage required
- **Modified Code**: Maintain or improve existing coverage
- **Critical Paths**: 100% coverage mandatory
- **Error Handling**: All error paths must be tested

### Writing Tests
```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_expected_behavior() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}

// Property test example
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn invariant_always_holds(input in any::<ValidInput>()) {
            let result = function_under_test(input);
            prop_assert!(invariant_condition(result));
        }
    }
}
```

## Code Standards

### Rust-Specific Guidelines
1. **Error Handling**: Use `Result<T, E>` for fallible operations
2. **Documentation**: All public items must have doc comments with examples
3. **Performance**: Prefer zero-allocation patterns where possible
4. **Safety**: Avoid `unsafe` code unless absolutely necessary
5. **Dependencies**: Minimize external dependencies, justify additions

### Code Organization
```rust
// File structure pattern
//! Module documentation
//!
//! Detailed description of module purpose and usage.

use std::collections::HashMap;  // Standard library first
use external_crate::Module;     // External crates
use crate::internal::Module;    // Internal crates

/// Public struct with comprehensive documentation
/// 
/// # Examples
/// 
/// ```
/// use rclean::StructName;
/// let instance = StructName::new();
/// ```
pub struct StructName {
    field: Type,
}

impl StructName {
    /// Constructor with example usage
    pub fn new() -> Self {
        Self { field: default_value }
    }
}

#[cfg(test)]
mod tests {
    // Test implementations
}
```

### Performance Guidelines
1. **Parallelization**: Use Rayon for CPU-intensive operations
2. **Memory Management**: Prefer borrowing over cloning
3. **File I/O**: Use buffered operations for large files
4. **Algorithms**: Document time/space complexity

## Architecture Guidelines

### Module Organization
- **lib.rs**: Core deduplication logic and public API
- **clustering.rs**: Similarity detection and clustering algorithms
- **outliers.rs**: Statistical analysis and outlier detection
- **mcp_server/**: Model Context Protocol implementation

### Dependency Management
- **Core Dependencies**: Essential for functionality
- **Dev Dependencies**: Testing and development tools only
- **Optional Features**: Use feature flags for optional functionality

### API Design Principles
1. **Consistency**: Similar functions should have similar signatures
2. **Ergonomics**: Make common use cases simple
3. **Flexibility**: Support advanced use cases without complexity
4. **Documentation**: Every public item needs examples

## Pull Request Process

### 1. Pre-PR Checklist
- [ ] All quality gates pass (`make quality-gate`)
- [ ] New functionality includes comprehensive tests
- [ ] Documentation updated for API changes
- [ ] Examples added for new features
- [ ] Performance impact assessed
- [ ] Security implications reviewed

### 2. PR Requirements
- **Title**: Follow conventional commit format
- **Description**: Explain what, why, and how
- **Tests**: Demonstrate new functionality works
- **Documentation**: Update relevant docs
- **Breaking Changes**: Clearly document any breaking changes

### 3. Review Process
1. **Automated Checks**: All CI/CD pipelines must pass
2. **Code Review**: At least one maintainer approval required
3. **Quality Verification**: Manual verification of quality standards
4. **Integration Testing**: Verify changes work in full system

### 4. Merge Requirements
- All conversations resolved
- CI/CD passes on latest commit
- No merge conflicts
- Quality standards maintained

## Release Process

### Version Management
- Follow [Semantic Versioning](https://semver.org/)
- Update version in `Cargo.toml`
- Update `CHANGELOG.md` with changes
- Create git tag: `git tag -a v0.1.2 -m "Release v0.1.2"`

### Automated Release Pipeline
```bash
# Version bump
git add . && git commit -m "feat: bump version to v0.1.2"
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin main && git push origin v0.1.2
```

**Automated Steps**:
1. Cross-platform binary builds (Linux, macOS, Windows)
2. GitHub release creation with artifacts
3. Automatic crates.io publishing
4. Documentation deployment

## Documentation Standards

### Required Documentation
1. **README.md**: Project overview and quick start
2. **API Documentation**: Comprehensive rustdoc comments
3. **Architecture Docs**: System design and structure
4. **User Guides**: Step-by-step usage instructions
5. **Contributing Guide**: This document

### Documentation Format
All documentation files must include the standard header:
```markdown
**Status**: Draft | Active | Archived | Deprecated
**Type**: Guide | Reference | Decision | Specification  
**Created**: YYYY-MM-DD
**Updated**: YYYY-MM-DD
**Author**: Name or Team
```

### Example Documentation
```rust
/// Find duplicate files using MD5 hashing
///
/// This function performs parallel duplicate detection across the provided
/// file paths using MD5 content hashing for comparison.
///
/// # Arguments
///
/// * `files` - A slice of file paths to analyze
/// * `options` - Configuration options for the operation
///
/// # Returns
///
/// Returns a `Result` containing a DataFrame with duplicate analysis results
///
/// # Examples
///
/// ```
/// use rclean::{dedupe, WalkOptions};
///
/// let options = WalkOptions::default();
/// let result = dedupe(&["/path/to/files"], &options)?;
/// println!("Found {} duplicate groups", result.height());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - File paths are inaccessible
/// - Insufficient permissions
/// - I/O errors during file reading
pub fn dedupe(files: &[&str], options: &WalkOptions) -> Result<DataFrame, Error>
```

## Getting Help

### Resources
- **Documentation**: Check `docs/` directory first
- **Examples**: See `examples/` for usage patterns
- **Tests**: Review test files for expected behavior
- **Issues**: Search existing GitHub issues

### Contact
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community support
- **Email**: For security-related concerns only

### Community Guidelines
- Be respectful and constructive
- Search existing issues before creating new ones
- Provide minimal reproducible examples
- Include relevant system information

---

Thank you for contributing to RClean! Your efforts help maintain the highest quality standards for this PMAT-certified project.