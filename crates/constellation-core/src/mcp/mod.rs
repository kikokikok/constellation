//! Model Context Protocol (MCP) module.
//!
//! Provides cryptographic security for agent communications with
//! signing, verification, encryption, and decryption capabilities.

pub mod crypto;
pub mod key_management;

pub use crypto::{
    AlgorithmInfo, AlgorithmType, CryptoError, KeyMetadata, KeyStore, KeyUsage, McpCrypto,
    PrivateKey, PublicKey,
};
pub use key_management::{
    DeactivationReason, KeyExport, KeyManager, KeyRotationPolicy, KeyUsageStats, KeyVersion,
    RotationNeeded, RotationReason, RotationStrategy, RotationUrgency,
};