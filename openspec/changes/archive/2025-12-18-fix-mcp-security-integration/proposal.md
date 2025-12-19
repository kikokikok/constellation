# Change: Fix MCP Security Integration Implementation

## Why
The current MCP security integration implementation has fundamental design flaws and compilation errors:

1. **Broken cryptography**: The integration tries to use non-existent methods (`encrypt_message`, `sign_message`) and has incorrect assumptions about public key vs symmetric encryption
2. **Missing implementations**: Core structs (`AccessControl`, `AuditLogging`, `KeyManagement`) are data containers without actual methods
3. **Type mismatches**: `McpSecureEnvelope` was missing the `security_level` field, causing compilation errors
4. **Debug hacks**: Uses `println!` instead of proper implementations
5. **Integration disabled**: The integration module is commented out in `lib.rs` due to compilation errors

This prevents the "incorporate-edge-research" vision from being realized, as secure agent communications are a critical component.

## What Changes
- **REPLACE** buggy custom `ring` implementation with `dryoc` crate
- **SIMPLIFY** crypto API using `dryoc`'s high-level, hard-to-misuse interface
- **IMPLEMENT** proper methods for `AccessControl`, `AuditLogging`, and `KeyManagement` structs
- **ADD** missing `security_level` field to `McpSecureEnvelope`
- **REPLACE** debug `println!` statements with proper implementations
- **RE-ENABLE** integration module in `lib.rs`
- **UPDATE** test files to use correct struct definitions
- **REDUCE** codebase by ~800 lines (from 1100+ to 200-300)
- **BREAKING** Crypto API changes to use `dryoc` types and patterns

## Impact
- **Affected specs**: `mcp-security`, `system-integration`
- **Affected code**: 
  - `crates/constellation-core/src/models/mcp.rs` - Add method implementations
  - `crates/constellation-core/src/integration/mcp_security_integration.rs` - Fix crypto usage
  - `crates/constellation-core/src/lib.rs` - Re-enable integration module
  - `crates/constellation-core/tests/integration_tests.rs` - Update test structs
- **Security**: Proper encryption, signing, and access control for agent communications
- **Reliability**: Compilation fixes and proper error handling
- **Maintainability**: Clear cryptography patterns and proper API design