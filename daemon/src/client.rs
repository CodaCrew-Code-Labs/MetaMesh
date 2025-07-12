use clap::Parser;

pub mod metamesh {
    tonic::include_proto!("metamesh");
}

use metamesh::meta_mesh_service_client::MetaMeshServiceClient;
use metamesh::*;

#[derive(Parser)]
#[command(name = "metamesh-client")]
#[command(about = "MetaMesh gRPC client")]
struct Args {
    #[arg(short, long, default_value = "http://127.0.0.1:50051")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Health,
    CreateAddress,
    RecoverKeys { mnemonic: String },
    ListAddresses,
    DeleteAddress { seed_ids: Vec<String> },
    DeleteAllAddresses,
    PingCheck { seed_id: String },
    Deserialize { packet_hex: String },
    PendingPackets,
    ListApis,
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut client = MetaMeshServiceClient::connect(args.server).await?;

    match args.command {
        Commands::Health => {
            let response = client.health(HealthRequest {}).await?;
            println!(
                "Status: {}, Service: {}",
                response.get_ref().status,
                response.get_ref().service
            );
        }
        Commands::CreateAddress => {
            let response = client.create_address(CreateAddressRequest {}).await?;
            let addr = response.get_ref();
            println!("Seed ID: {}", addr.seed_id);
            println!("Mnemonic: {}", addr.mnemonic);
        }
        Commands::RecoverKeys { mnemonic } => {
            let response = client.recover_keys(RecoverKeysRequest { mnemonic }).await?;
            let keys = response.get_ref();
            println!("Seed ID: {}", keys.seed_id);
        }
        Commands::ListAddresses => {
            let response = client.list_addresses(ListAddressesRequest {}).await?;
            let addresses = &response.get_ref().addresses;
            if addresses.is_empty() {
                println!("No addresses found");
            } else {
                for addr in addresses {
                    println!("Seed ID: {} | Created: {}", addr.seed_id, addr.created_at);
                }
            }
        }
        Commands::DeleteAddress { seed_ids } => {
            let response = client
                .delete_address(DeleteAddressRequest { seed_ids })
                .await?;
            let result = response.get_ref();
            println!("Deleted: {} addresses", result.deleted_count);
            if !result.deleted_seed_ids.is_empty() {
                println!("Deleted IDs: {:?}", result.deleted_seed_ids);
            }
            if !result.not_found_seed_ids.is_empty() {
                println!("Not found IDs: {:?}", result.not_found_seed_ids);
            }
        }
        Commands::DeleteAllAddresses => {
            let response = client
                .delete_all_addresses(DeleteAllAddressesRequest {})
                .await?;
            let result = response.get_ref();
            println!("{}", result.message);
        }
        Commands::PingCheck { seed_id } => {
            let response = client.ping_check(PingCheckRequest { seed_id }).await?;
            println!("Message: {}", response.get_ref().message);
            println!("Packet: {}", response.get_ref().packet_hex);
        }
        Commands::Deserialize { packet_hex } => {
            let response = client
                .deserialize(DeserializeRequest { packet_hex })
                .await?;
            println!("{}", response.get_ref().analysis);
        }
        Commands::PendingPackets => {
            let response = client.pending_packets(PendingPacketsRequest {}).await?;
            let result = response.get_ref();

            if result.total_packets == 0 {
                println!("No pending packets");
            } else {
                println!("📦 {} pending packets:", result.total_packets);
                println!();
                for packet in &result.packets {
                    println!(
                        "ID: {} | Type: {} | Retries: {}/{} | Created: {}",
                        packet.packet_id,
                        packet.packet_type,
                        packet.retry_count,
                        packet.max_retries,
                        packet.created_at
                    );
                }
            }
        }
        Commands::ListApis => {
            println!("Available MetaMesh APIs:");
            println!();
            println!("📊 Status:");
            println!("  health                    - Check daemon health status");
            println!();
            println!("🔑 Address Management:");
            println!("  create-address            - Generate new identity with mnemonic");
            println!("  recover-keys <mnemonic>   - Recover identity from mnemonic phrase");
            println!("  list-addresses            - List all stored addresses");
            println!();
            println!("🗑️  Delete Operations:");
            println!("  delete-address <seed_ids> - Delete specific addresses (space-separated)");
            println!("  delete-all-addresses      - Delete all stored addresses");
            println!();
            println!("🔗 Transport:");
            println!("  ping-check <seed_id>      - Create MetaMesh ping packet");
            println!("  deserialize <packet_hex>  - Deserialize and analyze packet");
            println!("  pending-packets           - List queued packets awaiting delivery");
            println!();
            println!("⚙️  System:");
            println!("  list-apis                 - Show this API list");
            println!("  shutdown                  - Stop the daemon");
            println!();
            println!("Examples:");
            println!("  ./metamesh-client create-address");
            println!("  ./metamesh-client ping-check 123456789");
            println!("  ./metamesh-client pending-packets");
            println!("  ./metamesh-client deserialize 4d4d030105...");
            println!("  ./metamesh-client delete-address 123456789 987654321");
            println!("  ./metamesh-client delete-all-addresses");
        }
        Commands::Shutdown => {
            let response = client.shutdown(ShutdownRequest {}).await?;
            println!("{}", response.get_ref().message);
        }
    }

    Ok(())
}
