use clap::Parser;
use metamesh_identity::{generate_seed_identity, recover_from_mnemonic};
use metamesh_transport::{
    MetaMeshPacket, PacketFlags, PacketHeader, PacketType, TransportMonitor, MAGIC_HEADER,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tonic::{transport::Server, Request, Response, Status};

mod config;
mod storage;
use config::Config;
use storage::{get_current_timestamp, QueuedPacket, SecureStorage, StoredIdentity};

pub mod metamesh {
    tonic::include_proto!("metamesh");
}

use metamesh::meta_mesh_service_server::{MetaMeshService, MetaMeshServiceServer};
use metamesh::*;

struct AppState {
    _config: Config,
    storage: SecureStorage,
    transport_monitor: Arc<Mutex<TransportMonitor>>,
}

impl AppState {
    fn new(transport_monitor: Arc<Mutex<TransportMonitor>>) -> Self {
        let config = Config::load();
        config.ensure_storage_dir();
        let storage = SecureStorage::new(config.storage_path.clone());
        Self {
            _config: config,
            storage,
            transport_monitor,
        }
    }
}

#[derive(Parser)]
#[command(name = "metamesh-daemon")]
#[command(about = "MetaMesh gRPC daemon service")]
struct Args {
    #[arg(short, long, default_value = "50051")]
    port: u16,
}

pub struct MetaMeshServiceImpl {
    state: AppState,
}

impl MetaMeshServiceImpl {
    fn new(transport_monitor: Arc<Mutex<TransportMonitor>>) -> Self {
        Self {
            state: AppState::new(transport_monitor),
        }
    }
}

async fn retry_pending_packets(
    storage: &SecureStorage,
    transport_monitor: &Arc<Mutex<TransportMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending_packets = storage.get_pending_packets()?;
    let current_time = get_current_timestamp();

    for packet in pending_packets {
        // Check if it's time to retry (5 minutes = 300 seconds)
        if current_time - packet.last_retry >= 300 {
            println!(
                "🔄 Retrying packet {} (attempt {})",
                packet.packet_id,
                packet.retry_count + 1
            );

            let mut guard = transport_monitor.lock().await;
            let results = guard.send_to_all_transports(&packet.packet_bytes).await;
            drop(guard);

            storage.increment_retry_count(&packet.packet_id)?;

            for result in results {
                println!("  {result}");
            }
        }
    }

    // Clean up expired packets
    let removed = storage.remove_expired_packets()?;
    if removed > 0 {
        println!("🗑️  Removed {removed} expired packets");
    }

    Ok(())
}

#[tonic::async_trait]
impl MetaMeshService for MetaMeshServiceImpl {
    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "metamesh-daemon".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn create_address(
        &self,
        _request: Request<CreateAddressRequest>,
    ) -> Result<Response<CreateAddressResponse>, Status> {
        let identity = generate_seed_identity();
        let stored_identity = StoredIdentity {
            seed_id: identity.seed_id.clone(),
            private_key: identity.private_key,
            public_key: identity.public_key.clone(),
            mnemonic: identity.mnemonic.clone(),
            created_at: get_current_timestamp(),
        };

        self.state
            .storage
            .store_identity(stored_identity)
            .map_err(|e| Status::internal(format!("Storage error: {e}")))?;

        let response = CreateAddressResponse {
            seed_id: identity.seed_id,
            public_key: identity.public_key,
            mnemonic: identity.mnemonic,
        };
        Ok(Response::new(response))
    }

    async fn recover_keys(
        &self,
        request: Request<RecoverKeysRequest>,
    ) -> Result<Response<RecoverKeysResponse>, Status> {
        let req = request.into_inner();

        // Check if identity already exists
        if let Ok(Some(existing)) = self.state.storage.find_by_mnemonic(&req.mnemonic) {
            let response = RecoverKeysResponse {
                seed_id: existing.seed_id,
                public_key: existing.public_key,
                private_key: existing.private_key,
            };
            return Ok(Response::new(response));
        }

        // Create new identity if not found
        let identity = recover_from_mnemonic(&req.mnemonic);
        let stored_identity = StoredIdentity {
            seed_id: identity.seed_id.clone(),
            private_key: identity.private_key.clone(),
            public_key: identity.public_key.clone(),
            mnemonic: identity.mnemonic,
            created_at: get_current_timestamp(),
        };

        self.state
            .storage
            .store_identity(stored_identity)
            .map_err(|e| Status::internal(format!("Storage error: {e}")))?;

        let response = RecoverKeysResponse {
            seed_id: identity.seed_id,
            public_key: identity.public_key,
            private_key: identity.private_key,
        };
        Ok(Response::new(response))
    }

    async fn list_addresses(
        &self,
        _request: Request<ListAddressesRequest>,
    ) -> Result<Response<ListAddressesResponse>, Status> {
        match self.state.storage.list_identities() {
            Ok(identities) => {
                let addresses: Vec<AddressInfo> = identities
                    .into_iter()
                    .map(|id| AddressInfo {
                        seed_id: id.seed_id,
                        created_at: id.created_at as i64,
                    })
                    .collect();

                let response = ListAddressesResponse { addresses };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Storage error: {e}"))),
        }
    }

    async fn delete_address(
        &self,
        request: Request<DeleteAddressRequest>,
    ) -> Result<Response<DeleteAddressResponse>, Status> {
        let req = request.into_inner();

        if req.seed_ids.is_empty() {
            return Err(Status::invalid_argument("At least one seed_id is required"));
        }

        match self.state.storage.delete_identities(&req.seed_ids) {
            Ok((deleted, not_found)) => {
                let response = DeleteAddressResponse {
                    deleted_count: deleted.len() as i32,
                    deleted_seed_ids: deleted,
                    not_found_seed_ids: not_found,
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Storage error: {e}"))),
        }
    }

    async fn delete_all_addresses(
        &self,
        _request: Request<DeleteAllAddressesRequest>,
    ) -> Result<Response<DeleteAllAddressesResponse>, Status> {
        match self.state.storage.delete_all_identities() {
            Ok(count) => {
                let response = DeleteAllAddressesResponse {
                    deleted_count: count as i32,
                    message: format!("Deleted {count} addresses"),
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Storage error: {e}"))),
        }
    }

    async fn ping_check(
        &self,
        request: Request<PingCheckRequest>,
    ) -> Result<Response<PingCheckResponse>, Status> {
        let req = request.into_inner();

        if req.seed_id.is_empty() {
            return Err(Status::invalid_argument("seed_id is required"));
        }

        // Get identity to use as from_seed
        let identity = match self.state.storage.get_identity(&req.seed_id) {
            Ok(Some(identity)) => identity,
            Ok(None) => return Err(Status::not_found("Identity not found")),
            Err(e) => return Err(Status::internal(format!("Storage error: {e}"))),
        };

        // Convert seed_id to 16-byte array
        let mut from_seed = [0u8; 16];
        let seed_bytes = identity.seed_id.as_bytes();
        let copy_len = std::cmp::min(seed_bytes.len(), 16);
        from_seed[..copy_len].copy_from_slice(&seed_bytes[..copy_len]);

        // Generate random nonce
        let mut nonce = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut nonce);

        // Create MetaMesh ping packet
        let packet = MetaMeshPacket {
            magic: MAGIC_HEADER,
            header: PacketHeader {
                packet_type: PacketType::Ping,
                version: 1,
                ttl: 5,
                flags: PacketFlags::BROADCAST,
                from_seed,
                to_seed: [0u8; 16], // Broadcast
                nonce,
                payload_len: 0,
            },
            payload: vec![],
        };

        // Serialize packet using postcard
        let packet_bytes = postcard::to_allocvec(&packet)
            .map_err(|e| Status::internal(format!("Serialization error: {e}")))?;
        let packet_hex = hex::encode(&packet_bytes);

        // Generate packet ID (hash of packet bytes)
        let mut hasher = Sha256::new();
        hasher.update(&packet_bytes);
        let packet_id = hex::encode(hasher.finalize())[..16].to_string();

        // Queue packet for store-and-forward
        let queued_packet = QueuedPacket {
            packet_id: packet_id.clone(),
            packet_bytes: packet_bytes.clone(),
            packet_type: "Ping".to_string(),
            created_at: get_current_timestamp(),
            retry_count: 0,
            last_retry: get_current_timestamp(),
            max_retries: packet.header.ttl as u32,
            ack_received: false,
        };

        self.state
            .storage
            .queue_packet(queued_packet)
            .map_err(|e| Status::internal(format!("Queue error: {e}")))?;

        // Send to all available transports using the shared transport monitor
        let mut transport_monitor = self.state.transport_monitor.lock().await;
        let send_results = transport_monitor
            .send_to_all_transports(&packet_bytes)
            .await;

        let mut message = format!(
            "Ping packet created for seed_id: {} ({} bytes)\n",
            req.seed_id,
            packet_bytes.len()
        );
        message.push_str(&format!("Packet ID: {packet_id} (queued for retry)\n"));
        message.push_str("Transport results:\n");
        for result in send_results {
            message.push_str(&format!("  {result}\n"));
        }

        let response = PingCheckResponse {
            message,
            packet_hex,
        };

        Ok(Response::new(response))
    }

    async fn deserialize(
        &self,
        request: Request<DeserializeRequest>,
    ) -> Result<Response<DeserializeResponse>, Status> {
        let req = request.into_inner();

        if req.packet_hex.is_empty() {
            return Err(Status::invalid_argument("packet_hex is required"));
        }

        // Decode hex to bytes
        let packet_bytes = hex::decode(&req.packet_hex)
            .map_err(|e| Status::invalid_argument(format!("Invalid hex: {e}")))?;

        // Deserialize using postcard
        let packet: MetaMeshPacket = postcard::from_bytes(&packet_bytes)
            .map_err(|e| Status::internal(format!("Deserialization error: {e}")))?;

        // Format analysis
        let magic_str = String::from_utf8_lossy(&packet.magic);
        let from_seed_str = String::from_utf8_lossy(&packet.header.from_seed)
            .trim_end_matches('\0')
            .to_string();
        let to_seed_str = String::from_utf8_lossy(&packet.header.to_seed)
            .trim_end_matches('\0')
            .to_string();
        let to_seed_display = if to_seed_str.is_empty() {
            "BROADCAST".to_string()
        } else {
            to_seed_str
        };
        let nonce_hex = hex::encode(packet.header.nonce);

        let analysis = format!(
            "🔍 MetaMesh Packet Analysis:\n\
            ========================================\n\
            Magic Header: {}\n\
            Packet Type: {:?}\n\
            Version: {}\n\
            TTL: {}\n\
            Flags: {:?}\n\
            From Seed: {}\n\
            To Seed: {}\n\
            Nonce: {}\n\
            Payload Length: {}\n\
            Payload TLVs: {}\n\
            \n\
            Total packet size: {} bytes",
            magic_str,
            packet.header.packet_type,
            packet.header.version,
            packet.header.ttl,
            packet.header.flags,
            from_seed_str,
            to_seed_display,
            nonce_hex,
            packet.header.payload_len,
            packet.payload.len(),
            packet_bytes.len()
        );

        let response = DeserializeResponse { analysis };
        Ok(Response::new(response))
    }

    async fn pending_packets(
        &self,
        _request: Request<PendingPacketsRequest>,
    ) -> Result<Response<PendingPacketsResponse>, Status> {
        match self.state.storage.get_pending_packets() {
            Ok(packets) => {
                let packet_infos: Vec<PacketInfo> = packets
                    .into_iter()
                    .map(|p| PacketInfo {
                        packet_id: p.packet_id,
                        packet_type: p.packet_type,
                        created_at: p.created_at as i64,
                        retry_count: p.retry_count as i32,
                        max_retries: p.max_retries as i32,
                    })
                    .collect();

                let response = PendingPacketsResponse {
                    total_packets: packet_infos.len() as i32,
                    packets: packet_infos,
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!("Storage error: {e}"))),
        }
    }

    async fn shutdown(
        &self,
        _request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            process::exit(0);
        });

        let response = ShutdownResponse {
            message: "Daemon shutting down".to_string(),
        };
        Ok(Response::new(response))
    }
}

async fn start_transport_monitor(storage: SecureStorage) -> Arc<Mutex<TransportMonitor>> {
    let monitor = Arc::new(Mutex::new(TransportMonitor::new()));
    let monitor_clone = monitor.clone();

    // Start transport monitoring
    tokio::spawn(async move {
        let mut guard = monitor_clone.lock().await;
        if let Err(e) = guard.start().await {
            println!("Transport monitor failed: {e}");
        }
    });

    // Start retry loop for pending packets
    let monitor_retry = monitor.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(300)).await; // 5 minutes
            if let Err(e) = retry_pending_packets(&storage, &monitor_retry).await {
                println!("Retry failed: {e}");
            }
        }
    });

    monitor
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 MetaMesh Daemon Starting...");

    // Create storage for transport monitor
    let config = Config::load();
    config.ensure_storage_dir();
    let storage = SecureStorage::new(config.storage_path.clone());

    // Start transport monitor with retry loop
    let transport_monitor = start_transport_monitor(storage).await;

    // Small delay to ensure transport messages are printed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("0.0.0.0:{}", args.port).parse()?;
    let service = MetaMeshServiceImpl::new(transport_monitor);

    println!("🌐 gRPC daemon listening on {addr}");
    println!("✨ MetaMesh daemon ready!");

    Server::builder()
        .add_service(MetaMeshServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
