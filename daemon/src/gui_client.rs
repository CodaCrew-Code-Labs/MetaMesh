use std::process::Command;

fn main() {
    println!("Starting MetaMesh GUI Client...");

    // Start the daemon in the background
    let daemon_child = Command::new("./target/release/metamesh-daemon")
        .args(["--daemon", "--port", "50051"])
        .spawn()
        .expect("Failed to start daemon");

    println!("Daemon started with PID: {}", daemon_child.id());

    // Here you would start your GUI framework
    // For now, just keep the process alive
    println!("GUI would start here...");

    // Wait for user input to shutdown
    println!("Press Enter to shutdown...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("Shutting down...");
}
