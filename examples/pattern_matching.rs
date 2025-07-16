//! Pattern matching examples for rclean
//!
//! This example demonstrates the different pattern matching capabilities.

use rclean::{Glob, GlobSetBuilder, PatternType, Regex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RDedupe Pattern Matching Examples\n");

    // Sample file list
    let files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
        "tests/test_utils.rs".to_string(),
        "examples/basic.rs".to_string(),
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "build.rs".to_string(),
        ".gitignore".to_string(),
        "docs/guide.md".to_string(),
    ];

    // Example 1: Literal pattern matching
    println!("Example 1: Literal pattern matching");
    let literal_pattern = PatternType::Literal("src".to_string());
    let matches = rclean::find_advanced(&files, &literal_pattern);
    println!("Files containing 'src': {:?}", matches);

    // Example 2: Glob pattern matching
    println!("\nExample 2: Glob pattern matching");
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("*.rs")?);
    builder.add(Glob::new("**/*.md")?);
    let globset = builder.build()?;
    let glob_pattern = PatternType::Glob(globset);
    let matches = rclean::find_advanced(&files, &glob_pattern);
    println!("Files matching '*.rs' or '**/*.md': {:?}", matches);

    // Example 3: Regex pattern matching
    println!("\nExample 3: Regex pattern matching");
    let regex = Regex::new(r"^(src|tests)/.*\.rs$")?;
    let regex_pattern = PatternType::Regex(regex);
    let matches = rclean::find_advanced(&files, &regex_pattern);
    println!(
        "Files matching regex '^(src|tests)/.*\\.rs$': {:?}",
        matches
    );

    // Example 4: Complex glob patterns
    println!("\nExample 4: Complex glob patterns");
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("**/test*.rs")?);
    builder.add(Glob::new("!.gitignore")?); // Note: negation in globs might not work as expected
    let globset = builder.build()?;
    let pattern = PatternType::Glob(globset);
    let matches = rclean::find_advanced(&files, &pattern);
    println!("Files matching '**/test*.rs': {:?}", matches);

    // Example 5: Case sensitivity
    println!("\nExample 5: Case sensitivity in patterns");
    let pattern1 = PatternType::Literal("README".to_string());
    let pattern2 = PatternType::Literal("readme".to_string());
    let matches1 = rclean::find_advanced(&files, &pattern1);
    let matches2 = rclean::find_advanced(&files, &pattern2);
    println!("Files containing 'README': {:?}", matches1);
    println!("Files containing 'readme': {:?}", matches2);

    Ok(())
}
