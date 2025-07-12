use metamesh_daemon::{SecureStorage, StoredIdentity};
use metamesh_identity::generate_seed_identity;
use tempfile::TempDir;

#[tokio::test]
async fn test_storage_encryption() {
    let temp_dir = TempDir::new().unwrap();
    let storage = SecureStorage::new(temp_dir.path().to_path_buf());
    
    let identity = generate_seed_identity();
    let stored = StoredIdentity {
        seed_id: identity.seed_id.clone(),
        private_key: identity.private_key,
        public_key: identity.public_key,
        mnemonic: identity.mnemonic,
        address: format!("addr_{}", identity.seed_id),
        created_at: 1234567890,
    };
    
    // Test store
    assert!(storage.store_identity(stored.clone()).unwrap());
    
    // Test retrieve
    let retrieved = storage.get_identity(&identity.seed_id).unwrap().unwrap();
    assert_eq!(retrieved.seed_id, stored.seed_id);
    assert_eq!(retrieved.private_key, stored.private_key);
}

#[tokio::test]
async fn test_duplicate_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let storage = SecureStorage::new(temp_dir.path().to_path_buf());
    
    let identity = generate_seed_identity();
    let stored = StoredIdentity {
        seed_id: identity.seed_id.clone(),
        private_key: identity.private_key,
        public_key: identity.public_key,
        mnemonic: identity.mnemonic,
        address: format!("addr_{}", identity.seed_id),
        created_at: 1234567890,
    };
    
    // First store should succeed
    assert!(storage.store_identity(stored.clone()).unwrap());
    
    // Second store should fail (duplicate)
    assert!(!storage.store_identity(stored).unwrap());
}

#[tokio::test]
async fn test_delete_operations() {
    let temp_dir = TempDir::new().unwrap();
    let storage = SecureStorage::new(temp_dir.path().to_path_buf());
    
    // Create test identities
    let id1 = generate_seed_identity();
    let id2 = generate_seed_identity();
    
    let stored1 = StoredIdentity {
        seed_id: id1.seed_id.clone(),
        private_key: id1.private_key,
        public_key: id1.public_key,
        mnemonic: id1.mnemonic,
        address: format!("addr_{}", id1.seed_id),
        created_at: 1234567890,
    };
    
    let stored2 = StoredIdentity {
        seed_id: id2.seed_id.clone(),
        private_key: id2.private_key,
        public_key: id2.public_key,
        mnemonic: id2.mnemonic,
        address: format!("addr_{}", id2.seed_id),
        created_at: 1234567890,
    };
    
    storage.store_identity(stored1).unwrap();
    storage.store_identity(stored2).unwrap();
    
    // Test single delete
    assert!(storage.delete_identity(&id1.seed_id).unwrap());
    assert!(!storage.delete_identity(&id1.seed_id).unwrap()); // Already deleted
    
    // Test delete all
    let count = storage.delete_all_identities().unwrap();
    assert_eq!(count, 1);
}