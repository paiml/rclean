//! Outlier detection module for finding files that consume disproportionate disk space.
//!
//! This module provides functionality to detect various types of storage outliers:
//! - Large files that are statistical outliers
//! - Rapidly growing files and directories
//! - Common space-wasting patterns
//! - Hidden space consumers
//! - Sparse files and empty directories

use crate::{walk_with_options, WalkOptions};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Options for outlier detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierOptions {
    /// Minimum file size to consider (in bytes)
    pub min_size: Option<u64>,
    /// Maximum number of results to return
    pub top_n: Option<usize>,
    /// Number of standard deviations from mean to consider as outlier
    pub std_dev_threshold: f64,
    /// Include hidden space consumers (node_modules, .git, etc.)
    pub check_hidden_consumers: bool,
    /// Include empty directories in results
    pub include_empty_dirs: bool,
    /// Check for common patterns (logs, backups, etc.)
    pub check_patterns: bool,
    /// Enable clustering of similar large files
    pub enable_clustering: bool,
    /// Minimum similarity percentage for clustering (50-100)
    pub cluster_similarity_threshold: u8,
    /// Minimum files to form a cluster
    pub min_cluster_size: usize,
}

impl Default for OutlierOptions {
    fn default() -> Self {
        Self {
            min_size: None,
            top_n: Some(20),
            std_dev_threshold: 2.0,
            check_hidden_consumers: true,
            include_empty_dirs: false,
            check_patterns: true,
            enable_clustering: false,
            cluster_similarity_threshold: 70,
            min_cluster_size: 2,
        }
    }
}

/// Represents a file that is a statistical outlier by size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileOutlier {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub percentage_of_total: f64,
    pub std_devs_from_mean: f64,
}

/// Represents a known space consumer pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenConsumer {
    pub path: PathBuf,
    pub pattern_type: String,
    pub total_size_bytes: u64,
    pub file_count: usize,
    pub recommendation: String,
}

/// Represents a group of files with similar naming patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternGroup {
    pub pattern: String,
    pub count: usize,
    pub total_size_bytes: u64,
    pub sample_files: Vec<PathBuf>,
}

/// Report containing all detected outliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierReport {
    pub large_files: Vec<LargeFileOutlier>,
    pub hidden_consumers: Vec<HiddenConsumer>,
    pub pattern_groups: Vec<PatternGroup>,
    pub large_file_clusters: Vec<crate::clustering::LargeFileCluster>,
    pub total_size_analyzed: u64,
    pub total_files_analyzed: usize,
}

/// Simple file info structure that doesn't compute hashes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleFileInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub ssdeep_hash: Option<String>,
}

/// Known patterns that commonly consume space
const HIDDEN_CONSUMER_PATTERNS: &[(&str, &str, &str)] = &[
    ("node_modules", "Node.js dependencies", "Consider using npm prune or clearing unused dependencies"),
    (".git", "Git repository data", "Run git gc to clean up unnecessary files"),
    ("target", "Rust build artifacts", "Run cargo clean to remove build artifacts"),
    ("build", "Build output directory", "Clean build artifacts if not needed"),
    ("dist", "Distribution files", "Remove old distribution builds"),
    (".venv", "Python virtual environment", "Recreate virtual environment if needed"),
    ("__pycache__", "Python cache files", "Safe to delete, will be regenerated"),
    (".cache", "Application cache", "Review and clean old cache files"),
    ("tmp", "Temporary files", "Clean up old temporary files"),
    ("logs", "Log files", "Archive or delete old logs"),
];

/// Detect outliers in the given path
///
/// # Examples
///
/// ```no_run
/// use rclean::outliers::{detect_outliers, OutlierOptions};
///
/// let options = OutlierOptions::default();
/// let report = detect_outliers("/home/user", &options).unwrap();
/// 
/// println!("Found {} large file outliers", report.large_files.len());
/// for outlier in &report.large_files {
///     println!("{}: {:.2} MB ({:.1}% of total)", 
///         outlier.path.display(), 
///         outlier.size_mb,
///         outlier.percentage_of_total
///     );
/// }
/// ```
pub fn detect_outliers(path: &str, options: &OutlierOptions) -> Result<OutlierReport, Box<dyn std::error::Error>> {
    let walk_options = WalkOptions::default();
    let files = walk_with_options(path, &walk_options)?;
    
    if files.is_empty() {
        return Ok(OutlierReport {
            large_files: vec![],
            hidden_consumers: vec![],
            pattern_groups: vec![],
            large_file_clusters: vec![],
            total_size_analyzed: 0,
            total_files_analyzed: 0,
        });
    }
    
    // Collect file information without hashing
    let file_infos: Vec<SimpleFileInfo> = files
        .iter()
        .filter_map(|path_str| {
            let path = Path::new(path_str);
            fs::metadata(path).ok().map(|metadata| {
                // Only compute SSDEEP hash for large files if clustering is enabled
                let ssdeep_hash = if options.enable_clustering && metadata.len() >= 1024 * 1024 {
                    fs::read(path).ok().and_then(|content| ssdeep::hash(&content).ok())
                } else {
                    None
                };
                
                SimpleFileInfo {
                    path: path.to_path_buf(),
                    size_bytes: metadata.len(),
                    ssdeep_hash,
                }
            })
        })
        .collect();
    
    let total_size: u64 = file_infos.iter().map(|f| f.size_bytes).sum();
    let total_files = file_infos.len();
    
    // Detect large file outliers
    let large_files = detect_large_file_outliers(&file_infos, total_size, options);
    
    // Detect hidden consumers
    let hidden_consumers = if options.check_hidden_consumers {
        detect_hidden_consumers(&files, &file_infos)
    } else {
        vec![]
    };
    
    // Detect pattern groups
    let pattern_groups = if options.check_patterns {
        detect_pattern_groups(&file_infos)
    } else {
        vec![]
    };
    
    // Detect large file clusters if enabled
    let large_file_clusters = if options.enable_clustering {
        // Only cluster large files that have SSDEEP hashes
        let large_files_for_clustering: Vec<SimpleFileInfo> = file_infos
            .iter()
            .filter(|f| f.ssdeep_hash.is_some() && f.size_bytes >= options.min_size.unwrap_or(1024 * 1024))
            .cloned()
            .collect();
        
        crate::clustering::detect_large_file_clusters(
            &large_files_for_clustering,
            options.cluster_similarity_threshold,
            options.min_cluster_size,
        ).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };
    
    Ok(OutlierReport {
        large_files,
        hidden_consumers,
        pattern_groups,
        large_file_clusters,
        total_size_analyzed: total_size,
        total_files_analyzed: total_files,
    })
}

fn detect_large_file_outliers(
    files: &[SimpleFileInfo],
    total_size: u64,
    options: &OutlierOptions,
) -> Vec<LargeFileOutlier> {
    if files.is_empty() {
        return vec![];
    }
    
    // Calculate statistics
    let sizes: Vec<f64> = files.iter().map(|f| f.size_bytes as f64).collect();
    let mean = sizes.iter().sum::<f64>() / sizes.len() as f64;
    
    // Calculate standard deviation
    let variance = sizes.iter()
        .map(|size| {
            let diff = size - mean;
            diff * diff
        })
        .sum::<f64>() / sizes.len() as f64;
    
    let std_dev = variance.sqrt();
    
    // Find outliers
    let mut outliers: Vec<LargeFileOutlier> = files
        .iter()
        .filter_map(|f| {
            // Apply min size filter if specified
            if let Some(min_size) = options.min_size {
                if f.size_bytes < min_size {
                    return None;
                }
            }
            
            let z_score = if std_dev > 0.0 {
                (f.size_bytes as f64 - mean) / std_dev
            } else {
                0.0
            };
            
            if z_score > options.std_dev_threshold {
                let outlier = LargeFileOutlier {
                    path: f.path.clone(),
                    size_bytes: f.size_bytes,
                    size_mb: f.size_bytes as f64 / (1024.0 * 1024.0),
                    percentage_of_total: (f.size_bytes as f64 / total_size as f64) * 100.0,
                    std_devs_from_mean: z_score,
                };
                Some(outlier)
            } else {
                None
            }
        })
        .collect();
    
    // Sort by size descending
    outliers.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    
    // Apply top_n limit if specified
    if let Some(top_n) = options.top_n {
        outliers.truncate(top_n);
    }
    
    outliers
}

fn detect_hidden_consumers(paths: &[String], file_infos: &[SimpleFileInfo]) -> Vec<HiddenConsumer> {
    let mut consumers = Vec::new();
    let mut path_to_info: HashMap<&Path, &SimpleFileInfo> = HashMap::new();
    
    for info in file_infos {
        path_to_info.insert(&info.path, info);
    }
    
    // Group paths by directory
    let mut dir_contents: HashMap<PathBuf, Vec<&str>> = HashMap::new();
    for path in paths {
        if let Some(parent) = Path::new(path).parent() {
            dir_contents.entry(parent.to_path_buf())
                .or_default()
                .push(path);
        }
    }
    
    // Check each directory for known patterns
    for (dir, contents) in dir_contents {
        for &(pattern, description, recommendation) in HIDDEN_CONSUMER_PATTERNS {
            let dir_name = dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if dir_name == pattern || dir.ends_with(pattern) {
                // Calculate total size and file count
                let mut total_size = 0u64;
                let mut file_count = 0;
                
                for path_str in &contents {
                    let path = Path::new(path_str);
                    if path.starts_with(&dir) {
                        if let Some(info) = path_to_info.get(path) {
                            total_size += info.size_bytes;
                            file_count += 1;
                        }
                    }
                }
                
                if total_size > 0 {
                    consumers.push(HiddenConsumer {
                        path: dir.clone(),
                        pattern_type: description.to_string(),
                        total_size_bytes: total_size,
                        file_count,
                        recommendation: recommendation.to_string(),
                    });
                }
                break;
            }
        }
    }
    
    // Sort by size descending
    consumers.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes));
    consumers
}

fn detect_pattern_groups(files: &[SimpleFileInfo]) -> Vec<PatternGroup> {
    let mut pattern_map: HashMap<String, Vec<&SimpleFileInfo>> = HashMap::new();
    
    for file in files {
        if let Some(file_name) = file.path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            
            // Check for numbered patterns (e.g., backup-001.tar, backup-002.tar)
            if let Some((prefix, suffix)) = detect_numbered_pattern(&file_name_str) {
                let pattern = format!("{}*{}", prefix, suffix);
                pattern_map.entry(pattern).or_default().push(file);
            }
            // Check for dated patterns (e.g., log-2024-01-01.txt)
            else if let Some((prefix, suffix)) = detect_dated_pattern(&file_name_str) {
                let pattern = format!("{}*{}", prefix, suffix);
                pattern_map.entry(pattern).or_default().push(file);
            }
        }
    }
    
    // Convert to PatternGroup and filter by minimum count
    let mut groups: Vec<PatternGroup> = pattern_map
        .into_iter()
        .filter(|(_, files)| files.len() >= 3) // At least 3 files to be considered a pattern
        .map(|(pattern, files)| {
            let total_size: u64 = files.iter().map(|f| f.size_bytes).sum();
            let sample_files: Vec<PathBuf> = files.iter()
                .take(5)
                .map(|f| f.path.clone())
                .collect();
                
            PatternGroup {
                pattern,
                count: files.len(),
                total_size_bytes: total_size,
                sample_files,
            }
        })
        .collect();
    
    // Sort by total size descending
    groups.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes));
    groups
}

fn detect_numbered_pattern(filename: &str) -> Option<(&str, &str)> {
    // Look for patterns like: prefix-001.ext, prefix_123.ext, prefix001.ext
    let re = regex::Regex::new(r"^(.+?)[-_]?(\d{2,})(\..+)?$").ok()?;
    
    if let Some(captures) = re.captures(filename) {
        let prefix = captures.get(1)?.as_str();
        let suffix = captures.get(3).map_or("", |m| m.as_str());
        Some((prefix, suffix))
    } else {
        None
    }
}

fn detect_dated_pattern(filename: &str) -> Option<(&str, &str)> {
    // Look for patterns with dates: prefix-2024-01-01.ext, prefix_2024_01_01.ext
    let re = regex::Regex::new(r"^(.+?)[-_]?(\d{4}[-_]?\d{2}[-_]?\d{2})(\..+)?$").ok()?;
    
    if let Some(captures) = re.captures(filename) {
        let prefix = captures.get(1)?.as_str();
        let suffix = captures.get(3).map_or("", |m| m.as_str());
        Some((prefix, suffix))
    } else {
        None
    }
}

/// Convert outlier report to a Polars DataFrame for further analysis
///
/// # Examples
///
/// ```no_run
/// use rclean::outliers::{detect_outliers, outliers_to_dataframe, OutlierOptions};
///
/// let options = OutlierOptions::default();
/// let report = detect_outliers("/home/user", &options).unwrap();
/// let df = outliers_to_dataframe(&report).unwrap();
/// 
/// // Now you can use Polars operations on the DataFrame
/// println!("{}", df);
/// ```
pub fn outliers_to_dataframe(report: &OutlierReport) -> Result<DataFrame, PolarsError> {
    let file_paths: Vec<String> = report.large_files.iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();
    
    let size_mb: Vec<f64> = report.large_files.iter()
        .map(|f| f.size_mb)
        .collect();
    
    let percentage: Vec<f64> = report.large_files.iter()
        .map(|f| f.percentage_of_total)
        .collect();
    
    let std_devs: Vec<f64> = report.large_files.iter()
        .map(|f| f.std_devs_from_mean)
        .collect();
    
    let df = DataFrame::new(vec![
        Series::new("file_path", file_paths),
        Series::new("size_mb", size_mb),
        Series::new("percentage_of_total", percentage),
        Series::new("std_devs_from_mean", std_devs),
    ])?;
    
    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_numbered_pattern() {
        assert_eq!(detect_numbered_pattern("backup-001.tar"), Some(("backup", ".tar")));
        assert_eq!(detect_numbered_pattern("file_123.log"), Some(("file", ".log")));
        assert_eq!(detect_numbered_pattern("test123"), Some(("test", "")));
        assert_eq!(detect_numbered_pattern("no-numbers.txt"), None);
    }
    
    #[test]
    fn test_detect_dated_pattern() {
        assert_eq!(detect_dated_pattern("log-2024-01-01.txt"), Some(("log", ".txt")));
        assert_eq!(detect_dated_pattern("backup_2024_12_31.tar"), Some(("backup", ".tar")));
        assert_eq!(detect_dated_pattern("report-2024-01-01"), Some(("report", "")));
        assert_eq!(detect_dated_pattern("no-date.txt"), None);
    }
}