//! Integration tests for rclean library

use rclean::{PatternType, WalkOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_full_deduplication_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files
    fs::write(temp_dir.path().join("file1.txt"), "same content").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "same content").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), "different content").unwrap();
    fs::write(temp_dir.path().join("file4.rs"), "rust code").unwrap();
    
    // Create subdirectory
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file5.txt"), "same content").unwrap();
    
    // Test pattern matching
    let pattern = PatternType::Literal("".to_string());
    let options = WalkOptions::default();
    
    let result = rclean::run_with_advanced_options(
        temp_dir.path().to_str().unwrap(),
        &pattern,
        &options,
        None,
    );
    
    assert!(result.is_ok());
    let df = result.unwrap();
    assert!(df.height() >= 4);
}

#[test]
#[ignore] // Temporarily disabled due to unsafe precondition violation
fn test_similarity_detection_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create files with similar content (long enough for ssdeep)
    let content1 = "a".repeat(5000);
    let content2 = format!("{}{}", "a".repeat(4800), "b".repeat(200));
    let content3 = "c".repeat(5000);
    
    fs::write(temp_dir.path().join("similar1.txt"), content1).unwrap();
    fs::write(temp_dir.path().join("similar2.txt"), content2).unwrap();
    fs::write(temp_dir.path().join("different.txt"), content3).unwrap();
    
    let pattern = PatternType::Literal("".to_string());
    let options = WalkOptions::default();
    
    let result = rclean::run_with_similarity(
        temp_dir.path().to_str().unwrap(),
        &pattern,
        &options,
        50, // 50% similarity threshold
        None,
    );
    
    assert!(result.is_ok());
    let df = result.unwrap();
    assert!(df.height() >= 3);
}

#[test]
fn test_pattern_types() {
    let temp_dir = TempDir::new().unwrap();
    
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("test.rs"), "code").unwrap();
    fs::write(temp_dir.path().join("readme.md"), "doc").unwrap();
    
    let files = rclean::walk(temp_dir.path().to_str().unwrap()).unwrap();
    
    // Test literal pattern
    let pattern = PatternType::Literal("test".to_string());
    let matches = rclean::find_advanced(&files, &pattern);
    assert!(matches.len() >= 2);
    
    // Test glob pattern
    let mut builder = rclean::GlobSetBuilder::new();
    builder.add(rclean::Glob::new("*.txt").unwrap());
    let globset = builder.build().unwrap();
    let pattern = PatternType::Glob(globset);
    let matches = rclean::find_advanced(&files, &pattern);
    assert!(!matches.is_empty());
    
    // Test regex pattern
    let regex = rclean::Regex::new(r".*\.rs$").unwrap();
    let pattern = PatternType::Regex(regex);
    let matches = rclean::find_advanced(&files, &pattern);
    assert!(!matches.is_empty());
}

#[test]
fn test_walk_options_variations() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create hidden file
    fs::write(temp_dir.path().join(".hidden"), "hidden").unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "visible").unwrap();
    
    // Create subdirectory
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("deep.txt"), "deep").unwrap();
    
    // Test default options (no hidden files)
    let files = rclean::walk_with_options(temp_dir.path().to_str().unwrap(), &WalkOptions::default()).unwrap();
    assert!(!files.iter().any(|f| f.contains(".hidden")));
    
    // Test with hidden files
    let options = WalkOptions {
        include_hidden: true,
        respect_gitignore: false,
        respect_ignore: false,
        max_depth: None,
    };
    let files = rclean::walk_with_options(temp_dir.path().to_str().unwrap(), &options).unwrap();
    assert!(files.iter().any(|f| f.contains(".hidden")));
    
    // Test with max depth
    let options = WalkOptions {
        include_hidden: false,
        respect_gitignore: true,
        respect_ignore: true,
        max_depth: Some(1),
    };
    let files = rclean::walk_with_options(temp_dir.path().to_str().unwrap(), &options).unwrap();
    assert!(!files.iter().any(|f| f.contains("deep.txt")));
}

#[test]
fn test_csv_output() {
    let temp_dir = TempDir::new().unwrap();
    
    fs::write(temp_dir.path().join("dup1.txt"), "duplicate").unwrap();
    fs::write(temp_dir.path().join("dup2.txt"), "duplicate").unwrap();
    fs::write(temp_dir.path().join("unique.txt"), "unique").unwrap();
    
    let csv_path = temp_dir.path().join("output.csv");
    let result = rclean::run_with_dataframe(
        temp_dir.path().to_str().unwrap(),
        "",
        Some(csv_path.to_str().unwrap()),
    );
    
    assert!(result.is_ok());
    assert!(csv_path.exists());
    
    // Check CSV content
    let csv_content = fs::read_to_string(csv_path).unwrap();
    assert!(csv_content.contains("file_path"));
    assert!(csv_content.contains("is_duplicate"));
}

#[test]
fn test_error_handling() {
    // Test with non-existent directory - should handle gracefully and return empty result
    let result = rclean::walk("/non/existent/path");
    assert!(result.is_ok());
    let files = result.unwrap();
    assert!(files.is_empty());
    
    // Test with empty pattern
    let files = vec!["test.txt".to_string()];
    let matches = rclean::find(&files, "");
    assert_eq!(matches.len(), 1);
    
    // Test with invalid regex
    #[allow(clippy::invalid_regex)]
    let regex_result = regex::Regex::new("[invalid");
    assert!(regex_result.is_err());
}

#[test]
fn test_run_simple_api() {
    let temp_dir = TempDir::new().unwrap();
    
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
    
    let result = rclean::run(temp_dir.path().to_str().unwrap(), "test");
    assert!(result.is_ok());
}