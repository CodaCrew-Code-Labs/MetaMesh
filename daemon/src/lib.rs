pub mod config;
pub mod storage;

pub use config::Config;
pub use storage::{SecureStorage, StoredIdentity};

// Re-export gRPC types for external use
pub mod metamesh {
    tonic::include_proto!("metamesh");
}

pub use metamesh::*;
