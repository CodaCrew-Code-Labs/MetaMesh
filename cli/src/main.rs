use clap::Parser;
use metamesh_identity::{generate_seed_identity, recover_from_mnemonic};

#[derive(Parser)]
#[command(name = "metamesh")]
#[command(about = "MetaMesh CLI - Post-quantum cryptographic identity management")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate a new cryptographic identity
    Generate,
    /// Recover identity from mnemonic phrase
    Recover { 
        /// Mnemonic phrase to recover from
        mnemonic: String 
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Generate => {
            let identity = generate_seed_identity();
            
            println!("🆔 New MetaMesh Identity Generated");
            println!("================================");
            println!("Seed ID: {}", identity.seed_id);
            println!("Mnemonic: {}", identity.mnemonic);
            println!("Public Key: {}...", &identity.public_key[..32]);
            println!("\n⚠️  Store the mnemonic phrase securely - it's needed for recovery!");
        }
        Commands::Recover { mnemonic } => {
            println!("🔄 Recovering identity from mnemonic...");
            
            let recovered = recover_from_mnemonic(&mnemonic);
            
            println!("✅ Identity Recovered Successfully");
            println!("=================================");
            println!("Seed ID: {}", recovered.seed_id);
            println!("Public Key: {}...", &recovered.public_key[..32]);
        }
    }
}