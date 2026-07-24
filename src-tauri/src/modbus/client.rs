use std::net::SocketAddr;
use tokio_modbus::client::Context;
use tokio_serial::SerialStream;

pub async fn connect_tcp(ip: &str, port: u16) -> Result<Context, String> {
    let addr_str = format!("{}:{}", ip, port);
    let socket_addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| format!("Invalid IP/Port format '{}': {}", addr_str, e))?;

    let ctx = tokio_modbus::client::tcp::connect(socket_addr)
        .await
        .map_err(|e| format!("Modbus TCP Connection failed: {}", e))?;

    Ok(ctx)
}

pub async fn connect_rtu(port_name: &str, baud_rate: u32) -> Result<Context, String> {
    let builder = tokio_serial::new(port_name, baud_rate)
        .data_bits(tokio_serial::DataBits::Eight)
        .stop_bits(tokio_serial::StopBits::One)
        .parity(tokio_serial::Parity::None);

    let serial_stream = SerialStream::open(&builder)
        .map_err(|e| format!("Failed to open serial port {}: {}", port_name, e))?;

    let ctx = tokio_modbus::client::rtu::attach(serial_stream);

    Ok(ctx)
}
