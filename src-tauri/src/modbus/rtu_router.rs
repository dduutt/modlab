use super::wire::{emit_frame, WireSink, WireTap};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tokio_serial::SerialStream;

use super::server::SessionMemory;
use super::types::ConnectionConfig;

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &byte in data {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[derive(Clone)]
pub struct SlaveRoute {
    pub session_id: String,
    pub session_title: String,
    pub memory: SessionMemory,
}

pub struct SerialPortRouter {
    pub config: ConnectionConfig,
    #[allow(dead_code)]
    pub port_name: String,
    pub slaves: Arc<Mutex<HashMap<u8, SlaveRoute>>>,
    pub stop_tx: broadcast::Sender<()>,
    pub task: tokio::task::JoinHandle<()>,
}

impl SerialPortRouter {
    pub fn new(
        app_handle: AppHandle,
        port_name: String,
        serial: SerialStream,
        config: ConnectionConfig,
    ) -> Self {
        let slaves = Arc::new(Mutex::new(HashMap::<u8, SlaveRoute>::new()));
        let (stop_tx, mut stop_rx) = broadcast::channel::<()>(1);
        let slaves_clone = slaves.clone();
        let port_label = port_name.clone();
        let log_slaves = slaves.clone();
        let log_app = app_handle.clone();
        let sink: WireSink = Arc::new(move |direction, bytes, complete| {
            let ids: Vec<_> = {
                let routes = log_slaves.lock().unwrap_or_else(|err| err.into_inner());
                if let Some(route) = bytes.first().and_then(|id| routes.get(id)) {
                    vec![route.session_id.clone()]
                } else {
                    routes
                        .values()
                        .map(|route| route.session_id.clone())
                        .collect()
                }
            };
            for id in ids {
                emit_frame(&log_app, &id, direction, bytes, false, complete);
            }
        });
        let mut serial = WireTap::new(serial, sink, false, false);

        let task = tokio::spawn(async move {
            let mut read_buf = [0u8; 512];
            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        break;
                    }
                    read_res = serial.read(&mut read_buf) => {
                        match read_res {
                            Ok(0) => {
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            }
                            Ok(n) => {
                                let mut frame = Vec::with_capacity(512);
                                frame.extend_from_slice(&read_buf[..n]);

                                // Modbus RTU 3.5T inter-character timeout accumulation
                                loop {
                                    match tokio::time::timeout(Duration::from_millis(20), serial.read(&mut read_buf)).await {
                                        Ok(Ok(more)) if more > 0 => {
                                            frame.extend_from_slice(&read_buf[..more]);
                                        }
                                        _ => break,
                                    }
                                }

                                serial.flush_partial();
                                Self::handle_frame(&mut serial, &slaves_clone, &frame).await;
                            }
                            Err(err) => {
                                eprintln!("[{port_label}] Serial read error: {err}");
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            }
        });

        Self {
            config,
            port_name,
            slaves,
            stop_tx,
            task,
        }
    }

    async fn handle_frame(
        serial: &mut WireTap<SerialStream>,
        slaves: &Arc<Mutex<HashMap<u8, SlaveRoute>>>,
        frame: &[u8],
    ) {
        if frame.len() < 4 {
            return;
        }

        let len = frame.len();
        let expected_crc = u16::from_le_bytes([frame[len - 2], frame[len - 1]]);
        let calculated_crc = crc16(&frame[..len - 2]);
        if expected_crc != calculated_crc {
            return;
        }

        let unit_id = frame[0];
        let target_slave = {
            let map = match slaves.lock() {
                Ok(m) => m,
                Err(_) => return,
            };
            map.get(&unit_id).cloned()
        };

        // If no slave registered for this Unit ID on this port, silently discard
        let Some(slave) = target_slave else {
            return;
        };

        let pdu = &frame[1..len - 2];
        if let Some(resp_pdu) = process_pdu(&slave.memory, pdu) {
            let mut resp_frame = Vec::with_capacity(resp_pdu.len() + 3);
            resp_frame.push(unit_id);
            resp_frame.extend_from_slice(&resp_pdu);
            let crc = crc16(&resp_frame);
            resp_frame.extend_from_slice(&crc.to_le_bytes());

            if let Err(e) = serial.write_all(&resp_frame).await {
                eprintln!("Failed to write serial response: {e}");
                return;
            }
        }
    }
}

fn process_pdu(memory: &SessionMemory, pdu: &[u8]) -> Option<Vec<u8>> {
    if pdu.is_empty() {
        return None;
    }
    let fc = pdu[0];
    match fc {
        // 0x01: Read Coils
        0x01 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            match memory.get_coil_range(start, count) {
                Ok(coils) => {
                    let byte_count = ((count + 7) / 8) as usize;
                    let mut bytes = vec![0u8; byte_count];
                    for (i, &coil) in coils.iter().enumerate() {
                        if coil {
                            bytes[i / 8] |= 1 << (i % 8);
                        }
                    }
                    let mut resp = Vec::with_capacity(2 + byte_count);
                    resp.push(fc);
                    resp.push(byte_count as u8);
                    resp.extend(bytes);
                    Some(resp)
                }
                Err(_) => Some(vec![fc | 0x80, 0x02]),
            }
        }
        // 0x02: Read Discrete Inputs
        0x02 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            match memory.get_discrete_input_range(start, count) {
                Ok(coils) => {
                    let byte_count = ((count + 7) / 8) as usize;
                    let mut bytes = vec![0u8; byte_count];
                    for (i, &coil) in coils.iter().enumerate() {
                        if coil {
                            bytes[i / 8] |= 1 << (i % 8);
                        }
                    }
                    let mut resp = Vec::with_capacity(2 + byte_count);
                    resp.push(fc);
                    resp.push(byte_count as u8);
                    resp.extend(bytes);
                    Some(resp)
                }
                Err(_) => Some(vec![fc | 0x80, 0x02]),
            }
        }
        // 0x03: Read Holding Registers
        0x03 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            match memory.get_holding_range(start, count) {
                Ok(words) => {
                    let mut resp = Vec::with_capacity(2 + words.len() * 2);
                    resp.push(fc);
                    resp.push((words.len() * 2) as u8);
                    for w in words {
                        resp.extend_from_slice(&w.to_be_bytes());
                    }
                    Some(resp)
                }
                Err(_) => Some(vec![fc | 0x80, 0x02]),
            }
        }
        // 0x04: Read Input Registers
        0x04 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            match memory.get_input_range(start, count) {
                Ok(words) => {
                    let mut resp = Vec::with_capacity(2 + words.len() * 2);
                    resp.push(fc);
                    resp.push((words.len() * 2) as u8);
                    for w in words {
                        resp.extend_from_slice(&w.to_be_bytes());
                    }
                    Some(resp)
                }
                Err(_) => Some(vec![fc | 0x80, 0x02]),
            }
        }
        // 0x05: Write Single Coil
        0x05 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let addr = u16::from_be_bytes([pdu[1], pdu[2]]);
            let val = u16::from_be_bytes([pdu[3], pdu[4]]);
            memory.set_coil_value(addr, val == 0xFF00);
            Some(pdu[..5].to_vec())
        }
        // 0x06: Write Single Register
        0x06 => {
            if pdu.len() < 5 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let addr = u16::from_be_bytes([pdu[1], pdu[2]]);
            let val = u16::from_be_bytes([pdu[3], pdu[4]]);
            memory.set_holding(addr, val);
            Some(pdu[..5].to_vec())
        }
        // 0x0F: Write Multiple Coils
        0x0F => {
            if pdu.len() < 6 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            let byte_count = pdu[5] as usize;
            if count == 0
                || count > 1968
                || byte_count != (usize::from(count) + 7) / 8
                || pdu.len() != 6 + byte_count
            {
                return Some(vec![fc | 0x80, 0x03]);
            }
            if u32::from(start) + u32::from(count) > 65_536 {
                return Some(vec![fc | 0x80, 0x02]);
            }
            let data = &pdu[6..6 + byte_count];
            for i in 0..count {
                let byte_idx = (i / 8) as usize;
                let bit_idx = i % 8;
                let val = (data[byte_idx] & (1 << bit_idx)) != 0;
                memory.set_coil_value(start + i, val);
            }
            Some(pdu[..5].to_vec())
        }
        // 0x10: Write Multiple Registers
        0x10 => {
            if pdu.len() < 6 {
                return Some(vec![fc | 0x80, 0x03]);
            }
            let start = u16::from_be_bytes([pdu[1], pdu[2]]);
            let count = u16::from_be_bytes([pdu[3], pdu[4]]);
            let byte_count = pdu[5] as usize;
            if count == 0
                || count > 123
                || pdu.len() != 6 + byte_count
                || byte_count != (usize::from(count) * 2)
            {
                return Some(vec![fc | 0x80, 0x03]);
            }
            if u32::from(start) + u32::from(count) > 65_536 {
                return Some(vec![fc | 0x80, 0x02]);
            }
            let data = &pdu[6..6 + byte_count];
            for i in 0..count {
                let idx = (i * 2) as usize;
                let val = u16::from_be_bytes([data[idx], data[idx + 1]]);
                memory.set_holding(start + i, val);
            }
            Some(pdu[..5].to_vec())
        }
        _ => Some(vec![fc | 0x80, 0x01]), // Illegal function code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_multiple_writes_leave_memory_unchanged() {
        let cases: Vec<(Vec<u8>, u8)> = vec![
            (vec![0x0F, 0, 0, 0, 9, 1, 0xFF], 0x03),
            (vec![0x0F, 0, 0, 0, 1, 0], 0x03),
            (vec![0x0F, 0, 0, 0, 0, 0], 0x03),
            (vec![0x0F, 0, 0, 0, 1, 1], 0x03),
            (vec![0x0F, 0xFF, 0xFF, 0, 2, 1, 3], 0x02),
            (vec![0x10, 0, 0, 0, 2, 2, 0, 99], 0x03),
            (vec![0x10, 0, 0, 0, 0, 0], 0x03),
            (vec![0x10, 0xFF, 0xFF, 0, 2, 4, 0, 99, 0, 88], 0x02),
        ];
        for (pdu, exception) in cases {
            let memory = SessionMemory::new();
            memory.set_holding(65_535, 42);
            assert_eq!(
                process_pdu(&memory, &pdu),
                Some(vec![pdu[0] | 0x80, exception])
            );
            assert_eq!(memory.get_coil_range(0, 9).unwrap(), vec![false; 9]);
            assert_eq!(memory.get_coil_range(65_535, 1).unwrap(), vec![false]);
            assert_eq!(memory.get_holding_range(0, 2).unwrap(), vec![0; 2]);
            assert_eq!(memory.get_holding_range(65_535, 1).unwrap(), vec![42]);
            // A rejected request must not prevent the next valid request.
            assert_eq!(
                process_pdu(&memory, &[0x06, 0, 0, 0, 7]),
                Some(vec![0x06, 0, 0, 0, 7])
            );
        }
    }

    #[test]
    fn multiple_writes_enforce_limits_and_allow_last_address() {
        for (fc, count, bytes) in [(0x0F, 1969u16, 247u8), (0x10, 124, 248)] {
            let mut pdu = vec![fc, 0, 0, (count >> 8) as u8, count as u8, bytes];
            pdu.extend(vec![0xFF; usize::from(bytes)]);
            assert_eq!(
                process_pdu(&SessionMemory::new(), &pdu),
                Some(vec![fc | 0x80, 0x03])
            );
        }
        let memory = SessionMemory::new();
        for pdu in [
            vec![0x0F, 0xFF, 0xFF, 0, 1, 1, 1],
            vec![0x10, 0xFF, 0xFF, 0, 1, 2, 0x12, 0x34],
        ] {
            assert_eq!(process_pdu(&memory, &pdu), Some(pdu[..5].to_vec()));
        }
        assert_eq!(memory.get_coil_range(65_535, 1).unwrap(), vec![true]);
        assert_eq!(memory.get_holding_range(65_535, 1).unwrap(), vec![0x1234]);
    }
}
