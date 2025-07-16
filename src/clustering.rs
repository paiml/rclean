//! Clustering module for detecting groups of similar files
//!
//! This module implements DBSCAN clustering to identify groups of similar large files
//! based on their SSDEEP fuzzy hashes.

use crate::outliers::SimpleFileInfo;
use ndarray::Array2;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Context for DBSCAN clustering operations
struct DbscanContext<'a> {
    epsilon: f64,
    min_points: usize,
    distances: &'a Array2<f64>,
    labels: &'a mut [Option<usize>],
    visited: &'a mut [bool],
}

/// Error types for clustering operations
#[derive(Debug, Error)]
pub enum ClusteringError {
    #[error("Insufficient files for clustering: {0} < {1}")]
    InsufficientFiles(usize, usize),

    #[error("Invalid similarity threshold: {0} (must be 50-100)")]
    InvalidSimilarity(u8),

    #[error("DBSCAN failed: {0}")]
    DbscanError(String),

    #[error("Hash computation failed: {0}")]
    HashError(String),
}

/// Result type for clustering operations
pub type ClusteringResult<T> = Result<T, ClusteringError>;

/// Represents a cluster of similar large files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LargeFileCluster {
    pub cluster_id: usize,
    pub files: Vec<SimpleFileInfo>,
    pub total_size: u64,
    pub avg_similarity: f64,
    pub density: f64,
}

/// Converts SSDEEP similarity score to distance metric
///
/// # Arguments
/// * `similarity` - SSDEEP similarity score [0, 100]
///
/// # Returns
/// Distance value where 0 = identical, 100 = completely dissimilar
pub fn similarity_to_distance(similarity: u8) -> f64 {
    100.0 - f64::from(similarity)
}

/// Builds pairwise distance matrix with parallel computation
///
/// # Arguments
/// * `files` - Slice of files with SSDEEP hashes
///
/// # Returns
/// Symmetric distance matrix
pub fn build_distance_matrix(files: &[SimpleFileInfo]) -> Array2<f64> {
    let n = files.len();
    let mut distances = Array2::zeros((n, n));

    // Parallel computation of upper triangle
    let upper_triangle: Vec<_> = (0..n)
        .into_par_iter()
        .flat_map(|i| (i + 1..n).into_par_iter().map(move |j| (i, j)))
        .map(|(i, j)| {
            let sim = calculate_similarity_safe(&files[i], &files[j]);
            (i, j, similarity_to_distance(sim))
        })
        .collect();

    // Fill symmetric matrix
    for (i, j, dist) in upper_triangle {
        distances[[i, j]] = dist;
        distances[[j, i]] = dist;
    }

    distances
}

/// Safe similarity calculation with validation
fn calculate_similarity_safe(a: &SimpleFileInfo, b: &SimpleFileInfo) -> u8 {
    match (&a.ssdeep_hash, &b.ssdeep_hash) {
        (Some(h1), Some(h2)) => {
            // Use ssdeep to calculate similarity
            ssdeep::compare(h1, h2).unwrap_or(0)
        },
        _ => 0,
    }
}

/// Performs DBSCAN clustering with configurable parameters
///
/// # Arguments
/// * `files` - Files to cluster
/// * `min_similarity` - Minimum similarity percentage for clustering (50-100)
/// * `min_cluster_size` - Minimum files to form a cluster
///
/// # Returns
/// Vector of detected clusters
pub fn detect_large_file_clusters(
    files: &[SimpleFileInfo],
    min_similarity: u8,
    min_cluster_size: usize,
) -> ClusteringResult<Vec<LargeFileCluster>> {
    if !(50..=100).contains(&min_similarity) {
        return Err(ClusteringError::InvalidSimilarity(min_similarity));
    }

    if files.len() < min_cluster_size {
        return Ok(vec![]);
    }

    // Filter files with SSDEEP hashes
    let hashable_files: Vec<_> = files
        .iter()
        .filter(|f| f.ssdeep_hash.is_some())
        .cloned()
        .collect();

    if hashable_files.len() < min_cluster_size {
        return Ok(vec![]);
    }

    let distances = build_distance_matrix(&hashable_files);
    let epsilon = similarity_to_distance(min_similarity);

    // Use custom DBSCAN implementation since linfa's API is complex
    let cluster_labels = simple_dbscan(&distances, epsilon, min_cluster_size);

    Ok(aggregate_clusters(
        &hashable_files,
        cluster_labels,
        &distances,
    ))
}

/// Simple DBSCAN implementation
fn simple_dbscan(distances: &Array2<f64>, epsilon: f64, min_points: usize) -> Vec<Option<usize>> {
    let n = distances.shape()[0];
    let mut labels = vec![None; n];
    let mut visited = vec![false; n];
    let mut cluster_id = 0;

    for i in 0..n {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        // Find neighbors within epsilon distance
        let neighbors: Vec<usize> = (0..n).filter(|&j| distances[[i, j]] <= epsilon).collect();

        if neighbors.len() >= min_points {
            // Start a new cluster
            let mut ctx = DbscanContext {
                epsilon,
                min_points,
                distances,
                labels: &mut labels,
                visited: &mut visited,
            };
            expand_cluster(i, &neighbors, cluster_id, &mut ctx);
            cluster_id += 1;
        }
    }

    labels
}

/// Expand cluster by recursively adding density-reachable points
fn expand_cluster(point: usize, neighbors: &[usize], cluster_id: usize, ctx: &mut DbscanContext) {
    ctx.labels[point] = Some(cluster_id);
    let mut seed_set = neighbors.to_vec();
    let mut i = 0;

    while i < seed_set.len() {
        let q = seed_set[i];

        if !ctx.visited[q] {
            ctx.visited[q] = true;

            // Find neighbors of q
            let q_neighbors: Vec<usize> = (0..ctx.distances.shape()[0])
                .filter(|&j| ctx.distances[[q, j]] <= ctx.epsilon)
                .collect();

            if q_neighbors.len() >= ctx.min_points {
                // Add new neighbors to seed set
                for &neighbor in &q_neighbors {
                    if !seed_set.contains(&neighbor) {
                        seed_set.push(neighbor);
                    }
                }
            }
        }

        if ctx.labels[q].is_none() {
            ctx.labels[q] = Some(cluster_id);
        }

        i += 1;
    }
}

/// Aggregates clustering results with statistics
fn aggregate_clusters(
    files: &[SimpleFileInfo],
    cluster_labels: Vec<Option<usize>>,
    distances: &Array2<f64>,
) -> Vec<LargeFileCluster> {
    let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();

    for (idx, label) in cluster_labels.iter().enumerate() {
        if let Some(cluster_id) = label {
            cluster_map.entry(*cluster_id).or_default().push(idx);
        }
    }

    cluster_map
        .into_iter()
        .map(|(id, indices)| build_cluster_info(id, &indices, files, distances))
        .collect()
}

/// Builds cluster information with metrics
fn build_cluster_info(
    cluster_id: usize,
    indices: &[usize],
    files: &[SimpleFileInfo],
    distances: &Array2<f64>,
) -> LargeFileCluster {
    let cluster_files: Vec<_> = indices.iter().map(|&i| files[i].clone()).collect();

    let total_size = cluster_files.iter().map(|f| f.size_bytes).sum();
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

/// Calculate average similarity within cluster
fn calculate_avg_similarity(indices: &[usize], distances: &Array2<f64>) -> f64 {
    if indices.len() <= 1 {
        return 100.0;
    }

    let mut total_similarity = 0.0;
    let mut count = 0;

    for i in 0..indices.len() {
        for j in i + 1..indices.len() {
            let distance = distances[[indices[i], indices[j]]];
            total_similarity += 100.0 - distance;
            count += 1;
        }
    }

    if count > 0 {
        total_similarity / count as f64
    } else {
        100.0
    }
}

/// Calculate cluster density (ratio of edges within epsilon)
fn calculate_density(indices: &[usize], distances: &Array2<f64>) -> f64 {
    if indices.len() <= 1 {
        return 1.0;
    }

    let max_edges = indices.len() * (indices.len() - 1) / 2;
    let mut edges_within_epsilon = 0;

    // Count edges with distance < 30 (similarity > 70)
    for i in 0..indices.len() {
        for j in i + 1..indices.len() {
            if distances[[indices[i], indices[j]]] < 30.0 {
                edges_within_epsilon += 1;
            }
        }
    }

    edges_within_epsilon as f64 / max_edges as f64
}

/// Locality-sensitive hashing for pre-filtering
pub fn compute_lsh_buckets(files: &[SimpleFileInfo]) -> HashMap<u64, Vec<SimpleFileInfo>> {
    let mut buckets = HashMap::new();

    for file in files {
        if let Some(hash) = &file.ssdeep_hash {
            // Extract block size and chunk for bucketing
            let parts: Vec<_> = hash.split(':').collect();
            if parts.len() >= 3 {
                let block_size = parts[0].parse::<u64>().unwrap_or(0);
                let chunk = &parts[1][..parts[1].len().min(8)];
                let bucket_key = hash_combine(block_size, chunk);
                buckets
                    .entry(bucket_key)
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }
        }
    }

    buckets
}

/// Combine hash components into bucket key
fn hash_combine(block_size: u64, chunk: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    block_size.hash(&mut hasher);
    chunk.hash(&mut hasher);
    hasher.finish()
}

/// Batch processing for memory-constrained environments
pub fn detect_clusters_batched(
    files: &[SimpleFileInfo],
    min_similarity: u8,
    min_cluster_size: usize,
    batch_size: usize,
) -> ClusteringResult<Vec<LargeFileCluster>> {
    if files.len() <= batch_size {
        return detect_large_file_clusters(files, min_similarity, min_cluster_size);
    }

    // Use LSH for pre-filtering
    let lsh_buckets = compute_lsh_buckets(files);
    let mut all_clusters = Vec::new();

    for bucket in lsh_buckets.values() {
        if bucket.len() >= min_cluster_size {
            let batch_clusters =
                detect_large_file_clusters(bucket, min_similarity, min_cluster_size)?;
            all_clusters.extend(batch_clusters);
        }
    }

    // Merge overlapping clusters
    Ok(merge_overlapping_clusters(all_clusters))
}

/// Merge clusters that share files
fn merge_overlapping_clusters(clusters: Vec<LargeFileCluster>) -> Vec<LargeFileCluster> {
    if clusters.is_empty() {
        return vec![];
    }

    let mut merged = Vec::new();
    let mut processed = HashSet::new();

    for (i, cluster) in clusters.iter().enumerate() {
        if processed.contains(&i) {
            continue;
        }

        let mut merged_cluster = cluster.clone();
        let mut file_paths: HashSet<_> = cluster.files.iter().map(|f| &f.path).collect();

        // Find overlapping clusters
        for (j, other) in clusters.iter().enumerate().skip(i + 1) {
            if processed.contains(&j) {
                continue;
            }

            let overlap = other.files.iter().any(|f| file_paths.contains(&f.path));

            if overlap {
                // Merge clusters
                for file in &other.files {
                    if file_paths.insert(&file.path) {
                        merged_cluster.files.push(file.clone());
                    }
                }
                merged_cluster.total_size += other.total_size;
                processed.insert(j);
            }
        }

        // Recalculate metrics
        merged_cluster.total_size = merged_cluster.files.iter().map(|f| f.size_bytes).sum();

        merged.push(merged_cluster);
        processed.insert(i);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file(path: &str, size: u64, hash: Option<String>) -> SimpleFileInfo {
        SimpleFileInfo {
            path: PathBuf::from(path),
            size_bytes: size,
            ssdeep_hash: hash,
        }
    }

    #[test]
    fn test_similarity_to_distance() {
        assert_eq!(similarity_to_distance(100), 0.0);
        assert_eq!(similarity_to_distance(70), 30.0);
        assert_eq!(similarity_to_distance(0), 100.0);
    }

    #[test]
    fn test_distance_matrix_symmetry() {
        let files = vec![
            create_test_file("a.txt", 1000, Some("3:abc:def".to_string())),
            create_test_file("b.txt", 1000, Some("3:abc:ghi".to_string())),
            create_test_file("c.txt", 1000, Some("3:xyz:123".to_string())),
        ];

        let matrix = build_distance_matrix(&files);

        for i in 0..files.len() {
            for j in 0..files.len() {
                assert_eq!(
                    matrix[[i, j]],
                    matrix[[j, i]],
                    "Distance matrix must be symmetric"
                );
            }
        }
    }

    #[test]
    fn test_cluster_detection_edge_cases() {
        // Empty input
        assert_eq!(detect_large_file_clusters(&[], 70, 2).unwrap().len(), 0);

        // Single file
        let single = vec![create_test_file(
            "single.txt",
            1000,
            Some("3:abc:def".to_string()),
        )];
        assert_eq!(detect_large_file_clusters(&single, 70, 2).unwrap().len(), 0);

        // Files without hashes
        let no_hashes = vec![
            create_test_file("a.txt", 1000, None),
            create_test_file("b.txt", 1000, None),
        ];
        assert_eq!(
            detect_large_file_clusters(&no_hashes, 70, 2).unwrap().len(),
            0
        );
    }

    #[test]
    fn test_invalid_similarity_threshold() {
        let files = vec![create_test_file(
            "a.txt",
            1000,
            Some("3:abc:def".to_string()),
        )];

        assert!(matches!(
            detect_large_file_clusters(&files, 49, 2),
            Err(ClusteringError::InvalidSimilarity(49))
        ));

        assert!(matches!(
            detect_large_file_clusters(&files, 101, 2),
            Err(ClusteringError::InvalidSimilarity(101))
        ));
    }

    #[test]
    fn test_lsh_buckets() {
        let files = vec![
            create_test_file("a.txt", 1000, Some("3:abc123:def456".to_string())),
            create_test_file("b.txt", 1000, Some("3:abc123:ghi789".to_string())),
            create_test_file("c.txt", 1000, Some("6:xyz123:123456".to_string())),
        ];

        let buckets = compute_lsh_buckets(&files);

        // Files with same block size and similar chunks should be in same bucket
        assert!(!buckets.is_empty());
    }

    #[test]
    fn test_cluster_aggregation() {
        let files = vec![
            create_test_file("a.txt", 1000, Some("3:abc:def".to_string())),
            create_test_file("b.txt", 2000, Some("3:abc:def".to_string())),
            create_test_file("c.txt", 3000, Some("3:xyz:123".to_string())),
        ];

        let distances = build_distance_matrix(&files);
        let cluster_labels = vec![Some(0), Some(0), Some(1)];

        let clusters = aggregate_clusters(&files, cluster_labels, &distances);

        assert_eq!(clusters.len(), 2);

        // Sort clusters by size to ensure consistent ordering
        let mut sorted_clusters = clusters.clone();
        sorted_clusters.sort_by_key(|c| c.files.len());

        assert_eq!(sorted_clusters[1].files.len(), 2); // Cluster with 2 files
        assert_eq!(sorted_clusters[1].total_size, 3000); // 1000 + 2000
        assert_eq!(sorted_clusters[0].files.len(), 1); // Cluster with 1 file
        assert_eq!(sorted_clusters[0].total_size, 3000); // 3000
    }

    #[test]
    fn test_dbscan_implementation() {
        // Create a simple distance matrix
        let mut distances = Array2::zeros((4, 4));
        // Group 1: points 0 and 1 are close
        distances[[0, 1]] = 10.0;
        distances[[1, 0]] = 10.0;
        // Group 2: points 2 and 3 are close
        distances[[2, 3]] = 10.0;
        distances[[3, 2]] = 10.0;
        // Groups are far apart
        distances[[0, 2]] = 90.0;
        distances[[0, 3]] = 90.0;
        distances[[1, 2]] = 90.0;
        distances[[1, 3]] = 90.0;
        distances[[2, 0]] = 90.0;
        distances[[3, 0]] = 90.0;
        distances[[2, 1]] = 90.0;
        distances[[3, 1]] = 90.0;

        let labels = simple_dbscan(&distances, 20.0, 2);

        // Should have two clusters
        assert!(labels[0].is_some());
        assert!(labels[1].is_some());
        assert!(labels[2].is_some());
        assert!(labels[3].is_some());
        assert_eq!(labels[0], labels[1]); // Same cluster
        assert_eq!(labels[2], labels[3]); // Same cluster
        assert_ne!(labels[0], labels[2]); // Different clusters
    }

    #[test]
    fn test_cluster_density_calculation() {
        let indices = vec![0, 1, 2];
        let mut distances = Array2::zeros((3, 3));
        // All points very close (high density)
        distances[[0, 1]] = 10.0;
        distances[[1, 0]] = 10.0;
        distances[[0, 2]] = 10.0;
        distances[[2, 0]] = 10.0;
        distances[[1, 2]] = 10.0;
        distances[[2, 1]] = 10.0;

        let density = calculate_density(&indices, &distances);
        assert!(density > 0.9); // Should be close to 1.0 for dense cluster
    }

    #[test]
    fn test_average_similarity_calculation() {
        let indices = vec![0, 1, 2];
        let mut distances = Array2::zeros((3, 3));
        // Set known distances
        distances[[0, 1]] = 20.0; // 80% similarity
        distances[[1, 0]] = 20.0;
        distances[[0, 2]] = 30.0; // 70% similarity
        distances[[2, 0]] = 30.0;
        distances[[1, 2]] = 10.0; // 90% similarity
        distances[[2, 1]] = 10.0;

        let avg_sim = calculate_avg_similarity(&indices, &distances);
        // Average of 80, 70, 90 = 80
        assert!((avg_sim - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_merge_overlapping_clusters() {
        let cluster1 = LargeFileCluster {
            cluster_id: 0,
            files: vec![
                create_test_file("a.txt", 1000, Some("3:abc:def".to_string())),
                create_test_file("b.txt", 2000, Some("3:abc:def".to_string())),
            ],
            total_size: 3000,
            avg_similarity: 90.0,
            density: 0.8,
        };

        let cluster2 = LargeFileCluster {
            cluster_id: 1,
            files: vec![
                create_test_file("b.txt", 2000, Some("3:abc:def".to_string())), // Overlapping file
                create_test_file("c.txt", 3000, Some("3:abc:ghi".to_string())),
            ],
            total_size: 5000,
            avg_similarity: 85.0,
            density: 0.7,
        };

        let merged = merge_overlapping_clusters(vec![cluster1, cluster2]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].files.len(), 3); // a, b, c (no duplicates)
        assert_eq!(merged[0].total_size, 6000); // Recalculated
    }

    #[test]
    fn test_batch_clustering() {
        let files: Vec<_> = (0..10)
            .map(|i| {
                create_test_file(
                    &format!("file{}.txt", i),
                    1000 * (i as u64 + 1),
                    Some(format!("3:abc{}:def", i % 3)),
                )
            })
            .collect();

        let result = detect_clusters_batched(&files, 70, 2, 5);
        assert!(result.is_ok());
        let clusters = result.unwrap();

        // Should have found some clusters based on the pattern
        assert!(!clusters.is_empty());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generate arbitrary file sets for testing
    prop_compose! {
        fn arb_file_set()(
            count in 2..50usize,
            base_hash in "[0-9a-f]{6}",
        ) -> Vec<SimpleFileInfo> {
            (0..count).map(|i| {
                SimpleFileInfo {
                    path: std::path::PathBuf::from(format!("file_{}.dat", i)),
                    size_bytes: 1024 * (i as u64 + 1),
                    ssdeep_hash: Some(format!("3:{}:{}", base_hash, i % 5)),
                }
            }).collect()
        }
    }

    proptest! {
        // Property: All files in a cluster have similarity >= threshold
        #[test]
        fn prop_cluster_similarity_invariant(
            files in arb_file_set(),
            min_sim in 50..100u8,
        ) {
            let result = detect_large_file_clusters(&files, min_sim, 2);

            if let Ok(clusters) = result {
                for cluster in clusters {
                    for i in 0..cluster.files.len() {
                        for j in i+1..cluster.files.len() {
                            if let (Some(h1), Some(h2)) = (&cluster.files[i].ssdeep_hash, &cluster.files[j].ssdeep_hash) {
                                if let Ok(sim) = ssdeep::compare(h1, h2) {
                                    prop_assert!(sim >= min_sim,
                                        "Files in cluster must have similarity {} >= {}",
                                        sim, min_sim);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Property: No file appears in multiple clusters
        #[test]
        fn prop_cluster_disjoint_invariant(
            files in arb_file_set(),
            min_sim in 50..100u8,
        ) {
            let result = detect_large_file_clusters(&files, min_sim, 2);

            if let Ok(clusters) = result {
                let mut seen = HashSet::new();

                for cluster in &clusters {
                    for file in &cluster.files {
                        prop_assert!(seen.insert(&file.path),
                            "File {:?} appears in multiple clusters", file.path);
                    }
                }
            }
        }

        // Property: Cluster size respects minimum
        #[test]
        fn prop_minimum_cluster_size(
            files in arb_file_set(),
            min_sim in 50..100u8,
            min_size in 2..10usize,
        ) {
            let result = detect_large_file_clusters(&files, min_sim, min_size);

            if let Ok(clusters) = result {
                for cluster in clusters {
                    prop_assert!(cluster.files.len() >= min_size,
                        "Cluster size {} < minimum {}",
                        cluster.files.len(), min_size);
                }
            }
        }

        // Property: Distance matrix is symmetric
        #[test]
        fn prop_distance_matrix_symmetric(files in arb_file_set()) {
            let matrix = build_distance_matrix(&files);
            let n = files.len();

            for i in 0..n {
                for j in 0..n {
                    prop_assert_eq!(
                        matrix[[i, j]], matrix[[j, i]],
                        "Distance matrix must be symmetric at [{}, {}]", i, j
                    );
                }
            }
        }

        // Property: Similarity to distance conversion is monotonic
        #[test]
        fn prop_similarity_distance_monotonic(s1 in 0..=100u8, s2 in 0..=100u8) {
            let d1 = similarity_to_distance(s1);
            let d2 = similarity_to_distance(s2);

            if s1 < s2 {
                prop_assert!(d1 > d2, "Higher similarity must yield lower distance");
            } else if s1 > s2 {
                prop_assert!(d1 < d2, "Lower similarity must yield higher distance");
            } else {
                prop_assert_eq!(d1, d2, "Equal similarity must yield equal distance");
            }
        }
    }
}
