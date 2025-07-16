# Ripgrep Integration Specification for RDedupe

## Overview

This specification outlines the integration of ripgrep-style pattern matching into rdedupe, replacing the current simple string contains matching with a more powerful and flexible system.

**Status Update v0.1.1**: Ripgrep-style pattern matching is implemented with glob, regex, and literal pattern support. Quality improvements completed - all clippy warnings resolved and enhanced test coverage.

## Goals

1. **Enhanced Pattern Matching**: Support glob patterns, regex patterns, and file type filtering
2. **Performance**: Leverage ripgrep's optimized file walking and filtering
3. **Gitignore Support**: Automatically respect `.gitignore` and `.ignore` files
4. **Backward Compatibility**: Maintain existing CLI interface while adding new options

## Technical Approach

### Dependencies

Add the following to `Cargo.toml`:
```toml
[dependencies]
ignore = "0.4"  # Ripgrep's gitignore-aware file walker
regex = "1.10"  # For regex pattern support
globset = "0.4" # For glob pattern compilation
```

### Architecture Changes

1. **Replace `walkdir` with `ignore::Walk`**
   - The `ignore` crate provides gitignore-aware directory traversal
   - Automatically skips hidden files and respects ignore rules
   - Better performance than walkdir

2. **Pattern Matching Types**
   - **Literal**: Current behavior (backward compatibility)
   - **Glob**: Shell-style patterns (e.g., `*.txt`, `**/*.rs`)
   - **Regex**: Full regular expression support

### Implementation Plan

#### Phase 1: Core Integration

1. **Update `walk()` function**:
```rust
use ignore::{Walk, WalkBuilder};

pub fn walk_with_ignore(path: &str, options: &WalkOptions) -> Result<Vec<String>, Box<dyn Error>> {
    let mut builder = WalkBuilder::new(path);
    
    // Configure options
    builder
        .hidden(!options.include_hidden)
        .ignore(options.respect_ignore)
        .git_ignore(options.respect_gitignore)
        .max_depth(options.max_depth);
    
    let walker = builder.build();
    let mut files = Vec::new();
    
    for entry in walker {
        let entry = entry?;
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            if let Some(path_str) = entry.path().to_str() {
                files.push(path_str.to_string());
            }
        }
    }
    
    Ok(files)
}
```

2. **Enhanced Pattern Matching**:
```rust
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;

pub enum PatternType {
    Literal(String),
    Glob(GlobSet),
    Regex(Regex),
}

pub fn find_advanced(files: &[String], pattern: &PatternType) -> Vec<String> {
    files
        .iter()
        .filter(|file| match pattern {
            PatternType::Literal(s) => file.contains(s),
            PatternType::Glob(g) => g.is_match(file),
            PatternType::Regex(r) => r.is_match(file),
        })
        .cloned()
        .collect()
}
```

#### Phase 2: CLI Updates

1. **Add pattern type flags**:
```rust
#[derive(Parser)]
struct SearchCommand {
    #[clap(long, default_value = ".")]
    path: String,
    
    #[clap(long, default_value = "")]
    pattern: String,
    
    #[clap(long, value_enum, default_value = "literal")]
    pattern_type: PatternTypeArg,
    
    #[clap(long, help = "Include hidden files")]
    hidden: bool,
    
    #[clap(long, help = "Ignore .gitignore rules")]
    no_ignore: bool,
    
    #[clap(long, help = "Maximum depth to traverse")]
    max_depth: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum PatternTypeArg {
    Literal,
    Glob,
    Regex,
}
```

2. **Support file type filtering**:
```rust
#[clap(long, help = "Filter by file type (e.g., rust, python, txt)")]
file_type: Option<Vec<String>>,
```

#### Phase 3: Performance Optimizations

1. **Parallel Pattern Compilation**
   - Compile regex/glob patterns once before filtering
   - Use rayon for parallel matching on large file sets

2. **Streaming Results**
   - For very large directories, stream results instead of collecting all files first

### Testing Strategy

1. **Unit Tests**
   - Test each pattern type (literal, glob, regex)
   - Test ignore file handling
   - Test hidden file filtering

2. **Integration Tests**
   - Test with real directory structures
   - Test with various `.gitignore` configurations
   - Test performance on large directories

3. **Backward Compatibility Tests**
   - Ensure existing CLI commands work unchanged
   - Verify output format remains consistent

### Migration Path

1. **Phase 1**: Add new functionality alongside existing
2. **Phase 2**: Deprecate old pattern flag in favor of pattern-type
3. **Phase 3**: Remove deprecated code in next major version

### Example Usage

```bash
# Current (backward compatible)
rdedupe search --path . --pattern ".txt"

# Glob pattern
rdedupe search --path . --pattern "*.txt" --pattern-type glob

# Recursive glob
rdedupe search --path . --pattern "**/*.rs" --pattern-type glob

# Regex pattern
rdedupe search --path . --pattern "test_.*\.rs$" --pattern-type regex

# Include hidden files
rdedupe search --path . --pattern "*.conf" --pattern-type glob --hidden

# Ignore .gitignore rules
rdedupe search --path . --pattern "*.log" --pattern-type glob --no-ignore

# File type filtering
rdedupe search --path . --file-type rust --file-type python
```

### Success Criteria

1. ✅ All existing tests pass
2. ✅ New pattern types work correctly
3. ✅ Performance is equal or better than current implementation
4. ✅ Gitignore rules are respected by default
5. ✅ Hidden files are excluded by default
6. ✅ Documentation is updated
7. ✅ Extreme linting still passes

### Risks and Mitigations

1. **Breaking Changes**: Mitigated by maintaining backward compatibility
2. **Performance Regression**: Mitigated by benchmarking before/after
3. **Increased Binary Size**: Accept reasonable increase for functionality
4. **Complexity**: Mitigated by clear separation of pattern types

### Timeline

- Phase 1 (Core): 2 hours
- Phase 2 (CLI): 1 hour  
- Phase 3 (Testing): 1 hour
- Documentation: 30 minutes

Total: ~4.5 hours of implementation work