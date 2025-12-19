use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestMessage {
    id: u32,
    name: String,
    data: Vec<u8>,
}

fn main() {
    let original = TestMessage {
        id: 42,
        name: "Test".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    
    // Try to serialize with toon crate
    match toon::to_string(&original) {
        Ok(toon_str) => {
            println!("Serialized to TOON: {} bytes", toon_str.len());
            
            match toon::from_str::<TestMessage>(&toon_str) {
                Ok(deserialized) => {
                    if original == deserialized {
                        println!("✓ Deserialized correctly");
                    } else {
                        println!("✗ Deserialized object doesn't match");
                    }
                }
                Err(e) => println!("✗ Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("✗ Serialization failed: {}", e),
    }
}
