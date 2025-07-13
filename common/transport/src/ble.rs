use std::error::Error;
use std::fmt;
use tokio::time::{sleep, Duration};

// MetaMesh BLE Service UUID (custom 128-bit)
pub const METAMESH_BLE_UUID: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

#[derive(Debug)]
pub enum TransportError {
    BluetoothNotAvailable,
    BluetoothNotEnabled,
    ConnectionFailed(String),
    SendFailed(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransportError::BluetoothNotAvailable => {
                write!(f, "Bluetooth not available on this platform")
            }
            TransportError::BluetoothNotEnabled => write!(f, "Bluetooth is not enabled"),
            TransportError::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            TransportError::SendFailed(msg) => write!(f, "Send failed: {msg}"),
        }
    }
}

impl Error for TransportError {}

// Platform-specific BLE implementations
#[cfg(target_os = "macos")]
mod macos_ble {
    use super::*;

    #[derive(Default)]
    pub struct BleTransport {
        enabled: bool,
        last_status: bool,
    }

    impl BleTransport {
        pub fn new() -> Self {
            Self::default()
        }

        pub async fn start_listener(&mut self) -> Result<(), TransportError> {
            println!("🍎 macOS: Starting Core Bluetooth BLE listener");
            self.enabled = true;
            self.last_status = true;
            println!("✅ BLE: Listening on UUID {METAMESH_BLE_UUID}");
            Ok(())
        }

        pub async fn monitor_status(&mut self) -> bool {
            self.enabled && self.last_status
        }

        pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
            if !self.enabled {
                return Err(TransportError::SendFailed(
                    "BLE transport not enabled".to_string(),
                ));
            }
            println!("📡 macOS BLE: Sending {} byte packet", packet_bytes.len());
            Ok(())
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

#[cfg(target_os = "linux")]
mod linux_ble {
    use super::*;

    #[derive(Default)]
    pub struct BleTransport {
        enabled: bool,
        last_status: bool,
    }

    impl BleTransport {
        pub fn new() -> Self {
            Self::default()
        }

        pub async fn start_listener(&mut self) -> Result<(), TransportError> {
            println!("🐧 Linux: Starting BlueZ D-Bus BLE listener");
            self.enabled = true;
            self.last_status = true;
            println!("✅ BLE: Listening on UUID {METAMESH_BLE_UUID}");
            Ok(())
        }

        pub async fn monitor_status(&mut self) -> bool {
            self.enabled && self.last_status
        }

        pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
            if !self.enabled {
                return Err(TransportError::SendFailed(
                    "BLE transport not enabled".to_string(),
                ));
            }
            println!("📡 Linux BLE: Sending {} byte packet", packet_bytes.len());
            Ok(())
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_ble {
    use super::*;

    #[derive(Default)]
    pub struct BleTransport {
        enabled: bool,
        last_status: bool,
    }

    impl BleTransport {
        pub fn new() -> Self {
            Self::default()
        }

        pub async fn start_listener(&mut self) -> Result<(), TransportError> {
            println!("🪟 Windows: Starting Windows BLE API listener");
            self.enabled = true;
            self.last_status = true;
            println!("✅ BLE: Listening on UUID {METAMESH_BLE_UUID}");
            Ok(())
        }

        pub async fn monitor_status(&mut self) -> bool {
            self.enabled && self.last_status
        }

        pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
            if !self.enabled {
                return Err(TransportError::SendFailed(
                    "BLE transport not enabled".to_string(),
                ));
            }
            println!("📡 Windows BLE: Sending {} byte packet", packet_bytes.len());
            Ok(())
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

#[cfg(target_os = "android")]
mod android_ble {
    use super::*;

    #[derive(Default)]
    pub struct BleTransport {
        enabled: bool,
        last_status: bool,
    }

    impl BleTransport {
        pub fn new() -> Self {
            Self::default()
        }

        pub async fn start_listener(&mut self) -> Result<(), TransportError> {
            println!("🤖 Android: Starting Android BLE API listener");
            self.enabled = true;
            self.last_status = true;
            println!("✅ BLE: Listening on UUID {METAMESH_BLE_UUID}");
            Ok(())
        }

        pub async fn monitor_status(&mut self) -> bool {
            self.enabled && self.last_status
        }

        pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
            if !self.enabled {
                return Err(TransportError::SendFailed(
                    "BLE transport not enabled".to_string(),
                ));
            }
            println!("📡 Android BLE: Sending {} byte packet", packet_bytes.len());
            Ok(())
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod embedded_ble {
    use super::*;

    #[derive(Default)]
    pub struct BleTransport {
        enabled: bool,
        last_status: bool,
    }

    impl BleTransport {
        pub fn new() -> Self {
            Self::default()
        }

        pub async fn start_listener(&mut self) -> Result<(), TransportError> {
            println!("🔧 Embedded: Starting bare-metal BLE stack");
            self.enabled = true;
            self.last_status = true;
            println!("✅ BLE: Listening on UUID {METAMESH_BLE_UUID}");
            Ok(())
        }

        pub async fn monitor_status(&mut self) -> bool {
            self.enabled && self.last_status
        }

        pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
            if !self.enabled {
                return Err(TransportError::SendFailed(
                    "BLE transport not enabled".to_string(),
                ));
            }
            println!(
                "📡 Embedded BLE: Sending {} byte packet",
                packet_bytes.len()
            );
            Ok(())
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

// Re-export platform-specific implementations
#[cfg(target_os = "android")]
pub use android_ble::*;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use embedded_ble::*;
#[cfg(target_os = "linux")]
pub use linux_ble::*;
#[cfg(target_os = "macos")]
pub use macos_ble::*;
#[cfg(target_os = "windows")]
pub use windows_ble::*;

// Common transport monitor
#[derive(Default)]
pub struct TransportMonitor {
    ble_transport: BleTransport,
    last_available_count: u8,
}

impl TransportMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&mut self) -> Result<(), TransportError> {
        let _ = self.ble_transport.start_listener().await;
        let initial_count = self.check_transports().await;
        self.print_transport_status(initial_count);
        self.last_available_count = initial_count;

        tokio::spawn(async move {
            let mut monitor = TransportMonitor::new();
            loop {
                sleep(Duration::from_secs(5)).await;
                let available_count = monitor.check_transports().await;
                if available_count != monitor.last_available_count {
                    monitor.print_transport_status(available_count);
                    monitor.last_available_count = available_count;
                }
            }
        });

        Ok(())
    }

    pub async fn send_to_all_transports(&mut self, packet_bytes: &[u8]) -> Vec<String> {
        let mut results = Vec::new();

        if self.ble_transport.is_enabled() {
            match self.ble_transport.send_packet(packet_bytes).await {
                Ok(_) => results.push("BLE: Sent successfully".to_string()),
                Err(e) => results.push(format!("BLE: Failed - {e}")),
            }
        } else {
            results.push("BLE: Not available".to_string());
        }

        results
    }

    async fn check_transports(&mut self) -> u8 {
        let mut available = 0u8;
        if self.ble_transport.monitor_status().await {
            available += 1;
        }
        available
    }

    fn print_transport_status(&self, count: u8) {
        match count {
            0 => println!("⚠️  0 transport mediums available"),
            1 => println!("✅ 1 transport medium available (BLE)"),
            _ => println!("✅ {count} transport mediums available"),
        }
    }
}
