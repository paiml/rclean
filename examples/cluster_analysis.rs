//! Example demonstrating cluster analysis functionality
//!
//! This example shows how to use rclean's clustering feature to find
//! groups of similar large files using DBSCAN clustering algorithm.

use rclean::clustering::LargeFileCluster;
use rclean::outliers::{detect_outliers, OutlierOptions};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("RClean Cluster Analysis Example\n");

    // Configure outlier detection with clustering enabled
    let options = OutlierOptions {
        min_size: Some(1024 * 1024), // 1MB minimum
        top_n: Some(50),
        std_dev_threshold: 2.0,
        check_hidden_consumers: true,
        check_patterns: true,
        // Enable clustering
        enable_clustering: true,
        cluster_similarity_threshold: 70, // 70% similarity threshold
        min_cluster_size: 2,              // At least 2 files to form a cluster
        ..Default::default()
    };

    // Run outlier detection with clustering
    println!("Analyzing current directory for outliers and clusters...\n");
    let report = detect_outliers(".", &options)?;

    // Display cluster analysis results
    if !report.large_file_clusters.is_empty() {
        println!("=== Similar Large File Clusters ===");
        println!(
            "Found {} clusters of similar files\n",
            report.large_file_clusters.len()
        );

        for cluster in &report.large_file_clusters {
            display_cluster(cluster);
            println!();
        }

        // Summary statistics
        let total_clustered_files: usize = report
            .large_file_clusters
            .iter()
            .map(|c| c.files.len())
            .sum();

        let total_clustered_size: u64 = report
            .large_file_clusters
            .iter()
            .map(|c| c.total_size)
            .sum();

        println!("=== Cluster Summary ===");
        println!("Total clusters: {}", report.large_file_clusters.len());
        println!("Total files in clusters: {}", total_clustered_files);
        println!(
            "Total size in clusters: {:.2} GB",
            total_clustered_size as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        // Find the largest cluster
        if let Some(largest) = report
            .large_file_clusters
            .iter()
            .max_by_key(|c| c.total_size)
        {
            println!(
                "\nLargest cluster (ID {}): {} files, {:.2} MB total",
                largest.cluster_id,
                largest.files.len(),
                largest.total_size as f64 / (1024.0 * 1024.0)
            );
        }

        // Recommendations
        println!("\n=== Recommendations ===");
        for cluster in &report.large_file_clusters {
            if cluster.avg_similarity > 90.0 {
                println!("- Cluster {}: Files are very similar ({:.1}% avg). Consider consolidating or using version control.",
                    cluster.cluster_id, cluster.avg_similarity);
            } else if cluster.avg_similarity > 70.0 {
                println!("- Cluster {}: Files are moderately similar ({:.1}% avg). Review for potential deduplication.",
                    cluster.cluster_id, cluster.avg_similarity);
            }
        }
    } else {
        println!("No clusters of similar large files found.");
    }

    Ok(())
}

fn display_cluster(cluster: &LargeFileCluster) {
    println!(
        "Cluster {} ({} files)",
        cluster.cluster_id,
        cluster.files.len()
    );
    println!(
        "  Total size: {:.2} MB",
        cluster.total_size as f64 / (1024.0 * 1024.0)
    );
    println!("  Average similarity: {:.1}%", cluster.avg_similarity);
    println!("  Cluster density: {:.2}", cluster.density);
    println!("  Files:");

    for file in &cluster.files {
        println!(
            "    - {} ({:.2} MB)",
            file.path.display(),
            file.size_bytes as f64 / (1024.0 * 1024.0)
        );
    }
}
