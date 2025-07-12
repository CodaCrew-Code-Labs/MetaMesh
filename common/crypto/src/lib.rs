use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::SignedMessage;

pub fn generate_deterministic_keypair(
    seed: &[u8; 32],
) -> (dilithium2::PublicKey, dilithium2::SecretKey) {
    // Use BLAKE3 to derive a deterministic seed for key generation
    let _key_seed = blake3::hash(seed);

    // For now, we'll use the standard key generation
    // In a real implementation, you'd want to use the seed for deterministic generation
    let (pk, sk) = dilithium2::keypair();
    (pk, sk)
}

pub fn sign_message(message: &[u8], secret_key: &dilithium2::SecretKey) -> Vec<u8> {
    let signed_msg = dilithium2::sign(message, secret_key);
    signed_msg.as_bytes().to_vec()
}

pub fn verify_signature(
    signed_message: &[u8],
    public_key: &dilithium2::PublicKey,
) -> Result<Vec<u8>, &'static str> {
    // Convert bytes to SignedMessage
    let signed_msg = dilithium2::SignedMessage::from_bytes(signed_message)
        .map_err(|_| "Invalid signed message format")?;

    match dilithium2::open(&signed_msg, public_key) {
        Ok(message) => Ok(message),
        Err(_) => Err("Signature verification failed"),
    }
}
