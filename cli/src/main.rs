use metamesh_identity::{generate_seed_identity, recover_from_mnemonic};

fn main() {
    let identity = generate_seed_identity();

    println!("🌱 Seed ID: {}", identity.seed_id);
    println!("📝 Sentence: {}", identity.mnemonic);
    println!(
        "🔐 Private Key (base64): {}...",
        &identity.private_key[..30]
    );
    println!("🔓 Public Key (base64): {}...", &identity.public_key[..30]);

    println!("\n🔁 Recovering from sentence...");
    let recovered = recover_from_mnemonic(&identity.mnemonic);

    println!("✅ Recovered Seed ID: {}", recovered.seed_id);
}
