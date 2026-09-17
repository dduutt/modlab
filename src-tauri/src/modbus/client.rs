use std::net::SocketAddr;
use std::time::Duration;

use tokio_modbus::client::Context;
use tokio_modbus::prelude::Slave;
use tokio_serial::{DataBits, Parity, SerialStream, StopBits};

use super::types::ConnectionConfig;
use super::wire::{WireSink, WireTap};

fn data_bits(value: u8) -> Result<DataBits, String> {
    match value {
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => Err(format!("Unsupported data bits: {value}. Use 7 or 8.")),
    }
}

fn stop_bits(value: u8) -> Result<StopBits, String> {
    match value {
        1 => Ok(StopBits::One),
        2 => Ok(StopBits::Two),
        _ => Err(format!("Unsupported stop bits: {value}. Use 1 or 2.")),
    }
}

fn parity(value: &str) -> Result<Parity, String> {
    match value {
        "None" => Ok(Parity::None),
        "Even" => Ok(Parity::Even),
        "Odd" => Ok(Parity::Odd),
        _ => Err(format!(
            "Unsupported parity: {value}. Use None, Even, or Odd."
        )),
    }
}

pub fn open_serial(config: &ConnectionConfig) -> Result<SerialStream, String> {
    let builder = tokio_serial::new(&config.serial_port, config.baud_rate)
        .data_bits(data_bits(config.data_bits)?)
        .stop_bits(stop_bits(config.stop_bits)?)
        .parity(parity(&config.parity)?);

    SerialStream::open(&builder)
        .map_err(|e| format!("Failed to open serial port {}: {e}", config.serial_port))
}

#[cfg(test)]
pub async fn connect_tcp(
    ip: &str,
    port: u16,
    unit_id: u8,
    timeout_ms: u32,
    retries: u8,
) -> Result<Context, String> {
    connect_tcp_logged(ip, port, unit_id, timeout_ms, retries, None).await
}

pub async fn connect_tcp_logged(
    ip: &str,
    port: u16,
    unit_id: u8,
    timeout_ms: u32,
    retries: u8,
    sink: Option<WireSink>,
) -> Result<Context, String> {
    let addr_str = format!("{ip}:{port}");
    let socket_addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| format!("Invalid IP/Port format '{addr_str}': {e}"))?;
    let timeout = Duration::from_millis(u64::from(timeout_ms));
    let mut last_error = String::new();

    for _ in 0..=retries {
        match tokio::time::timeout(timeout, async {
            let stream = tokio::net::TcpStream::connect(socket_addr).await?;
            let sink = sink
                .clone()
                .unwrap_or_else(|| std::sync::Arc::new(|_, _, _| {}));
            Ok::<_, std::io::Error>(tokio_modbus::client::tcp::attach_slave(
                WireTap::new(stream, sink, true, true),
                Slave(unit_id),
            ))
        })
        .await
        {
            Ok(Ok(ctx)) => return Ok(ctx),
            Ok(Err(err)) => last_error = err.to_string(),
            Err(_) => last_error = format!("connection timed out after {timeout_ms}ms"),
        }
    }

    Err(format!("Modbus TCP connection failed: {last_error}"))
}

pub fn connect_rtu_logged(
    config: &ConnectionConfig,
    unit_id: u8,
    sink: WireSink,
) -> Result<Context, String> {
    let serial_stream = WireTap::new(open_serial(config)?, sink, false, true);
    Ok(tokio_modbus::client::rtu::attach_slave(
        serial_stream,
        Slave(unit_id),
    ))
}
