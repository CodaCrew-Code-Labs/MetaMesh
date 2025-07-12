use serde::{Serialize, Deserialize};

pub mod ble;
pub use ble::{BleTransport, TransportError, TransportMonitor, METAMESH_BLE_UUID};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PacketType {
    Message = 0x01,
    KeyRequest = 0x02,
    KeyResponse = 0x03,
    Ping = 0x04,
    EmergencyBroadcast = 0x05,
    RouteDiscovery = 0x06,
    RouteResponse = 0x07,
    Ack = 0x08,
    Reserved1 = 0x09,
    Reserved2 = 0x0A,
    Reserved3 = 0x0B,
    Reserved4 = 0x0C,
    Reserved5 = 0x0D,
    Reserved6 = 0x10,
    Reserved7 = 0x11,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct PacketFlags: u16 {
        const ENCRYPTED  = 0b0000000000000001;
        const SIGNED     = 0b0000000000000010;
        const BROADCAST  = 0b0000000000000100;
        const COMPRESSED = 0b0000000000001000;
        const PRIORITY   = 0b0000000000010000;
        const RESERVED1  = 0b0000000000100000;
        const RESERVED2  = 0b0000000001000000;
        const RESERVED3  = 0b0000000010000000;
        const RESERVED4  = 0b0000000100000000;
        const RESERVED5  = 0b0000001000000000;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PacketHeader {
    pub packet_type: PacketType,     // 1 byte
    pub version: u8,                 // 1 byte
    pub ttl: u8,                     // 1 byte
    pub flags: PacketFlags,          // 2 bytes
    pub from_seed: [u8; 16],         // 16 bytes
    pub to_seed: [u8; 16],           // 16 bytes (0s if broadcast)
    pub nonce: [u8; 8],              // 8 bytes (anti-replay)
    pub payload_len: u32,            // 4 bytes
    // Total: 49 bytes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TLV {
    pub type_id: u8,
    pub length: u16,
    pub value: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaMeshPacket {
    pub magic: [u8; 2],              // 2 bytes - "MM"
    pub header: PacketHeader,        // 49 bytes
    pub payload: Vec<TLV>,           // Variable
    // Total: 51+ bytes
}

// Magic header constant
pub const MAGIC_HEADER: [u8; 2] = *b"MM";

// TLV Type IDs
pub const TLV_SIG: u8 = 0x01;
pub const TLV_PUBKEY: u8 = 0x02;
pub const TLV_DATA: u8 = 0x03;
pub const TLV_ROUTE: u8 = 0x04;