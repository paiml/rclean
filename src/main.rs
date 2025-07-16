//! Command-line interface for the rclean disk cleanup tool.
//!
//! This tool provides efficient duplicate file detection and storage outlier analysis
//! using parallel processing to help clean up disk space.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::io::IsTerminal;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Common search options shared by commands
struct SearchOptions {
    path: String,
    pattern: String,
    pattern_type: PatternTypeArg,
    hidden: bool,
    no_ignore: bool,
    max_depth: Option<usize>,
}

/// Outlier detection parameters
struct OutlierParams {
    path: String,
    min_size: Option<String>,
    top: usize,
    std_dev: f64,
    check_hidden: bool,
    check_patterns: bool,
    cluster: bool,
    cluster_similarity: u8,
    min_cluster_size: usize,
    format: OutputFormat,
    csv: Option<String>,
}

#[derive(Parser)]
//add extended help
#[clap(
    name = "rclean",
    version = "0.1.1",
    author = "Noah Gift",
    about = "A disk cleanup tool that finds duplicates and storage outliers",
    after_help = "Examples:\n  rclean /path/to/directory                       # Find duplicate files\n  rclean ~/Documents --pattern '*.pdf' --pattern-type glob\n  rclean . --csv report.csv\n  rclean ~/Documents --similarity 70              # Find similar files\n  rclean search /path --pattern '*.txt'\n  rclean count ~/Documents\n  rclean outliers /path --min-size 100MB         # Find large file outliers\n  rclean outliers ~ --check-hidden --format json # Find hidden space consumers"
)]
struct Cli {
    /// Path to scan for duplicates
    #[clap(default_value = ".")]
    path: String,

    /// Pattern to match files
    #[clap(long, default_value = "")]
    pattern: String,

    /// Pattern type for matching
    #[clap(long, value_enum, default_value = "literal")]
    pattern_type: PatternTypeArg,

    /// Include hidden files
    #[clap(long)]
    hidden: bool,

    /// Ignore .gitignore rules
    #[clap(long)]
    no_ignore: bool,

    /// Maximum depth to traverse
    #[clap(long)]
    max_depth: Option<usize>,

    /// Generate detailed CSV report
    #[clap(long)]
    csv: Option<String>,

    /// Find similar files (fuzzy matching), value is similarity threshold 0-100
    #[clap(long)]
    similarity: Option<u32>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Pattern matching type for CLI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum PatternTypeArg {
    /// Simple string contains matching
    Literal,
    /// Glob pattern matching (e.g., *.txt, **/*.rs)
    Glob,
    /// Regular expression matching
    Regex,
}

#[derive(Parser)]
enum Commands {
    Search {
        /// Path to search in
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
        #[clap(long, value_enum, default_value = "literal")]
        pattern_type: PatternTypeArg,
        #[clap(long, help = "Include hidden files")]
        hidden: bool,
        #[clap(long, help = "Ignore .gitignore rules")]
        no_ignore: bool,
        #[clap(long, help = "Maximum depth to traverse")]
        max_depth: Option<usize>,
    },

    Dedupe {
        /// Path to scan for duplicates
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
        #[clap(long, value_enum, default_value = "literal")]
        pattern_type: PatternTypeArg,
        #[clap(long, help = "Include hidden files")]
        hidden: bool,
        #[clap(long, help = "Ignore .gitignore rules")]
        no_ignore: bool,
        #[clap(long, help = "Maximum depth to traverse")]
        max_depth: Option<usize>,
        #[clap(long, help = "Generate detailed CSV report")]
        csv: Option<String>,
        #[clap(
            long,
            help = "Find similar files (fuzzy matching), value is similarity threshold 0-100"
        )]
        similarity: Option<u32>,
    },

    //create count with path and pattern defaults for both
    Count {
        /// Path to count files in
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
        #[clap(long, value_enum, default_value = "literal")]
        pattern_type: PatternTypeArg,
        #[clap(long, help = "Include hidden files")]
        hidden: bool,
        #[clap(long, help = "Ignore .gitignore rules")]
        no_ignore: bool,
        #[clap(long, help = "Maximum depth to traverse")]
        max_depth: Option<usize>,
    },

    Outliers {
        /// Path to analyze for outliers
        path: String,
        #[clap(long, help = "Minimum file size to consider (e.g., 100MB, 1GB)")]
        min_size: Option<String>,
        #[clap(long, help = "Number of top outliers to show", default_value = "20")]
        top: usize,
        #[clap(long, help = "Standard deviations from mean to consider as outlier", default_value = "2.0")]
        std_dev: f64,
        #[clap(long, help = "Check for hidden space consumers (node_modules, .git, etc.)")]
        check_hidden: bool,
        #[clap(long, help = "Check for file patterns (backups, logs, etc.)")]
        check_patterns: bool,
        #[clap(long, help = "Enable clustering of similar large files")]
        cluster: bool,
        #[clap(long, default_value_t = 70, help = "Similarity threshold for clustering (50-100)")]
        cluster_similarity: u8,
        #[clap(long, default_value_t = 2, help = "Minimum files to form a cluster")]
        min_cluster_size: usize,
        #[clap(long, help = "Output format", value_enum, default_value = "table")]
        format: OutputFormat,
        #[clap(long, help = "Export results to CSV")]
        csv: Option<String>,
    },
}

/// Output format for results
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Table format (default)
    Table,
    /// JSON format
    Json,
    /// Simple text format
    Text,
}

/// Convert CLI pattern type to library pattern type.
fn create_pattern(
    pattern: &str,
    pattern_type: PatternTypeArg,
) -> Result<rclean::PatternType, Box<dyn std::error::Error>> {
    match pattern_type {
        PatternTypeArg::Literal => Ok(rclean::PatternType::Literal(pattern.to_string())),
        PatternTypeArg::Glob => {
            let glob = globset::Glob::new(pattern)?;
            let mut builder = globset::GlobSetBuilder::new();
            builder.add(glob);
            let globset = builder.build()?;
            Ok(rclean::PatternType::Glob(globset))
        },
        PatternTypeArg::Regex => {
            let regex = regex::Regex::new(pattern)?;
            Ok(rclean::PatternType::Regex(regex))
        },
    }
}

/// Create walk options from CLI arguments.
const fn create_walk_options(
    hidden: bool,
    no_ignore: bool,
    max_depth: Option<usize>,
) -> rclean::WalkOptions {
    rclean::WalkOptions {
        include_hidden: hidden,
        respect_gitignore: !no_ignore,
        respect_ignore: !no_ignore,
        max_depth,
    }
}

fn handle_search(options: &SearchOptions) {
    println!(
        "Searching for files in {} matching {}",
        options.path, options.pattern
    );

    let walk_options = create_walk_options(options.hidden, options.no_ignore, options.max_depth);
    match rclean::walk_with_options(&options.path, &walk_options) {
        Ok(files) => match create_pattern(&options.pattern, options.pattern_type) {
            Ok(pattern_matcher) => {
                let files = rclean::find_advanced(&files, &pattern_matcher);
                println!("Found {} files matching pattern", files.len());
                for file in files {
                    println!("{file}");
                }
            },
            Err(e) => eprintln!("Error creating pattern: {e}"),
        },
        Err(e) => eprintln!("Error walking directory: {e}"),
    }
}

fn handle_dedupe(options: &SearchOptions, csv: Option<&str>, similarity: Option<u32>) {
    println!("{}", rclean::display_thread_info());
    println!(
        "Analyzing files in {} matching '{}'",
        options.path, options.pattern
    );

    let walk_options = create_walk_options(options.hidden, options.no_ignore, options.max_depth);
    match create_pattern(&options.pattern, options.pattern_type) {
        Ok(pattern_matcher) => {
            let result = similarity.map_or_else(
                || {
                    rclean::run_with_advanced_options(
                        &options.path,
                        &pattern_matcher,
                        &walk_options,
                        csv,
                    )
                },
                |threshold| {
                    rclean::run_with_similarity(
                        &options.path,
                        &pattern_matcher,
                        &walk_options,
                        threshold,
                        csv,
                    )
                },
            );

            match result {
                Ok(df) => {
                    println!("\n=== Analysis Complete ===");
                    println!("Total files analyzed: {}", df.height());
                    if let Some(csv_path) = csv {
                        println!("Detailed CSV report saved to: {csv_path}");
                    }
                },
                Err(e) => println!("Error: {e}"),
            }
        },
        Err(e) => eprintln!("Error creating pattern: {e}"),
    }
}

fn handle_count(options: &SearchOptions) {
    println!(
        "Counting files in {} matching {}",
        options.path, options.pattern
    );

    let walk_options = create_walk_options(options.hidden, options.no_ignore, options.max_depth);
    match rclean::walk_with_options(&options.path, &walk_options) {
        Ok(files) => match create_pattern(&options.pattern, options.pattern_type) {
            Ok(pattern_matcher) => {
                let files = rclean::find_advanced(&files, &pattern_matcher);
                println!("Found {} files matching pattern", files.len());
            },
            Err(e) => eprintln!("Error creating pattern: {e}"),
        },
        Err(e) => eprintln!("Error walking directory: {e}"),
    }
}

fn handle_outliers(params: OutlierParams) {
    println!("🔍 Analyzing outliers in {}", params.path);
    
    // Parse min_size if provided
    let min_size_bytes = params.min_size.as_ref().and_then(|s| parse_size(s).ok());
    
    let options = rclean::outliers::OutlierOptions {
        min_size: min_size_bytes,
        top_n: Some(params.top),
        std_dev_threshold: params.std_dev,
        check_hidden_consumers: params.check_hidden,
        include_empty_dirs: false,
        check_patterns: params.check_patterns,
        enable_clustering: params.cluster,
        cluster_similarity_threshold: params.cluster_similarity,
        min_cluster_size: params.min_cluster_size,
    };
    
    match rclean::outliers::detect_outliers(&params.path, &options) {
        Ok(report) => {
            println!("\n📊 Analysis Complete");
            println!("Total files analyzed: {}", report.total_files_analyzed);
            println!(
                "Total size analyzed: {:.2} GB",
                report.total_size_analyzed as f64 / (1024.0 * 1024.0 * 1024.0)
            );
            
            // Display results based on format
            match params.format {
                OutputFormat::Table => display_outliers_table(&report),
                OutputFormat::Json => display_outliers_json(&report),
                OutputFormat::Text => display_outliers_text(&report),
            }
            
            // Export to CSV if requested
            if let Some(csv_path) = params.csv {
                if let Ok(mut df) = rclean::outliers::outliers_to_dataframe(&report) {
                    match rclean::generate_csv_report(&mut df, &csv_path) {
                        Ok(()) => println!("\n💾 Results exported to: {}", csv_path),
                        Err(e) => eprintln!("Error writing CSV: {}", e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Error detecting outliers: {}", e),
    }
}

fn display_outliers_table(report: &rclean::outliers::OutlierReport) {
    use rclean::comfy_table::{Table, presets::UTF8_FULL};
    
    if !report.large_files.is_empty() {
        println!("\n🚨 Large File Outliers:");
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
            .set_header(vec!["File Path", "Size (MB)", "% of Total", "Std Devs"]);
        
        for outlier in &report.large_files {
            table.add_row(vec![
                outlier.path.to_string_lossy().to_string(),
                format!("{:.2}", outlier.size_mb),
                format!("{:.1}%", outlier.percentage_of_total),
                format!("{:.1}σ", outlier.std_devs_from_mean),
            ]);
        }
        
        println!("{table}");
    }
    
    if !report.hidden_consumers.is_empty() {
        println!("\n🗂️  Hidden Space Consumers:");
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
            .set_header(vec!["Path", "Type", "Size (MB)", "Files", "Recommendation"]);
        
        for consumer in &report.hidden_consumers {
            table.add_row(vec![
                consumer.path.to_string_lossy().to_string(),
                consumer.pattern_type.clone(),
                format!("{:.2}", consumer.total_size_bytes as f64 / (1024.0 * 1024.0)),
                consumer.file_count.to_string(),
                consumer.recommendation.clone(),
            ]);
        }
        
        println!("{table}");
    }
    
    if !report.pattern_groups.is_empty() {
        println!("\n📁 Pattern Groups:");
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
            .set_header(vec!["Pattern", "Count", "Total Size (MB)"]);
        
        for group in &report.pattern_groups {
            table.add_row(vec![
                group.pattern.clone(),
                group.count.to_string(),
                format!("{:.2}", group.total_size_bytes as f64 / (1024.0 * 1024.0)),
            ]);
        }
        
        println!("{table}");
    }
    
    if !report.large_file_clusters.is_empty() {
        println!("\n🔗 Similar Large File Clusters:");
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
            .set_header(vec!["Cluster ID", "File Count", "Total Size", "Avg Sim", "Density", "File Path"]);
        
        for cluster in &report.large_file_clusters {
            // First row with cluster info and first file
            if !cluster.files.is_empty() {
                table.add_row(vec![
                    cluster.cluster_id.to_string(),
                    cluster.files.len().to_string(),
                    format!("{:.2} MB", cluster.total_size as f64 / (1024.0 * 1024.0)),
                    format!("{:.1}%", cluster.avg_similarity),
                    format!("{:.2}", cluster.density),
                    cluster.files[0].path.to_string_lossy().to_string(),
                ]);
                
                // Additional rows for remaining files in cluster
                for file in cluster.files.iter().skip(1) {
                    table.add_row(vec![
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        file.path.to_string_lossy().to_string(),
                    ]);
                }
            }
        }
        
        println!("{table}");
    }
}

fn display_outliers_json(report: &rclean::outliers::OutlierReport) {
    match serde_json::to_string_pretty(report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

fn display_outliers_text(report: &rclean::outliers::OutlierReport) {
    if !report.large_files.is_empty() {
        println!("\nLarge File Outliers:");
        for outlier in &report.large_files {
            println!(
                "  {} - {:.2} MB ({:.1}% of total, {:.1}σ from mean)",
                outlier.path.display(),
                outlier.size_mb,
                outlier.percentage_of_total,
                outlier.std_devs_from_mean
            );
        }
    }
    
    if !report.hidden_consumers.is_empty() {
        println!("\nHidden Space Consumers:");
        for consumer in &report.hidden_consumers {
            println!(
                "  {} ({}) - {:.2} MB in {} files - {}",
                consumer.path.display(),
                consumer.pattern_type,
                consumer.total_size_bytes as f64 / (1024.0 * 1024.0),
                consumer.file_count,
                consumer.recommendation
            );
        }
    }
    
    if !report.pattern_groups.is_empty() {
        println!("\nPattern Groups:");
        for group in &report.pattern_groups {
            println!(
                "  {} - {} files, {:.2} MB total",
                group.pattern,
                group.count,
                group.total_size_bytes as f64 / (1024.0 * 1024.0)
            );
        }
    }
    
    if !report.large_file_clusters.is_empty() {
        println!("\nSimilar Large File Clusters:");
        for cluster in &report.large_file_clusters {
            println!(
                "  Cluster {} - {} files, {:.2} MB total ({:.1}% avg similarity, {:.2} density)",
                cluster.cluster_id,
                cluster.files.len(),
                cluster.total_size as f64 / (1024.0 * 1024.0),
                cluster.avg_similarity,
                cluster.density
            );
            for file in &cluster.files {
                println!(
                    "    {} - {:.2} MB",
                    file.path.display(),
                    file.size_bytes as f64 / (1024.0 * 1024.0)
                );
            }
        }
    }
}

fn parse_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim().to_uppercase();
    
    if let Some(num_str) = size_str.strip_suffix("KB") {
        num_str.trim().parse::<f64>()
            .map(|n| (n * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("MB") {
        num_str.trim().parse::<f64>()
            .map(|n| (n * 1024.0 * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("GB") {
        num_str.trim().parse::<f64>()
            .map(|n| (n * 1024.0 * 1024.0 * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("B") {
        num_str.trim().parse::<u64>()
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else {
        size_str.parse::<u64>()
            .map_err(|_| format!("Invalid size: {} (use B, KB, MB, or GB suffix)", size_str))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing()?;

    info!("Starting rclean v{}", env!("CARGO_PKG_VERSION"));

    match detect_execution_mode() {
        ExecutionMode::Mcp => {
            info!("Running in MCP server mode");
            let server = rclean::mcp_server::server::McpServer::new();
            server.run().await
        },
        ExecutionMode::Cli => {
            info!("Running in CLI mode");
            run_cli().await
        },
    }
}

enum ExecutionMode {
    Mcp,
    Cli,
}

fn detect_execution_mode() -> ExecutionMode {
    let is_mcp = !std::io::stdin().is_terminal() && std::env::args().len() == 1
        || std::env::var("MCP_VERSION").is_ok();

    if is_mcp {
        debug!("Detected MCP server mode");
        ExecutionMode::Mcp
    } else {
        debug!("Detected CLI mode");
        ExecutionMode::Cli
    }
}

fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    Ok(())
}

async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // If no subcommand is provided, default to dedupe
    let command = cli.command.unwrap_or(Commands::Dedupe {
        path: cli.path,
        pattern: cli.pattern,
        pattern_type: cli.pattern_type,
        hidden: cli.hidden,
        no_ignore: cli.no_ignore,
        max_depth: cli.max_depth,
        csv: cli.csv,
        similarity: cli.similarity,
    });

    match command {
        Commands::Search {
            path,
            pattern,
            pattern_type,
            hidden,
            no_ignore,
            max_depth,
        } => {
            let options = SearchOptions {
                path,
                pattern,
                pattern_type,
                hidden,
                no_ignore,
                max_depth,
            };
            handle_search(&options);
        },
        Commands::Dedupe {
            path,
            pattern,
            pattern_type,
            hidden,
            no_ignore,
            max_depth,
            csv,
            similarity,
        } => {
            let options = SearchOptions {
                path,
                pattern,
                pattern_type,
                hidden,
                no_ignore,
                max_depth,
            };
            handle_dedupe(&options, csv.as_deref(), similarity);
        },
        Commands::Count {
            path,
            pattern,
            pattern_type,
            hidden,
            no_ignore,
            max_depth,
        } => {
            let options = SearchOptions {
                path,
                pattern,
                pattern_type,
                hidden,
                no_ignore,
                max_depth,
            };
            handle_count(&options);
        },
        Commands::Outliers {
            path,
            min_size,
            top,
            std_dev,
            check_hidden,
            check_patterns,
            cluster,
            cluster_similarity,
            min_cluster_size,
            format,
            csv,
        } => {
            handle_outliers(OutlierParams {
                path,
                min_size,
                top,
                std_dev,
                check_hidden,
                check_patterns,
                cluster,
                cluster_similarity,
                min_cluster_size,
                format,
                csv,
            });
        },
    }
    Ok(())
}
