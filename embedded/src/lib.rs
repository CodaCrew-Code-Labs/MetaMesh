#![no_std]

use base64::{engine::general_purpose, Engine as _};
use heapless::String;

#[derive(Debug)]
pub struct EmbeddedIdentity {
    pub seed_id: String<32>,
    pub public_key: String<128>,
}

pub fn generate_embedded_identity() -> EmbeddedIdentity {
    // Simplified identity generation for embedded systems
    let dummy_key = b"embedded_public_key_placeholder";
    let hash = blake3::hash(dummy_key);
    let hash_bytes = hash.as_bytes();

    // Create truncated seed ID
    let truncated = &hash_bytes[..12];
    let seed_id_str = general_purpose::STANDARD.encode(truncated);
    let seed_id = String::try_from(&seed_id_str[..16]).unwrap_or_else(|_| String::new());

    // Create public key string
    let public_key_str = general_purpose::STANDARD.encode(dummy_key);
    let public_key = String::try_from(&public_key_str[..64]).unwrap_or_else(|_| String::new());

    EmbeddedIdentity {
        seed_id,
        public_key,
    }
}
