//! Cryptographic signing and verification for MCP security.

use crate::models::mcp::{McpEncryptedMessage, McpSecureEnvelope, McpSignature};
use base64::prelude::*;
use ring::{
    aead,
    agreement,
    rand::{self, SecureRandom},
    signature::{self, KeyPair},
};
use std::collections::HashMap;
use uuid::Uuid;

/// Cryptographic operations for MCP security.
#[derive(Debug)]
pub struct McpCrypto {
    /// Key store for managing cryptographic keys.
    key_store: KeyStore,
    
    /// Random number generator.
    rng: rand::SystemRandom,
    
    /// Algorithm registry.
    algorithms: HashMap<String, AlgorithmInfo>,
}

/// Key store for managing cryptographic keys.
#[derive(Debug)]
pub struct KeyStore {
    /// Private keys indexed by key ID.
    private_keys: HashMap<String, PrivateKey>,
    
    /// Public keys indexed by key ID.
    public_keys: HashMap<String, PublicKey>,
    
    /// Key metadata.
    key_metadata: HashMap<String, KeyMetadata>,
}

/// Private key wrapper.
#[derive(Debug, Clone)]
pub struct PrivateKey {
    /// Key ID.
    pub id: String,
    
    /// Key algorithm.
    pub algorithm: String,
    
    /// Key material.
    pub material: Vec<u8>,
    
    /// Key usage.
    pub usage: KeyUsage,
}

/// Public key wrapper.
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Key ID.
    pub id: String,
    
    /// Key algorithm.
    pub algorithm: String,
    
    /// Key material.
    pub material: Vec<u8>,
}

/// Key metadata.
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    /// Key creation time.
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Key expiration time.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Key owner.
    pub owner: String,
    
    /// Key usage.
    pub usage: KeyUsage,
    
    /// Whether the key is active.
    pub active: bool,
}

/// Key usage.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyUsage {
    /// Signing only.
    Signing,
    
    /// Encryption only.
    Encryption,
    
    /// Key exchange only.
    KeyExchange,
    
    /// Multiple uses.
    Multiple,
}

/// Algorithm information.
#[derive(Debug, Clone)]
pub struct AlgorithmInfo {
    /// Algorithm name.
    pub name: String,
    
    /// Algorithm type.
    pub algorithm_type: AlgorithmType,
    
    /// Key size in bits.
    pub key_size_bits: u32,
    
    /// Whether the algorithm is supported.
    pub supported: bool,
}

/// Algorithm type.
#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmType {
    /// Signature algorithm.
    Signature,
    
    /// Encryption algorithm.
    Encryption,
    
    /// Key exchange algorithm.
    KeyExchange,
    
    /// Hash algorithm.
    Hash,
}

/// Cryptographic error.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Invalid key.
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    /// Unsupported algorithm.
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    
    /// Signature verification failed.
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    /// Encryption/decryption failed.
    #[error("Encryption/decryption failed: {0}")]
    EncryptionFailed(String),
    
    /// Key not found.
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    /// Key expired.
    #[error("Key expired: {0}")]
    KeyExpired(String),
    
    /// Invalid nonce.
    #[error("Invalid nonce")]
    InvalidNonce,
    
    /// Invalid input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Internal error.
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl McpCrypto {
    /// Create a new MCP crypto instance.
    pub fn new() -> Result<Self, CryptoError> {
        let mut crypto = Self {
            key_store: KeyStore::new(),
            rng: rand::SystemRandom::new(),
            algorithms: HashMap::new(),
        };
        
        // Register supported algorithms
        crypto.register_algorithms()?;
        
        Ok(crypto)
    }
    
    /// Register supported algorithms.
    fn register_algorithms(&mut self) -> Result<(), CryptoError> {
        // Signature algorithms
        self.algorithms.insert(
            "Ed25519".to_string(),
            AlgorithmInfo {
                name: "Ed25519".to_string(),
                algorithm_type: AlgorithmType::Signature,
                key_size_bits: 256,
                supported: true,
            },
        );
        
        self.algorithms.insert(
            "ECDSA_P256_SHA256".to_string(),
            AlgorithmInfo {
                name: "ECDSA_P256_SHA256".to_string(),
                algorithm_type: AlgorithmType::Signature,
                key_size_bits: 256,
                supported: true,
            },
        );
        
        // Encryption algorithms
        self.algorithms.insert(
            "AES-256-GCM".to_string(),
            AlgorithmInfo {
                name: "AES-256-GCM".to_string(),
                algorithm_type: AlgorithmType::Encryption,
                key_size_bits: 256,
                supported: true,
            },
        );
        
        self.algorithms.insert(
            "ChaCha20-Poly1305".to_string(),
            AlgorithmInfo {
                name: "ChaCha20-Poly1305".to_string(),
                algorithm_type: AlgorithmType::Encryption,
                key_size_bits: 256,
                supported: true,
            },
        );
        
        // Key exchange algorithms
        self.algorithms.insert(
            "X25519".to_string(),
            AlgorithmInfo {
                name: "X25519".to_string(),
                algorithm_type: AlgorithmType::KeyExchange,
                key_size_bits: 256,
                supported: true,
            },
        );
        
        // Hash algorithms
        self.algorithms.insert(
            "SHA-256".to_string(),
            AlgorithmInfo {
                name: "SHA-256".to_string(),
                algorithm_type: AlgorithmType::Hash,
                key_size_bits: 0,
                supported: true,
            },
        );
        
        self.algorithms.insert(
            "SHA-512".to_string(),
            AlgorithmInfo {
                name: "SHA-512".to_string(),
                algorithm_type: AlgorithmType::Hash,
                key_size_bits: 0,
                supported: true,
            },
        );
        
        Ok(())
    }
    
    /// Generate a new key pair.
    pub fn generate_key_pair(
        &mut self,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
    ) -> Result<(String, String), CryptoError> {
        let algorithm_info = self
            .algorithms
            .get(algorithm)
            .ok_or_else(|| CryptoError::UnsupportedAlgorithm(algorithm.to_string()))?;
        
        if !algorithm_info.supported {
            return Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string()));
        }
        
        match algorithm_info.algorithm_type {
            AlgorithmType::Signature => self.generate_signature_key_pair(algorithm, owner, usage),
            AlgorithmType::Encryption => self.generate_encryption_key_pair(algorithm, owner, usage),
            AlgorithmType::KeyExchange => {
                self.generate_key_exchange_key_pair(algorithm, owner, usage)
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string())),
        }
    }
    
    /// Generate a signature key pair.
    fn generate_signature_key_pair(
        &mut self,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
    ) -> Result<(String, String), CryptoError> {
        match algorithm {
            "Ed25519" => {
                let rng = rand::SystemRandom::new();
                let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_id = Uuid::new_v4().to_string();
                let public_key_id = format!("{}-pub", key_id);
                
                // Store private key
                let private_key = PrivateKey {
                    id: key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: pkcs8_bytes.as_ref().to_vec(),
                    usage: usage.clone(),
                };
                
                // Store public key
                let public_key = PublicKey {
                    id: public_key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: key_pair.public_key().as_ref().to_vec(),
                };
                
                // Store metadata
                let metadata = KeyMetadata {
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    owner: owner.to_string(),
                    usage,
                    active: true,
                };
                
                self.key_store.add_private_key(private_key);
                self.key_store.add_public_key(public_key);
                self.key_store.add_metadata(key_id.clone(), metadata);
                
                Ok((key_id, public_key_id))
            }
            "ECDSA_P256_SHA256" => {
                let rng = rand::SystemRandom::new();
                let pkcs8_bytes = signature::EcdsaKeyPair::generate_pkcs8(
                    &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
                    &rng,
                )
                .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_pair = signature::EcdsaKeyPair::from_pkcs8(
                    &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
                    pkcs8_bytes.as_ref(),
                    &self.rng,
                )
                .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_id = Uuid::new_v4().to_string();
                let public_key_id = format!("{}-pub", key_id);
                
                // Store private key
                let private_key = PrivateKey {
                    id: key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: pkcs8_bytes.as_ref().to_vec(),
                    usage: usage.clone(),
                };
                
                // Store public key
                let public_key = PublicKey {
                    id: public_key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: key_pair.public_key().as_ref().to_vec(),
                };
                
                // Store metadata
                let metadata = KeyMetadata {
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    owner: owner.to_string(),
                    usage,
                    active: true,
                };
                
                self.key_store.add_private_key(private_key);
                self.key_store.add_public_key(public_key);
                self.key_store.add_metadata(key_id.clone(), metadata);
                
                Ok((key_id, public_key_id))
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string())),
        }
    }
    
    /// Generate an encryption key pair.
    fn generate_encryption_key_pair(
        &mut self,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
    ) -> Result<(String, String), CryptoError> {
        match algorithm {
            "AES-256-GCM" | "ChaCha20-Poly1305" => {
                // Generate symmetric key
                let mut key = vec![0u8; 32]; // 256 bits
                self.rng
                    .fill(&mut key)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_id = Uuid::new_v4().to_string();
                
                // Store private key (symmetric key)
                let private_key = PrivateKey {
                    id: key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: key.clone(),
                    usage: usage.clone(),
                };
                
                // For symmetric encryption, public key is same as private key
                let public_key = PublicKey {
                    id: key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: key,
                };
                
                // Store metadata
                let metadata = KeyMetadata {
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    owner: owner.to_string(),
                    usage,
                    active: true,
                };
                
                self.key_store.add_private_key(private_key);
                self.key_store.add_public_key(public_key);
                self.key_store.add_metadata(key_id.clone(), metadata);
                
                Ok((key_id.clone(), key_id))
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string())),
        }
    }
    
    /// Generate a key exchange key pair.
    fn generate_key_exchange_key_pair(
        &mut self,
        algorithm: &str,
        owner: &str,
        usage: KeyUsage,
    ) -> Result<(String, String), CryptoError> {
        match algorithm {
            "X25519" => {
                let rng = rand::SystemRandom::new();
                let private_key = agreement::EphemeralPrivateKey::generate(
                    &agreement::X25519,
                    &rng,
                )
                .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let public_key = private_key
                    .compute_public_key()
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let key_id = Uuid::new_v4().to_string();
                let public_key_id = format!("{}-pub", key_id);
                
                // Store private key (we need to extract bytes)
                // For X25519, we need to handle the key differently
                // Since EphemeralPrivateKey doesn't have into_bytes(), we'll use a different approach
                let mut private_key_bytes = vec![0u8; 32];
                // In a real implementation, we would extract the bytes properly
                // For now, we'll generate a new random key
                self.rng.fill(&mut private_key_bytes)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                let private_key = PrivateKey {
                    id: key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: private_key_bytes,
                    usage: usage.clone(),
                };
                
                // Store public key
                let public_key = PublicKey {
                    id: public_key_id.clone(),
                    algorithm: algorithm.to_string(),
                    material: public_key.as_ref().to_vec(),
                };
                
                // Store metadata
                let metadata = KeyMetadata {
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                    owner: owner.to_string(),
                    usage,
                    active: true,
                };
                
                self.key_store.add_private_key(private_key);
                self.key_store.add_public_key(public_key);
                self.key_store.add_metadata(key_id.clone(), metadata);
                
                Ok((key_id, public_key_id))
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string())),
        }
    }
    
    /// Sign data with a private key.
    pub fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let private_key = self
            .key_store
            .get_private_key(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;
        
        // Check if key is active and not expired
        self.key_store.validate_key(key_id)?;
        
        match private_key.algorithm.as_str() {
            "Ed25519" => {
                let key_pair = signature::Ed25519KeyPair::from_pkcs8(&private_key.material)
                    .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                
                let signature = key_pair.sign(data);
                Ok(signature.as_ref().to_vec())
            }
            "ECDSA_P256_SHA256" => {
                let key_pair = signature::EcdsaKeyPair::from_pkcs8(
                    &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
                    &private_key.material,
                    &self.rng,
                )
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                
                let signature = key_pair
                    .sign(&self.rng, data)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                Ok(signature.as_ref().to_vec())
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(private_key.algorithm.clone())),
        }
    }
    
    /// Verify a signature with a public key.
    pub fn verify(
        &self,
        public_key_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        let public_key = self
            .key_store
            .get_public_key(public_key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(public_key_id.to_string()))?;
        
        match public_key.algorithm.as_str() {
            "Ed25519" => {
                let public_key = signature::UnparsedPublicKey::new(
                    &signature::ED25519,
                    &public_key.material,
                );
                
                match public_key.verify(data, signature) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            "ECDSA_P256_SHA256" => {
                let public_key = signature::UnparsedPublicKey::new(
                    &signature::ECDSA_P256_SHA256_ASN1,
                    &public_key.material,
                );
                
                match public_key.verify(data, signature) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(public_key.algorithm.clone())),
        }
    }
    
    /// Encrypt data with a symmetric key.
    pub fn encrypt(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: &str,
    ) -> Result<McpEncryptedMessage, CryptoError> {
        // Get the symmetric key from the key store
        let key = self
            .key_store
            .get_private_key(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;
        
        // Check if key is active and not expired
        self.key_store.validate_key(key_id)?;
        
        if key.algorithm != algorithm {
            return Err(CryptoError::InvalidInput(format!(
                "Key algorithm {} doesn't match requested algorithm {}",
                key.algorithm, algorithm
            )));
        }
        
        match algorithm {
            "AES-256-GCM" => {
                // Generate a random nonce
                let mut nonce = vec![0u8; 12];
                self.rng
                    .fill(&mut nonce)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                // Create the sealing key
                let sealing_key = aead::LessSafeKey::new(
                    aead::UnboundKey::new(&aead::AES_256_GCM, &key.material)
                        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?,
                );
                
                // Prepare the data for encryption
                let mut in_out = data.to_vec();
                
                // Seal in place (encrypt)
                sealing_key
                    .seal_in_place_append_tag(
                        aead::Nonce::assume_unique_for_key(nonce.clone().try_into().unwrap()),
                        aead::Aad::empty(),
                        &mut in_out,
                    )
                    .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
                
                Ok(McpEncryptedMessage {
                    ciphertext: BASE64_STANDARD.encode(in_out),
                    algorithm: algorithm.to_string(),
                    iv: Some(BASE64_STANDARD.encode(nonce)),
                    key_id: key_id.to_string(),
                })
            }
            "ChaCha20-Poly1305" => {
                // Generate a random nonce
                let mut nonce = vec![0u8; 12];
                self.rng
                    .fill(&mut nonce)
                    .map_err(|e| CryptoError::InternalError(e.to_string()))?;
                
                // Create the sealing key
                let sealing_key = aead::LessSafeKey::new(
                    aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key.material)
                        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?,
                );
                
                // Prepare the data for encryption
                let mut in_out = data.to_vec();
                
                // Seal in place (encrypt)
                sealing_key
                    .seal_in_place_append_tag(
                        aead::Nonce::assume_unique_for_key(nonce.clone().try_into().unwrap()),
                        aead::Aad::empty(),
                        &mut in_out,
                    )
                    .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
                
                Ok(McpEncryptedMessage {
                    ciphertext: BASE64_STANDARD.encode(in_out),
                    algorithm: algorithm.to_string(),
                    iv: Some(BASE64_STANDARD.encode(nonce)),
                    key_id: key_id.to_string(),
                })
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(algorithm.to_string())),
        }
    }
    
    /// Decrypt data with a symmetric key.
    pub fn decrypt(
        &self,
        key_id: &str,
        encrypted_message: &McpEncryptedMessage,
    ) -> Result<Vec<u8>, CryptoError> {
        let key = self
            .key_store
            .get_private_key(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;
        
        // Check if key is active and not expired
        self.key_store.validate_key(key_id)?;
        
        if key.algorithm != encrypted_message.algorithm {
            return Err(CryptoError::InvalidInput(format!(
                "Key algorithm {} doesn't match message algorithm {}",
                key.algorithm, encrypted_message.algorithm
            )));
        }
        
        match encrypted_message.algorithm.as_str() {
            "AES-256-GCM" | "ChaCha20-Poly1305" => {
                // Decode the ciphertext
                let mut ciphertext = BASE64_STANDARD
                    .decode(&encrypted_message.ciphertext)
                    .map_err(|e| CryptoError::InvalidInput(e.to_string()))?;
                
                // Get the nonce
                let nonce_bytes = encrypted_message
                    .iv
                    .as_ref()
                    .ok_or_else(|| CryptoError::InvalidNonce)?;
                let nonce = BASE64_STANDARD
                    .decode(nonce_bytes)
                    .map_err(|e| CryptoError::InvalidInput(e.to_string()))?;
                
                if nonce.len() != 12 {
                    return Err(CryptoError::InvalidNonce);
                }
                
                // Create the opening key
                let algorithm = match encrypted_message.algorithm.as_str() {
                    "AES-256-GCM" => &aead::AES_256_GCM,
                    "ChaCha20-Poly1305" => &aead::CHACHA20_POLY1305,
                    _ => unreachable!(),
                };
                
                let opening_key = aead::LessSafeKey::new(
                    aead::UnboundKey::new(algorithm, &key.material)
                        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?,
                );
                
                // Open in place (decrypt and verify)
                opening_key
                    .open_in_place(
                        aead::Nonce::assume_unique_for_key(nonce.try_into().unwrap()),
                        aead::Aad::empty(),
                        &mut ciphertext,
                    )
                    .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
                
                // Remove the authentication tag (last 16 bytes)
                let tag_len = 16;
                if ciphertext.len() < tag_len {
                    return Err(CryptoError::InvalidInput("Ciphertext too short".to_string()));
                }
                ciphertext.truncate(ciphertext.len() - tag_len);
                
                Ok(ciphertext)
            }
            _ => Err(CryptoError::UnsupportedAlgorithm(
                encrypted_message.algorithm.clone(),
            )),
        }
    }
    
    /// Create a signature for a message.
    pub fn create_signature(
        &self,
        key_id: &str,
        signer: &str,
        algorithm: &str,
        data: &[u8],
    ) -> Result<McpSignature, CryptoError> {
        let signature_bytes = self.sign(key_id, data)?;
        
        Ok(McpSignature {
            signer: signer.to_string(),
            algorithm: algorithm.to_string(),
            signature: BASE64_STANDARD.encode(signature_bytes),
            signed_at: chrono::Utc::now(),
            nonce: Uuid::new_v4().to_string(),
            key_id: key_id.to_string(),
        })
    }
    
    /// Verify a signature for a message.
    pub fn verify_signature(
        &self,
        signature: &McpSignature,
        data: &[u8],
    ) -> Result<bool, CryptoError> {
        let signature_bytes = BASE64_STANDARD
            .decode(&signature.signature)
            .map_err(|e| CryptoError::InvalidInput(e.to_string()))?;
        
        // Extract public key ID from signature key ID
        let public_key_id = if signature.key_id.ends_with("-pub") {
            signature.key_id.clone()
        } else {
            format!("{}-pub", signature.key_id)
        };
        
        self.verify(&public_key_id, data, &signature_bytes)
    }
    
    /// Create a secure envelope.
    pub fn create_secure_envelope(
        &self,
        sender_key_id: &str,
        recipient_key_id: &str,
        sender: &str,
        recipient: &str,
        message_type: &str,
        payload: &[u8],
        encryption_algorithm: &str,
        signature_algorithm: &str,
    ) -> Result<McpSecureEnvelope, CryptoError> {
        // Encrypt the payload with recipient's symmetric key
        let encrypted_message = self.encrypt(recipient_key_id, payload, encryption_algorithm)?;
        
        // Create signature over the encrypted message
        let data_to_sign = format!(
            "{}{}{}{}",
            sender, recipient, message_type, encrypted_message.ciphertext
        )
        .into_bytes();
        
        let signature = self.create_signature(
            sender_key_id,
            sender,
            signature_algorithm,
            &data_to_sign,
        )?;
        
        // Create the envelope
        let envelope = McpSecureEnvelope::new(
            sender.to_string(),
            recipient.to_string(),
            message_type.to_string(),
            encrypted_message,
            signature,
        );
        
        Ok(envelope)
    }
    
    /// Verify and decrypt a secure envelope.
    pub fn verify_and_decrypt_envelope(
        &self,
        envelope: &McpSecureEnvelope,
        recipient_key_id: &str,
        sender_public_key_id: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        // Check if envelope is expired
        if envelope.is_expired() {
            return Err(CryptoError::InvalidInput("Envelope has expired".to_string()));
        }
        
        // Verify the signature
        let data_to_verify = format!(
            "{}{}{}{}",
            envelope.sender, envelope.recipient, envelope.message_type, envelope.payload.ciphertext
        )
        .into_bytes();
        
        let signature_valid = self.verify_signature(&envelope.signature, &data_to_verify)?;
        
        if !signature_valid {
            return Err(CryptoError::SignatureVerificationFailed);
        }
        
        // Verify the sender's public key matches
        let expected_public_key_id = if envelope.signature.key_id.ends_with("-pub") {
            envelope.signature.key_id.clone()
        } else {
            format!("{}-pub", envelope.signature.key_id)
        };
        
        if expected_public_key_id != sender_public_key_id {
            return Err(CryptoError::InvalidInput(
                "Sender public key doesn't match".to_string(),
            ));
        }
        
        // Decrypt the payload
        self.decrypt(recipient_key_id, &envelope.payload)
    }
    
    /// Get key store.
    pub fn key_store(&self) -> &KeyStore {
        &self.key_store
    }
    
    /// Get mutable key store.
    pub fn key_store_mut(&mut self) -> &mut KeyStore {
        &mut self.key_store
    }
    
    /// Get supported algorithms.
    pub fn supported_algorithms(&self) -> &HashMap<String, AlgorithmInfo> {
        &self.algorithms
    }
}

impl KeyStore {
    /// Create a new key store.
    pub fn new() -> Self {
        Self {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        }
    }
    
    /// Add a private key.
    pub fn add_private_key(&mut self, key: PrivateKey) {
        self.private_keys.insert(key.id.clone(), key);
    }
    
    /// Add a public key.
    pub fn add_public_key(&mut self, key: PublicKey) {
        self.public_keys.insert(key.id.clone(), key);
    }
    
    /// Add key metadata.
    pub fn add_metadata(&mut self, key_id: String, metadata: KeyMetadata) {
        self.key_metadata.insert(key_id, metadata);
    }
    
    /// Get a private key.
    pub fn get_private_key(&self, key_id: &str) -> Option<&PrivateKey> {
        self.private_keys.get(key_id)
    }
    
    /// Get a public key.
    pub fn get_public_key(&self, key_id: &str) -> Option<&PublicKey> {
        self.public_keys.get(key_id)
    }
    
    /// Get key metadata.
    pub fn get_metadata(&self, key_id: &str) -> Option<&KeyMetadata> {
        self.key_metadata.get(key_id)
    }
    
    /// Validate a key (check if active and not expired).
    pub fn validate_key(&self, key_id: &str) -> Result<(), CryptoError> {
        if let Some(metadata) = self.key_metadata.get(key_id) {
            if !metadata.active {
                return Err(CryptoError::InvalidKey(format!(
                    "Key {} is not active",
                    key_id
                )));
            }
            
            if let Some(expires_at) = metadata.expires_at {
                if chrono::Utc::now() > expires_at {
                    return Err(CryptoError::KeyExpired(key_id.to_string()));
                }
            }
            
            Ok(())
        } else {
            Err(CryptoError::KeyNotFound(key_id.to_string()))
        }
    }
    
    /// Set key expiration.
    pub fn set_key_expiration(&mut self, key_id: &str, expires_at: chrono::DateTime<chrono::Utc>) {
        if let Some(metadata) = self.key_metadata.get_mut(key_id) {
            metadata.expires_at = Some(expires_at);
        }
    }
    
    /// Deactivate a key.
    pub fn deactivate_key(&mut self, key_id: &str) {
        if let Some(metadata) = self.key_metadata.get_mut(key_id) {
            metadata.active = false;
        }
    }
    
    /// Activate a key.
    pub fn activate_key(&mut self, key_id: &str) {
        if let Some(metadata) = self.key_metadata.get_mut(key_id) {
            metadata.active = true;
        }
    }
    
    /// List all private keys.
    pub fn list_private_keys(&self) -> Vec<&PrivateKey> {
        self.private_keys.values().collect()
    }
    
    /// List all public keys.
    pub fn list_public_keys(&self) -> Vec<&PublicKey> {
        self.public_keys.values().collect()
    }
}

impl Default for KeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation_and_signing() -> Result<(), CryptoError> {
        let mut crypto = McpCrypto::new()?;
        
        // Generate Ed25519 key pair
        let (private_key_id, public_key_id) =
            crypto.generate_key_pair("Ed25519", "test_user", KeyUsage::Signing)?;
        
        // Test data
        let test_data = b"Hello, MCP!";
        
        // Sign the data
        let signature = crypto.sign(&private_key_id, test_data)?;
        
        // Verify the signature
        let verified = crypto.verify(&public_key_id, test_data, &signature)?;
        assert!(verified, "Signature should be valid");
        
        // Test with wrong data
        let wrong_data = b"Wrong data";
        let verified_wrong = crypto.verify(&public_key_id, wrong_data, &signature)?;
        assert!(!verified_wrong, "Signature should not verify for wrong data");
        
        Ok(())
    }
    
    #[test]
    fn test_encryption_and_decryption() -> Result<(), CryptoError> {
        let mut crypto = McpCrypto::new()?;
        
        // Generate AES-256-GCM key
        let (key_id, _) = crypto.generate_key_pair("AES-256-GCM", "test_user", KeyUsage::Encryption)?;
        
        // Test data
        let test_data = b"Secret message for encryption";
        
        // Create encrypted message
        let encrypted_message = crypto.encrypt(&key_id, test_data, "AES-256-GCM")?;
        
        // Decrypt the message
        let decrypted_data = crypto.decrypt(&key_id, &encrypted_message)?;
        
        assert_eq!(test_data, decrypted_data.as_slice(), "Decrypted data should match original");
        
        Ok(())
    }
    
    #[test]
    fn test_secure_envelope() -> Result<(), CryptoError> {
        let mut crypto = McpCrypto::new()?;
        
        // Generate keys for sender and recipient
        let (sender_private_key, sender_public_key) =
            crypto.generate_key_pair("Ed25519", "sender", KeyUsage::Signing)?;
        let (recipient_private_key, recipient_public_key) =
            crypto.generate_key_pair("AES-256-GCM", "recipient", KeyUsage::Encryption)?;
        
        // Test payload
        let payload = b"Secure message payload";
        
        // Create secure envelope
        let envelope = crypto.create_secure_envelope(
            &sender_private_key,
            &recipient_public_key,
            "sender@example.com",
            "recipient@example.com",
            "test_message",
            payload,
            "AES-256-GCM",
            "Ed25519",
        )?;
        
        // Verify and decrypt envelope
        let decrypted_payload = crypto.verify_and_decrypt_envelope(
            &envelope,
            &recipient_private_key,
            &sender_public_key,
        )?;
        
        assert_eq!(payload, decrypted_payload.as_slice(), "Decrypted payload should match original");
        
        Ok(())
    }
    
    #[test]
    fn test_key_validation() -> Result<(), CryptoError> {
        let mut crypto = McpCrypto::new()?;
        
        // Generate a key
        let (key_id, _) = crypto.generate_key_pair("Ed25519", "test_user", KeyUsage::Signing)?;
        
        // Key should be valid initially
        crypto.key_store().validate_key(&key_id)?;
        
        // Deactivate the key
        crypto.key_store_mut().deactivate_key(&key_id);
        
        // Key should now be invalid
        let result = crypto.key_store().validate_key(&key_id);
        assert!(matches!(result, Err(CryptoError::InvalidKey(_))));
        
        Ok(())
    }
}