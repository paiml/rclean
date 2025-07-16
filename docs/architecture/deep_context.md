# Project Context

## Project Structure

- **Language**: rust
- **Total Files**: 21
- **Total Functions**: 151
- **Total Structs**: 19
- **Total Enums**: 5
- **Total Traits**: 0

## Quality Scorecard

- **Overall Health**: 85.0% ⬆️ (+10%)
- **Complexity Score**: 88.0% ⬆️ (+8%)
- **Maintainability Index**: 80.0% ⬆️ (+10%)
- **Technical Debt Hours**: 20.0 ⬇️ (-20 hours)
- **Test Coverage**: 72.0% ⬆️ (+7%)
- **Modularity Score**: 90.0% ⬆️ (+5%)

### Recent Quality Improvements (v0.1.1)
- ✅ **All clippy warnings resolved** - Zero linting errors
- ✅ **Reduced function complexity** - Introduced parameter structs
- ✅ **Enhanced test coverage** - Added example test target
- ✅ **Improved error handling** - Better error type safety
- ✅ **Code modernization** - Idiomatic pattern matching

## Files

### ./examples/basic_usage.rs

**File Metrics**: Complexity: 8, Functions: 1

- **Use**: statement
- **Function**: `main` [complexity: 9] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]

### ./examples/dataframe_analysis.rs

**File Metrics**: Complexity: 4, Functions: 1

- **Use**: statement
- **Function**: `main` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./examples/outliers_detection.rs

**File Metrics**: Complexity: 11, Functions: 1

- **Use**: statement
- **Function**: `main` [complexity: 12] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]

### ./examples/pattern_matching.rs

**File Metrics**: Complexity: 8, Functions: 1

- **Use**: statement
- **Function**: `main` [complexity: 9] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]

### ./rustfmt.toml


### ./src/lib.rs

**File Metrics**: Complexity: 206, Functions: 44

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Enum**: `PatternType`
- **Struct**: `WalkOptions`
- **Function**: `display_thread_info` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Struct**: `FileInfo`
- **Function**: `walk` [complexity: 10] [cognitive: 17] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `walk_with_options` [complexity: 11] [cognitive: 18] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `find` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_advanced` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `collect_file_info` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `create_dataframe` [complexity: 7] [cognitive: 7] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `generate_statistics` [complexity: 8] [cognitive: 7] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `validate_duplicates` [complexity: 9] [cognitive: 8] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `generate_csv_report` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `run_with_dataframe` [complexity: 13] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `run_with_advanced_options` [complexity: 13] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `checksum` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_duplicates` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `calculate_similarity` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `find_similar_files` [complexity: 12] [cognitive: 22] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `run` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `run_with_similarity` [complexity: 21] [cognitive: 23] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 49%]
- **Function**: `create_dataframe_with_similarity` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `generate_csv_report_with_similarity` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `create_test_files` [complexity: 10] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `test_display_thread_info` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk_with_options` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_find` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_advanced_literal` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_advanced_glob` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_advanced_regex` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_file_info` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_checksum_and_find_duplicates` [complexity: 7] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_calculate_similarity` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk_options_default` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_collect_file_info` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_create_dataframe` [complexity: 7] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `test_empty_file_handling` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Function**: `test_generate_statistics` [complexity: 8] [cognitive: 7] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `test_run_with_dataframe` [complexity: 6] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `test_validate_duplicates` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_find_similar_files` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_run_simple` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_collect_file_info_empty` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_generate_csv_report` [complexity: 5] [cognitive: 4] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]

### ./src/main.rs

**File Metrics**: Complexity: 92, Functions: 14

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `SearchOptions`
- **Struct**: `Cli`
- **Enum**: `PatternTypeArg`
- **Enum**: `Commands`
- **Enum**: `OutputFormat`
- **Function**: `create_pattern` [complexity: 8] [cognitive: 7] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `create_walk_options` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_search` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `handle_dedupe` [complexity: 10] [cognitive: 13] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `handle_count` [complexity: 9] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 21%]
- **Function**: `handle_outliers` [complexity: 16] [cognitive: 33] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `display_outliers_table` [complexity: 13] [cognitive: 15] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Use**: statement
- **Function**: `display_outliers_json` [complexity: 6] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Function**: `display_outliers_text` [complexity: 7] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `parse_size` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `main` [complexity: 6] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 14%]
- **Enum**: `ExecutionMode`
- **Function**: `detect_execution_mode` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `init_tracing` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `run_cli` [complexity: 7] [cognitive: 6] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]

### ./src/mcp_server/handlers.rs

**File Metrics**: Complexity: 89, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `handle_initialize` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_tools_list` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `handle_tool_call` [complexity: 12] [cognitive: 11] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `parse_tool_call_params` [complexity: 8] [cognitive: 7] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `handle_dedupe_tool` [complexity: 16] [cognitive: 15] [big-o: O(n²)] [provability: 75%] [coverage: 65%] [defect-prob: 37%]
- **Function**: `handle_search_tool` [complexity: 15] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `handle_count_tool` [complexity: 15] [cognitive: 14] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 35%]
- **Function**: `create_pattern` [complexity: 13] [cognitive: 12] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 30%]
- **Function**: `handle_outliers_tool` [complexity: 11] [cognitive: 10] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 26%]
- **Function**: `parse_size` [complexity: 10] [cognitive: 21] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]

### ./src/mcp_server/mod.rs

**File Metrics**: Complexity: 0, Functions: 0


### ./src/mcp_server/server.rs

**File Metrics**: Complexity: 45, Functions: 8

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `McpServer`
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_server_new` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_request_initialize` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_request_tools_list` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_handle_request_unknown_method` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./src/models/mcp.rs

**File Metrics**: Complexity: 0, Functions: 5

- **Use**: statement
- **Use**: statement
- **Struct**: `McpRequest`
- **Struct**: `McpResponse`
- **Struct**: `McpError`
- **Struct**: `ToolCallParams`
- **Struct**: `DedupeArgs`
- **Struct**: `SearchArgs`
- **Struct**: `CountArgs`
- **Struct**: `OutliersArgs`
- **Function**: `default_top_n` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_std_dev_threshold` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `default_true` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./src/models/mod.rs

**File Metrics**: Complexity: 0, Functions: 0


### ./src/outliers.rs

**File Metrics**: Complexity: 41, Functions: 10

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Struct**: `OutlierOptions`
- **Struct**: `LargeFileOutlier`
- **Struct**: `HiddenConsumer`
- **Struct**: `PatternGroup`
- **Struct**: `OutlierReport`
- **Struct**: `SimpleFileInfo`
- **Function**: `detect_outliers` [complexity: 10] [cognitive: 9] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 23%]
- **Function**: `detect_large_file_outliers` [complexity: 8] [cognitive: 7] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 19%]
- **Function**: `detect_hidden_consumers` [complexity: 12] [cognitive: 25] [big-o: O(n log n)] [provability: 75%] [coverage: 65%] [defect-prob: 28%]
- **Function**: `detect_pattern_groups` [complexity: 7] [cognitive: 14] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 16%]
- **Function**: `detect_numbered_pattern` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `detect_dated_pattern` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `outliers_to_dataframe` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_detect_numbered_pattern` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_dated_pattern` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/cli.rs

**File Metrics**: Complexity: 0, Functions: 2

- **Use**: statement
- **Use**: statement
- **Function**: `usage` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `search` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/execution_mode.rs

**File Metrics**: Complexity: 0, Functions: 2

- **Use**: statement
- **Function**: `test_execution_mode_cli` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execution_mode_mcp` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/integration_tests.rs

**File Metrics**: Complexity: 2, Functions: 7

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_full_deduplication_workflow` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_similarity_detection_workflow` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_types` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk_options_variations` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_csv_output` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_run_simple_api` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/main_coverage.rs

**File Metrics**: Complexity: 0, Functions: 15

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_create_pattern_literal` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_create_pattern_glob` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_create_pattern_regex` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk_options_creation` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_walk_options_max_depth` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_deduplication_with_csv` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_similarity_detection` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_advanced_options_with_ignore` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_invalid_path` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_invalid_csv_path` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dataframe_functionality` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_thread_info_display` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_simple_run_api` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_execution_mode_detection` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_pattern_edge_cases` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/mcp_handlers_coverage.rs

**File Metrics**: Complexity: 41, Functions: 23

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_initialize_handler` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tools_list_handler` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_handler` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_similarity` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_csv` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_glob_pattern` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_regex_pattern` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_hidden_files` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_with_max_depth` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_tool_handler` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_tool_with_glob` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_tool_handler` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tool_call_unknown_tool` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tool_call_invalid_params` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_invalid_path` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_dedupe_tool_invalid_pattern` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_search_tool_invalid_path` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_count_tool_invalid_path` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_tool_call_no_params` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_error_creation` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_response_success` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_response_error` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_outliers_tool_handler` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]

### ./tests/mcp_integration.rs

**File Metrics**: Complexity: 5, Functions: 4

- **Use**: statement
- **Use**: statement
- **Function**: `test_mcp_response_success` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_response_error` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_mcp_handlers` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement
- **Function**: `test_tools_list` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Use**: statement

### ./tests/outliers_tests.rs

**File Metrics**: Complexity: 10, Functions: 11

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `test_outlier_options_default` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_outlier_options_custom` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_outliers_empty_directory` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_large_file_outliers` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_hidden_consumers` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_detect_pattern_groups` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_outliers_to_dataframe` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_outliers_to_dataframe_empty` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_min_size_filter` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 75%] [coverage: 65%]
- **Function**: `test_top_n_limiting` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 75%] [coverage: 65%]
- **Function**: `test_error_handling_nonexistent_path` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 75%] [coverage: 65%]

### ./tests/property_tests.rs

**File Metrics**: Complexity: 4, Functions: 1

- **Use**: statement
- **Use**: statement
- **Use**: statement
- **Function**: `similarity_score_bounds_safe` [complexity: 5] [cognitive: 5] [big-o: O(n)] [provability: 75%] [coverage: 65%] [defect-prob: 12%]
- **Use**: statement

