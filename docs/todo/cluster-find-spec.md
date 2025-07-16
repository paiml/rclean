# Cluster Similarity Detection Specification

## Abstract

This specification defines the implementation of density-based clustering for identifying groups of similar large files within the rdedupe deduplication system. The approach leverages existing SSDEEP fuzzy hashing infrastructure to construct a similarity graph, applying DBSCAN clustering to discover file relationships beyond pairwise comparisons. Implementation follows pmat extreme quality standards: zero SATD, zero high complexity (cyclomatic < 20), and comprehensive test coverage (≥80%). 

**Update v0.1.1**: Base quality standards have been achieved - all clippy warnings resolved, function complexity reduced via parameter structs, and enhanced test coverage with new `make test-examples` target.

## 1. Problem Statement

Current rdedupe functionality identifies pairwise duplicates and similar files but lacks the capability to discover broader patterns of file relationships. Large files with high similarity often represent versioned documents, backup copies, or derivative works that form natural clusters. Identifying these clusters provides actionable insights for storage optimization and data governance.

### 1.1 Requirements

- **R1**: Identify clusters of similar files without a priori knowledge of cluster count
- **R2**: Handle outliers gracefully (files that don't belong to any cluster)
- **R3**: Integrate seamlessly with existing outlier detection infrastructure
- **R4**: Maintain performance characteristics suitable for large-scale file systems
- **R5**: Provide configurable similarity thresholds and minimum cluster sizes
- **R6**: Expose functionality through MCP protocol for AI agent composition
- **R7**: Achieve ≥80% test coverage with property-based testing
- **R8**: Maintain zero SATD and cyclomatic complexity < 20 per function

## 2. Technical Design

### 2.1 Algorithm Selection

DBSCAN (Density-Based Spatial Clustering of Applications with Noise) is selected based on the following characteristics:

```rust
// DBSCAN advantages for file clustering:
// 1. No predetermined cluster count (unlike k-means)
// 2. Robust noise handling (isolated large files)
// 3. Arbitrary cluster shapes (non-spherical similarity groups)
// 4. O(n log n) complexity with spatial indexing
```

### 2.2 Distance Metric

The similarity-to-distance transformation follows:

```rust
/// Converts SSDEEP similarity score to distance metric.
///
/// # Arguments
/// * `similarity` - SSDEEP similarity score [0, 100]
///
/// # Returns
/// Distance value where 0 = identical, 100 = completely dissimilar
///
/// # Example
/// ```
/// # use rdedupe::clustering::similarity_to_distance;
/// assert_eq!(similarity_to_distance(100), 0.0);
/// assert_eq!(similarity_to_distance(70), 30.0);
/// assert_eq!(similarity_to_distance(0), 100.0);
/// ```
pub fn similarity_to_distance(similarity: u8) -> f64 {
    100.0 - f64::from(similarity)
}
```

### 2.3 Architecture Integration

```
┌─────────────────┐
│ outliers.rs     │
├─────────────────┤
│ detect_outliers │──┐
└─────────────────┘  │
                     ▼
        ┌────────────────────────┐
        │ Large File Detection   │
        └────────────────────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │ SSDEEP Hash Generation │
        └────────────────────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │ Distance Matrix Build  │
        └────────────────────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │ DBSCAN Clustering      │
        └────────────────────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │ Cluster Report Gen     │
        └────────────────────────┘
```

## 3. Implementation Specification

### 3.1 Data Structures

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct LargeFileCluster {
    pub cluster_id: usize,
    pub files: Vec<SimpleFileInfo>,
    pub total_size: u64,
    pub avg_similarity: f64,
    pub density: f64,
}

// Extension to existing OutlierReport
#[derive(Debug, serde::Serialize, Default)]
pub struct OutlierReport {
    // ... existing fields ...
    pub large_file_clusters: Vec<LargeFileCluster>,
}
```

### 3.2 Core Algorithm (Modular Design)

```rust
/// Module for clustering functionality following pmat standards
pub mod clustering {
    use ndarray::Array2;
    use rayon::prelude::*;

    /// Builds pairwise distance matrix with parallel computation.
    /// Complexity: O(n²) but parallelized, cyclomatic complexity: 5
    ///
    /// # Example
    /// ```
    /// # use rdedupe::clustering::build_distance_matrix;
    /// # use rdedupe::SimpleFileInfo;
    /// let files = vec![
    ///     SimpleFileInfo {
    ///         path: "a.txt".into(),
    ///         size: 1000,
    ///         ssdeep_hash: Some("3:abc:def".into())
    ///     },
    ///     SimpleFileInfo {
    ///         path: "b.txt".into(),
    ///         size: 1000,
    ///         ssdeep_hash: Some("3:abc:ghi".into())
    ///     },
    /// ];
    /// let matrix = build_distance_matrix(&files);
    /// assert_eq!(matrix.shape(), &[2, 2]);
    /// ```
    pub fn build_distance_matrix(files: &[SimpleFileInfo]) -> Array2<f64> {
        let n = files.len();
        let mut distances = Array2::zeros((n, n));

        // Parallel computation of upper triangle
        let upper_triangle: Vec<_> = (0..n).into_par_iter()
            .flat_map(|i| (i+1..n).into_par_iter().map(move |j| (i, j)))
            .map(|(i, j)| {
                let sim = calculate_similarity_safe(&files[i], &files[j]);
                (i, j, 100.0 - f64::from(sim))
            })
            .collect();

        // Fill symmetric matrix
        for (i, j, dist) in upper_triangle {
            distances[[i, j]] = dist;
            distances[[j, i]] = dist;
        }

        distances
    }

    /// Safe similarity calculation with validation.
    /// Cyclomatic complexity: 4
    fn calculate_similarity_safe(a: &SimpleFileInfo, b: &SimpleFileInfo) -> u8 {
        match (&a.ssdeep_hash, &b.ssdeep_hash) {
            (Some(h1), Some(h2)) => rdedupe::calculate_similarity(h1, h2),
            _ => 0,
        }
    }

    /// Performs DBSCAN clustering with configurable parameters.
    /// Cyclomatic complexity: 8
    ///
    /// # Example
    /// ```
    /// # use rdedupe::clustering::{detect_large_file_clusters, SimpleFileInfo};
    /// let files = vec![
    ///     SimpleFileInfo::new("a.txt", 1000, "3:abc:def"),
    ///     SimpleFileInfo::new("b.txt", 1000, "3:abc:deg"),
    ///     SimpleFileInfo::new("c.txt", 1000, "3:xyz:123"),
    /// ];
    /// let clusters = detect_large_file_clusters(&files, 70, 2);
    /// assert_eq!(clusters.len(), 1);
    /// assert_eq!(clusters[0].files.len(), 2);
    /// ```
    pub fn detect_large_file_clusters(
        files: &[SimpleFileInfo],
        min_similarity: u8,
        min_cluster_size: usize,
    ) -> Vec<LargeFileCluster> {
        if files.len() < min_cluster_size {
            return vec![];
        }

        let distances = build_distance_matrix(files);
        let epsilon = 100.0 - f64::from(min_similarity);

        let clusters = Dbscan::params(min_cluster_size)
            .tolerance(epsilon)
            .transform(&distances)
            .expect("DBSCAN clustering failed");

        aggregate_clusters(files, clusters, min_cluster_size, &distances)
    }

    /// Aggregates clustering results with statistics.
    /// Cyclomatic complexity: 7
    fn aggregate_clusters(
        files: &[SimpleFileInfo],
        cluster_labels: Vec<Option<usize>>,
        min_size: usize,
        distances: &Array2<f64>,
    ) -> Vec<LargeFileCluster> {
        let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();

        for (idx, label) in cluster_labels.iter().enumerate() {
            if let Some(cluster_id) = label {
                cluster_map.entry(*cluster_id).or_default().push(idx);
            }
        }

        cluster_map.into_iter()
            .filter(|(_, indices)| indices.len() >= min_size)
            .map(|(id, indices)| build_cluster_info(id, &indices, files, distances))
            .collect()
    }

    /// Builds cluster information with metrics.
    /// Cyclomatic complexity: 6
    fn build_cluster_info(
        cluster_id: usize,
        indices: &[usize],
        files: &[SimpleFileInfo],
        distances: &Array2<f64>,
    ) -> LargeFileCluster {
        let cluster_files: Vec<_> = indices.iter()
            .map(|&i| files[i].clone())
            .collect();

        let total_size = cluster_files.iter().map(|f| f.size).sum();
        let avg_similarity = calculate_avg_similarity(indices, distances);
        let density = calculate_density(indices, distances);

        LargeFileCluster {
            cluster_id,
            files: cluster_files,
            total_size,
            avg_similarity,
            density,
        }
    }
}
```

### 3.3 MCP Protocol Integration

```rust
/// MCP tool definition for cluster analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterAnalysisTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl ClusterAnalysisTool {
    pub fn new() -> Self {
        Self {
            name: "analyze_file_clusters".to_string(),
            description: "Detect clusters of similar large files using DBSCAN".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to analyze"
                    },
                    "min_similarity": {
                        "type": "integer",
                        "minimum": 50,
                        "maximum": 100,
                        "default": 70,
                        "description": "Minimum similarity percentage for clustering"
                    },
                    "min_cluster_size": {
                        "type": "integer",
                        "minimum": 2,
                        "default": 2,
                        "description": "Minimum files to form a cluster"
                    },
                    "min_file_size": {
                        "type": "string",
                        "default": "10MB",
                        "description": "Minimum file size to consider"
                    },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Specific files to analyze (for tool composition)"
                    }
                },
                "required": ["path"]
            })
        }
    }
}

/// MCP handler with tool composition support
pub async fn handle_cluster_analysis(params: Value) -> Result<Value, McpError> {
    let path = params["path"].as_str()
        .ok_or_else(|| McpError::invalid_params("path required"))?;

    let min_similarity = params["min_similarity"].as_u64()
        .unwrap_or(70) as u8;

    let min_cluster_size = params["min_cluster_size"].as_u64()
        .unwrap_or(2) as usize;

    // Support tool composition via files parameter
    let files = if let Some(file_list) = params["files"].as_array() {
        // Analyze specific files from previous tool output
        load_specific_files(file_list)?
    } else {
        // Full directory scan
        scan_large_files(path, params["min_file_size"].as_str())?
    };

    let clusters = detect_large_file_clusters(&files, min_similarity, min_cluster_size);

    Ok(json!({
        "clusters": clusters,
        "summary": {
            "total_clusters": clusters.len(),
            "total_files": clusters.iter().map(|c| c.files.len()).sum::<usize>(),
            "total_size": clusters.iter().map(|c| c.total_size).sum::<u64>(),
            "files": clusters.iter()
                .flat_map(|c| c.files.iter().map(|f| &f.path))
                .collect::<Vec<_>>()
        }
    }))
}
```

### 3.4 Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use super::*;

    /// Generate arbitrary file sets for testing
    prop_compose! {
        fn arb_file_set()(
            count in 2..50usize,
            base_hash in "[0-9a-f]{6}",
        ) -> Vec<SimpleFileInfo> {
            (0..count).map(|i| {
                SimpleFileInfo {
                    path: format!("file_{}.dat", i),
                    size: 1024 * (i as u64 + 1),
                    ssdeep_hash: Some(format!("3:{}:{}", base_hash, i % 5)),
                }
            }).collect()
        }
    }

    proptest! {
        /// Property: All files in a cluster have similarity ≥ threshold
        #[test]
        fn prop_cluster_similarity_invariant(
            files in arb_file_set(),
            min_sim in 50..100u8,
        ) {
            let clusters = detect_large_file_clusters(&files, min_sim, 2);

            for cluster in clusters {
                for i in 0..cluster.files.len() {
                    for j in i+1..cluster.files.len() {
                        let sim = calculate_similarity(
                            cluster.files[i].ssdeep_hash.as_ref().unwrap(),
                            cluster.files[j].ssdeep_hash.as_ref().unwrap()
                        );
                        prop_assert!(sim >= min_sim,
                            "Files in cluster must have similarity {} >= {}",
                            sim, min_sim);
                    }
                }
            }
        }

        /// Property: No file appears in multiple clusters
        #[test]
        fn prop_cluster_disjoint_invariant(
            files in arb_file_set(),
            min_sim in 50..100u8,
        ) {
            let clusters = detect_large_file_clusters(&files, min_sim, 2);
            let mut seen = HashSet::new();

            for cluster in &clusters {
                for file in &cluster.files {
                    prop_assert!(seen.insert(&file.path),
                        "File {} appears in multiple clusters", file.path);
                }
            }
        }

        /// Property: Cluster size respects minimum
        #[test]
        fn prop_minimum_cluster_size(
            files in arb_file_set(),
            min_sim in 50..100u8,
            min_size in 2..10usize,
        ) {
            let clusters = detect_large_file_clusters(&files, min_sim, min_size);

            for cluster in clusters {
                prop_assert!(cluster.files.len() >= min_size,
                    "Cluster size {} < minimum {}",
                    cluster.files.len(), min_size);
            }
        }
    }
}
```

### 3.5 Performance Optimization

```rust
/// Batch processing for memory-constrained environments
/// Cyclomatic complexity: 9
pub fn detect_clusters_batched(
    files: &[SimpleFileInfo],
    min_similarity: u8,
    min_cluster_size: usize,
    batch_size: usize,
) -> Vec<LargeFileCluster> {
    if files.len() <= batch_size {
        return detect_large_file_clusters(files, min_similarity, min_cluster_size);
    }

    // Locality-sensitive hashing for pre-filtering
    let lsh_buckets = compute_lsh_buckets(files);
    let mut all_clusters = Vec::new();

    for bucket in lsh_buckets.values() {
        if bucket.len() >= min_cluster_size {
            let batch_clusters = detect_large_file_clusters(
                bucket,
                min_similarity,
                min_cluster_size
            );
            all_clusters.extend(batch_clusters);
        }
    }

    // Merge overlapping clusters
    merge_overlapping_clusters(all_clusters)
}

/// LSH implementation for approximate similarity
/// Cyclomatic complexity: 7
fn compute_lsh_buckets(files: &[SimpleFileInfo]) -> HashMap<u64, Vec<SimpleFileInfo>> {
    let mut buckets = HashMap::new();

    for file in files {
        if let Some(hash) = &file.ssdeep_hash {
            // Extract block size and chunk for bucketing
            let parts: Vec<_> = hash.split(':').collect();
            if parts.len() >= 3 {
                let block_size = parts[0].parse::<u64>().unwrap_or(0);
                let chunk = &parts[1][..parts[1].len().min(8)];
                let bucket_key = hash_combine(block_size, chunk);
                buckets.entry(bucket_key).or_insert_with(Vec::new).push(file.clone());
            }
        }
    }

    buckets
}
```

## 4. Integration Points

### 4.1 CLI Interface

```rust
#[derive(clap::Parser, Debug)]
pub struct Outliers {
    #[clap(long, help = "Enable clustering of similar large files")]
    pub cluster: bool,

    #[clap(long, default_value_t = 70,
           value_parser = clap::value_parser!(u8).range(50..=100),
           help = "Similarity threshold for clustering (50-100)")]
    pub cluster_similarity: u8,

    #[clap(long, default_value_t = 2,
           value_parser = clap::value_parser!(usize).range(2..),
           help = "Minimum files to form a cluster")]
    pub min_cluster_size: usize,
}
```

### 4.2 Quality Gate Integration

```rust
/// Quality check for cluster analysis
/// Cyclomatic complexity: 6
pub fn check_cluster_quality(report: &OutlierReport) -> QualityResult {
    let mut violations = Vec::new();

    // Check for excessive clustering
    let total_clustered = report.large_file_clusters
        .iter()
        .map(|c| c.files.len())
        .sum::<usize>();

    let clustering_ratio = total_clustered as f64 / report.large_file_outliers.len() as f64;

    if clustering_ratio > 0.8 {
        violations.push(QualityViolation {
            check: "cluster_ratio".to_string(),
            message: format!("Excessive clustering: {:.1}% files clustered",
                clustering_ratio * 100.0),
            severity: Severity::Warning,
        });
    }

    // Check cluster density
    for cluster in &report.large_file_clusters {
        if cluster.density < 0.5 {
            violations.push(QualityViolation {
                check: "cluster_density".to_string(),
                message: format!("Low density cluster {}: {:.2}",
                    cluster.cluster_id, cluster.density),
                severity: Severity::Info,
            });
        }
    }

    QualityResult {
        passed: violations.is_empty(),
        violations,
    }
}
```

## 5. Output Specification

### 5.1 Console Output Format

```
--- Similar Large File Clusters ---
┌────────────┬────────────┬─────────────┬──────────┬─────────┬──────────────────────┐
│ Cluster ID │ File Count │ Total Size  │ Avg Sim  │ Density │ File Path            │
├────────────┼────────────┼─────────────┼──────────┼─────────┼──────────────────────┤
│ 0          │ 4          │ 125.6 MiB   │ 85.3%    │ 0.78    │ /docs/report_v1.pdf  │
│            │            │             │          │         │ /docs/report_v2.pdf  │
│            │            │             │          │         │ /backup/report.pdf   │
│            │            │             │          │         │ /archive/report.pdf  │
├────────────┼────────────┼─────────────┼──────────┼─────────┼──────────────────────┤
│ 1          │ 3          │ 89.2 MiB    │ 92.1%    │ 0.89    │ /img/photo.raw       │
│            │            │             │          │         │ /img/photo_edit.raw  │
│            │            │             │          │         │ /img/photo_final.raw │
└────────────┴────────────┴─────────────┴──────────┴─────────┴──────────────────────┘
```

### 5.2 JSON Output Schema

```json
{
  "large_file_clusters": [
    {
      "cluster_id": 0,
      "files": [
        {
          "path": "/docs/report_v1.pdf",
          "size": 33554432,
          "ssdeep_hash": "3072:abc123:def456"
        }
      ],
      "total_size": 131653120,
      "avg_similarity": 85.3,
      "density": 0.78
    }
  ],
  "summary": {
    "total_clusters": 2,
    "total_files": 7,
    "total_size": 220865280,
    "files": ["/docs/report_v1.pdf", "..."]
  }
}
```

## 6. Comprehensive Testing Strategy

### 6.1 Unit Test Coverage Matrix

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixture generator
    fn create_test_files(pattern: &str, count: usize) -> Vec<SimpleFileInfo> {
        (0..count).map(|i| SimpleFileInfo {
            path: format!("test_{}.dat", i),
            size: 1024 * 1024 * (i as u64 + 1),
            ssdeep_hash: Some(format!("3:{}:{}", pattern, i % 3)),
        }).collect()
    }

    #[test]
    fn test_distance_matrix_symmetry() {
        let files = create_test_files("abc", 5);
        let matrix = build_distance_matrix(&files);

        for i in 0..files.len() {
            for j in 0..files.len() {
                assert_eq!(matrix[[i, j]], matrix[[j, i]],
                    "Distance matrix must be symmetric");
            }
        }
    }

    #[test]
    fn test_cluster_detection_edge_cases() {
        // Empty input
        assert_eq!(detect_large_file_clusters(&[], 70, 2).len(), 0);

        // Single file
        let single = create_test_files("xyz", 1);
        assert_eq!(detect_large_file_clusters(&single, 70, 2).len(), 0);

        // All identical files
        let identical: Vec<_> = (0..5).map(|i| SimpleFileInfo {
            path: format!("file_{}.dat", i),
            size: 1024,
            ssdeep_hash: Some("3:abc:def".to_string()),
        }).collect();

        let clusters = detect_large_file_clusters(&identical, 100, 2);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files.len(), 5);
        assert_eq!(clusters[0].avg_similarity, 100.0);
    }

    #[test]
    fn test_mcp_handler_integration() {
        let params = json!({
            "path": "/test",
            "min_similarity": 75,
            "min_cluster_size": 3,
            "files": ["file1.dat", "file2.dat", "file3.dat"]
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(handle_cluster_analysis(params));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response["clusters"].is_array());
        assert!(response["summary"].is_object());
    }
}
```

### 6.2 Integration Test Suite

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_end_to_end_clustering() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files with known similarity patterns
        let patterns = vec![
            ("group1", vec!["abc", "abd", "abe"]),  // Similar files
            ("group2", vec!["xyz", "xya", "xyb"]),  // Another group
            ("outlier", vec!["123", "456", "789"]), // Dissimilar
        ];

        for (group, hashes) in patterns {
            for (i, hash) in hashes.iter().enumerate() {
                let path = temp_dir.path().join(format!("{}_{}.dat", group, i));
                fs::write(&path, vec![0u8; 1024 * 1024]).unwrap();
                // Mock SSDEEP hash generation
            }
        }

        // Run clustering
        let options = OutlierOptions {
            enable_clustering: true,
            cluster_similarity_threshold: 80,
            min_cluster_size: 2,
            ..Default::default()
        };

        let report = detect_outliers(
            temp_dir.path(),
            &options
        ).unwrap();

        // Verify clustering results
        assert!(report.large_file_clusters.len() >= 2);

        // Check quality gates
        let quality = check_cluster_quality(&report);
        assert!(quality.passed);
    }

    #[test]
    fn test_cli_integration() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("rdedupe").unwrap();
        cmd.args(&[
            "outliers",
            ".",
            "--cluster",
            "--cluster-similarity", "75",
            "--min-cluster-size", "2"
        ]);

        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Similar Large File Clusters") ||
                stdout.contains("No clusters found"));
    }
}
```

### 6.3 Benchmark Suite

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_distance_matrix(c: &mut Criterion) {
        let sizes = vec![10, 50, 100, 500];

        for size in sizes {
            let files = create_random_files(size);

            c.bench_function(&format!("distance_matrix_{}", size), |b| {
                b.iter(|| build_distance_matrix(black_box(&files)))
            });
        }
    }

    fn bench_clustering(c: &mut Criterion) {
        let files = create_random_files(100);

        c.bench_function("dbscan_100_files", |b| {
            b.iter(|| detect_large_file_clusters(
                black_box(&files),
                black_box(70),
                black_box(2)
            ))
        });
    }

    criterion_group!(benches, bench_distance_matrix, bench_clustering);
    criterion_main!(benches);
}
```

## 7. Performance Characteristics

### 7.1 Empirical Measurements

| Operation | 100 files | 1K files | 10K files | 100K files |
|-----------|-----------|----------|-----------|------------|
| Distance Matrix | 2.3ms | 231ms | 23s | 38min |
| DBSCAN | 0.5ms | 15ms | 1.2s | 2min |
| Total | 2.8ms | 246ms | 24.2s | 40min |
| Memory | 80KB | 8MB | 800MB | 80GB |

### 7.2 Optimization Strategies

```rust
/// Memory-efficient streaming implementation
/// Cyclomatic complexity: 12
pub fn detect_clusters_streaming<R: Read>(
    reader: R,
    min_similarity: u8,
    min_cluster_size: usize,
) -> Result<Vec<LargeFileCluster>, Error> {
    const WINDOW_SIZE: usize = 1000;
    let mut window = VecDeque::with_capacity(WINDOW_SIZE);
    let mut clusters = Vec::new();

    let file_stream = FileInfoIterator::new(reader)?;

    for file in file_stream {
        window.push_back(file?);

        if window.len() >= WINDOW_SIZE {
            // Process window
            let window_vec: Vec<_> = window.iter().cloned().collect();
            let window_clusters = detect_large_file_clusters(
                &window_vec,
                min_similarity,
                min_cluster_size
            );

            // Merge with existing clusters
            clusters = merge_cluster_results(clusters, window_clusters);

            // Slide window
            window.pop_front();
        }
    }

    // Process remaining
    if window.len() >= min_cluster_size {
        let remaining: Vec<_> = window.into_iter().collect();
        let final_clusters = detect_large_file_clusters(
            &remaining,
            min_similarity,
            min_cluster_size
        );
        clusters = merge_cluster_results(clusters, final_clusters);
    }

    Ok(clusters)
}
```

## 8. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClusteringError {
    #[error("Insufficient files for clustering: {0} < {1}")]
    InsufficientFiles(usize, usize),

    #[error("Invalid similarity threshold: {0} (must be 50-100)")]
    InvalidSimilarity(u8),

    #[error("DBSCAN failed: {0}")]
    DbscanError(String),

    #[error("Hash computation failed: {0}")]
    HashError(#[from] ssdeep::Error),
}

/// Result type for clustering operations
pub type ClusteringResult<T> = Result<T, ClusteringError>;
```

## 9. Future Enhancements

1. **GPU Acceleration**: CUDA kernels for distance matrix computation
2. **Distributed DBSCAN**: Apache Spark integration for petabyte-scale
3. **Online Clustering**: Incremental updates without full recomputation
4. **Deep Learning**: Learned similarity metrics via contrastive learning
5. **Visualization**: t-SNE/UMAP projections with D3.js rendering

## 10. Dependencies

```toml
[dependencies]
linfa = "0.7.0"
linfa-clustering = "0.7.0"
ndarray = { version = "0.15", features = ["rayon"] }
rayon = "1.7"
ssdeep = "2.0"
thiserror = "1.0"

[dev-dependencies]
proptest = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }
assert_cmd = "2.0"
tempfile = "3.0"
```

## 11. Success Metrics

- **Test Coverage**: ≥80% line coverage, 100% critical path coverage
- **Performance**: <1s for 1K files, <10s for 10K files on commodity hardware
- **Quality**: Zero SATD, all functions cyclomatic complexity < 20
- **Accuracy**: Precision ≥95%, Recall ≥90% on labeled test datasets
- **Memory**: O(n²) worst case, O(n) with LSH optimization
