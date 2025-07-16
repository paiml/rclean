# Fuzzy Matching Specification for RDedupe

## Overview

Add fuzzy/similarity matching to detect files that are similar but not identical.

**Status Update v0.1.1**: Core similarity detection is implemented and working. Quality improvements completed - all clippy warnings resolved and test coverage enhanced. This is useful for finding:
- Different versions of the same document
- Images with slight edits or different compression
- Code files with minor changes
- Text files with small modifications

## Use Cases

1. **Document versioning**: Find different versions of reports, presentations
2. **Image similarity**: Detect resized, cropped, or slightly edited images
3. **Code similarity**: Find code files with minor refactoring
4. **Near-duplicates**: Files that are 90%+ similar but not exact matches

## Technical Approaches

### 1. Fuzzy Hashing (Context Triggered Piecewise Hashing - CTPH)
- **Library**: `ssdeep` algorithm via `fuzzyhash-rs` crate
- **Pros**: Designed specifically for similarity detection, handles file size differences
- **Cons**: Slower than cryptographic hashing
- **Use case**: General file similarity

### 2. SimHash (Locality Sensitive Hashing)
- **Library**: `simhash` crate
- **Pros**: Fast, good for text similarity
- **Cons**: Best for text, not binary files
- **Use case**: Text documents, source code

### 3. MinHash
- **Library**: `minhash-rs` crate
- **Pros**: Scalable, good for large datasets
- **Cons**: More complex implementation
- **Use case**: Large-scale similarity detection

### 4. Perceptual Hashing (for images)
- **Library**: `img_hash` crate
- **Pros**: Specifically designed for image similarity
- **Cons**: Only works for images
- **Use case**: Image deduplication

## Proposed Implementation

### Phase 1: Text File Similarity (SimHash)
1. Add `simhash` dependency
2. Create `SimilarityType` enum:
   ```rust
   pub enum SimilarityType {
       Exact,      // Current MD5 matching
       Fuzzy(f64), // Similarity threshold (0.0-1.0)
   }
   ```
3. Add similarity calculation for text files
4. Group files by similarity threshold

### Phase 2: General File Similarity (Fuzzy Hash)
1. Add `fuzzyhash` dependency
2. Implement SSDEEP algorithm for all file types
3. Add similarity scoring and grouping

### Phase 3: Specialized Similarity (Future)
1. Image similarity with perceptual hashing
2. Audio similarity detection
3. Video similarity detection

## CLI Interface

```bash
# Find exact duplicates (current behavior)
rdedupe ~/Documents

# Find similar files (90% threshold)
rdedupe ~/Documents --similarity 0.9

# Find similar text files using simhash
rdedupe ~/Documents --similarity 0.8 --similarity-type simhash

# Find similar files with fuzzy hashing
rdedupe ~/Documents --similarity 0.7 --similarity-type fuzzy

# Combine with other options
rdedupe ~/code --pattern "*.rs" --pattern-type glob --similarity 0.85
```

## Output Format

```
=== Similar Files Found (≥90% similarity) ===
Group 1 (95% similar):
  - report_v1.docx (1.2 MB)
  - report_v2.docx (1.3 MB)
  - report_final.docx (1.3 MB)

Group 2 (92% similar):
  - image.jpg (2.1 MB)
  - image_edited.jpg (2.0 MB)
  - image_compressed.jpg (1.8 MB)

=== Exact Duplicates ===
[Current output format]
```

## Performance Considerations

1. **Two-pass approach**:
   - First pass: Calculate fuzzy hashes in parallel
   - Second pass: Compare hashes and group by similarity

2. **Optimization strategies**:
   - Skip similarity check for files with very different sizes
   - Use bloom filters for initial filtering
   - Cache fuzzy hashes for large files

3. **Memory usage**:
   - Store fuzzy hashes in memory during comparison
   - Use streaming for large files

## Testing Strategy

1. Create test files with known similarity levels
2. Verify similarity scores match expectations
3. Performance benchmarks for large directories
4. Test different file types (text, binary, images)

## Dependencies to Add

```toml
# For general fuzzy hashing
fuzzyhash = "0.2"

# For text similarity
simhash = "0.2"

# For image similarity (future)
img_hash = "3.0"

# For edit distance calculations
strsim = "0.10"
```

## Implementation Priority

1. ✅ Basic similarity infrastructure
2. ✅ Fuzzy hashing for all files
3. ✅ CLI integration
4. ⬜ SimHash for text files
5. ⬜ Perceptual hashing for images
6. ⬜ Performance optimizations
7. ⬜ Advanced grouping algorithms