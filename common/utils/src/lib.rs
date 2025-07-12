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

    // Take first 12 bytes and encode as base64 for shorter ID
    let truncated = &hash_bytes[..12];
    encode_base64(truncated)
        .replace(['/', '+'], "")
        .chars()
        .take(16)
        .collect()
}

pub fn sentence_to_entropy(sentence: &str) -> [u8; 32] {
    let hash = blake3::hash(sentence.as_bytes());
    *hash.as_bytes()
}
