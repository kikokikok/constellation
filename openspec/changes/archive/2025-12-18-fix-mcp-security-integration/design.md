# Design: Fix MCP Security Integration

## Context
The MCP (Model Context Protocol) security integration is broken due to:
1. Mismatch between integration expectations and actual crypto API
2. Missing method implementations in core structs
3. Fundamental cryptography design issues (public key vs symmetric encryption confusion)

The integration module is currently disabled in `lib.rs` due to compilation errors, preventing secure agent communications.

## Goals
1. **Fix compilation**: Enable integration module to compile without errors
2. **Proper cryptography**: Implement correct encryption/signing patterns for agent communications
3. **Complete implementations**: Add missing methods to core structs
4. **Working tests**: Update tests to use correct struct definitions

## Non-Goals
1. **Redesign crypto module**: Work within existing `McpCrypto` API
2. **Public key infrastructure**: Use symmetric encryption with key exchange for now
3. **Production-grade security**: Provide working foundation, not NSA-level security

## Decisions

### Decision 1: Fix `ring` Crypto Implementation (Proper Solution)
**What**: Fix the existing `ring`-based crypto module to properly implement X25519 key exchange and all required cryptography, rather than using workarounds.

**Why**: 
- **`ring` already supports all requirements**: Research shows `ring` provides Ed25519, ECDSA, AES-256-GCM, ChaCha20-Poly1305, and X25519
- **Current bugs are fixable**:
  1. X25519 key generation bug: Use `private_key.private_key_bytes()` instead of random bytes
  2. Missing key agreement: Add method using `ring::agreement::agree_ephemeral()`
  3. Key storage: Store actual key material, not just IDs
- **Better than workarounds**: Proper crypto is more secure and maintainable
- **Already integrated**: `ring` is already a dependency, no new crates needed

**Implementation**:
1. Fix X25519 key extraction in `generate_key_exchange_key_pair()`
2. Add `perform_key_agreement()` method using `ring::agreement`
3. Update key storage to handle actual key material
4. Implement proper X25519 key exchange protocol for agents

**Alternatives considered**:
- Use pre-shared keys workaround (less secure, manual management)
- Switch to `libsodium-rs` (adds dependency, migration effort)
- Use RustCrypto ecosystem (multiple crates, more complex)
- Keep broken implementation (not acceptable)

### Decision 2: Add Metadata Fields with Serialization Skip
**What**: Add `metadata: HashMap<String, Value>` fields to `AccessControl`, `AuditLogging`, and `KeyManagement` structs with `#[serde(skip)]` attribute.

**Why**:
- Need somewhere to store rules, logs, and key metadata
- Structs are currently data containers without storage
- Metadata field provides flexibility without over-engineering
- `#[serde(skip)]` prevents serialization issues with in-memory data
- Clear separation between configuration and runtime state

**Alternatives considered**:
- Add specific fields for each data type (too rigid)
- Create separate storage structs (over-complex)
- Use external database (overkill for MVP)
- Serialize metadata (could cause issues with complex data)

### Decision 3: Implement In-Memory Storage for MVP
**What**: Store access rules, audit logs, and key metadata in memory via the `metadata` fields.

**Why**:
- Simplest working implementation
- No external dependencies
- Can be replaced with persistent storage later

**Alternatives considered**:
- SQLite database (adds dependency)
- File-based storage (IO complexity)
- External service (overkill)

### Decision 4: Fix McpSecureEnvelope Structure
**What**: Add missing `security_level: SecurityLevel` field to `McpSecureEnvelope`.

**Why**:
- Required by integration code
- Matches security context concept
- Already referenced in code causing compilation errors

## Risks / Trade-offs

### Risk: Thread Safety Violation
**Risk**: Integration code tries to mutate `crypto` through read lock (`RwLockReadGuard`).

**Mitigation**: 
- Use write lock when generating keys: `let mut crypto = self.crypto.write().await`
- Or modify `McpCrypto` to support immutable key generation methods
- Document thread safety requirements

### Risk: API Mismatch
**Risk**: `decrypt()` expects `&McpEncryptedMessage` but integration passes `&Vec<u8>`.

**Mitigation**:
- Use proper API: `crypto.decrypt(&encrypted_message_struct)`
- Update integration to handle full encrypted message structs
- Add type conversion helpers if needed

### Risk: Type Conversion Issues
**Risk**: `generate_key_pair()` returns `(String, String)` but `PrivateKey.material` expects `Vec<u8>`.

**Mitigation**:
- Proper string-to-bytes conversion: `key_string.into_bytes()`
- Validate key material format
- Add conversion helper methods

### Risk: In-Memory Storage Limitations
**Risk**: Data lost on restart, not scalable.

**Mitigation**:
- Clear documentation of limitations with `#[serde(skip)]` attribute
- Design for easy replacement with persistent storage
- Suitable only for development/testing
- Plan SQLite/PostgreSQL persistence for production

### Risk: `ring` API Complexity
**Risk**: `ring` API can be complex and easy to misuse.

**Mitigation**:
- Follow `ring` documentation and examples carefully
- Add comprehensive tests for crypto operations
- Use type-safe wrappers where possible
- Consider adding `secrecy` crate for secret memory handling

### Risk: Key Material Management
**Risk**: Storing and handling raw key material increases attack surface.

**Mitigation**:
- Use `ring`'s secure key handling APIs
- **Optional**: Add `secrecy` crate for `Secret` type wrappers
- **Optional**: Add `zeroize` crate for secure memory zeroization
- Implement proper key lifecycle management (generation, rotation, destruction)
- Document security considerations for production use

## Migration Plan
1. **Phase 1**: Fix compilation errors and enable integration module
2. **Phase 2**: Implement working symmetric encryption with key exchange
3. **Phase 3**: Add proper method implementations to core structs
4. **Phase 4**: Update tests and validate functionality

**Rollback**: Simply re-comment integration module in `lib.rs` if issues arise.

## Open Questions
1. Should we implement a simple key exchange protocol now or later?
2. What are the performance implications of in-memory storage at scale?
3. How should we handle key distribution in a distributed agent system?