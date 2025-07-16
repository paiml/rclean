//! Comprehensive tests for outliers detection functionality

use rclean::outliers::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_outlier_options_default() {
    let options = OutlierOptions::default();
    assert_eq!(options.top_n, Some(20));
    assert_eq!(options.std_dev_threshold, 2.0);
    assert!(options.check_hidden_consumers);
    assert!(options.check_patterns);
    assert!(!options.include_empty_dirs);
    assert!(!options.enable_clustering);
    assert_eq!(options.cluster_similarity_threshold, 70);
    assert_eq!(options.min_cluster_size, 2);
}

#[test]
fn test_outlier_options_custom() {
    let options = OutlierOptions {
        min_size: Some(1024 * 1024), // 1MB
        top_n: Some(10),
        std_dev_threshold: 3.0,
        check_hidden_consumers: false,
        include_empty_dirs: true,
        check_patterns: false,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    assert_eq!(options.min_size, Some(1024 * 1024));
    assert_eq!(options.top_n, Some(10));
    assert_eq!(options.std_dev_threshold, 3.0);
    assert!(!options.check_hidden_consumers);
    assert!(options.include_empty_dirs);
    assert!(!options.check_patterns);
    assert!(!options.enable_clustering);
    assert_eq!(options.cluster_similarity_threshold, 70);
    assert_eq!(options.min_cluster_size, 2);
}

#[test]
fn test_detect_outliers_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let options = OutlierOptions::default();
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    assert_eq!(report.large_files.len(), 0);
    assert_eq!(report.hidden_consumers.len(), 0);
    assert_eq!(report.pattern_groups.len(), 0);
    assert_eq!(report.large_file_clusters.len(), 0);
    assert_eq!(report.total_files_analyzed, 0);
    assert_eq!(report.total_size_analyzed, 0);
}

#[test]
fn test_detect_large_file_outliers() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files with varying sizes (all large enough to avoid ssdeep issues)
    fs::write(temp_dir.path().join("small1.txt"), "a".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("small2.txt"), "a".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("small3.txt"), "a".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("medium.txt"), "a".repeat(50000)).unwrap();
    fs::write(temp_dir.path().join("large.txt"), "a".repeat(500000)).unwrap();
    
    let options = OutlierOptions {
        min_size: None,
        top_n: Some(5),
        std_dev_threshold: 1.5,
        check_hidden_consumers: false,
        include_empty_dirs: false,
        check_patterns: false,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    assert!(!report.large_files.is_empty());
    assert!(report.large_files[0].path.to_string_lossy().contains("large.txt"));
    assert!(report.large_files[0].size_bytes > 400000);
    assert!(report.large_files[0].std_devs_from_mean > 1.5);
}

#[test]
fn test_detect_hidden_consumers() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a node_modules directory
    let node_modules = temp_dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("package1.js"), "a".repeat(50000)).unwrap();
    fs::write(node_modules.join("package2.js"), "a".repeat(50000)).unwrap();
    
    // Create a .git directory
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("objects"), "a".repeat(100000)).unwrap();
    
    let options = OutlierOptions {
        min_size: None,
        top_n: Some(10),
        std_dev_threshold: 2.0,
        check_hidden_consumers: true,
        include_empty_dirs: false,
        check_patterns: false,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    assert!(!report.hidden_consumers.is_empty());
    let node_modules_found = report.hidden_consumers.iter()
        .any(|c| c.pattern_type.contains("Node.js"));
    assert!(node_modules_found);
}

#[test]
fn test_detect_pattern_groups() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files with patterns
    fs::write(temp_dir.path().join("backup-001.tar"), "data".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("backup-002.tar"), "data".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("backup-003.tar"), "data".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("backup-004.tar"), "data".repeat(10000)).unwrap();
    
    fs::write(temp_dir.path().join("log-2024-01-01.txt"), "log".repeat(5000)).unwrap();
    fs::write(temp_dir.path().join("log-2024-01-02.txt"), "log".repeat(5000)).unwrap();
    fs::write(temp_dir.path().join("log-2024-01-03.txt"), "log".repeat(5000)).unwrap();
    
    let options = OutlierOptions {
        min_size: None,
        top_n: Some(10),
        std_dev_threshold: 2.0,
        check_hidden_consumers: false,
        include_empty_dirs: false,
        check_patterns: true,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    assert!(!report.pattern_groups.is_empty(), "Expected at least 1 pattern group, but found {}", report.pattern_groups.len());
    let backup_pattern_found = report.pattern_groups.iter()
        .any(|g| g.pattern.contains("backup"));
    assert!(backup_pattern_found, "Expected to find a backup pattern");
}

#[test]
fn test_outliers_to_dataframe() {
    let report = OutlierReport {
        large_files: vec![
            LargeFileOutlier {
                path: std::path::PathBuf::from("/tmp/test1.txt"),
                size_bytes: 1024 * 1024 * 10, // 10MB
                size_mb: 10.0,
                percentage_of_total: 50.0,
                std_devs_from_mean: 3.2,
            },
            LargeFileOutlier {
                path: std::path::PathBuf::from("/tmp/test2.txt"),
                size_bytes: 1024 * 1024 * 5, // 5MB
                size_mb: 5.0,
                percentage_of_total: 25.0,
                std_devs_from_mean: 2.1,
            },
        ],
        hidden_consumers: vec![],
        pattern_groups: vec![],
        large_file_clusters: vec![],
        total_size_analyzed: 1024 * 1024 * 20, // 20MB
        total_files_analyzed: 10,
    };
    
    let df = outliers_to_dataframe(&report).unwrap();
    
    assert_eq!(df.height(), 2);
    assert_eq!(df.width(), 4);
    assert!(df.column("file_path").is_ok());
    assert!(df.column("size_mb").is_ok());
    assert!(df.column("percentage_of_total").is_ok());
    assert!(df.column("std_devs_from_mean").is_ok());
}

#[test]
fn test_outliers_to_dataframe_empty() {
    let report = OutlierReport {
        large_files: vec![],
        hidden_consumers: vec![],
        pattern_groups: vec![],
        large_file_clusters: vec![],
        total_size_analyzed: 0,
        total_files_analyzed: 0,
    };
    
    let df = outliers_to_dataframe(&report).unwrap();
    
    assert_eq!(df.height(), 0);
    assert_eq!(df.width(), 4);
}

#[test]
fn test_min_size_filter() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files of different sizes
    fs::write(temp_dir.path().join("tiny.txt"), "a".repeat(1000)).unwrap();
    fs::write(temp_dir.path().join("small.txt"), "a".repeat(10000)).unwrap();
    fs::write(temp_dir.path().join("large.txt"), "a".repeat(100000)).unwrap();
    
    let options = OutlierOptions {
        min_size: Some(5000), // Only consider files > 5KB
        top_n: Some(10),
        std_dev_threshold: 0.5, // Low threshold to catch more files
        check_hidden_consumers: false,
        include_empty_dirs: false,
        check_patterns: false,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    // Only the large file should be considered
    for outlier in &report.large_files {
        assert!(outlier.size_bytes >= 5000);
    }
}

#[test]
fn test_top_n_limiting() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create many files
    for i in 0..10 {
        let size = 10000 + (i * 10000);
        fs::write(
            temp_dir.path().join(format!("file{}.txt", i)),
            "a".repeat(size),
        ).unwrap();
    }
    
    let options = OutlierOptions {
        min_size: None,
        top_n: Some(3), // Limit to top 3
        std_dev_threshold: 0.1, // Very low threshold
        check_hidden_consumers: false,
        include_empty_dirs: false,
        check_patterns: false,
        enable_clustering: false,
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };
    
    let report = detect_outliers(temp_dir.path().to_str().unwrap(), &options).unwrap();
    
    assert!(report.large_files.len() <= 3);
    
    // Verify they are sorted by size (largest first)
    for i in 1..report.large_files.len() {
        assert!(report.large_files[i-1].size_bytes >= report.large_files[i].size_bytes);
    }
}

// Pattern detection functions are tested implicitly through
// test_detect_pattern_groups above

#[test]
fn test_error_handling_nonexistent_path() {
    let options = OutlierOptions::default();
    
    let result = detect_outliers("/nonexistent/path/that/does/not/exist", &options);
    
    // Should handle gracefully and return ok with empty results
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.total_files_analyzed, 0);
}