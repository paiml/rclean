//! Basic usage example for rclean library
//!
//! This example demonstrates how to use the library API to find duplicate files.

use rclean::{PatternType, WalkOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic rclean usage example\n");
    
    // Display threading information
    println!("{}", rclean::display_thread_info());
    println!();
    
    // Example 1: Simple walk and find
    println!("Example 1: Walking current directory and finding .txt files");
    let files = rclean::walk(".")?;
    let txt_files = rclean::find(&files, ".txt");
    println!("Found {} .txt files out of {} total files", txt_files.len(), files.len());
    
    // Example 2: Using advanced pattern matching with glob
    println!("\nExample 2: Using glob pattern to find Rust files");
    let glob = rclean::Glob::new("**/*.rs")?;
    let mut builder = rclean::GlobSetBuilder::new();
    builder.add(glob);
    let globset = builder.build()?;
    let pattern = PatternType::Glob(globset);
    
    let rust_files = rclean::find_advanced(&files, &pattern);
    println!("Found {} Rust files", rust_files.len());
    
    // Example 3: Using walk with options (respecting .gitignore)
    println!("\nExample 3: Walking with options (hidden files, gitignore)");
    let walk_options = WalkOptions {
        include_hidden: true,
        respect_gitignore: false,
        respect_ignore: false,
        max_depth: Some(3),
    };
    
    let all_files = rclean::walk_with_options(".", &walk_options)?;
    println!("Found {} files (including hidden, ignoring .gitignore, max depth 3)", all_files.len());
    
    // Example 4: Finding duplicates with checksum
    println!("\nExample 4: Finding duplicate files");
    let test_files = vec![
        "Cargo.toml".to_string(),
        "README.md".to_string(),
    ];
    
    let checksums = rclean::checksum(&test_files)?;
    let duplicates = rclean::find_duplicates(checksums);
    
    if duplicates.is_empty() {
        println!("No duplicates found in the test set");
    } else {
        println!("Found {} duplicate groups:", duplicates.len());
        for (i, group) in duplicates.iter().enumerate() {
            println!("  Group {}: {:?}", i + 1, group);
        }
    }
    
    Ok(())
}