use constellation_core::mcp::crypto::{McpCrypto, KeyStore, PrivateKey, McpPublicKey, KeyMetadata, KeyUsage};
use dryoc::{
    constants::CRYPTO_SECRETBOX_NONCEBYTES,
    dryocsecretbox::{self, DryocSecretBox},
    keypair::{KeyPair, PublicKey as DryocPublicKey, SecretKey},
    rng::randombytes_buf,
    sign::{Signature, SigningKeyPair, SignedMessage},
};

fn main() {
    println!("Testing McpCrypto integration with dryoc...");
    
    // Create a key store
    let mut key_store = KeyStore {
        private_keys: std::collections::HashMap::new(),
        public_keys: std::collections::HashMap::new(),
        key_metadata: std::collections::HashMap::new(),
    };
    
    // Test 1: Generate and store Ed25519 keypair
    println!("\n1. Testing Ed25519 key generation...");
    let ed25519_keypair = SigningKeyPair::gen_with_defaults();
    
    let private_key = PrivateKey {
        id: "ed25519_key".to_string(),
        algorithm: "Ed25519".to_string(),
        material: ed25519_keypair.secret_key().as_ref().to_vec(),
        usage: KeyUsage::Signing,
    };
    
    let public_key = McpPublicKey {
        id: "ed25519_key-pub".to_string(),
        algorithm: "Ed25519".to_string(),
        material: ed25519_keypair.public_key().as_ref().to_vec(),
        metadata: KeyMetadata {
            created_at: chrono::Utc::now(),
            expires_at: None,
            usage: KeyUsage::Signing,
            active: true,
        },
    };
    
    key_store.private_keys.insert("ed25519_key".to_string(), private_key);
    key_store.public_keys.insert("ed25519_key-pub".to_string(), public_key);
    
    println!("✓ Ed25519 keypair generated and stored");
    
    // Test 2: Generate and store X25519 keypair
    println!("\n2. Testing X25519 key generation...");
    let x25519_keypair = KeyPair::gen_with_defaults();
    
    let x25519_private = PrivateKey {
        id: "x25519_key".to_string(),
        algorithm: "X25519".to_string(),
        material: x25519_keypair.secret_key().as_ref().to_vec(),
        usage: KeyUsage::KeyExchange,
    };
    
    let x25519_public = McpPublicKey {
        id: "x25519_key-pub".to_string(),
        algorithm: "X25519".to_string(),
        material: x25519_keypair.public_key().as_ref().to_vec(),
        metadata: KeyMetadata {
            created_at: chrono::Utc::now(),
            expires_at: None,
            usage: KeyUsage::KeyExchange,
            active: true,
        },
    };
    
    key_store.private_keys.insert("x25519_key".to_string(), x25519_private);
    key_store.public_keys.insert("x25519_key-pub".to_string(), x25519_public);
    
    println!("✓ X25519 keypair generated and stored");
    
    // Test 3: Generate symmetric key
    println!("\n3. Testing symmetric key generation...");
    let mut symmetric_key = [0u8; 32];
    randombytes_buf(&mut symmetric_key);
    
    let symmetric_private = PrivateKey {
        id: "symmetric_key".to_string(),
        algorithm: "XSalsa20Poly1305".to_string(),
        material: symmetric_key.to_vec(),
        usage: KeyUsage::Encryption,
    };
    
    key_store.private_keys.insert("symmetric_key".to_string(), symmetric_private);
    
    println!("✓ Symmetric key generated and stored");
    
    // Create McpCrypto instance
    let crypto = McpCrypto::new(key_store);
    
    println!("\n✅ McpCrypto integration test setup complete!");
    println!("Keys available:");
    println!("  - ed25519_key (Ed25519 signing)");
    println!("  - x25519_key (X25519 key exchange)");
    println!("  - symmetric_key (XSalsa20Poly1305 encryption)");
}