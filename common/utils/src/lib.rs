use base64::{engine::general_purpose, Engine as _};

pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(data)
}

pub fn derive_seed_id(public_key: &[u8]) -> String {
    let hash = blake3::hash(public_key);
    let hash_bytes = hash.as_bytes();
    
    // Take first 8 bytes and convert to u64 for numeric ID
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash_bytes[..8]);
    let numeric_id = u64::from_be_bytes(bytes);
    
    // Format to exactly 16 digits with leading zeros if needed
    format!("{:016}", numeric_id % 10000000000000000) // Ensure exactly 16 digits
}

pub fn sentence_to_entropy(sentence: &str) -> [u8; 32] {
    let hash = blake3::hash(sentence.as_bytes());
    *hash.as_bytes()
}