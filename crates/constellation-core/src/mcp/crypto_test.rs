#[cfg(test)]
mod tests {
    use super::*;
    use dryoc::{
        constants::CRYPTO_SECRETBOX_NONCEBYTES,
        dryocsecretbox::{self, DryocSecretBox},
        keypair::{KeyPair, PublicKey as DryocPublicKey, SecretKey},
        rng::randombytes_buf,
        sign::{Signature, SigningKeyPair, SignedMessage},
    };
    use std::collections::HashMap;

    #[test]
    fn test_key_generation_and_storage() {
        let mut key_store = KeyStore {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        };

        // Generate Ed25519 keypair
        let ed25519_keypair = SigningKeyPair::gen_with_defaults();
        
        let private_key = PrivateKey {
            id: "test_ed25519".to_string(),
            algorithm: "Ed25519".to_string(),
            material: ed25519_keypair.secret_key().as_ref().to_vec(),
            usage: KeyUsage::Signing,
        };

        let public_key = McpPublicKey {
            id: "test_ed25519-pub".to_string(),
            algorithm: "Ed25519".to_string(),
            material: ed25519_keypair.public_key().as_ref().to_vec(),
            metadata: KeyMetadata {
                created_at: chrono::Utc::now(),
                expires_at: None,
                usage: KeyUsage::Signing,
                active: true,
            },
        };

        key_store.private_keys.insert("test_ed25519".to_string(), private_key);
        key_store.public_keys.insert("test_ed25519-pub".to_string(), public_key);

        assert!(key_store.private_keys.contains_key("test_ed25519"));
        assert!(key_store.public_keys.contains_key("test_ed25519-pub"));
    }

    #[test]
    fn test_encryption_decryption() {
        let mut key_store = KeyStore {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        };

        // Generate symmetric key
        let mut key = [0u8; 32];
        randombytes_buf(&mut key);

        let private_key = PrivateKey {
            id: "test_symmetric".to_string(),
            algorithm: "XSalsa20Poly1305".to_string(),
            material: key.to_vec(),
            usage: KeyUsage::Encryption,
        };

        key_store.private_keys.insert("test_symmetric".to_string(), private_key);

        let crypto = McpCrypto::new(key_store);

        let plaintext = b"Hello, world! This is a secret message.";
        
        // Encrypt
        let encrypted = crypto.encrypt("test_symmetric", plaintext, "XSalsa20Poly1305").unwrap();
        
        // Decrypt
        let decrypted = crypto.decrypt("test_symmetric", &encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_signature_creation_and_verification() {
        let mut key_store = KeyStore {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        };

        // Generate Ed25519 keypair
        let ed25519_keypair = SigningKeyPair::gen_with_defaults();
        
        let private_key = PrivateKey {
            id: "test_signing".to_string(),
            algorithm: "Ed25519".to_string(),
            material: ed25519_keypair.secret_key().as_ref().to_vec(),
            usage: KeyUsage::Signing,
        };

        let public_key = McpPublicKey {
            id: "test_signing-pub".to_string(),
            algorithm: "Ed25519".to_string(),
            material: ed25519_keypair.public_key().as_ref().to_vec(),
            metadata: KeyMetadata {
                created_at: chrono::Utc::now(),
                expires_at: None,
                usage: KeyUsage::Signing,
                active: true,
            },
        };

        key_store.private_keys.insert("test_signing".to_string(), private_key);
        key_store.public_keys.insert("test_signing-pub".to_string(), public_key);

        let crypto = McpCrypto::new(key_store);

        let message = b"Important message that needs signing";
        
        // Create signature
        let signature = crypto.create_signature("test_signing", "test_signer", "Ed25519", message).unwrap();
        
        // Verify signature
        let is_valid = crypto.verify_signature(&signature, message).unwrap();
        
        assert!(is_valid, "Signature should be valid");
        
        // Test with tampered message
        let tampered_message = b"Tampered message that needs signing";
        let is_valid_tampered = crypto.verify_signature(&signature, tampered_message).unwrap();
        assert!(!is_valid_tampered, "Tampered message should not verify");
    }

    #[test]
    fn test_key_exchange() {
        let mut key_store = KeyStore {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        };

        // Generate two X25519 keypairs
        let alice_keypair = KeyPair::gen_with_defaults();
        let bob_keypair = KeyPair::gen_with_defaults();

        // Store Alice's keys
        let alice_private = PrivateKey {
            id: "alice_x25519".to_string(),
            algorithm: "X25519".to_string(),
            material: alice_keypair.secret_key().as_ref().to_vec(),
            usage: KeyUsage::KeyExchange,
        };

        let alice_public = McpPublicKey {
            id: "alice_x25519-pub".to_string(),
            algorithm: "X25519".to_string(),
            material: alice_keypair.public_key().as_ref().to_vec(),
            metadata: KeyMetadata {
                created_at: chrono::Utc::now(),
                expires_at: None,
                usage: KeyUsage::KeyExchange,
                active: true,
            },
        };

        // Store Bob's keys
        let bob_private = PrivateKey {
            id: "bob_x25519".to_string(),
            algorithm: "X25519".to_string(),
            material: bob_keypair.secret_key().as_ref().to_vec(),
            usage: KeyUsage::KeyExchange,
        };

        let bob_public = McpPublicKey {
            id: "bob_x25519-pub".to_string(),
            algorithm: "X25519".to_string(),
            material: bob_keypair.public_key().as_ref().to_vec(),
            metadata: KeyMetadata {
                created_at: chrono::Utc::now(),
                expires_at: None,
                usage: KeyUsage::KeyExchange,
                active: true,
            },
        };

        key_store.private_keys.insert("alice_x25519".to_string(), alice_private);
        key_store.public_keys.insert("alice_x25519-pub".to_string(), alice_public);
        key_store.private_keys.insert("bob_x25519".to_string(), bob_private);
        key_store.public_keys.insert("bob_x25519-pub".to_string(), bob_public);

        let crypto = McpCrypto::new(key_store);

        // Alice computes shared secret with Bob's public key
        let alice_shared = crypto.key_exchange("alice_x25519", "bob_x25519-pub").unwrap();
        
        // Bob computes shared secret with Alice's public key
        let bob_shared = crypto.key_exchange("bob_x25519", "alice_x25519-pub").unwrap();
        
        // Both should have the same shared secret
        assert_eq!(alice_shared, bob_shared, "Key exchange should produce same shared secret");
    }

    #[test]
    fn test_secure_envelope() {
        let mut key_store = KeyStore {
            private_keys: HashMap::new(),
            public_keys: HashMap::new(),
            key_metadata: HashMap::new(),
        };

        // Generate keys for sender and recipient
        let sender_keypair = SigningKeyPair::gen_with_defaults();
        let mut symmetric_key = [0u8; 32];
        randombytes_buf(&mut symmetric_key);

        // Sender's signing key
        let sender_private = PrivateKey {
            id: "sender_ed25519".to_string(),
            algorithm: "Ed25519".to_string(),
            material: sender_keypair.secret_key().as_ref().to_vec(),
            usage: KeyUsage::Signing,
        };

        let sender_public = McpPublicKey {
            id: "sender_ed25519-pub".to_string(),
            algorithm: "Ed25519".to_string(),
            material: sender_keypair.public_key().as_ref().to_vec(),
            metadata: KeyMetadata {
                created_at: chrono::Utc::now(),
                expires_at: None,
                usage: KeyUsage::Signing,
                active: true,
            },
        };

        // Recipient's symmetric key
        let recipient_private = PrivateKey {
            id: "recipient_symmetric".to_string(),
            algorithm: "XSalsa20Poly1305".to_string(),
            material: symmetric_key.to_vec(),
            usage: KeyUsage::Encryption,
        };

        key_store.private_keys.insert("sender_ed25519".to_string(), sender_private);
        key_store.public_keys.insert("sender_ed25519-pub".to_string(), sender_public);
        key_store.private_keys.insert("recipient_symmetric".to_string(), recipient_private);

        let crypto = McpCrypto::new(key_store);

        let payload = b"Secure message payload";
        
        // Create secure envelope
        let envelope = crypto.create_secure_envelope(
            "sender_ed25519",
            "recipient_symmetric",
            "alice",
            "bob",
            "test_message",
            payload,
            "XSalsa20Poly1305",
            "Ed25519",
        ).unwrap();

        // Verify and decrypt envelope
        let decrypted_payload = crypto.verify_and_decrypt_envelope(
            "recipient_symmetric",
            "sender_ed25519-pub",
            &envelope,
        ).unwrap();

        assert_eq!(payload, decrypted_payload.as_slice(), "Decrypted payload should match original");
    }
}