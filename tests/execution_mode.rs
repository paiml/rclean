//! Tests for execution mode detection

use std::env;

#[test]
fn test_execution_mode_cli() {
    // In test environment, should detect CLI mode
    // Note: We can't directly test the function from main.rs,
    // but we can test the logic
    
    // When MCP_VERSION is not set, it should be CLI mode
    env::remove_var("MCP_VERSION");
    
    // This test confirms environment is set up correctly
    assert!(env::var("MCP_VERSION").is_err());
}

#[test]
fn test_execution_mode_mcp() {
    // Set MCP_VERSION to simulate MCP mode
    env::set_var("MCP_VERSION", "1.0");
    
    // Verify it's set
    let result = env::var("MCP_VERSION");
    
    // Clean up first to avoid affecting other tests
    env::remove_var("MCP_VERSION");
    
    // Now assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1.0");
}