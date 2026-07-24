use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusRole {
    Slave,
    Master,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModbusProtocol {
    Tcp,
    Rtu,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub role: String,
    pub protocol: String,
    pub ip: String,
    pub port: u16,
    #[serde(rename = "serialPort")]
    pub serial_port: String,
    #[serde(rename = "baudRate")]
    pub baud_rate: u32,
    #[serde(rename = "dataBits")]
    pub data_bits: u8,
    #[serde(rename = "stopBits")]
    pub stop_bits: u8,
    pub parity: String,
    #[serde(rename = "timeoutMs", default = "default_timeout_ms")]
    pub timeout_ms: u32,
    #[serde(rename = "retries", default = "default_retries")]
    pub retries: u8,
}

fn default_timeout_ms() -> u32 {
    1000
}

fn default_retries() -> u8 {
    3
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficFrame {
    pub id: String,
    pub time: String,
    pub direction: String,
    pub message: String,
    pub bytes: String,
}
