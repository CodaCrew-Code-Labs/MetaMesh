use std::process::Command;
use std::thread;
use std::time::Duration;

pub mod metamesh {
    tonic::include_proto!("metamesh");
}

use metamesh::meta_mesh_service_client::MetaMeshServiceClient;
use metamesh::*;

pub struct HeadlessClient {
    server_url: String,
}

impl HeadlessClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
        }
    }

    pub async fn start_daemon_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if daemon is running
        if self.health_check().await.is_ok() {
            return Ok(());
        }

        // Start daemon in background
        Command::new("./metamesh-daemon")
            .args(&["--daemon", "--port", "50051"])
            .spawn()?;

        // Wait for daemon to start
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(100));
            if self.health_check().await.is_ok() {
                return Ok(());
            }
        }

        Err("Failed to start daemon".into())
    }

    pub async fn health_check(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut client = MetaMeshServiceClient::connect(self.server_url.clone()).await?;
        let response = client.health(HealthRequest {}).await?;
        Ok(response.get_ref().status.clone())
    }

    pub async fn create_address(&self) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let mut client = MetaMeshServiceClient::connect(self.server_url.clone()).await?;
        let response = client.create_address(CreateAddressRequest {}).await?;
        let addr = response.get_ref();
        Ok((addr.seed_id.clone(), addr.public_key.clone(), addr.mnemonic.clone()))
    }

    pub async fn recover_keys(&self, mnemonic: &str) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let mut client = MetaMeshServiceClient::connect(self.server_url.clone()).await?;
        let response = client.recover_keys(RecoverKeysRequest { 
            mnemonic: mnemonic.to_string() 
        }).await?;
        let keys = response.get_ref();
        Ok((keys.seed_id.clone(), keys.public_key.clone(), keys.private_key.clone()))
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MetaMeshServiceClient::connect(self.server_url.clone()).await?;
        client.shutdown(ShutdownRequest {}).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HeadlessClient::new("http://127.0.0.1:50051");
    
    // Auto-start daemon if not running
    client.start_daemon_if_needed().await?;
    
    // Example usage
    println!("Creating new address...");
    let (seed_id, _public_key, mnemonic) = client.create_address().await?;
    println!("Seed ID: {}", seed_id);
    println!("Mnemonic: {}", mnemonic);
    
    Ok(())
}