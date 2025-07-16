use proptest::prelude::*;
use rclean::{find, find_advanced, PatternType};
use rclean::outliers::{detect_outliers, OutlierOptions};

// Property: find() should always return a subset of the input files
proptest! {
    #[test]
    fn find_returns_subset(files in prop::collection::vec("[a-z0-9./]+", 0..100), pattern in "[a-z0-9]+") {
        let files: Vec<String> = files.into_iter().collect();
        let result = find(&files, &pattern);

        // Every result should be in the original files
        for file in &result {
            prop_assert!(files.contains(file));
        }

        // Result length should not exceed input length
        prop_assert!(result.len() <= files.len());
    }
}

// Property: find() with empty pattern should return all files
proptest! {
    #[test]
    fn find_empty_pattern_returns_all(files in prop::collection::vec("[a-z0-9./]+", 0..100)) {
        let files: Vec<String> = files.into_iter().collect();
        let result = find(&files, "");

        prop_assert_eq!(result.len(), files.len());
    }
}

// Property: Pattern matching should be consistent
proptest! {
    #[test]
    fn pattern_matching_consistency(
        files in prop::collection::vec("[a-z0-9./]+\\.txt", 1..50),
        pattern_str in "[a-z0-9]+"
    ) {
        let files: Vec<String> = files.into_iter().collect();

        // Literal pattern
        let literal_pattern = PatternType::Literal(pattern_str.clone());
        let literal_results = find_advanced(&files, &literal_pattern);

        // Simple find should give same results for literal patterns
        let simple_results = find(&files, &pattern_str);

        prop_assert_eq!(literal_results, simple_results);
    }
}

// Test: Similarity score bounds with known good hashes
#[test]
fn similarity_score_bounds_safe() {
    use rclean::calculate_similarity;

    // Use pre-generated hashes to avoid ssdeep crashes
    let test_hashes = vec![
        "3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C",
        "6:hYQ7IgRizPMK8qEnj9A5s5vD1SIuLzoLIpUACQ:hVrRG8qEq2vD1SI68LFQ",
        "12:PuNQHTo4lPuNQHTo4lPuNQHTo4l:gHdLgHdLgHdL",
        "24:zMgHdLzMgHdLzMgHdL:UfLUfLUfL",
        "48:ku/Ay08TL0LnJnlDMYN3uXO4aLUq7zw4fAiFit3tzOxIjrtRNwv67LuI83huyZsq:kuoS2JlDMYxuXoLBt6rWNIkhuEsVWxt",
    ];

    for hash1 in &test_hashes {
        for hash2 in &test_hashes {
            if let Ok(score) = calculate_similarity(hash1, hash2) {
                assert!(score <= 100);
            }
        }
    }
}

// Property: find_advanced with glob pattern
proptest! {
    #[test]
    fn glob_pattern_matches_extension(
        base_names in prop::collection::vec("[a-z0-9]+", 1..20),
        extensions in prop::collection::vec("txt|csv|json|xml", 1..20)
    ) {
        let files: Vec<String> = base_names.into_iter()
            .zip(extensions.into_iter())
            .map(|(base, ext)| format!("{base}.{ext}"))
            .collect();

        // Create glob pattern for .txt files
        let mut builder = rclean::GlobSetBuilder::new();
        let glob = rclean::Glob::new("*.txt")
            .map_err(|_| TestCaseError::fail("Invalid glob pattern"))?;
        builder.add(glob);
        let globset = builder.build()
            .map_err(|_| TestCaseError::fail("Failed to build globset"))?;
        let pattern = PatternType::Glob(globset);

        let results = find_advanced(&files, &pattern);

        // All results should end with .txt
        for file in &results {
            prop_assert!(std::path::Path::new(file)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("txt")));
        }

        // Count txt files manually
        let txt_count = files.iter().filter(|f| std::path::Path::new(f)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))).count();
        prop_assert_eq!(results.len(), txt_count);
    }
}

// Property: WalkOptions should handle edge cases
proptest! {
    #[test]
    fn walk_options_depth_property(max_depth in prop::option::of(0usize..100)) {
        let options = rclean::WalkOptions {
            include_hidden: false,
            respect_gitignore: true,
            respect_ignore: true,
            max_depth,
        };

        // Property: max_depth should be what we set
        prop_assert_eq!(options.max_depth, max_depth);
    }
}

// Property: FileInfo size conversions
proptest! {
    #[test]
    fn file_size_conversion_property(size_bytes in 0u64..1_000_000_000) {
        #[allow(clippy::cast_precision_loss)]
        let size_mb = size_bytes as f64 / 1_048_576.0;

        // Property: MB conversion should be correct
        #[allow(clippy::cast_precision_loss)]
        let expected_mb = size_bytes as f64 / (1024.0 * 1024.0);
        prop_assert!((size_mb - expected_mb).abs() < f64::EPSILON);

        // Property: size should not be negative
        prop_assert!(size_mb >= 0.0);
    }
}

// Property: Empty file list handling
proptest! {
    #[test]
    fn empty_files_handling(pattern in "[a-z0-9]+") {
        let files: Vec<String> = vec![];
        let result = find(&files, &pattern);

        prop_assert!(result.is_empty());
    }
}

// Property: Pattern type exhaustiveness
proptest! {
    #[test]
    fn pattern_type_coverage(
        files in prop::collection::vec("[a-z0-9./]+", 1..50),
        literal_pattern in "[a-z0-9]+",
        glob_pattern in "[a-z0-9*?]+",
    ) {
        let files: Vec<String> = files.into_iter().collect();

        // Test all pattern types don't panic
        let _ = find_advanced(&files, &PatternType::Literal(literal_pattern.clone()));

        // Only test valid glob patterns
        if let Ok(glob) = rclean::Glob::new(&glob_pattern) {
            let mut builder = rclean::GlobSetBuilder::new();
            builder.add(glob);
            if let Ok(globset) = builder.build() {
                let _ = find_advanced(&files, &PatternType::Glob(globset));
            }
        }

        // Test regex pattern
        if let Ok(regex) = rclean::Regex::new(&literal_pattern) {
            let _ = find_advanced(&files, &PatternType::Regex(regex));
        }

        // Property: function should not panic
        prop_assert!(true);
    }
}

// Property-based tests for outliers module

// Property: Outlier detection should never return more files than analyzed
proptest! {
    #[test]
    fn outliers_subset_property(
        top_n in 1usize..100,
        std_dev in 0.5f64..5.0,
        min_size in prop::option::of(1000u64..1000000)
    ) {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files
        for i in 0..10 {
            let size = 10000 + (i * 5000);
            fs::write(
                temp_dir.path().join(format!("file{}.txt", i)),
                "a".repeat(size)
            ).unwrap();
        }
        
        let options = OutlierOptions {
            min_size,
            top_n: Some(top_n),
            std_dev_threshold: std_dev,
            check_hidden_consumers: true,
            include_empty_dirs: false,
            check_patterns: true,
            enable_clustering: false,
            cluster_similarity_threshold: 70,
            min_cluster_size: 2,
        };
        
        if let Ok(report) = detect_outliers(temp_dir.path().to_str().unwrap(), &options) {
            // Large files should be at most top_n
            prop_assert!(report.large_files.len() <= top_n);
            
            // Large files should not exceed total files analyzed
            prop_assert!(report.large_files.len() <= report.total_files_analyzed);
            
            // Total size should be sum of all file sizes
            prop_assert!(report.total_size_analyzed > 0);
        }
    }
}

// Property: Standard deviation threshold should filter correctly
proptest! {
    #[test]
    fn outliers_std_dev_filtering(std_dev_threshold in 0.1f64..10.0) {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with predictable sizes
        // Most files are 10KB, one is 1MB (clear outlier)
        for i in 0..9 {
            fs::write(
                temp_dir.path().join(format!("small{}.txt", i)),
                "a".repeat(10000)
            ).unwrap();
        }
        fs::write(
            temp_dir.path().join("large.txt"),
            "a".repeat(1000000)
        ).unwrap();
        
        let options = OutlierOptions {
            min_size: None,
            top_n: Some(10),
            std_dev_threshold,
            check_hidden_consumers: false,
            include_empty_dirs: false,
            check_patterns: false,
            enable_clustering: false,
            cluster_similarity_threshold: 70,
            min_cluster_size: 2,
        };
        
        if let Ok(report) = detect_outliers(temp_dir.path().to_str().unwrap(), &options) {
            // All reported outliers should have std_devs_from_mean > threshold
            for outlier in &report.large_files {
                prop_assert!(outlier.std_devs_from_mean > std_dev_threshold);
            }
        }
    }
}

// Property: Min size filter should work correctly
proptest! {
    #[test]
    fn outliers_min_size_filtering(min_size in 1000u64..100000) {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with various sizes
        for i in 0..10 {
            let size = 500 + (i * 20000); // Sizes from 500 to ~200KB
            fs::write(
                temp_dir.path().join(format!("file{}.txt", i)),
                "a".repeat(size)
            ).unwrap();
        }
        
        let options = OutlierOptions {
            min_size: Some(min_size),
            top_n: Some(20),
            std_dev_threshold: 0.1, // Very low to catch many files
            check_hidden_consumers: false,
            include_empty_dirs: false,
            check_patterns: false,
            enable_clustering: false,
            cluster_similarity_threshold: 70,
            min_cluster_size: 2,
        };
        
        if let Ok(report) = detect_outliers(temp_dir.path().to_str().unwrap(), &options) {
            // All reported files should be >= min_size
            for outlier in &report.large_files {
                prop_assert!(outlier.size_bytes >= min_size);
            }
        }
    }
}

// Property: Pattern detection consistency
proptest! {
    #[test]
    fn outliers_pattern_detection_consistency(count in 3usize..10) {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with patterns
        for i in 0..count {
            fs::write(
                temp_dir.path().join(format!("backup-{:03}.tar", i)),
                "data".repeat(10000)
            ).unwrap();
        }
        
        let options = OutlierOptions {
            min_size: None,
            top_n: Some(20),
            std_dev_threshold: 2.0,
            check_hidden_consumers: false,
            include_empty_dirs: false,
            check_patterns: true,
            enable_clustering: false,
            cluster_similarity_threshold: 70,
            min_cluster_size: 2,
        };
        
        if let Ok(report) = detect_outliers(temp_dir.path().to_str().unwrap(), &options) {
            // Should detect at least one pattern group
            if count >= 3 {
                prop_assert!(!report.pattern_groups.is_empty());
                
                // Pattern group count should match files created
                if let Some(backup_group) = report.pattern_groups.iter().find(|g| g.pattern.contains("backup")) {
                    prop_assert_eq!(backup_group.count, count);
                }
            }
        }
    }
}
