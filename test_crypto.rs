use dryoc::{
    constants::CRYPTO_SECRETBOX_NONCEBYTES,
    dryocsecretbox::{self, DryocSecretBox},
    keypair::{KeyPair, PublicKey as DryocPublicKey, SecretKey},
    rng::randombytes_buf,
    sign::{Signature, SigningKeyPair, SignedMessage},
};

fn main() {
    println!("Testing dryoc crypto functionality...");
    
    // Test 1: Ed25519 signing
    println!("\n1. Testing Ed25519 signing...");
    let keypair = SigningKeyPair::gen_with_defaults();
    let message = b"Hello, world!";
    
    let signed_message = keypair.sign_with_defaults(message).expect("signing failed");
    signed_message.verify(&keypair.public_key).expect("verification failed");
    println!("✓ Ed25519 signing and verification works");
    
    // Test 2: X25519 key exchange
    println!("\n2. Testing X25519 key exchange...");
    let alice_keypair = KeyPair::gen_with_defaults();
    let bob_keypair = KeyPair::gen_with_defaults();
    
    let alice_shared = dryocsecretbox::DryocSecretBox::precompute(&bob_keypair.public_key, &alice_keypair.secret_key)
        .expect("Alice key exchange failed");
    let bob_shared = dryocsecretbox::DryocSecretBox::precompute(&alice_keypair.public_key, &bob_keypair.secret_key)
        .expect("Bob key exchange failed");
    
    assert_eq!(alice_shared.to_bytes(), bob_shared.to_bytes());
    println!("✓ X25519 key exchange works");
    
    // Test 3: Symmetric encryption
    println!("\n3. Testing symmetric encryption...");
    let mut key = [0u8; 32];
    randombytes_buf(&mut key);
    
    let mut nonce = [0u8; CRYPTO_SECRETBOX_NONCEBYTES];
    randombytes_buf(&mut nonce);
    
    let plaintext = b"Secret message";
    let ciphertext = dryocsecretbox::DryocSecretBox::encrypt_to_vecbox(plaintext, &nonce, &key)
        .expect("encryption failed");
    
    let decrypted = dryocsecretbox::DryocSecretBox::decrypt_to_vec(&ciphertext, &nonce, &key)
        .expect("decryption failed");
    
    assert_eq!(plaintext, decrypted.as_slice());
    println!("✓ Symmetric encryption works");
    
    println!("\n✅ All dryoc crypto tests passed!");
}