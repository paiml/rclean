# RClean MCP Server API

**Status**: Active  
**Type**: Reference  
**Created**: 2025-01-16  
**Updated**: 2025-01-16  
**Author**: PAIML Team  

## Overview

RClean provides a comprehensive Model Context Protocol (MCP) server implementation that enables Large Language Models to perform advanced file system analysis and cleanup operations through a secure, controlled interface.

## MCP Server Features

### 🛠️ **Tools Provided**

#### 1. **dedupe** - Duplicate File Detection
Find and analyze duplicate files using MD5 content hashing with parallel processing.

**Parameters:**
- `path` (string): Directory path to scan (default: current directory)
- `pattern` (string, optional): File pattern to match
- `pattern_type` (enum): "literal", "glob", or "regex" (default: "literal")
- `include_hidden` (boolean): Include hidden files (default: false)
- `max_depth` (number, optional): Maximum directory depth to scan
- `csv_output` (string, optional): Path to save CSV report

**Example:**
```json
{
  "name": "dedupe",
  "arguments": {
    "path": "/home/user/Documents",
    "pattern": "*.pdf",
    "pattern_type": "glob",
    "csv_output": "/tmp/duplicates.csv"
  }
}
```

#### 2. **search** - File Pattern Search
Search for files matching specific patterns across the filesystem.

**Parameters:**
- `path` (string): Directory path to search
- `pattern` (string): Pattern to match against filenames
- `pattern_type` (enum): "literal", "glob", or "regex"
- `include_hidden` (boolean): Include hidden files

**Example:**
```json
{
  "name": "search",
  "arguments": {
    "path": "/var/log",
    "pattern": "error.*\\.log$",
    "pattern_type": "regex"
  }
}
```

#### 3. **count** - File Count Analysis
Count files matching patterns for quick filesystem analysis.

**Parameters:**
- `path` (string): Directory path to analyze
- `pattern` (string, optional): Pattern to match
- `pattern_type` (enum): Pattern matching type

**Example:**
```json
{
  "name": "count",
  "arguments": {
    "path": "/home/user/photos",
    "pattern": "*.{jpg,png,gif}",
    "pattern_type": "glob"
  }
}
```

#### 4. **outliers** - Storage Outlier Detection
Detect large files, hidden space consumers, and unusual storage patterns.

**Parameters:**
- `path` (string): Directory to analyze
- `min_size_mb` (number): Minimum file size in MB (default: 10)
- `std_dev_threshold` (number): Standard deviation threshold (default: 2.0)
- `top_n` (number): Number of top outliers to return (default: 20)

**Example:**
```json
{
  "name": "outliers",
  "arguments": {
    "path": "/home/user",
    "min_size_mb": 100,
    "std_dev_threshold": 3.0,
    "top_n": 10
  }
}
```

#### 5. **analyze_clusters** - File Similarity Clustering
Perform DBSCAN clustering analysis to find groups of similar files.

**Parameters:**
- `path` (string): Directory to analyze
- `min_similarity` (number): Minimum similarity threshold (0-100, default: 70)
- `min_cluster_size` (number): Minimum files per cluster (default: 2)

**Example:**
```json
{
  "name": "analyze_clusters",
  "arguments": {
    "path": "/home/user/downloads",
    "min_similarity": 85,
    "min_cluster_size": 3
  }
}
```

## Usage with MCP Clients

### Claude Desktop Configuration

Add to your `claude_desktop_config.json`:

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

### VS Code with Continue

Configure in your Continue config:

```json
{
  "experimental": {
    "modelContextProtocol": true
  },
  "mcpServers": {
    "rclean": {
      "command": "rclean",
      "args": [],
      "env": {"MCP_VERSION": "2024-11-05"}
    }
  }
}
```

### Python MCP Client

```python
import asyncio
from mcp import ClientSession
from mcp.client.stdio import stdio_client

async def use_rclean_mcp():
    import os
    env = os.environ.copy()
    env["MCP_VERSION"] = "2024-11-05"
    async with stdio_client(["rclean"], env=env) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            
            # Find duplicates
            result = await session.call_tool("dedupe", {
                "path": "/home/user/documents",
                "pattern": "*.pdf",
                "pattern_type": "glob"
            })
            print(result.content)

asyncio.run(use_rclean_mcp())
```

## Security Features

### 🔒 **Built-in Safety**
- **Read-only operations**: No file modification capabilities
- **Path validation**: Prevents directory traversal attacks
- **Resource limits**: Configurable scanning depth and timeouts
- **Error handling**: Graceful handling of permission errors

### 🛡️ **Access Control**
- **Sandboxed execution**: MCP protocol isolation
- **Permission awareness**: Respects filesystem permissions
- **Audit logging**: All operations logged for security review

## Performance Characteristics

### ⚡ **Optimized for Scale**
- **Parallel processing**: Multi-threaded file operations using Rayon
- **Memory efficient**: Streaming operations for large datasets
- **Configurable limits**: Prevent resource exhaustion
- **Progress reporting**: Real-time feedback for long operations

### 📊 **Benchmark Results**
- **100K files**: < 30 seconds typical scan time
- **Memory usage**: < 1GB for typical workloads
- **CPU scaling**: Near-linear performance with core count

## Real-World Use Cases

### 🧹 **System Cleanup**
```json
{
  "name": "dedupe",
  "arguments": {
    "path": "/home/user",
    "csv_output": "/tmp/cleanup_report.csv"
  }
}
```

### 🔍 **Security Auditing**
```json
{
  "name": "search", 
  "arguments": {
    "path": "/var/log",
    "pattern": ".*error.*|.*fail.*|.*security.*",
    "pattern_type": "regex"
  }
}
```

### 📈 **Storage Analysis**
```json
{
  "name": "outliers",
  "arguments": {
    "path": "/data",
    "min_size_mb": 1000,
    "top_n": 25
  }
}
```

### 🎯 **Content Organization**
```json
{
  "name": "analyze_clusters",
  "arguments": {
    "path": "/media/photos",
    "min_similarity": 90
  }
}
```

## Error Handling

The MCP server provides detailed error information for common scenarios:

- **Invalid paths**: Clear error messages for non-existent directories
- **Permission errors**: Graceful handling with informative responses
- **Pattern errors**: Validation and helpful suggestions for invalid patterns
- **Resource limits**: Informative messages when limits are exceeded

## Installation for MCP Usage

### From crates.io
```bash
cargo install rclean
MCP_VERSION=2024-11-05 rclean
```

### From source
```bash
git clone https://github.com/paiml/rclean.git
cd rclean
cargo build --release
MCP_VERSION=2024-11-05 ./target/release/rclean
```

### Docker (coming soon)
```bash
docker run -v /data:/data -e MCP_VERSION=2024-11-05 paiml/rclean
```

## Protocol Compliance

- **MCP Version**: 2024-11-05 specification
- **Transport**: JSON-RPC over stdio
- **SDK**: Built with official Rust MCP SDK patterns
- **Standards**: Follows MCP best practices for tool design

## Contributing

RClean's MCP server is open source and welcomes contributions:

- **Bug reports**: [GitHub Issues](https://github.com/paiml/rclean/issues)
- **Feature requests**: [GitHub Discussions](https://github.com/paiml/rclean/discussions)
- **Pull requests**: See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## License

MIT License - see [LICENSE](../../LICENSE) for details.

---

*RClean MCP Server: Bringing advanced file system analysis to Large Language Models through the Model Context Protocol.*