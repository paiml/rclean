//! DataFrame analysis example for rclean
//!
//! This example shows how to use the DataFrame functionality for duplicate analysis.

use rclean::{PatternType, WalkOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RDedupe DataFrame Analysis Example\n");
    
    // Set up pattern matching for source files
    let pattern = PatternType::Literal("".to_string()); // Empty pattern matches all files
    
    // Configure walking options
    let walk_options = WalkOptions {
        include_hidden: false,
        respect_gitignore: true,
        respect_ignore: true,
        max_depth: None,
    };
    
    // Run deduplication with DataFrame support
    println!("Analyzing current directory for duplicates...\n");
    let df = rclean::run_with_advanced_options(
        ".",
        &pattern,
        &walk_options,
        None, // No CSV output in this example
    )?;
    
    // Display results
    println!("Analysis complete!");
    println!("Total files analyzed: {}", df.height());
    
    // Show statistics
    if let Ok(stats_df) = rclean::generate_statistics(&df) {
        println!("\n=== Statistics ===");
        println!("{}", stats_df);
    }
    
    // Validate duplicates
    if let Err(e) = rclean::validate_duplicates(&df) {
        eprintln!("Validation error: {}", e);
    }
    
    // Example of using similarity detection
    println!("\n=== Similarity Analysis ===");
    println!("Running similarity analysis with 70% threshold...");
    
    let _similar_df = rclean::run_with_similarity(
        ".",
        &pattern,
        &walk_options,
        70, // 70% similarity threshold
        None,
    )?;
    
    println!("Similarity analysis complete!");
    
    Ok(())
}