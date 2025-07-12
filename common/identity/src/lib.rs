use fake::{faker::address::en::*, faker::company::en::*, faker::name::en::*, Fake};
use metamesh_crypto::generate_deterministic_keypair;
use metamesh_entropy::collect_hardware_entropy;
use metamesh_utils::{derive_seed_id, encode_base64, sentence_to_entropy};
use pqcrypto_traits::sign::{PublicKey, SecretKey};
use rand::{rngs::StdRng, SeedableRng};

pub struct SeedIdentity {
    pub mnemonic: String,
    pub private_key: String,
    pub public_key: String,
    pub seed_id: String,
}

pub fn generate_seed_identity() -> SeedIdentity {
    let entropy = collect_hardware_entropy(); // 256 bits from hardware sources

    let sentence = entropy_to_sentence(&entropy);
    derive_keys_from_sentence(&sentence)
}

pub fn recover_from_mnemonic(sentence: &str) -> SeedIdentity {
    derive_keys_from_sentence(sentence)
}

fn derive_keys_from_sentence(sentence: &str) -> SeedIdentity {
    let entropy = sentence_to_entropy(sentence);
    let (pk, sk) = generate_deterministic_keypair(&entropy);
    let seed_id = derive_seed_id(pk.as_bytes());

    SeedIdentity {
        mnemonic: sentence.to_string(),
        private_key: encode_base64(sk.as_bytes()),
        public_key: encode_base64(pk.as_bytes()),
        seed_id,
    }
}

fn entropy_to_sentence(entropy: &[u8]) -> String {
    let seed = u64::from_be_bytes({
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&entropy[..8]);
        bytes
    });

    let mut sentences = Vec::new();
    let mut current_seed = seed;

    for i in 0..3 {
        let mut rng = StdRng::seed_from_u64(current_seed);

        let sentence = match i {
            0 => {
                let name: String = FirstName().fake_with_rng(&mut rng);
                let city: String = CityName().fake_with_rng(&mut rng);
                format!("The person named {} lives in {}", name, city)
            }
            1 => {
                let company: String = CompanyName().fake_with_rng(&mut rng);
                let name: String = LastName().fake_with_rng(&mut rng);
                format!("Mr {} works at {}", name, company)
            }
            _ => {
                let street: String = StreetName().fake_with_rng(&mut rng);
                let name: String = FirstName().fake_with_rng(&mut rng);
                format!("{} walks down {} street", name, street)
            }
        };

        sentences.push(sentence);
        current_seed = current_seed.wrapping_add(1);
    }

    sentences.join(", ")
}
