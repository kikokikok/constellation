## 1. Research and Design Phase
- [ ] 1.1 Analyze current MCP crypto module capabilities vs integration requirements
- [ ] 1.2 Research proper cryptography patterns for agent communications
- [ ] 1.3 Have Architect agent review design decisions
- [ ] 1.4 Create design document with technical decisions

## 2. Fix Core MCP Models
- [ ] 2.1 Add `metadata` field with `#[serde(skip)]` to `AccessControl`, `AuditLogging`, `KeyManagement` structs
- [ ] 2.2 Implement proper methods: `is_authorized`, `add_rule`, `log_event`, `get_logs`, `register_key`, `rotate_key`
- [ ] 2.3 Update `Default` implementations to include new fields
- [ ] 2.4 Add `security_level` field to `McpSecureEnvelope` struct
- [ ] 2.5 Update `McpSecureEnvelope::new()` method signature
- [ ] 2.6 Fix thread safety: Use write locks for mutable crypto operations

## 3. Fix Crypto Module Implementation
- [ ] 3.1 Fix X25519 key extraction bug in `generate_key_exchange_key_pair()`
- [ ] 3.2 Add `perform_key_agreement()` method using `ring::agreement::agree_ephemeral()`
- [ ] 3.3 Update key storage to handle actual key material (not just IDs)
- [ ] 3.4 Add proper key serialization/deserialization methods
- [ ] 3.5 Consider adding `secrecy` crate for secret memory handling

## 4. Fix MCP Security Integration
- [ ] 4.1 Remove debug `println!` statements
- [ ] 4.2 Fix crypto method calls to use actual API (`encrypt`, `sign`, `verify`, `decrypt`)
- [ ] 4.3 Implement proper X25519 key exchange protocol for agents
- [ ] 4.4 Fix type conversions: `String` to `Vec<u8>` for key material
- [ ] 4.5 Fix API usage: `decrypt()` expects `&McpEncryptedMessage`, not `&Vec<u8>`
- [ ] 4.6 Fix access control checks using new methods
- [ ] 4.7 Fix audit logging using new methods
- [ ] 4.8 Fix key rotation using new methods
- [ ] 4.9 Add `AccessDenied` variant to `CryptoError` enum

## 4. Enable Integration Module
- [ ] 4.1 Uncomment integration module in `lib.rs`
- [ ] 4.2 Fix any remaining compilation errors
- [ ] 4.3 Run `cargo check` to verify compilation

## 5. Update Tests
- [ ] 5.1 Fix `integration_tests.rs` to use correct struct definitions
- [ ] 5.2 Update DTG metrics usage (no `execution_time_ms` or `cost` fields)
- [ ] 5.3 Update AgentSkill struct usage (no `level` or `metadata` fields)
- [ ] 5.4 Run tests to verify functionality

## 6. Documentation and Validation
- [ ] 6.1 Document cryptography patterns for agent communications
- [ ] 6.2 Add code comments explaining design decisions
- [ ] 6.3 Run `openspec validate --strict` on the change
- [ ] 6.4 Create validation test for secure agent communications