use crate::models::mcp::{
    CountArgs, DedupeArgs, McpRequest, McpResponse, OutliersArgs, SearchArgs, ToolCallParams,
};
use crate::{PatternType, WalkOptions};
use polars::prelude::IntoLazy;
use serde_json::{json, Value};
use tracing::{error, info};

pub fn handle_initialize(request: McpRequest) -> McpResponse {
    info!("Handling initialize request");

    McpResponse::success(
        request.id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "rclean",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

pub fn handle_tools_list(request: McpRequest) -> McpResponse {
    info!("Handling tools/list request");

    McpResponse::success(
        request.id,
        json!({
            "tools": [
                {
                    "name": "dedupe",
                    "description": "Find duplicate files in a directory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to scan for duplicates"
                            },
                            "pattern": {
                                "type": "string",
                                "description": "Pattern to match files",
                                "default": ""
                            },
                            "pattern_type": {
                                "type": "string",
                                "enum": ["literal", "glob", "regex"],
                                "default": "literal"
                            },
                            "hidden": {
                                "type": "boolean",
                                "description": "Include hidden files",
                                "default": false
                            },
                            "no_ignore": {
                                "type": "boolean",
                                "description": "Ignore .gitignore rules",
                                "default": false
                            },
                            "max_depth": {
                                "type": "integer",
                                "description": "Maximum depth to traverse"
                            },
                            "similarity": {
                                "type": "integer",
                                "description": "Similarity threshold (0-100) for fuzzy matching",
                                "minimum": 0,
                                "maximum": 100
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "search",
                    "description": "Search for files matching a pattern",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to search in"
                            },
                            "pattern": {
                                "type": "string",
                                "description": "Pattern to match files",
                                "default": ""
                            },
                            "pattern_type": {
                                "type": "string",
                                "enum": ["literal", "glob", "regex"],
                                "default": "literal"
                            },
                            "hidden": {
                                "type": "boolean",
                                "description": "Include hidden files",
                                "default": false
                            },
                            "no_ignore": {
                                "type": "boolean",
                                "description": "Ignore .gitignore rules",
                                "default": false
                            },
                            "max_depth": {
                                "type": "integer",
                                "description": "Maximum depth to traverse"
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "count",
                    "description": "Count files matching a pattern",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to count files in"
                            },
                            "pattern": {
                                "type": "string",
                                "description": "Pattern to match files",
                                "default": ""
                            },
                            "pattern_type": {
                                "type": "string",
                                "enum": ["literal", "glob", "regex"],
                                "default": "literal"
                            },
                            "hidden": {
                                "type": "boolean",
                                "description": "Include hidden files",
                                "default": false
                            },
                            "no_ignore": {
                                "type": "boolean",
                                "description": "Ignore .gitignore rules",
                                "default": false
                            },
                            "max_depth": {
                                "type": "integer",
                                "description": "Maximum depth to traverse"
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "outliers",
                    "description": "Detect storage outliers (large files, hidden consumers, patterns)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to analyze for outliers"
                            },
                            "min_size": {
                                "type": "string",
                                "description": "Minimum file size to consider (e.g., 100MB, 1GB)"
                            },
                            "top_n": {
                                "type": "integer",
                                "description": "Number of top outliers to return",
                                "default": 20
                            },
                            "std_dev_threshold": {
                                "type": "number",
                                "description": "Standard deviations from mean to consider as outlier",
                                "default": 2.0
                            },
                            "check_hidden_consumers": {
                                "type": "boolean",
                                "description": "Check for hidden space consumers (node_modules, .git, etc.)",
                                "default": true
                            },
                            "check_patterns": {
                                "type": "boolean",
                                "description": "Check for file patterns (backups, logs, etc.)",
                                "default": true
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "analyze_file_clusters",
                    "description": "Detect clusters of similar large files using DBSCAN",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Directory path to analyze"
                            },
                            "min_similarity": {
                                "type": "integer",
                                "minimum": 50,
                                "maximum": 100,
                                "default": 70,
                                "description": "Minimum similarity percentage for clustering"
                            },
                            "min_cluster_size": {
                                "type": "integer",
                                "minimum": 2,
                                "default": 2,
                                "description": "Minimum files to form a cluster"
                            },
                            "min_file_size": {
                                "type": "string",
                                "default": "10MB",
                                "description": "Minimum file size to consider"
                            },
                            "files": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Specific files to analyze (for tool composition)"
                            }
                        },
                        "required": ["path"]
                    }
                }
            ]
        }),
    )
}

pub async fn handle_tool_call(request: McpRequest) -> McpResponse {
    let tool_params = match parse_tool_call_params(request.params, &request.id) {
        Ok(params) => params,
        Err(response) => return *response,
    };

    match tool_params.name.as_str() {
        "dedupe" => handle_dedupe_tool(request.id, tool_params.arguments).await,
        "search" => handle_search_tool(request.id, tool_params.arguments).await,
        "count" => handle_count_tool(request.id, tool_params.arguments).await,
        "outliers" => handle_outliers_tool(request.id, tool_params.arguments).await,
        "analyze_file_clusters" => {
            handle_analyze_clusters_tool(request.id, tool_params.arguments).await
        },
        _ => McpResponse::error(
            request.id,
            -32602,
            format!("Unknown tool: {}", tool_params.name),
        ),
    }
}

fn parse_tool_call_params(
    params: Option<Value>,
    request_id: &Value,
) -> Result<ToolCallParams, Box<McpResponse>> {
    let params = match params {
        Some(p) => p,
        None => {
            return Err(Box::new(McpResponse::error(
                request_id.clone(),
                -32602,
                "Invalid params: missing tool call parameters".to_string(),
            )));
        },
    };

    match serde_json::from_value(params) {
        Ok(p) => Ok(p),
        Err(e) => Err(Box::new(McpResponse::error(
            request_id.clone(),
            -32602,
            format!("Invalid params: {}", e),
        ))),
    }
}

async fn handle_dedupe_tool(id: Value, arguments: Value) -> McpResponse {
    let args: DedupeArgs = match serde_json::from_value(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid arguments for dedupe: {}", e));
        },
    };

    let walk_options = WalkOptions {
        include_hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        respect_ignore: !args.no_ignore,
        max_depth: args.max_depth,
    };

    let pattern = match create_pattern(&args.pattern, &args.pattern_type) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid pattern: {}", e));
        },
    };

    // Run deduplication
    let result = if let Some(threshold) = args.similarity {
        crate::run_with_similarity(&args.path, &pattern, &walk_options, threshold, None)
    } else {
        crate::run_with_advanced_options(&args.path, &pattern, &walk_options, None)
    };

    match result {
        Ok(df) => {
            // Convert DataFrame results to JSON
            let height = df.height();
            let duplicates = df
                .clone()
                .lazy()
                .filter(polars::prelude::col("is_duplicate").eq(polars::prelude::lit(true)))
                .collect()
                .map(|df| df.height())
                .unwrap_or(0);

            McpResponse::success(
                id,
                json!({
                    "total_files": height,
                    "duplicate_files": duplicates,
                    "duplicate_groups": df
                        .column("duplicate_group")
                        .ok()
                        .and_then(|col| col.unique().ok())
                        .map(|unique| unique.len().saturating_sub(1)) // Subtract 1 for null values, prevent underflow
                        .unwrap_or(0),
                    "message": format!("Found {} duplicate files in {} total files", duplicates, height)
                }),
            )
        },
        Err(e) => {
            error!("Dedupe error: {}", e);
            McpResponse::error(id, -32603, format!("Dedupe failed: {}", e))
        },
    }
}

async fn handle_search_tool(id: Value, arguments: Value) -> McpResponse {
    let args: SearchArgs = match serde_json::from_value(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid arguments for search: {}", e));
        },
    };

    let walk_options = WalkOptions {
        include_hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        respect_ignore: !args.no_ignore,
        max_depth: args.max_depth,
    };

    let pattern = match create_pattern(&args.pattern, &args.pattern_type) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid pattern: {}", e));
        },
    };

    match crate::walk_with_options(&args.path, &walk_options) {
        Ok(files) => {
            let matched_files = crate::find_advanced(&files, &pattern);
            McpResponse::success(
                id,
                json!({
                    "files": matched_files,
                    "count": matched_files.len(),
                    "message": format!("Found {} files matching pattern", matched_files.len())
                }),
            )
        },
        Err(e) => {
            error!("Search error: {}", e);
            McpResponse::error(id, -32603, format!("Search failed: {}", e))
        },
    }
}

async fn handle_count_tool(id: Value, arguments: Value) -> McpResponse {
    let args: CountArgs = match serde_json::from_value(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid arguments for count: {}", e));
        },
    };

    let walk_options = WalkOptions {
        include_hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        respect_ignore: !args.no_ignore,
        max_depth: args.max_depth,
    };

    let pattern = match create_pattern(&args.pattern, &args.pattern_type) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(id, -32602, format!("Invalid pattern: {}", e));
        },
    };

    match crate::walk_with_options(&args.path, &walk_options) {
        Ok(files) => {
            let matched_files = crate::find_advanced(&files, &pattern);
            McpResponse::success(
                id,
                json!({
                    "count": matched_files.len(),
                    "message": format!("Found {} files matching pattern", matched_files.len())
                }),
            )
        },
        Err(e) => {
            error!("Count error: {}", e);
            McpResponse::error(id, -32603, format!("Count failed: {}", e))
        },
    }
}

fn create_pattern(pattern: &str, pattern_type: &str) -> Result<PatternType, String> {
    match pattern_type {
        "literal" | "" => Ok(PatternType::Literal(pattern.to_string())),
        "glob" => {
            let glob =
                globset::Glob::new(pattern).map_err(|e| format!("Invalid glob pattern: {}", e))?;
            let mut builder = globset::GlobSetBuilder::new();
            builder.add(glob);
            let globset = builder
                .build()
                .map_err(|e| format!("Failed to build globset: {}", e))?;
            Ok(PatternType::Glob(globset))
        },
        "regex" => {
            let regex =
                regex::Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
            Ok(PatternType::Regex(regex))
        },
        _ => Err(format!("Unknown pattern type: {}", pattern_type)),
    }
}

async fn handle_outliers_tool(id: Value, arguments: Value) -> McpResponse {
    let args: OutliersArgs = match serde_json::from_value(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                id,
                -32602,
                format!("Invalid arguments for outliers: {}", e),
            );
        },
    };

    // Parse min_size if provided
    let min_size_bytes = args.min_size.as_ref().and_then(|s| parse_size(s).ok());

    let options = crate::outliers::OutlierOptions {
        min_size: min_size_bytes,
        top_n: Some(args.top_n),
        std_dev_threshold: args.std_dev_threshold,
        check_hidden_consumers: args.check_hidden_consumers,
        include_empty_dirs: false,
        check_patterns: args.check_patterns,
        enable_clustering: false, // Not enabled by default in outliers tool
        cluster_similarity_threshold: 70,
        min_cluster_size: 2,
    };

    match crate::outliers::detect_outliers(&args.path, &options) {
        Ok(report) => {
            // Convert report to JSON response
            let result = json!({
                "total_files_analyzed": report.total_files_analyzed,
                "total_size_analyzed": report.total_size_analyzed,
                "total_size_gb": report.total_size_analyzed as f64 / (1024.0 * 1024.0 * 1024.0),
                "large_files": report.large_files.iter().map(|o| json!({
                    "path": o.path.to_string_lossy(),
                    "size_bytes": o.size_bytes,
                    "size_mb": o.size_mb,
                    "percentage_of_total": o.percentage_of_total,
                    "std_devs_from_mean": o.std_devs_from_mean,
                })).collect::<Vec<_>>(),
                "hidden_consumers": report.hidden_consumers.iter().map(|c| json!({
                    "path": c.path.to_string_lossy(),
                    "pattern_type": c.pattern_type,
                    "total_size_bytes": c.total_size_bytes,
                    "file_count": c.file_count,
                    "recommendation": c.recommendation,
                })).collect::<Vec<_>>(),
                "pattern_groups": report.pattern_groups.iter().map(|g| json!({
                    "pattern": g.pattern,
                    "sample_files": g.sample_files.iter().map(|f| f.to_string_lossy()).collect::<Vec<_>>(),
                    "total_size_bytes": g.total_size_bytes,
                    "count": g.count,
                })).collect::<Vec<_>>(),
                "message": format!("Found {} outliers across {} files",
                    report.large_files.len() + report.hidden_consumers.len() + report.pattern_groups.len(),
                    report.total_files_analyzed
                )
            });

            McpResponse::success(id, result)
        },
        Err(e) => {
            error!("Outliers detection error: {}", e);
            McpResponse::error(id, -32603, format!("Outliers detection failed: {}", e))
        },
    }
}

async fn handle_analyze_clusters_tool(id: Value, arguments: Value) -> McpResponse {
    // Parse arguments
    let path = arguments["path"].as_str().unwrap_or(".");
    let min_similarity = arguments["min_similarity"].as_u64().unwrap_or(70) as u8;
    let min_cluster_size = arguments["min_cluster_size"].as_u64().unwrap_or(2) as usize;
    let min_file_size = arguments["min_file_size"].as_str().unwrap_or("10MB");

    // Parse min file size
    let min_size_bytes = parse_size(min_file_size).unwrap_or(10 * 1024 * 1024);

    // Support tool composition via files parameter
    let files = if let Some(file_list) = arguments["files"].as_array() {
        // Analyze specific files from previous tool output
        let mut file_infos = Vec::new();
        for file_path in file_list {
            if let Some(path_str) = file_path.as_str() {
                if let Ok(metadata) = std::fs::metadata(path_str) {
                    if metadata.is_file() && metadata.len() >= min_size_bytes {
                        // Compute SSDEEP hash for large files
                        let ssdeep_hash = if let Ok(content) = std::fs::read(path_str) {
                            ssdeep::hash(&content).ok()
                        } else {
                            None
                        };

                        file_infos.push(crate::outliers::SimpleFileInfo {
                            path: std::path::PathBuf::from(path_str),
                            size_bytes: metadata.len(),
                            ssdeep_hash,
                        });
                    }
                }
            }
        }
        file_infos
    } else {
        // Full directory scan for large files
        let walk_options = WalkOptions::default();
        let all_files = match crate::walk_with_options(path, &walk_options) {
            Ok(files) => files,
            Err(e) => {
                return McpResponse::error(id, -32603, format!("Error scanning directory: {}", e));
            },
        };

        // Collect large files with SSDEEP hashes
        let mut file_infos = Vec::new();
        for file_path in all_files {
            if let Ok(metadata) = std::fs::metadata(&file_path) {
                if metadata.is_file() && metadata.len() >= min_size_bytes {
                    // Compute SSDEEP hash
                    let ssdeep_hash = if let Ok(content) = std::fs::read(&file_path) {
                        ssdeep::hash(&content).ok()
                    } else {
                        None
                    };

                    file_infos.push(crate::outliers::SimpleFileInfo {
                        path: std::path::PathBuf::from(file_path),
                        size_bytes: metadata.len(),
                        ssdeep_hash,
                    });
                }
            }
        }
        file_infos
    };

    // Perform clustering
    match crate::clustering::detect_large_file_clusters(&files, min_similarity, min_cluster_size) {
        Ok(clusters) => {
            let result = json!({
                "clusters": clusters.iter().map(|c| json!({
                    "cluster_id": c.cluster_id,
                    "files": c.files.iter().map(|f| json!({
                        "path": f.path.to_string_lossy(),
                        "size_bytes": f.size_bytes,
                        "size_mb": f.size_bytes as f64 / (1024.0 * 1024.0),
                    })).collect::<Vec<_>>(),
                    "total_size": c.total_size,
                    "total_size_mb": c.total_size as f64 / (1024.0 * 1024.0),
                    "avg_similarity": c.avg_similarity,
                    "density": c.density,
                })).collect::<Vec<_>>(),
                "summary": {
                    "total_clusters": clusters.len(),
                    "total_files": clusters.iter().map(|c| c.files.len()).sum::<usize>(),
                    "total_size": clusters.iter().map(|c| c.total_size).sum::<u64>(),
                    "files": clusters.iter()
                        .flat_map(|c| c.files.iter().map(|f| f.path.to_string_lossy().to_string()))
                        .collect::<Vec<_>>()
                },
                "message": format!("Found {} clusters containing {} similar files",
                    clusters.len(),
                    clusters.iter().map(|c| c.files.len()).sum::<usize>()
                )
            });

            McpResponse::success(id, result)
        },
        Err(e) => McpResponse::error(id, -32603, format!("Error detecting clusters: {}", e)),
    }
}

fn parse_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim().to_uppercase();

    if let Some(num_str) = size_str.strip_suffix("KB") {
        num_str
            .trim()
            .parse::<f64>()
            .map(|n| (n * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("MB") {
        num_str
            .trim()
            .parse::<f64>()
            .map(|n| (n * 1024.0 * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("GB") {
        num_str
            .trim()
            .parse::<f64>()
            .map(|n| (n * 1024.0 * 1024.0 * 1024.0) as u64)
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else if let Some(num_str) = size_str.strip_suffix("B") {
        num_str
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("Invalid size: {}", size_str))
    } else {
        size_str
            .parse::<u64>()
            .map_err(|_| format!("Invalid size: {} (use B, KB, MB, or GB suffix)", size_str))
    }
}
