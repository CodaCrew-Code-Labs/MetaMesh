#![no_std]
extern crate alloc;
use alloc::{string::String, vec::Vec, format};
use blake3;
use base64::{Engine, engine::general_purpose};

pub fn derive_seed_id_embedded(public_key_bytes: &[u8]) -> String {
    let blake_hash = blake3::hash(public_key_bytes);
    let short_id = &blake_hash.as_bytes()[..6];
    format!("{}", u64::from_be_bytes({
        let mut padded = [0u8; 8];
        padded[2..].copy_from_slice(short_id);
        padded
    }))
}

pub fn encode_base64_embedded(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn sentence_to_entropy_embedded(sentence: &str) -> Vec<u8> {
    let hash = blake3::hash(sentence.as_bytes());
    hash.as_bytes()[..20].to_vec()
}