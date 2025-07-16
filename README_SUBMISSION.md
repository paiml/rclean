# MCP Server Submission for RClean

## Repository Entry for modelcontextprotocol/servers

Add to the Community Servers section in alphabetical order:

```markdown
<img height="12" width="12" src="https://github.com/paiml/rclean/raw/main/favicon.ico" alt="RClean Logo" /> **[RClean](https://github.com/paiml/rclean)** - High-performance file system analysis with duplicate detection, storage outlier analysis, and similarity clustering
```

## Submission Details

**Repository**: https://github.com/paiml/rclean
**MCP Support**: ✅ Full MCP 2024-11-05 protocol compliance
**Tools Provided**: 5 tools (dedupe, search, count, outliers, analyze_clusters)
**Language**: Rust
**Quality**: PMAT-certified with 126+ tests

## Installation
```bash
cargo install rclean
```

## Configuration
```json
{
  "mcpServers": {
    "rclean": {
      "command": "rclean",
      "args": [],
      "env": {"MCP_VERSION": "2024-11-05"}
    }
  }
}
```

## Features
- 🔍 Advanced duplicate file detection using MD5 hashing
- 📊 Statistical outlier analysis for storage optimization
- 🎯 DBSCAN clustering for similar file grouping
- ⚡ High-performance parallel processing with Rayon
- 🛡️ Security-focused read-only operations
- 📈 Comprehensive file pattern matching (literal, glob, regex)
- 💾 CSV export capabilities for detailed reporting
- 🏗️ PMAT-certified quality standards

## Description for Submission
RClean is a high-performance Rust-based MCP server that provides advanced file system analysis capabilities. It enables AI agents to perform sophisticated disk cleanup operations, duplicate file detection, storage outlier analysis, and file similarity clustering through a secure, read-only interface. Built with PMAT quality standards and comprehensive testing (126+ tests), RClean offers powerful tools for system administration, file organization, and storage optimization tasks.