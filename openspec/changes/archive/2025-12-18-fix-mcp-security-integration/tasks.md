## 1. Research and Design Phase
- [x] 1.1 Analyze current MCP crypto module capabilities vs integration requirements
- [x] 1.2 Research proper cryptography patterns for agent communications
- [x] 1.3 Have Architect agent review design decisions
- [x] 1.4 Create design document with technical decisions

## 2. Fix Core MCP Models
- [x] 2.1 Add `metadata` field with `#[serde(skip)]` to `AccessControl`, `AuditLogging`, `KeyManagement` structs
- [x] 2.2 Implement proper methods: `is_authorized`, `add_rule`, `log_event`, `get_logs`, `register_key`, `rotate_key`
- [x] 2.3 Update `Default` implementations to include new fields
- [x] 2.4 Add `security_level` field to `McpSecureEnvelope` struct
- [x] 2.5 Update `McpSecureEnvelope::new()` method signature
- [x] 2.6 Fix thread safety: Use write locks for mutable crypto operations

## 3. Fix Crypto Module Implementation
- [x] 3.1 Fix X25519 key extraction bug in `generate_key_exchange_key_pair()`
- [x] 3.2 Add `perform_key_agreement()` method using `ring::agreement::agree_ephemeral()` (Using dryoc instead) - **COMPLETED: Using x25519-dalek for X25519 key exchange**
- [x] 3.3 Update key storage to handle actual key material (not just IDs)
- [x] 3.4 Add proper key serialization/deserialization methods
- [x] 3.5 Consider adding `secrecy` crate for secret memory handling (Already in workspace dependencies)

## 4. Fix MCP Security Integration
- [x] 4.1 Remove debug `println!` statements
- [x] 4.2 Fix crypto method calls to use actual API (`encrypt`, `sign`, `verify`, `decrypt`)
- [x] 4.3 Implement proper X25519 key exchange protocol for agents (XOR stub still used) - **COMPLETED: Replaced XOR stubs with proper X25519 key exchange**
- [x] 4.4 Fix type conversions: `String` to `Vec<u8>` for key material
- [x] 4.5 Fix API usage: `decrypt()` expects `&McpEncryptedMessage`, not `&Vec<u8>`
- [x] 4.6 Fix access control checks using new methods
- [x] 4.7 Fix audit logging using new methods
- [x] 4.8 Fix key rotation using new methods
- [x] 4.9 Add `AccessDenied` variant to `CryptoError` enum

## 5. Enable Integration Module
- [x] 5.1 Uncomment integration module in `lib.rs`
- [x] 5.2 Fix any remaining compilation errors
- [x] 5.3 Run `cargo check` to verify compilation

## 6. Update Tests
- [x] 6.1 Fix `integration_tests.rs` to use correct struct definitions
- [x] 6.2 Update DTG metrics usage (no `execution_time_ms` or `cost` fields)
- [x] 6.3 Update AgentSkill struct usage (no `level` or `metadata` fields)
- [x] 6.4 Run tests to verify functionality

## 7. Documentation and Validation
- [x] 7.1 Document cryptography patterns for agent communications - **COMPLETED: X25519 key exchange with x25519-dalek, dryoc for encryption**
- [x] 7.2 Add code comments explaining design decisions - **COMPLETED: Added comments in crypto.rs about X25519 and dryoc usage**
- [x] 7.3 Run `openspec validate --strict` on the change
- [x] 7.4 Create validation test for secure agent communications - **COMPLETED: test_key_exchange validates X25519 key exchange**