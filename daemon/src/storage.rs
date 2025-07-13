use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredIdentity {
    pub seed_id: String,
    pub private_key: String,
    pub public_key: String,
    pub mnemonic: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueuedPacket {
    pub packet_id: String,     // Unique ID (hash of packet)
    pub packet_bytes: Vec<u8>, // Serialized packet
    pub packet_type: String,   // Packet type for display
    pub created_at: u64,       // Timestamp
    pub retry_count: u32,      // Number of send attempts
    pub last_retry: u64,       // Last retry timestamp
    pub max_retries: u32,      // Based on TTL
    pub ack_received: bool,    // Acknowledgment status
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityStorage {
    identities: HashMap<String, StoredIdentity>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacketStorage {
    packets: HashMap<String, QueuedPacket>,
}

pub struct SecureStorage {
    identities_path: PathBuf,
    packet_path: PathBuf,
    identities_key: [u8; 32],
    packets_key: [u8; 32],
}

impl SecureStorage {
    pub fn new(storage_dir: PathBuf) -> Self {
        let identities_path = storage_dir.join("identities.dat");
        let packet_path = storage_dir.join("packets.dat");
        let identities_key = Self::derive_identities_key();
        let packets_key = Self::derive_packets_key();
        Self {
            identities_path,
            packet_path,
            identities_key,
            packets_key,
        }
    }

    fn derive_identities_key() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"metamesh-identities-key-v1");
        hasher.finalize().into()
    }

    fn derive_packets_key() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"metamesh-packets-key-v1");
        hasher.finalize().into()
    }

    fn load_identities(&self) -> Result<IdentityStorage, Box<dyn std::error::Error>> {
        if !self.identities_path.exists() {
            return Ok(IdentityStorage {
                identities: HashMap::new(),
            });
        }

        let encrypted_data = fs::read(&self.identities_path)?;
        if encrypted_data.is_empty() {
            return Ok(IdentityStorage {
                identities: HashMap::new(),
            });
        }

        let cipher = Aes256Gcm::new_from_slice(&self.identities_key)?;
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let decrypted = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Identity decryption failed: {e:?}"))?;
        let storage: IdentityStorage = serde_json::from_slice(&decrypted)?;
        Ok(storage)
    }

    fn save_identities(&self, storage: &IdentityStorage) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_vec_pretty(storage)?;

        let cipher = Aes256Gcm::new_from_slice(&self.identities_key)?;
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json_data.as_ref())
            .map_err(|e| format!("Identity encryption failed: {e:?}"))?;

        let mut encrypted_data = Vec::new();
        encrypted_data.extend_from_slice(&nonce_bytes);
        encrypted_data.extend_from_slice(&ciphertext);

        fs::write(&self.identities_path, encrypted_data)?;
        Ok(())
    }

    fn load_packets(&self) -> Result<PacketStorage, Box<dyn std::error::Error>> {
        if !self.packet_path.exists() {
            return Ok(PacketStorage {
                packets: HashMap::new(),
            });
        }

        let encrypted_data = fs::read(&self.packet_path)?;
        if encrypted_data.is_empty() {
            return Ok(PacketStorage {
                packets: HashMap::new(),
            });
        }

        let cipher = Aes256Gcm::new_from_slice(&self.packets_key)?;
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let decrypted = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Packet decryption failed: {e:?}"))?;
        let storage: PacketStorage = serde_json::from_slice(&decrypted)?;
        Ok(storage)
    }

    fn save_packets(&self, storage: &PacketStorage) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_vec_pretty(storage)?;

        let cipher = Aes256Gcm::new_from_slice(&self.packets_key)?;
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json_data.as_ref())
            .map_err(|e| format!("Packet encryption failed: {e:?}"))?;

        let mut encrypted_data = Vec::new();
        encrypted_data.extend_from_slice(&nonce_bytes);
        encrypted_data.extend_from_slice(&ciphertext);

        fs::write(&self.packet_path, encrypted_data)?;
        Ok(())
    }

    pub fn store_identity(
        &self,
        identity: StoredIdentity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.load_identities()?;
        storage
            .identities
            .insert(identity.seed_id.clone(), identity);
        self.save_identities(&storage)?;
        Ok(())
    }

    pub fn get_identity(
        &self,
        seed_id: &str,
    ) -> Result<Option<StoredIdentity>, Box<dyn std::error::Error>> {
        let storage = self.load_identities()?;
        Ok(storage.identities.get(seed_id).cloned())
    }

    pub fn list_identities(&self) -> Result<Vec<StoredIdentity>, Box<dyn std::error::Error>> {
        let storage = self.load_identities()?;
        Ok(storage.identities.values().cloned().collect())
    }

    pub fn find_by_mnemonic(
        &self,
        mnemonic: &str,
    ) -> Result<Option<StoredIdentity>, Box<dyn std::error::Error>> {
        let storage = self.load_identities()?;
        for identity in storage.identities.values() {
            if identity.mnemonic == mnemonic {
                return Ok(Some(identity.clone()));
            }
        }
        Ok(None)
    }

    pub fn delete_identities(
        &self,
        seed_ids: &[String],
    ) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
        let mut storage = self.load_identities()?;
        let mut deleted = Vec::new();
        let mut not_found = Vec::new();

        for seed_id in seed_ids {
            if storage.identities.remove(seed_id).is_some() {
                deleted.push(seed_id.clone());
            } else {
                not_found.push(seed_id.clone());
            }
        }

        self.save_identities(&storage)?;
        Ok((deleted, not_found))
    }

    pub fn delete_all_identities(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let storage = self.load_identities()?;
        let count = storage.identities.len();
        let empty_storage = IdentityStorage {
            identities: HashMap::new(),
        };
        self.save_identities(&empty_storage)?;
        Ok(count)
    }

    // Packet queue methods
    pub fn queue_packet(&self, packet: QueuedPacket) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.load_packets()?;
        storage.packets.insert(packet.packet_id.clone(), packet);
        self.save_packets(&storage)?;
        Ok(())
    }

    pub fn get_pending_packets(&self) -> Result<Vec<QueuedPacket>, Box<dyn std::error::Error>> {
        let storage = self.load_packets()?;
        Ok(storage
            .packets
            .values()
            .filter(|p| !p.ack_received)
            .cloned()
            .collect())
    }

    #[allow(dead_code)]
    pub fn mark_packet_acked(&self, packet_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.load_packets()?;
        if let Some(packet) = storage.packets.get_mut(packet_id) {
            packet.ack_received = true;
        }
        self.save_packets(&storage)?;
        Ok(())
    }

    pub fn increment_retry_count(&self, packet_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.load_packets()?;
        if let Some(packet) = storage.packets.get_mut(packet_id) {
            packet.retry_count += 1;
            packet.last_retry = get_current_timestamp();
        }
        self.save_packets(&storage)?;
        Ok(())
    }

    pub fn remove_expired_packets(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut storage = self.load_packets()?;
        let current_time = get_current_timestamp();
        let mut removed_count = 0;

        storage.packets.retain(|_, packet| {
            let expired = packet.ack_received
                || packet.retry_count >= packet.max_retries
                || (current_time - packet.created_at) > 3600; // 1 hour max
            if expired {
                removed_count += 1;
            }
            !expired
        });

        self.save_packets(&storage)?;
        Ok(removed_count)
    }
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
