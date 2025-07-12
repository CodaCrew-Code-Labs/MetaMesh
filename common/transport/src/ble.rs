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
            TransportError::BluetoothNotAvailable => write!(f, "Bluetooth not available on this platform"),
            TransportError::BluetoothNotEnabled => write!(f, "Bluetooth is not enabled"),
            TransportError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            TransportError::SendFailed(msg) => write!(f, "Send failed: {}", msg),
        }
    }
}

impl Error for TransportError {}

pub struct BleTransport {
    enabled: bool,
    last_status: bool,
}

impl BleTransport {
    pub fn new() -> Self {
        Self { 
            enabled: false,
            last_status: false,
        }
    }

    pub async fn start_listener(&mut self) -> Result<(), TransportError> {
        println!("🔵 Checking BLE availability...");
        
        if !self.is_bluetooth_available() {
            println!("❌ BLE: Not available on this platform");
            return Err(TransportError::BluetoothNotAvailable);
        }

        let bt_enabled = self.is_bluetooth_enabled().await;
        
        if !bt_enabled {
            println!("❌ BLE: Bluetooth is not enabled");
            self.last_status = false;
            return Err(TransportError::BluetoothNotEnabled);
        }

        self.enabled = true;
        self.last_status = true;
        println!("✅ BLE: Listening on UUID {}", METAMESH_BLE_UUID);
        
        // Start advertising and scanning
        self.start_advertising().await?;
        self.start_scanning().await?;
        
        Ok(())
    }

    pub async fn monitor_status(&mut self) -> bool {
        let current_status = self.is_bluetooth_enabled().await;
        
        if current_status != self.last_status {
            if current_status {
                println!("🔵 BLE: Bluetooth enabled - restarting transport");
                if let Ok(_) = self.start_advertising().await {
                    let _ = self.start_scanning().await;
                    self.enabled = true;
                }
            } else {
                println!("🔴 BLE: Bluetooth disabled - transport stopped");
                self.enabled = false;
            }
            self.last_status = current_status;
        }
        
        self.enabled && current_status
    }

    pub async fn send_packet(&self, packet_bytes: &[u8]) -> Result<(), TransportError> {
        if !self.enabled {
            return Err(TransportError::SendFailed("BLE transport not enabled".to_string()));
        }
        
        println!("📡 BLE: Sending {} byte packet", packet_bytes.len());
        // TODO: Actual BLE packet transmission
        Ok(())
    }

    fn is_bluetooth_available(&self) -> bool {
        cfg!(any(
            target_os = "android",
            target_os = "ios", 
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_arch = "arm",     // Pi
            target_arch = "xtensa"   // ESP32
        ))
    }

    async fn is_bluetooth_enabled(&self) -> bool {
        // macOS - use system command as btleplug doesn't detect power state properly
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            match Command::new("system_profiler").args(&["SPBluetoothDataType"]).output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains("State: On") || output_str.contains("Discoverable: Yes")
                }
                Err(_) => false,
            }
        }
        
        // Linux/Windows - use btleplug
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            use btleplug::api::{Manager as _, Central};
            use btleplug::platform::Manager;
            
            match Manager::new().await {
                Ok(manager) => {
                    match manager.adapters().await {
                        Ok(adapters) => {
                            if adapters.is_empty() {
                                return false;
                            }
                            
                            for adapter in adapters {
                                match adapter.start_scan(btleplug::api::ScanFilter::default()).await {
                                    Ok(_) => {
                                        let _ = adapter.stop_scan().await;
                                        return true;
                                    }
                                    Err(_) => continue,
                                }
                            }
                            false
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
        
        // Other platforms - simplified for now
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            false
        }
    }

    async fn start_advertising(&self) -> Result<(), TransportError> {
        println!("📡 BLE: Starting advertising...");
        Ok(())
    }

    async fn start_scanning(&self) -> Result<(), TransportError> {
        println!("🔍 BLE: Starting scanning for MetaMesh devices...");
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub struct TransportMonitor {
    ble_transport: BleTransport,
    last_available_count: u8,
}

impl TransportMonitor {
    pub fn new() -> Self {
        Self {
            ble_transport: BleTransport::new(),
            last_available_count: 0,
        }
    }

    pub async fn start(&mut self) -> Result<(), TransportError> {
        // Try to start BLE
        let _ = self.ble_transport.start_listener().await;
        
        // Print initial status
        let initial_count = self.check_transports().await;
        self.print_transport_status(initial_count);
        self.last_available_count = initial_count;
        
        // Start monitoring loop
        tokio::spawn(async move {
            let mut monitor = TransportMonitor::new();
            loop {
                sleep(Duration::from_secs(5)).await;
                let available_count = monitor.check_transports().await;
                
                // Only print if count changed
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
        
        // Try BLE
        if self.ble_transport.is_enabled() {
            match self.ble_transport.send_packet(packet_bytes).await {
                Ok(_) => results.push("BLE: Sent successfully".to_string()),
                Err(e) => results.push(format!("BLE: Failed - {}", e)),
            }
        } else {
            results.push("BLE: Not available".to_string());
        }
        
        // TODO: Add WiFi Direct and LoRa when implemented
        
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
            _ => println!("✅ {} transport mediums available", count),
        }
    }
}