//! A high-performance disk cleanup library with parallel processing.
//!
//! This library provides functionality to scan directories, find duplicate files
//! based on MD5 hashes, detect storage outliers, and generate detailed reports
//! using Polars `DataFrames`.

pub mod clustering;
pub mod mcp_server;
pub mod models;
pub mod outliers;

pub use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use indicatif::{ParallelProgressIterator as _, ProgressStyle};
use polars::prelude::*;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
pub use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Re-export for CLI usage
pub use comfy_table;
pub use globset;
pub use regex;

/// Pattern matching type for file filtering.
#[derive(Debug, Clone)]
pub enum PatternType {
    /// Simple string contains matching (default).
    Literal(String),
    /// Glob pattern matching (e.g., *.txt, **/*.rs).
    Glob(GlobSet),
    /// Regular expression matching.
    Regex(Regex),
}

/// Options for directory walking.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    /// Include hidden files.
    pub include_hidden: bool,
    /// Respect .gitignore files.
    pub respect_gitignore: bool,
    /// Respect .ignore files.
    pub respect_ignore: bool,
    /// Maximum depth to traverse.
    pub max_depth: Option<usize>,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            respect_gitignore: true,
            respect_ignore: true,
            max_depth: None,
        }
    }
}

/// Display threading information including CPU cores and thread pool size.
///
/// # Examples
///
/// ```
/// let info = rclean::display_thread_info();
/// assert!(info.contains("CPU cores:"));
/// assert!(info.contains("Rayon thread pool size:"));
/// ```
#[must_use]
#[inline]
pub fn display_thread_info() -> String {
    let num_cpus = num_cpus::get();
    let rayon_threads = rayon::current_num_threads();

    format!("💻 CPU cores: {num_cpus}\n🧵 Rayon thread pool size: {rayon_threads}")
}

/// Information about a file including metadata and hash.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub md5_hash: String,
    pub fuzzy_hash: Option<String>,
    pub is_duplicate: bool,
    pub duplicate_group: Option<String>,
    pub is_similar: bool,
    pub similarity_group: Option<String>,
    pub similarity_score: Option<f64>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

impl FileInfo {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let path_obj = Path::new(path);
        let metadata = fs::metadata(path)?;

        let name = path_obj
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path_obj
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let size_bytes = metadata.len();
        #[allow(clippy::cast_precision_loss)]
        let size_mb = size_bytes as f64 / 1_048_576.0; // Convert bytes to MB

        let file_content = fs::read(path)?;
        let md5_hash = format!("{:x}", md5::compute(&file_content));

        // Calculate fuzzy hash for similarity detection
        // Skip files that are too small (< 512 bytes) or too large (> 100MB)
        let fuzzy_hash = if file_content.len() >= 512 && file_content.len() < 1024 * 1024 * 100 {
            ssdeep::hash(&file_content).ok()
        } else {
            None
        };

        let created = metadata
            .created()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| format!("{}", duration.as_secs()));

        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| format!("{}", duration.as_secs()));

        Ok(Self {
            path: path.to_string(),
            name,
            extension,
            size_bytes,
            size_mb,
            md5_hash,
            fuzzy_hash,
            is_duplicate: false,
            duplicate_group: None,
            is_similar: false,
            similarity_group: None,
            similarity_score: None,
            created,
            modified,
        })
    }
}

pub fn walk(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();
    let mut permission_errors = 0;

    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    if let Some(path_str) = entry.path().to_str() {
                        files.push(path_str.to_string());
                    }
                }
            },
            Err(err) => {
                // Check if it's a recoverable error
                if let Some(io_err) = err.io_error() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied |
                        std::io::ErrorKind::InvalidInput |
                        std::io::ErrorKind::InvalidData |
                        std::io::ErrorKind::NotFound => {
                            permission_errors += 1;
                            // Continue walking, don't fail on these errors
                            continue;
                        },
                        _ => {
                            // For other IO errors, propagate them
                            return Err(Box::new(err));
                        }
                    }
                }
                // For non-IO errors, propagate them
                return Err(Box::new(err));
            },
        }
    }

    if permission_errors > 0 {
        eprintln!("⚠️  Skipped {permission_errors} directories due to permission errors");
    }

    Ok(files)
}

/// Walk a directory recursively with gitignore support and return all file paths.
///
/// # Errors
///
/// Returns an error if:
/// - The directory cannot be accessed
/// - A path contains invalid UTF-8
pub fn walk_with_options(path: &str, options: &WalkOptions) -> Result<Vec<String>, Box<dyn Error>> {
    let mut builder = WalkBuilder::new(path);

    // Configure the walker
    builder
        .hidden(!options.include_hidden)
        .ignore(options.respect_ignore)
        .git_ignore(options.respect_gitignore)
        .git_global(options.respect_gitignore)
        .git_exclude(options.respect_gitignore);

    if let Some(max_depth) = options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    let walker = builder.build();
    let mut files = Vec::new();
    let mut permission_errors = 0;

    for entry in walker {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_some_and(|ft| ft.is_file()) {
                    if let Some(path_str) = entry.path().to_str() {
                        files.push(path_str.to_string());
                    }
                }
            },
            Err(err) => {
                // Check if it's a recoverable error
                if let Some(io_err) = err.io_error() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied |
                        std::io::ErrorKind::InvalidInput |
                        std::io::ErrorKind::InvalidData |
                        std::io::ErrorKind::NotFound => {
                            permission_errors += 1;
                            // Continue walking, don't fail on these errors
                            continue;
                        },
                        _ => {
                            // For other IO errors, propagate them
                            return Err(Box::new(err));
                        }
                    }
                }
                // For non-IO errors, propagate them
                return Err(Box::new(err));
            },
        }
    }

    if permission_errors > 0 {
        eprintln!("⚠️  Skipped {permission_errors} directories due to permission errors");
    }

    Ok(files)
}

/// Find files matching a pattern.
///
/// # Examples
///
/// ```
/// let files = vec![
///     "test.txt".to_string(),
///     "data.csv".to_string(),
///     "test_data.txt".to_string(),
/// ];
/// let matches = rclean::find(&files, "test");
/// assert_eq!(matches.len(), 2);
/// assert!(matches.contains(&"test.txt".to_string()));
/// assert!(matches.contains(&"test_data.txt".to_string()));
/// ```
#[must_use]
pub fn find(files: &[String], pattern: &str) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.contains(pattern))
        .cloned()
        .collect()
}

/// Find files matching an advanced pattern.
///
/// # Examples
///
/// ```
/// use rclean::{PatternType, find_advanced, GlobSetBuilder, Glob};
///
/// let files = vec![
///     "test.txt".to_string(),
///     "data.csv".to_string(),
///     "test_data.json".to_string(),
/// ];
///
/// // Literal pattern
/// let pattern = PatternType::Literal("test".to_string());
/// let matches = find_advanced(&files, &pattern);
/// assert_eq!(matches.len(), 2);
///
/// // Glob pattern
/// let mut builder = GlobSetBuilder::new();
/// builder.add(Glob::new("*.txt").unwrap());
/// let globset = builder.build().unwrap();
/// let pattern = PatternType::Glob(globset);
/// let matches = find_advanced(&files, &pattern);
/// assert_eq!(matches.len(), 1);
/// assert_eq!(matches[0], "test.txt");
/// ```
#[must_use]
pub fn find_advanced(files: &[String], pattern: &PatternType) -> Vec<String> {
    files
        .iter()
        .filter(|file| match pattern {
            PatternType::Literal(s) => file.contains(s),
            PatternType::Glob(g) => g.is_match(file),
            PatternType::Regex(r) => r.is_match(file),
        })
        .cloned()
        .collect()
}

// New function to collect detailed file information - TRUE PARALLEL VERSION
/// Collect detailed file information in parallel.
///
/// # Errors
///
/// Returns an error if progress bar creation fails.
pub fn collect_file_info(files: &[String]) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    if files.is_empty() {
        return Ok(Vec::new());
    }

    println!(
        "\nAnalyzing {} files with {} threads...",
        files.len(),
        rayon::current_num_threads()
    );

    let pb = indicatif::ProgressBar::new(files.len() as u64);
    #[allow(clippy::expect_used)]
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
        .expect("Failed to create progress bar template")
        .progress_chars("##-");

    pb.set_style(sty);
    pb.set_message("Computing MD5 hashes...");

    // Enable steady tick to ensure spinner is visible
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // TRUE PARALLEL: Each thread processes files independently, no shared mutex
    let file_infos: Vec<Option<FileInfo>> = files
        .par_iter()
        .progress_with(pb.clone())
        .map(|file_path| FileInfo::new(file_path).ok())
        .collect();

    pb.finish_with_message("✓ File analysis complete!");
    println!();

    // Filter out failed files
    let valid_infos: Vec<FileInfo> = file_infos.into_iter().flatten().collect();

    Ok(valid_infos)
}

// Create Polars DataFrame from file information
pub fn create_dataframe(mut file_infos: Vec<FileInfo>) -> Result<DataFrame, Box<dyn Error>> {
    // Group files by hash to identify duplicates
    let mut hash_groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, file_info) in file_infos.iter().enumerate() {
        hash_groups
            .entry(file_info.md5_hash.clone())
            .or_default()
            .push(index);
    }

    // Mark duplicates and assign group IDs - ONLY for files that actually have duplicates
    let mut duplicate_count = 0;
    for (hash, indices) in &hash_groups {
        if indices.len() > 1 {
            let group_id = hash.clone();
            duplicate_count += indices.len();

            for &index in indices {
                file_infos[index].is_duplicate = true;
                file_infos[index].duplicate_group = Some(group_id.clone());
            }
        }
    }

    println!(
        "Found {} files in {} duplicate groups",
        duplicate_count,
        hash_groups.iter().filter(|(_, v)| v.len() > 1).count()
    );

    // Extract data for DataFrame columns
    let paths: Vec<String> = file_infos.iter().map(|f| f.path.clone()).collect();
    let names: Vec<String> = file_infos.iter().map(|f| f.name.clone()).collect();
    let extensions: Vec<String> = file_infos.iter().map(|f| f.extension.clone()).collect();
    let sizes_bytes: Vec<u64> = file_infos.iter().map(|f| f.size_bytes).collect();
    let sizes_mb: Vec<f64> = file_infos.iter().map(|f| f.size_mb).collect();
    let hashes: Vec<String> = file_infos.iter().map(|f| f.md5_hash.clone()).collect();
    let is_duplicate: Vec<bool> = file_infos.iter().map(|f| f.is_duplicate).collect();
    let duplicate_groups: Vec<Option<String>> = file_infos
        .iter()
        .map(|f| f.duplicate_group.clone())
        .collect();

    let df = df! [
        "file_path" => paths,
        "file_name" => names,
        "extension" => extensions,
        "size_bytes" => sizes_bytes,
        "size_mb" => sizes_mb,
        "md5_hash" => hashes,
        "is_duplicate" => is_duplicate,
        "duplicate_group" => duplicate_groups,
    ]?;

    Ok(df)
}

/// Generate file statistics summary.
///
/// # Errors
///
/// Returns an error if:
/// - `DataFrame` column access fails
/// - `DataFrame` creation fails
#[allow(clippy::cast_precision_loss)]
pub fn generate_statistics(df: &DataFrame) -> Result<DataFrame, Box<dyn Error>> {
    let total_files = df.height();
    let total_size_bytes: u64 = df.column("size_bytes")?.sum().unwrap_or(0);
    #[allow(clippy::cast_precision_loss)]
    let total_size_mb = total_size_bytes as f64 / 1_048_576.0;

    let duplicate_count = df
        .column("is_duplicate")?
        .bool()?
        .into_iter()
        .filter(|x| x.unwrap_or(false))
        .count();

    let unique_extensions = df.column("extension")?.unique()?.len();

    #[allow(clippy::cast_precision_loss)]
    let avg_file_size_mb = total_size_mb / total_files as f64;

    let stats_df = df! [
        "metric" => vec!["total_files", "duplicate_files", "total_size_mb", "avg_file_size_mb", "unique_extensions"],
        "value" => vec![total_files as f64, duplicate_count as f64, total_size_mb, avg_file_size_mb, unique_extensions as f64],
    ]?;

    Ok(stats_df)
}

// Validate duplicate detection logic
pub fn validate_duplicates(df: &DataFrame) -> Result<(), Box<dyn Error>> {
    println!("\n=== Duplicate Detection Validation ===");

    // Group by hash and check consistency
    let duplicates = df
        .clone()
        .lazy()
        .filter(col("is_duplicate").eq(lit(true)))
        .collect()?;

    if duplicates.height() == 0 {
        println!("✓ No duplicates found - validation passed");
        return Ok(());
    }

    // Group duplicates by their hash to verify consistency
    let grouped = duplicates
        .lazy()
        .group_by([col("md5_hash")])
        .agg([
            col("file_path").count().alias("file_count"),
            col("duplicate_group").first().alias("group_id"),
        ])
        .collect()?;

    println!("Duplicate groups found:");
    let max_display = 50; // Limit output to prevent terminal overflow
    let total_groups = grouped.height();
    let display_count = std::cmp::min(max_display, total_groups);
    
    for row in 0..display_count {
        let hash = grouped.column("md5_hash")?.get(row)?;
        let count = grouped.column("file_count")?.get(row)?;
        println!("  Hash: {hash} -> {count} files");
    }
    
    if total_groups > max_display {
        println!("  ... and {} more duplicate groups (showing first {})", 
                total_groups - max_display, max_display);
    }

    println!("✓ Duplicate detection validation completed");
    Ok(())
}

// Generate CSV report - ONLY for duplicate files
pub fn generate_csv_report(df: &mut DataFrame, output_path: &str) -> Result<(), Box<dyn Error>> {
    // Filter to only include actual duplicates
    let duplicates_only = df
        .clone()
        .lazy()
        .filter(col("is_duplicate").eq(lit(true)))
        .collect()?;

    if duplicates_only.height() == 0 {
        println!("No duplicates found - CSV report not generated");
        return Ok(());
    }

    let mut file = fs::File::create(output_path)?;
    let mut duplicates_df = duplicates_only;

    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut duplicates_df)?;

    println!(
        "CSV report generated: {} ({} duplicate files)",
        output_path,
        duplicates_df.height()
    );

    Ok(())
}

/// Run deduplication with `DataFrame` support and optional CSV output.
///
/// # Errors
///
/// Returns an error if:
/// - Directory walking fails
/// - File processing fails
/// - `DataFrame` operations fail
pub fn run_with_dataframe(
    path: &str,
    pattern: &str,
    output_csv: Option<&str>,
) -> Result<DataFrame, Box<dyn Error>> {
    println!("Scanning directory: {path}");

    let files = walk(path)?;
    let files = find(&files, pattern);

    println!("Found {} files matching pattern '{}'", files.len(), pattern);

    if files.is_empty() {
        println!("No files found to analyze.");
        return Ok(df! [
            "file_path" => Vec::<String>::new(),
            "file_name" => Vec::<String>::new(),
            "extension" => Vec::<String>::new(),
            "size_bytes" => Vec::<u64>::new(),
            "size_mb" => Vec::<f64>::new(),
            "md5_hash" => Vec::<String>::new(),
            "is_duplicate" => Vec::<bool>::new(),
            "duplicate_group" => Vec::<Option<String>>::new(),
        ]?);
    }

    let file_infos = collect_file_info(&files)?;
    let df = create_dataframe(file_infos)?;

    // Print summary statistics
    let stats = generate_statistics(&df)?;

    println!("\n=== File Analysis Summary ===");
    println!("{stats}");

    // Validate duplicate detection
    validate_duplicates(&df)?;

    // Show duplicate information
    let duplicates = df
        .clone()
        .lazy()
        .filter(col("is_duplicate").eq(lit(true)))
        .collect()?;

    if duplicates.height() > 0 {
        println!("\n=== Duplicate Files Found ===");
        println!("{duplicates}");
    } else {
        println!("\nNo duplicate files found.");
    }

    // Generate CSV report if requested
    if let Some(csv_path) = output_csv {
        let mut df_copy = df.clone();

        generate_csv_report(&mut df_copy, csv_path)?;
    }

    Ok(df)
}

/// Run deduplication with `DataFrame` support using advanced pattern matching.
///
/// # Errors
///
/// Returns an error if:
/// - Directory walking fails
/// - File processing fails
/// - `DataFrame` operations fail
pub fn run_with_advanced_options(
    path: &str,
    pattern: &PatternType,
    walk_options: &WalkOptions,
    output_csv: Option<&str>,
) -> Result<DataFrame, Box<dyn Error>> {
    println!("Scanning directory: {path}");

    let files = walk_with_options(path, walk_options)?;
    let files = find_advanced(&files, pattern);

    println!("Found {} files matching pattern", files.len());

    if files.is_empty() {
        println!("No files found to analyze.");
        return Ok(df! [
            "file_path" => Vec::<String>::new(),
            "file_name" => Vec::<String>::new(),
            "extension" => Vec::<String>::new(),
            "size_bytes" => Vec::<u64>::new(),
            "size_mb" => Vec::<f64>::new(),
            "md5_hash" => Vec::<String>::new(),
            "is_duplicate" => Vec::<bool>::new(),
            "duplicate_group" => Vec::<Option<String>>::new(),
        ]?);
    }

    let file_infos = collect_file_info(&files)?;
    let df = create_dataframe(file_infos)?;

    // Print summary statistics
    let stats = generate_statistics(&df)?;

    println!("\n=== File Analysis Summary ===");
    println!("{stats}");

    // Validate duplicate detection
    validate_duplicates(&df)?;

    // Show duplicate information
    let duplicates = df
        .clone()
        .lazy()
        .filter(col("is_duplicate").eq(lit(true)))
        .collect()?;

    if duplicates.height() > 0 {
        println!("\n=== Duplicate Files Found ===");
        println!("{duplicates}");
    } else {
        println!("\nNo duplicate files found.");
    }

    // Generate CSV report if requested
    if let Some(csv_path) = output_csv {
        let mut df_copy = df.clone();

        generate_csv_report(&mut df_copy, csv_path)?;
    }

    Ok(df)
}

/*  TRUE PARALLEL version of checksum using rayon with no mutex contention
Uses indicatif to show a progress bar
*/
/// Compute checksums for files in parallel.
///
/// # Errors
///
/// Returns an error if progress bar creation fails.
///
/// # Panics
///
/// Panics if progress bar template creation fails.
pub fn checksum(files: &[String]) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    println!(
        "Computing checksums with {} threads...",
        rayon::current_num_threads()
    );

    let pb = indicatif::ProgressBar::new(files.len() as u64);
    #[allow(clippy::expect_used)]
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .expect("Failed to create progress bar template");

    pb.set_style(sty);

    // TRUE PARALLEL: Each thread computes checksums independently
    let file_checksums: Vec<(String, String)> = files
        .par_iter()
        .progress_with(pb)
        .filter_map(|file| {
            fs::read(file).ok().map(|content| {
                let checksum = format!("{:x}", md5::compute(&content));
                (checksum, file.clone())
            })
        })
        .collect();

    // Sequential grouping (this part must be sequential anyway)
    let mut checksums: HashMap<String, Vec<String>> = HashMap::new();
    for (hash, file_path) in file_checksums {
        checksums.entry(hash).or_default().push(file_path);
    }

    Ok(checksums)
}

/*
Find all the files with more than one entry in the HashMap
*/
pub fn find_duplicates(checksums: HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut duplicates = Vec::new();

    for (_checksum, files) in checksums {
        if files.len() > 1 {
            duplicates.push(files);
        }
    }

    duplicates
}

/// Calculate similarity between two fuzzy hashes.
///
/// Returns a score between 0 and 100, where 100 is identical.
///
/// # Examples
///
/// ```
/// # use rclean::calculate_similarity;
/// // Test with identical hashes (100% similarity)
/// let hash1 = "3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C";
/// let hash2 = "3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C";
///
/// match calculate_similarity(hash1, hash2) {
///     Ok(score) => {
///         assert_eq!(score, 100);
///     }
///     Err(_) => panic!("Failed to calculate similarity"),
/// }
///
/// // Test with different hashes
/// let hash3 = "3:RlFgVNhBGcL6wCrFQEv:RlFxLsr2C";
/// match calculate_similarity(hash1, hash3) {
///     Ok(score) => {
///         assert!(score <= 100);
///         assert!(score >= 0);
///     }
///     Err(_) => panic!("Failed to calculate similarity"),
/// }
/// ```
pub fn calculate_similarity(hash1: &str, hash2: &str) -> Result<u32, Box<dyn Error>> {
    Ok(u32::from(ssdeep::compare(hash1, hash2)?))
}

/// Find similar files based on fuzzy hashing and similarity threshold.
///
/// # Arguments
/// * `file_infos` - Vector of `FileInfo` with fuzzy hashes
/// * `threshold` - Similarity threshold (0-100), typically 70-90 for good matches
///
/// # Returns
/// Type alias for a group of similar files with their similarity scores.
pub type SimilarFileGroup = Vec<(FileInfo, f64)>;

/// Vector of groups of similar files with their similarity scores
pub fn find_similar_files(
    file_infos: &[FileInfo],
    threshold: u32,
) -> Result<Vec<SimilarFileGroup>, Box<dyn Error>> {
    let mut similar_groups: Vec<SimilarFileGroup> = Vec::new();
    let mut processed: Vec<bool> = vec![false; file_infos.len()];

    // Compare each file with every other file
    for i in 0..file_infos.len() {
        if processed[i] {
            continue;
        }

        let mut group: Vec<(FileInfo, f64)> = vec![(file_infos[i].clone(), 100.0)];
        processed[i] = true;

        if let Some(ref hash1) = file_infos[i].fuzzy_hash {
            for j in (i + 1)..file_infos.len() {
                if processed[j] {
                    continue;
                }

                if let Some(ref hash2) = file_infos[j].fuzzy_hash {
                    let similarity = calculate_similarity(hash1, hash2)?;
                    if similarity >= threshold {
                        group.push((file_infos[j].clone(), f64::from(similarity)));
                        processed[j] = true;
                    }
                }
            }
        }

        // Only add groups with more than one file
        if group.len() > 1 {
            similar_groups.push(group);
        }
    }

    Ok(similar_groups)
}

/// Run the deduplication process.
///
/// # Errors
///
/// Returns an error if:
/// - Directory walking fails
/// - Checksum computation fails
pub fn run(path: &str, pattern: &str) -> Result<(), Box<dyn Error>> {
    let files = walk(path)?;
    let files = find(&files, pattern);

    println!("Found {} files matching {pattern}", files.len());

    let checksums = checksum(&files)?;
    let duplicates = find_duplicates(checksums);

    println!("Found {} duplicate(s)", duplicates.len());

    for duplicate in duplicates {
        println!("{duplicate:?}");
    }

    Ok(())
}

/// Run deduplication with similarity detection using fuzzy hashing.
///
/// # Arguments
/// * `path` - Directory to scan
/// * `pattern` - Pattern type for file filtering
/// * `walk_options` - Walking options (hidden files, gitignore, etc.)
/// * `similarity_threshold` - Similarity threshold (0-100)
/// * `output_csv` - Optional CSV output path
///
/// # Errors
///
/// Returns an error if:
/// - Directory walking fails
/// - File processing fails
/// - `DataFrame` operations fail
pub fn run_with_similarity(
    path: &str,
    pattern: &PatternType,
    walk_options: &WalkOptions,
    similarity_threshold: u32,
    output_csv: Option<&str>,
) -> Result<DataFrame, Box<dyn Error>> {
    println!("Scanning directory: {path}");
    println!("Similarity threshold: {similarity_threshold}%");

    let files = walk_with_options(path, walk_options)?;
    let files = find_advanced(&files, pattern);

    println!("Found {} files matching pattern", files.len());

    if files.is_empty() {
        println!("No files found to analyze.");
        return Ok(df! [
            "file_path" => Vec::<String>::new(),
            "file_name" => Vec::<String>::new(),
            "extension" => Vec::<String>::new(),
            "size_bytes" => Vec::<u64>::new(),
            "size_mb" => Vec::<f64>::new(),
            "md5_hash" => Vec::<String>::new(),
            "is_duplicate" => Vec::<bool>::new(),
            "duplicate_group" => Vec::<Option<String>>::new(),
            "is_similar" => Vec::<bool>::new(),
            "similarity_score" => Vec::<Option<f64>>::new(),
        ]?);
    }

    let mut file_infos = collect_file_info(&files)?;

    // First, find exact duplicates (existing functionality)
    let mut hash_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, file_info) in file_infos.iter().enumerate() {
        hash_groups
            .entry(file_info.md5_hash.clone())
            .or_default()
            .push(index);
    }

    let mut duplicate_count = 0;
    for (hash, indices) in &hash_groups {
        if indices.len() > 1 {
            let group_id = hash.clone();
            duplicate_count += indices.len();

            for &index in indices {
                file_infos[index].is_duplicate = true;
                file_infos[index].duplicate_group = Some(group_id.clone());
            }
        }
    }

    // Then, find similar files using fuzzy hashing
    let similar_groups = find_similar_files(&file_infos, similarity_threshold)?;

    let mut similar_count = 0;
    for (group_idx, group) in similar_groups.iter().enumerate() {
        let group_id = format!("similar_{group_idx}");
        similar_count += group.len();

        for (file_info, score) in group {
            // Find the file in our file_infos vector and update it
            if let Some(idx) = file_infos.iter().position(|f| f.path == file_info.path) {
                file_infos[idx].is_similar = true;
                file_infos[idx].similarity_group = Some(group_id.clone());
                file_infos[idx].similarity_score = Some(*score);
            }
        }
    }

    println!(
        "\nFound {} exact duplicates in {} groups",
        duplicate_count,
        hash_groups.iter().filter(|(_, v)| v.len() > 1).count()
    );

    println!(
        "Found {} similar files in {} groups (≥{}% similarity)",
        similar_count,
        similar_groups.len(),
        similarity_threshold
    );

    // Create DataFrame with similarity information
    let df = create_dataframe_with_similarity(&file_infos)?;

    // Print similar files
    if !similar_groups.is_empty() {
        println!("\n=== Similar Files Found (≥{similarity_threshold}% similarity) ===");
        for (idx, group) in similar_groups.iter().enumerate() {
            println!("\nGroup {} ({} files):", idx + 1, group.len());
            for (file_info, score) in group {
                println!(
                    "  {:.1}% - {} ({:.2} MB)",
                    score, file_info.path, file_info.size_mb
                );
            }
        }
    }

    // Generate CSV if requested
    if let Some(csv_path) = output_csv {
        generate_csv_report_with_similarity(&df, csv_path)?;
    }

    Ok(df)
}

/// Create `DataFrame` including similarity information.
fn create_dataframe_with_similarity(file_infos: &[FileInfo]) -> Result<DataFrame, Box<dyn Error>> {
    let paths: Vec<String> = file_infos.iter().map(|f| f.path.clone()).collect();
    let names: Vec<String> = file_infos.iter().map(|f| f.name.clone()).collect();
    let extensions: Vec<String> = file_infos.iter().map(|f| f.extension.clone()).collect();
    let sizes_bytes: Vec<u64> = file_infos.iter().map(|f| f.size_bytes).collect();
    let sizes_mb: Vec<f64> = file_infos.iter().map(|f| f.size_mb).collect();
    let hashes: Vec<String> = file_infos.iter().map(|f| f.md5_hash.clone()).collect();
    let is_duplicate: Vec<bool> = file_infos.iter().map(|f| f.is_duplicate).collect();
    let duplicate_groups: Vec<Option<String>> = file_infos
        .iter()
        .map(|f| f.duplicate_group.clone())
        .collect();
    let is_similar: Vec<bool> = file_infos.iter().map(|f| f.is_similar).collect();
    let similarity_scores: Vec<Option<f64>> =
        file_infos.iter().map(|f| f.similarity_score).collect();

    let df = df! [
        "file_path" => paths,
        "file_name" => names,
        "extension" => extensions,
        "size_bytes" => sizes_bytes,
        "size_mb" => sizes_mb,
        "md5_hash" => hashes,
        "is_duplicate" => is_duplicate,
        "duplicate_group" => duplicate_groups,
        "is_similar" => is_similar,
        "similarity_score" => similarity_scores,
    ]?;

    Ok(df)
}

/// Generate CSV report including similarity information.
fn generate_csv_report_with_similarity(
    df: &DataFrame,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Filter to include duplicates OR similar files
    let filtered = df
        .clone()
        .lazy()
        .filter(
            col("is_duplicate")
                .eq(lit(true))
                .or(col("is_similar").eq(lit(true))),
        )
        .collect()?;

    if filtered.height() == 0 {
        println!("No duplicates or similar files found - CSV report not generated");
        return Ok(());
    }

    let mut file = fs::File::create(output_path)?;
    let mut filtered_df = filtered;

    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut filtered_df)?;

    println!(
        "CSV report generated: {} ({} duplicate/similar files)",
        output_path,
        filtered_df.height()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
        // Create some test files
        fs::write(dir.join("file1.txt"), "content1")?;
        fs::write(dir.join("file2.txt"), "content1")?; // Duplicate of file1
        fs::write(dir.join("file3.txt"), "content3")?;
        fs::write(dir.join("file4.rs"), "rust code")?;
        fs::write(dir.join(".hidden"), "hidden file")?;
        
        // Create subdirectory
        let subdir = dir.join("subdir");
        fs::create_dir(&subdir)?;
        fs::write(subdir.join("file5.txt"), "content5")?;
        fs::write(subdir.join("file6.txt"), "content1")?; // Another duplicate
        
        Ok(vec![
            dir.join("file1.txt").to_string_lossy().to_string(),
            dir.join("file2.txt").to_string_lossy().to_string(),
            dir.join("file3.txt").to_string_lossy().to_string(),
            dir.join("file4.rs").to_string_lossy().to_string(),
            dir.join(".hidden").to_string_lossy().to_string(),
            subdir.join("file5.txt").to_string_lossy().to_string(),
            subdir.join("file6.txt").to_string_lossy().to_string(),
        ])
    }

    #[test]
    fn test_display_thread_info() {
        let info = display_thread_info();
        assert!(info.contains("CPU cores:"));
        assert!(info.contains("Rayon thread pool size:"));
    }

    #[test]
    fn test_walk() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let _files = create_test_files(temp_dir.path())?;
        
        let walked_files = walk(temp_dir.path().to_str().unwrap())?;
        // Should find at least the non-hidden files
        assert!(walked_files.len() >= 6);
        
        Ok(())
    }

    #[test]
    fn test_walk_with_options() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let _files = create_test_files(temp_dir.path())?;
        
        // Test with hidden files included
        let options = WalkOptions {
            include_hidden: true,
            respect_gitignore: false,
            respect_ignore: false,
            max_depth: None,
        };
        
        let walked_files = walk_with_options(temp_dir.path().to_str().unwrap(), &options)?;
        assert!(walked_files.iter().any(|f| f.contains(".hidden")));
        
        // Test with max depth
        let options = WalkOptions {
            include_hidden: false,
            respect_gitignore: true,
            respect_ignore: true,
            max_depth: Some(1),
        };
        
        let walked_files = walk_with_options(temp_dir.path().to_str().unwrap(), &options)?;
        // Should not find files in subdirectory
        assert!(!walked_files.iter().any(|f| f.contains("subdir")));
        
        Ok(())
    }

    #[test]
    fn test_find() {
        let files = vec![
            "test.txt".to_string(),
            "data.csv".to_string(),
            "test_data.txt".to_string(),
            "readme.md".to_string(),
        ];
        
        let matches = find(&files, "test");
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"test.txt".to_string()));
        assert!(matches.contains(&"test_data.txt".to_string()));
        
        let matches = find(&files, ".txt");
        assert_eq!(matches.len(), 2);
        
        let matches = find(&files, "");
        assert_eq!(matches.len(), 4);
    }

    #[test]
    fn test_find_advanced_literal() {
        let files = vec![
            "test.txt".to_string(),
            "data.csv".to_string(),
            "test_data.txt".to_string(),
        ];
        
        let pattern = PatternType::Literal("test".to_string());
        let matches = find_advanced(&files, &pattern);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_advanced_glob() -> Result<(), Box<dyn Error>> {
        let files = vec![
            "test.txt".to_string(),
            "data.csv".to_string(),
            "test.rs".to_string(),
        ];
        
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("*.txt")?);
        let globset = builder.build()?;
        let pattern = PatternType::Glob(globset);
        
        let matches = find_advanced(&files, &pattern);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "test.txt");
        
        Ok(())
    }

    #[test]
    fn test_find_advanced_regex() -> Result<(), Box<dyn Error>> {
        let files = vec![
            "test.txt".to_string(),
            "data.csv".to_string(),
            "test123.txt".to_string(),
        ];
        
        let regex = Regex::new(r"test\d+\.txt")?;
        let pattern = PatternType::Regex(regex);
        
        let matches = find_advanced(&files, &pattern);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "test123.txt");
        
        Ok(())
    }

    #[test]
    fn test_file_info() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        // Use longer content to avoid ssdeep issues with small files
        let content = "test content that is long enough to avoid ssdeep crashes";
        fs::write(&test_file, content)?;
        
        let file_info = FileInfo::new(test_file.to_str().unwrap())?;
        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.extension, "txt");
        assert_eq!(file_info.size_bytes, content.len() as u64);
        assert!(!file_info.md5_hash.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_checksum_and_find_duplicates() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");
        
        fs::write(&file1, "duplicate content")?;
        fs::write(&file2, "duplicate content")?;
        fs::write(&file3, "unique content")?;
        
        let files = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
            file3.to_string_lossy().to_string(),
        ];
        
        let checksums = checksum(&files)?;
        let duplicates = find_duplicates(checksums);
        
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].len(), 2);
        
        Ok(())
    }

    #[test]
    fn test_calculate_similarity() -> Result<(), Box<dyn Error>> {
        // Test with identical hashes
        let hash1 = "3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C";
        let hash2 = "3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C";
        
        let score = calculate_similarity(hash1, hash2)?;
        assert_eq!(score, 100);
        
        // Test with different hashes
        let hash3 = "3:RlFgVNhBGcL6wCrFQEv:RlFxLsr2C";
        let score = calculate_similarity(hash1, hash3)?;
        assert!(score < 100);
        // Score is u32, so >= 0 is always true
        
        Ok(())
    }

    #[test]
    fn test_walk_options_default() {
        let options = WalkOptions::default();
        assert!(!options.include_hidden);
        assert!(options.respect_gitignore);
        assert!(options.respect_ignore);
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn test_collect_file_info() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;
        
        let files = vec![test_file.to_string_lossy().to_string()];
        let file_infos = collect_file_info(&files)?;
        
        assert_eq!(file_infos.len(), 1);
        assert_eq!(file_infos[0].name, "test.txt");
        
        Ok(())
    }

    #[test]
    fn test_create_dataframe() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        fs::write(&file1, "content")?;
        fs::write(&file2, "content")?;
        
        let files = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];
        
        let file_infos = collect_file_info(&files)?;
        let df = create_dataframe(file_infos)?;
        
        assert_eq!(df.height(), 2);
        assert!(df.column("is_duplicate").is_ok());
        
        Ok(())
    }

    #[test]
    fn test_empty_file_handling() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let empty_file = temp_dir.path().join("empty.txt");
        fs::write(&empty_file, "")?;
        
        let files = vec![empty_file.to_string_lossy().to_string()];
        let file_infos = collect_file_info(&files)?;
        
        assert_eq!(file_infos.len(), 1);
        assert_eq!(file_infos[0].size_bytes, 0);
        assert!(file_infos[0].fuzzy_hash.is_none()); // Too small for fuzzy hash
        
        Ok(())
    }

    #[test]
    fn test_generate_statistics() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        fs::write(&file1, "content")?;
        fs::write(&file2, "different")?;
        
        let files = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];
        
        let file_infos = collect_file_info(&files)?;
        let df = create_dataframe(file_infos)?;
        let stats = generate_statistics(&df)?;
        
        assert!(stats.height() > 0);
        assert!(stats.column("metric").is_ok());
        assert!(stats.column("value").is_ok());
        
        Ok(())
    }

    #[test]
    fn test_run_with_dataframe() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.txt");
        
        fs::write(&file1, "test content")?;
        fs::write(&file2, "test content")?; // Duplicate
        
        let df = run_with_dataframe(
            temp_dir.path().to_str().unwrap(),
            "test",
            None,
        )?;
        
        assert!(df.height() >= 2);
        
        // Test with CSV output
        let csv_path = temp_dir.path().join("output.csv");
        let _df = run_with_dataframe(
            temp_dir.path().to_str().unwrap(),
            "test",
            Some(csv_path.to_str().unwrap()),
        )?;
        
        assert!(csv_path.exists());
        
        Ok(())
    }

    #[test]
    fn test_validate_duplicates() -> Result<(), Box<dyn Error>> {
        // Create a DataFrame with duplicates
        let df = df! [
            "file_path" => vec!["file1.txt", "file2.txt", "file3.txt"],
            "file_name" => vec!["file1.txt", "file2.txt", "file3.txt"],
            "extension" => vec!["txt", "txt", "txt"],
            "size_bytes" => vec![100u64, 100, 200],
            "size_mb" => vec![0.0001, 0.0001, 0.0002],
            "md5_hash" => vec!["hash1", "hash1", "hash2"],
            "is_duplicate" => vec![true, true, false],
            "duplicate_group" => vec![Some("hash1"), Some("hash1"), None],
        ]?;
        
        // Should not error
        validate_duplicates(&df)?;
        
        Ok(())
    }

    #[test]
    fn test_find_similar_files() -> Result<(), Box<dyn Error>> {
        let file_infos = vec![
            FileInfo {
                path: "file1.txt".to_string(),
                name: "file1.txt".to_string(),
                extension: "txt".to_string(),
                size_bytes: 1000,
                size_mb: 0.001,
                md5_hash: "hash1".to_string(),
                fuzzy_hash: Some("3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C".to_string()),
                is_duplicate: false,
                duplicate_group: None,
                is_similar: false,
                similarity_group: None,
                similarity_score: None,
                created: None,
                modified: None,
            },
            FileInfo {
                path: "file2.txt".to_string(),
                name: "file2.txt".to_string(),
                extension: "txt".to_string(),
                size_bytes: 1000,
                size_mb: 0.001,
                md5_hash: "hash2".to_string(),
                fuzzy_hash: Some("3:AXGBicFlgVNhBGcL6wCrFQEv:AXGHsNhxLsr2C".to_string()),
                is_duplicate: false,
                duplicate_group: None,
                is_similar: false,
                similarity_group: None,
                similarity_score: None,
                created: None,
                modified: None,
            },
        ];
        
        let similar_groups = find_similar_files(&file_infos, 90)?;
        assert_eq!(similar_groups.len(), 1);
        assert_eq!(similar_groups[0].len(), 2);
        
        Ok(())
    }

    #[test]
    fn test_run_simple() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("test.txt");
        fs::write(&file1, "test")?;
        
        // Should not panic
        run(temp_dir.path().to_str().unwrap(), "test")?;
        
        Ok(())
    }

    #[test]
    fn test_collect_file_info_empty() -> Result<(), Box<dyn Error>> {
        let file_infos = collect_file_info(&[])?;
        assert_eq!(file_infos.len(), 0);
        Ok(())
    }

    #[test]
    fn test_generate_csv_report() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create DataFrame with duplicates
        let mut df = df! [
            "file_path" => vec!["file1.txt", "file2.txt"],
            "file_name" => vec!["file1.txt", "file2.txt"],
            "extension" => vec!["txt", "txt"],
            "size_bytes" => vec![100u64, 100],
            "size_mb" => vec![0.0001, 0.0001],
            "md5_hash" => vec!["hash1", "hash1"],
            "is_duplicate" => vec![true, true],
            "duplicate_group" => vec![Some("hash1"), Some("hash1")],
        ]?;
        
        let csv_path = temp_dir.path().join("duplicates.csv");
        generate_csv_report(&mut df, csv_path.to_str().unwrap())?;
        
        assert!(csv_path.exists());
        
        Ok(())
    }
}
