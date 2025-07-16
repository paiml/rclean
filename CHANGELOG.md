# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-07-16

### Added
- Enhanced quality gates with comprehensive checks
- Binary size optimization configurations
- Security audit target (`make security-audit`)
- Dependency check target (`make dependency-check`)
- Coverage reporting with thresholds
- Cross-platform build optimizations
- Zero-tolerance quality standards

### Changed
- Improved Makefile with better quality control
- Updated documentation to reflect quality improvements
- Enhanced lint settings with extreme quality checks
- Optimized release builds for smaller binaries (LTO, codegen-units=1)
- Version bump from 0.1.0 to 0.1.1

### Fixed
- All clippy warnings resolved
- DBSCAN clustering parameter optimization
- Large error variant boxing for MCP responses
- Unnecessary type casting and qualification warnings
- Pattern matching improvements

### Development
- Added `make test-examples` target for example tests
- Implemented comprehensive quality gate pipeline
- Added format checking without modifications
- Enhanced property-based testing coverage

## [0.1.0] - Initial Release

### Added
- Core deduplication functionality
- File pattern matching (literal, glob, regex)
- MCP server support
- Outlier detection with clustering
- Multi-threaded processing with Rayon
- CSV export capabilities
- CLI interface with multiple commands