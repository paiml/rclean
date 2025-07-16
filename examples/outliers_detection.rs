//! Example demonstrating outlier detection functionality
//!
//! This example shows how to use rclean's outlier detection to find:
//! - Large files that are statistical outliers
//! - Hidden space consumers (node_modules, .git, etc.)
//! - Pattern groups (backup files, logs, etc.)

use rclean::outliers::{detect_outliers, OutlierOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RClean Outliers Detection Example\n");

    // Configure outlier detection options
    let options = OutlierOptions {
        min_size: Some(1024 * 1024), // 1MB in bytes
        top_n: Some(10),
        std_dev_threshold: 2.0,
        check_hidden_consumers: true,
        check_patterns: true,
        ..Default::default()
    };

    // Run detection on current directory
    println!("Analyzing current directory for outliers...\n");
    let report = detect_outliers(".", &options)?;

    // Display summary
    println!("=== Analysis Summary ===");
    println!("Total files analyzed: {}", report.total_files_analyzed);
    println!(
        "Total size: {:.2} GB",
        report.total_size_analyzed as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!();

    // Display large file outliers
    if !report.large_files.is_empty() {
        println!("=== Large File Outliers ===");
        for (i, outlier) in report.large_files.iter().enumerate().take(5) {
            println!("{}. {}", i + 1, outlier.path.display());
            println!("   Size: {:.2} MB", outlier.size_mb);
            println!(
                "   Percentage of total: {:.1}%",
                outlier.percentage_of_total
            );
            println!(
                "   Standard deviations from mean: {:.1}σ",
                outlier.std_devs_from_mean
            );
            println!();
        }
    }

    // Display hidden consumers
    if !report.hidden_consumers.is_empty() {
        println!("=== Hidden Space Consumers ===");
        for consumer in &report.hidden_consumers {
            println!("- {} ({})", consumer.path.display(), consumer.pattern_type);
            println!(
                "  Size: {:.2} MB in {} files",
                consumer.total_size_bytes as f64 / (1024.0 * 1024.0),
                consumer.file_count
            );
            println!("  Recommendation: {}", consumer.recommendation);
            println!();
        }
    }

    // Display pattern groups
    if !report.pattern_groups.is_empty() {
        println!("=== Pattern Groups ===");
        for group in report.pattern_groups.iter().take(5) {
            println!("- Pattern: {}", group.pattern);
            println!("  Count: {} files", group.count);
            println!(
                "  Total size: {:.2} MB",
                group.total_size_bytes as f64 / (1024.0 * 1024.0)
            );
            println!();
        }
    }

    // Example: Create DataFrame for further analysis
    let df = rclean::outliers::outliers_to_dataframe(&report)?;
    println!("=== DataFrame Summary ===");
    println!("{}", df);

    // Example: Export to CSV
    if !report.large_files.is_empty() {
        let mut df = rclean::outliers::outliers_to_dataframe(&report)?;
        rclean::generate_csv_report(&mut df, "outliers_report.csv")?;
        println!("\nOutliers report saved to: outliers_report.csv");
    }

    Ok(())
}
