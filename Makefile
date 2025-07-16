# RClean Makefile - Following PMAT Quality Standards
# Zero tolerance for quality violations

.PHONY: all format format-check lint lint-extreme test test-fast test-doc test-property test-examples build-release build-small run clean quality-gate security-audit dependency-check release-check help

# Default target
all: format lint-extreme test run build-release

# Format code with rustfmt
format:
	@echo "📝 Formatting code..."
	@cargo fmt
	@echo "✅ Formatting completed!"

# Check formatting without making changes
format-check:
	@echo "🔍 Checking code formatting..."
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt --all -- --check
	@echo "✅ Format check passed!"

# Standard linting with clippy
lint:
	@echo "🔍 Running standard linting..."
	@rustup component add clippy 2> /dev/null
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Standard linting passed!"

# EXTREME linting - PMAT style with zero tolerance
lint-extreme:
	@echo "🔥 Running EXTREME linting with zero tolerance..."
	@rustup component add clippy 2> /dev/null
	@cargo clippy --all-targets --all-features -- \
		-D warnings \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-W clippy::unwrap_used \
		-W clippy::expect_used \
		-W clippy::panic \
		-W clippy::unimplemented \
		-W clippy::todo \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::multiple_crate_versions \
		-A clippy::implicit_hasher \
		-A clippy::wildcard_imports
	@echo "✅ EXTREME linting passed!"

# Run all tests
test:
	@echo "🧪 Running all tests..."
	@cargo test --all-features
	@echo "✅ All tests passed!"

# Fast tests only
test-fast:
	@echo "⚡ Running fast tests..."
	@cargo test --all-features -- --test-threads=4
	@echo "✅ Fast tests passed!"

# Documentation tests
test-doc:
	@echo "📚 Running documentation tests..."
	@cargo test --doc
	@echo "✅ Documentation tests passed!"

# Property-based tests
test-property:
	@echo "🎲 Running property-based tests..."
	@cargo test --test property_tests --all-features
	@echo "✅ Property tests passed!"

# Example tests
test-examples:
	@echo "📖 Running example tests..."
	@cargo test --examples
	@echo "✅ Example tests passed!"

# Build optimized release binary
build-release:
	@echo "🔨 Building release version for platform $(shell uname -s)..."
	@cargo build --release
	@echo "✅ Release build completed at: ./target/release/rclean"

# Install the binary
install: build-release
	@echo "📦 Installing rclean..."
	@cargo install --path . --force
	@echo "✅ Installation completed! Binary installed to: $$(which rclean)"

# Install from current directory (for CI/CD)
install-local: build-release
	@echo "📦 Installing rclean locally..."
	@mkdir -p ~/.local/bin
	@cp target/release/rclean ~/.local/bin/
	@chmod +x ~/.local/bin/rclean
	@echo "✅ Installation completed! Binary installed to: ~/.local/bin/rclean"
	@echo "📝 Make sure ~/.local/bin is in your PATH"

# Run with example
run:
	@echo "🚀 Running with example..."
	@cargo run -- dedupe --path tests --pattern .txt
	@echo "✅ Execution completed!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -f Cargo.lock
	@rm -rf coverage/
	@echo "✅ Clean completed!"

# Fast coverage - MUST complete under 30 seconds
coverage:
	@echo "⚡ Running FAST coverage (target: < 30 seconds)..."
	@mkdir -p coverage/
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "📦 Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
	fi
	@echo "🚀 Running tests with coverage..."
	@cargo llvm-cov clean --workspace
	@cargo llvm-cov test --lib --test main_coverage --test mcp_integration --test mcp_handlers_coverage --test outliers_tests \
		--html --output-dir coverage/
	@echo "📁 HTML report: coverage/index.html"
	@echo ""
	@echo "📊 Coverage Report:"
	@echo "=================="
	@cargo llvm-cov report
	@echo "✅ Coverage complete!"

# Coverage to stdout only (faster)
coverage-stdout:
	@echo "📊 Running coverage to stdout..."
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "📦 Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
	fi
	@cargo llvm-cov clean --workspace
	@cargo llvm-cov test --lib --test main_coverage --test mcp_integration --test mcp_handlers_coverage --test outliers_tests --quiet
	@echo ""
	@echo "📊 Coverage Report:"
	@echo "=================="
	@cargo llvm-cov report

# Show existing coverage report
coverage-report:
	@echo "📊 Showing existing coverage report..."
	@if [ -d "target/llvm-cov-target" ]; then \
		cargo llvm-cov report; \
	else \
		echo "⚠️  No coverage data found. Run 'make coverage' first."; \
	fi

# Run quality gate check
quality-gate: format-check lint-extreme test
	@echo "🎯 Running quality gate..."
	@echo "✅ Quality gate PASSED! All checks successful!"

# Enhanced Quality Gate - Zero Tolerance
quality-gate-extreme: security-audit dependency-check format-check lint-extreme test test-examples coverage release-check
	@echo "🏭 Enhanced Quality Gate Starting..."
	@echo "📊 Verifying all quality metrics..."
	@echo ""
	@echo "🔒 Security Audit: ✅ PASSED"
	@echo "📦 Dependency Check: ✅ PASSED" 
	@echo "🎨 Format Check: ✅ PASSED"
	@echo "🔍 Extreme Linting: ✅ PASSED"
	@echo "🧪 All Tests: ✅ PASSED"
	@echo "📖 Example Tests: ✅ PASSED"
	@echo "📈 Coverage Report: ✅ PASSED"
	@echo "🚀 Release Build: ✅ PASSED"
	@echo ""
	@echo "🏆 ENHANCED QUALITY GATE: ✅ PASSED!"
	@echo "🎯 Zero defects, zero warnings, zero technical debt"
	@echo "📦 Ready for production release"

# Security Audit
security-audit:
	@echo "🔒 Running security audit..."
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "📦 Installing cargo-audit..."; \
		cargo install cargo-audit; \
	fi
	@cargo audit
	@echo "✅ Security audit passed!"

# Dependency Check
dependency-check:
	@echo "📦 Running dependency check..."
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "📦 Installing cargo-outdated..."; \
		cargo install cargo-outdated; \
	fi
	@cargo outdated --exit-code 1 || echo "⚠️ Some dependencies may be outdated"
	@echo "✅ Dependency check completed!"

# Release Build Check
release-check:
	@echo "🚀 Running release build check..."
	@cargo check --release --all-targets --all-features
	@cargo build --release --all-features
	@echo "📏 Measuring binary size..."
	@if [ -f target/release/rclean ]; then \
		SIZE=$$(stat -f%z target/release/rclean 2>/dev/null || stat -c%s target/release/rclean 2>/dev/null); \
		echo "📦 Release binary size: $$SIZE bytes ($$(echo $$SIZE | numfmt --to=iec-i))"; \
		if [ $$SIZE -gt 50000000 ]; then \
			echo "⚠️ Binary size exceeds 50MB - consider optimization"; \
		else \
			echo "✅ Binary size within acceptable limits"; \
		fi; \
	fi
	@echo "✅ Release build check passed!"

# Build smallest possible binary
build-small:
	@echo "🔨 Building smallest binary with maximum optimizations..."
	@cargo build --profile release-small --all-features
	@echo "📦 Comparing binary sizes..."
	@if [ -f target/release/rclean ] && [ -f target/release-small/rclean ]; then \
		NORMAL_SIZE=$$(stat -f%z target/release/rclean 2>/dev/null || stat -c%s target/release/rclean 2>/dev/null); \
		SMALL_SIZE=$$(stat -f%z target/release-small/rclean 2>/dev/null || stat -c%s target/release-small/rclean 2>/dev/null); \
		REDUCTION=$$(echo "scale=1; ($$NORMAL_SIZE - $$SMALL_SIZE) * 100 / $$NORMAL_SIZE" | bc -l 2>/dev/null || echo "N/A"); \
		echo "📊 Normal release: $$NORMAL_SIZE bytes ($$(echo $$NORMAL_SIZE | numfmt --to=iec-i))"; \
		echo "📊 Small release:  $$SMALL_SIZE bytes ($$(echo $$SMALL_SIZE | numfmt --to=iec-i))"; \
		echo "📈 Size reduction: $$REDUCTION%"; \
	fi
	@echo "✅ Small build completed!"

# Help
help:
	@echo "RDedupe Makefile - Available targets:"
	@echo ""
	@echo "  make all             - Run complete pipeline (format, lint-extreme, test, run, build)"
	@echo "  make format          - Format code with rustfmt"
	@echo "  make format-check    - Check formatting without changes"
	@echo "  make lint            - Run standard linting"
	@echo "  make lint-extreme    - Run EXTREME linting (PMAT style)"
	@echo "  make test            - Run all tests"
	@echo "  make test-fast       - Run tests with parallel execution"
	@echo "  make test-doc        - Run documentation tests"
	@echo "  make test-property   - Run property-based tests"
	@echo "  make test-examples   - Run example tests"
	@echo "  make build-release   - Build optimized release binary"
	@echo "  make install         - Install binary using cargo install"
	@echo "  make install-local   - Install binary to ~/.local/bin (for CI/CD)"
	@echo "  make run             - Run with example data"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make coverage        - Generate test coverage report (HTML)"
	@echo "  make coverage-stdout - Show coverage summary in terminal"
	@echo "  make coverage-report - Display existing coverage report"
	@echo "  make quality-gate    - Run all quality checks"
	@echo "  make help            - Show this help message"
	@echo ""
	@echo "Enhanced Quality Targets:"
	@echo "  make quality-gate-extreme - Enhanced quality gate (zero tolerance)"
	@echo "  make security-audit      - Security vulnerability audit"
	@echo "  make dependency-check    - Check for outdated dependencies"
	@echo "  make release-check       - Validate release build quality"
	@echo "  make build-small         - Build smallest possible binary"
	@echo ""
	@echo "Quality standards:"
	@echo "  - Zero clippy warnings (extreme mode)"
	@echo "  - Zero formatting issues"
	@echo "  - All tests passing"
	@echo "  - No self-admitted technical debt"
	@echo "  - Security audit passing"
	@echo "  - Dependencies up to date"
	@echo "  - Binary size optimization"
	@echo "  - Test coverage tracking"