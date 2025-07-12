use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, SecretKey};
use blake3;

pub fn generate_deterministic_keypair(entropy: &[u8]) -> (dilithium2::PublicKey, dilithium2::SecretKey) {
    let seed_hash = blake3::hash(entropy);
    
    let mut attempt = 0u64;
    loop {
        let mut hasher = blake3::Hasher::new();
        hasher.update(seed_hash.as_bytes());
        hasher.update(&attempt.to_be_bytes());
        let attempt_seed = hasher.finalize();
        
        let mut key_material = Vec::new();
        let mut counter = 0u64;
        
        while key_material.len() < dilithium2::secret_key_bytes() {
            let mut material_hasher = blake3::Hasher::new();
            material_hasher.update(attempt_seed.as_bytes());
            material_hasher.update(&counter.to_be_bytes());
            key_material.extend_from_slice(material_hasher.finalize().as_bytes());
            counter += 1;
        }
        
        key_material.truncate(dilithium2::secret_key_bytes());
        
        if let Ok(sk) = dilithium2::SecretKey::from_bytes(&key_material) {
            let mut pub_material = Vec::new();
            let mut pub_counter = 1000u64;
            
            while pub_material.len() < dilithium2::public_key_bytes() {
                let mut pub_hasher = blake3::Hasher::new();
                pub_hasher.update(attempt_seed.as_bytes());
                pub_hasher.update(&pub_counter.to_be_bytes());
                pub_material.extend_from_slice(pub_hasher.finalize().as_bytes());
                pub_counter += 1;
            }
            
            pub_material.truncate(dilithium2::public_key_bytes());
            
            if let Ok(pk) = dilithium2::PublicKey::from_bytes(&pub_material) {
                return (pk, sk);
            }
        }
        
        attempt += 1;
        if attempt > 1000 {
            return dilithium2::keypair();
        }
    }
}