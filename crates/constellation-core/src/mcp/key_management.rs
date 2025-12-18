//! Key management with rotation policies for MCP security.
//!
//! Provides key lifecycle management, rotation policies, and key versioning
//! for cryptographic operations.

use crate::mcp::crypto::{CryptoError, KeyMetadata, KeyUsage, PrivateKey};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Key rotation policy.
#[derive(Debug, Clone)]
pub struct KeyRotationPolicy {
    /// Maximum key lifetime before rotation.
    pub max_lifetime: Duration,

    /// Warning period before rotation.
    pub warning_period: Duration,

    /// Whether to rotate keys automatically.
    pub auto_rotate: bool,

    /// Whether to keep old keys for decryption.
    pub keep_old_keys: bool,

    /// Maximum number of old keys to keep.
    pub max_old_keys: usize,

    /// Rotation strategy.
    pub strategy: RotationStrategy,
}

/// Key rotation strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum RotationStrategy {
    /// Rotate at fixed intervals.
    TimeBased,

    /// Rotate after a certain number of uses.
    UsageBased,

    /// Rotate on demand.
    OnDemand,

    /// Rotate when compromised.
    CompromiseBased,
}

/// Key version information.
#[derive(Debug, Clone)]
pub struct KeyVersion {
    /// Key ID.
    pub key_id: String,

    /// Version number.
    pub version: u32,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Activation time.
    pub activated_at: DateTime<Utc>,

    /// Deactivation time.
    pub deactivated_at: Option<DateTime<Utc>>,

    /// Whether this is the current active version.
    pub is_active: bool,

    /// Reason for deactivation.
    pub deactivation_reason: Option<DeactivationReason>,
}

/// Reason for key deactivation.
#[derive(Debug, Clone, PartialEq)]
pub enum DeactivationReason {
    /// Key expired.
    Expired,

    /// Key rotated.
    Rotated,

    /// Key compromised.
    Compromised,

    /// Key no longer needed.
    NoLongerNeeded,

    /// Administrative action.
    Administrative,
}

/// Key manager with rotation policies.
#[derive(Debug)]
pub struct KeyManager {
    /// Cryptographic operations.
    crypto: crate::mcp::crypto::McpCrypto,

    /// Rotation policies by key type.
    rotation_policies: HashMap<String, KeyRotationPolicy>,

    /// Key versions.
    key_versions: HashMap<String, Vec<KeyVersion>>,

    /// Key usage counters.
    key_usage: HashMap<String, u64>,
}

impl KeyManager {
    /// Create a new key manager.
    pub fn new() -> Result<Self, CryptoError> {
        Ok(Self {
            crypto: crate::mcp::crypto::McpCrypto::new()?,
            rotation_policies: HashMap::new(),
            key_versions: HashMap::new(),
            key_usage: HashMap::new(),
        })
    }

    /// Set rotation policy for a key type.
    pub fn set_rotation_policy(&mut self, key_type: String, policy: KeyRotationPolicy) {
        self.rotation_policies.insert(key_type, policy);
    }

    /// Generate a new key with versioning.
    pub fn generate_key(
        &mut self,
        key_type: &str,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
    ) -> Result<(String, String), CryptoError> {
        // Generate the key using our crypto instance
        let (private_key_id, public_key_id) =
            self.crypto.generate_key_pair(algorithm, owner, usage)?;

        // Create version information
        let version = KeyVersion {
            key_id: private_key_id.clone(),
            version: 1,
            created_at: Utc::now(),
            activated_at: Utc::now(),
            deactivated_at: None,
            is_active: true,
            deactivation_reason: None,
        };

        // Store version
        self.key_versions
            .entry(key_type.to_string())
            .or_default()
            .push(version);

        // Initialize usage counter
        self.key_usage.insert(private_key_id.clone(), 0);

        Ok((private_key_id, public_key_id))
    }

    /// Rotate a key according to policy.
    pub fn rotate_key(
        &mut self,
        key_type: &str,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
        reason: DeactivationReason,
    ) -> Result<(String, String), CryptoError> {
        // Get current active key ID for this type
        let current_key_id = self
            .get_current_active_key(key_type)
            .map(|k| k.key_id.clone())
            .ok_or_else(|| CryptoError::KeyNotFound(key_type.to_string()))?;

        // Deactivate current key
        self.deactivate_key(&current_key_id, reason.clone())?;

        // Generate new key
        let (new_private_key_id, new_public_key_id) =
            self.generate_key(key_type, algorithm, owner, usage)?;

        // Update version for new key
        if let Some(versions) = self.key_versions.get_mut(key_type) {
            let version_count = versions.len();
            if let Some(new_version) = versions.last_mut() {
                new_version.version = version_count as u32;
            }
        }

        Ok((new_private_key_id, new_public_key_id))
    }

    /// Get current active key for a type.
    pub fn get_current_active_key(&self, key_type: &str) -> Option<&KeyVersion> {
        self.key_versions
            .get(key_type)?
            .iter()
            .find(|version| version.is_active)
    }

    /// Deactivate a key.
    pub fn deactivate_key(
        &mut self,
        key_id: &str,
        reason: DeactivationReason,
    ) -> Result<(), CryptoError> {
        // Find and update the key version
        for versions in self.key_versions.values_mut() {
            for version in versions.iter_mut() {
                if version.key_id == key_id && version.is_active {
                    version.is_active = false;
                    version.deactivated_at = Some(Utc::now());
                    version.deactivation_reason = Some(reason.clone());
                    break;
                }
            }
        }

        // Deactivate in key store
        self.crypto.key_store_mut().deactivate_key(key_id);

        Ok(())
    }

    /// Check for keys needing rotation.
    pub fn check_rotation_needed(&self) -> Vec<RotationNeeded> {
        let mut needed = Vec::new();

        for (key_type, policy) in &self.rotation_policies {
            if let Some(active_key) = self.get_current_active_key(key_type) {
                match policy.strategy {
                    RotationStrategy::TimeBased => {
                        let age = Utc::now() - active_key.activated_at;
                        if age > policy.max_lifetime {
                            needed.push(RotationNeeded {
                                key_type: key_type.clone(),
                                key_id: active_key.key_id.clone(),
                                reason: RotationReason::Expired,
                                urgency: RotationUrgency::Critical,
                            });
                        } else if age > (policy.max_lifetime - policy.warning_period) {
                            needed.push(RotationNeeded {
                                key_type: key_type.clone(),
                                key_id: active_key.key_id.clone(),
                                reason: RotationReason::ExpiringSoon,
                                urgency: RotationUrgency::Warning,
                            });
                        }
                    }
                    RotationStrategy::UsageBased => {
                        let usage = self.key_usage.get(&active_key.key_id).unwrap_or(&0);
                        if *usage > 1000000 {
                            // Example threshold
                            needed.push(RotationNeeded {
                                key_type: key_type.clone(),
                                key_id: active_key.key_id.clone(),
                                reason: RotationReason::HighUsage,
                                urgency: RotationUrgency::High,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        needed
    }

    /// Record key usage.
    pub fn record_usage(&mut self, key_id: &str) {
        *self.key_usage.entry(key_id.to_string()).or_insert(0) += 1;
    }

    /// Get key usage statistics.
    pub fn get_usage_stats(&self, key_id: &str) -> Option<KeyUsageStats> {
        let version = self
            .key_versions
            .values()
            .flatten()
            .find(|v| v.key_id == key_id)?;

        let usage_count = self.key_usage.get(key_id).copied().unwrap_or(0);
        let age = Utc::now() - version.activated_at;

        Some(KeyUsageStats {
            key_id: key_id.to_string(),
            version: version.version,
            usage_count,
            age_seconds: age.num_seconds() as u64,
            is_active: version.is_active,
            activated_at: version.activated_at,
            deactivated_at: version.deactivated_at,
        })
    }

    /// Get all key versions for a type.
    pub fn get_key_versions(&self, key_type: &str) -> Option<&[KeyVersion]> {
        self.key_versions.get(key_type).map(|v| v.as_slice())
    }

    /// Clean up old keys according to policy.
    pub fn cleanup_old_keys(&mut self) -> Vec<String> {
        let mut cleaned = Vec::new();

        for (key_type, policy) in &self.rotation_policies {
            if let Some(versions) = self.key_versions.get_mut(key_type) {
                // Sort by creation time (oldest first)
                versions.sort_by_key(|v| v.created_at);

                // Keep only the most recent keys according to policy
                if versions.len() > policy.max_old_keys {
                    let to_remove = versions.len() - policy.max_old_keys;
                    for version in versions.drain(0..to_remove) {
                        // Remove from key store
                        self.crypto.key_store_mut().deactivate_key(&version.key_id);
                        cleaned.push(version.key_id);
                    }
                }
            }
        }

        cleaned
    }

    /// Export key for backup.
    pub fn export_key(&self, key_id: &str) -> Result<KeyExport, CryptoError> {
        // Get private key
        let private_key = self
            .crypto
            .key_store()
            .get_private_key(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;

        // Get version info
        let version = self
            .key_versions
            .values()
            .flatten()
            .find(|v| v.key_id == key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;

        // Get metadata
        let metadata = self
            .crypto
            .key_store()
            .get_metadata(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;

        Ok(KeyExport {
            key_id: key_id.to_string(),
            version: version.version,
            algorithm: private_key.algorithm.clone(),
            key_material: private_key.material.clone(),
            metadata: metadata.clone(),
            created_at: version.created_at,
            exported_at: Utc::now(),
        })
    }

    /// Import key from backup.
    pub fn import_key(&mut self, key_export: KeyExport) -> Result<(), CryptoError> {
        // Create private key
        let private_key = PrivateKey {
            id: key_export.key_id.clone(),
            algorithm: key_export.algorithm.clone(),
            material: key_export.key_material.clone(),
            usage: key_export.metadata.usage.clone(),
        };

        // Create version
        let version = KeyVersion {
            key_id: key_export.key_id.clone(),
            version: key_export.version,
            created_at: key_export.created_at,
            activated_at: Utc::now(),
            deactivated_at: None,
            is_active: true,
            deactivation_reason: None,
        };

        // Add to key store
        self.crypto.key_store_mut().add_private_key(private_key);
        self.crypto
            .key_store_mut()
            .add_metadata(key_export.key_id.clone(), key_export.metadata);

        // Add version
        self.key_versions
            .entry(key_export.algorithm.clone())
            .or_default()
            .push(version);

        Ok(())
    }

    /// Get crypto instance reference.
    pub fn crypto(&self) -> &crate::mcp::crypto::McpCrypto {
        &self.crypto
    }

    /// Get mutable crypto instance reference.
    pub fn crypto_mut(&mut self) -> &mut crate::mcp::crypto::McpCrypto {
        &mut self.crypto
    }
}

/// Rotation needed information.
#[derive(Debug, Clone)]
pub struct RotationNeeded {
    /// Key type.
    pub key_type: String,

    /// Key ID.
    pub key_id: String,

    /// Reason for rotation.
    pub reason: RotationReason,

    /// Urgency level.
    pub urgency: RotationUrgency,
}

/// Reason for rotation.
#[derive(Debug, Clone, PartialEq)]
pub enum RotationReason {
    /// Key expired.
    Expired,

    /// Key expiring soon.
    ExpiringSoon,

    /// High usage.
    HighUsage,

    /// Suspected compromise.
    SuspectedCompromise,

    /// Policy requirement.
    PolicyRequirement,
}

/// Urgency level for rotation.
#[derive(Debug, Clone, PartialEq)]
pub enum RotationUrgency {
    /// Critical - rotate immediately.
    Critical,

    /// High - rotate soon.
    High,

    /// Medium - rotate when convenient.
    Medium,

    /// Warning - monitor.
    Warning,
}

/// Key usage statistics.
#[derive(Debug, Clone)]
pub struct KeyUsageStats {
    /// Key ID.
    pub key_id: String,

    /// Version number.
    pub version: u32,

    /// Usage count.
    pub usage_count: u64,

    /// Age in seconds.
    pub age_seconds: u64,

    /// Whether key is active.
    pub is_active: bool,

    /// When key was activated.
    pub activated_at: DateTime<Utc>,

    /// When key was deactivated.
    pub deactivated_at: Option<DateTime<Utc>>,
}

/// Key export for backup.
#[derive(Debug, Clone)]
pub struct KeyExport {
    /// Key ID.
    pub key_id: String,

    /// Version number.
    pub version: u32,

    /// Algorithm.
    pub algorithm: String,

    /// Key material.
    pub key_material: Vec<u8>,

    /// Key metadata.
    pub metadata: KeyMetadata,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Export time.
    pub exported_at: DateTime<Utc>,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            max_lifetime: Duration::days(90),  // 90 days
            warning_period: Duration::days(7), // 7 days warning
            auto_rotate: true,
            keep_old_keys: true,
            max_old_keys: 5,
            strategy: RotationStrategy::TimeBased,
        }
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create key manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_with_versioning() -> Result<(), CryptoError> {
        let mut manager = KeyManager::new()?;

        // Generate a key
        let (private_key_id, _public_key_id) =
            manager.generate_key("signing", "Ed25519", "test_user", KeyUsage::Signing)?;

        // Check version was created
        let versions = manager.get_key_versions("signing").unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].key_id, private_key_id);
        assert_eq!(versions[0].version, 1);
        assert!(versions[0].is_active);

        Ok(())
    }

    #[test]
    fn test_key_rotation() -> Result<(), CryptoError> {
        let mut manager = KeyManager::new()?;

        // Generate initial key
        let (_key1_private, _) = manager.generate_key(
            "encryption",
            "AES-256-GCM",
            "test_user",
            KeyUsage::Encryption,
        )?;

        // Rotate the key
        let (_key2_private, _) = manager.rotate_key(
            "encryption",
            "AES-256-GCM",
            "test_user",
            KeyUsage::Encryption,
            DeactivationReason::Rotated,
        )?;

        // Check versions
        let versions = manager.get_key_versions("encryption").unwrap();
        assert_eq!(versions.len(), 2);

        // First key should be deactivated
        assert!(!versions[0].is_active);
        assert_eq!(
            versions[0].deactivation_reason,
            Some(DeactivationReason::Rotated)
        );

        // Second key should be active
        assert!(versions[1].is_active);
        assert_eq!(versions[1].version, 2);

        Ok(())
    }

    #[test]
    fn test_rotation_policy_check() -> Result<(), CryptoError> {
        let mut manager = KeyManager::new()?;

        // Set a short rotation policy
        let policy = KeyRotationPolicy {
            max_lifetime: Duration::seconds(1), // 1 second for testing
            warning_period: Duration::seconds(0),
            auto_rotate: true,
            keep_old_keys: true,
            max_old_keys: 5,
            strategy: RotationStrategy::TimeBased,
        };

        manager.set_rotation_policy("test".to_string(), policy);

        // Generate a key
        let (key_id, _) =
            manager.generate_key("test", "Ed25519", "test_user", KeyUsage::Signing)?;

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Check rotation needed
        let needed = manager.check_rotation_needed();
        assert!(!needed.is_empty());
        assert_eq!(needed[0].key_id, key_id);
        assert_eq!(needed[0].reason, RotationReason::Expired);
        assert_eq!(needed[0].urgency, RotationUrgency::Critical);

        Ok(())
    }

    #[test]
    fn test_key_usage_tracking() -> Result<(), CryptoError> {
        let mut manager = KeyManager::new()?;

        // Generate a key
        let (key_id, _) =
            manager.generate_key("signing", "Ed25519", "test_user", KeyUsage::Signing)?;

        // Record usage
        manager.record_usage(&key_id);
        manager.record_usage(&key_id);
        manager.record_usage(&key_id);

        // Check usage stats
        let stats = manager.get_usage_stats(&key_id).unwrap();
        assert_eq!(stats.usage_count, 3);
        assert!(stats.is_active);

        Ok(())
    }

    #[test]
    fn test_key_export_import() -> Result<(), CryptoError> {
        let mut manager = KeyManager::new()?;

        // Generate a key
        let (key_id, _) =
            manager.generate_key("signing", "Ed25519", "test_user", KeyUsage::Signing)?;

        // Export the key
        let export = manager.export_key(&key_id)?;
        assert_eq!(export.key_id, key_id);
        assert_eq!(export.version, 1);

        // Create new manager and import
        let mut new_manager = KeyManager::new()?;
        new_manager.import_key(export)?;

        // Check key was imported
        let versions = new_manager.get_key_versions("Ed25519").unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].key_id, key_id);

        Ok(())
    }
}
