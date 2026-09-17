use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub type WireSink = Arc<dyn Fn(&str, &[u8], bool) + Send + Sync>;

// Observe only bytes actually accepted/read by the transport. Frame buffering
// handles TCP fragmentation/coalescing; never reconstruct a packet from values.
pub struct WireTap<T> {
    inner: T,
    sink: WireSink,
    tcp: bool,
    master: bool,
    rx: Vec<u8>,
    tx: Vec<u8>,
}
impl<T> std::fmt::Debug for WireTap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WireTap")
            .field("tcp", &self.tcp)
            .finish_non_exhaustive()
    }
}
impl<T> WireTap<T> {
    pub fn new(inner: T, sink: WireSink, tcp: bool, master: bool) -> Self {
        Self {
            inner,
            sink,
            tcp,
            master,
            rx: Vec::new(),
            tx: Vec::new(),
        }
    }
    fn record(&mut self, direction: &str, bytes: &[u8]) {
        let request = if self.master {
            direction == "TX"
        } else {
            direction == "RX"
        };
        let buffer = if direction == "TX" {
            &mut self.tx
        } else {
            &mut self.rx
        };
        buffer.extend_from_slice(bytes);
        loop {
            let length = if self.tcp {
                if buffer.len() < 6 {
                    break;
                }
                let length = u16::from_be_bytes([buffer[4], buffer[5]]) as usize;
                if !(2..=254).contains(&length) {
                    (self.sink)(direction, buffer, false);
                    buffer.clear();
                    break;
                }
                6 + length
            } else {
                if buffer.len() < 2 {
                    break;
                }
                let fc = buffer[1];
                if fc & 0x80 != 0 {
                    5
                } else if request {
                    match fc {
                        1..=6 => 8,
                        15 | 16 => {
                            if buffer.len() < 7 {
                                break;
                            }
                            9 + buffer[6] as usize
                        }
                        _ => {
                            (self.sink)(direction, buffer, false);
                            buffer.clear();
                            break;
                        }
                    }
                } else {
                    match fc {
                        1..=4 => {
                            if buffer.len() < 3 {
                                break;
                            }
                            5 + buffer[2] as usize
                        }
                        5 | 6 | 15 | 16 => 8,
                        _ => {
                            (self.sink)(direction, buffer, false);
                            buffer.clear();
                            break;
                        }
                    }
                }
            };
            if buffer.len() < length {
                break;
            }
            let frame: Vec<_> = buffer.drain(..length).collect();
            (self.sink)(direction, &frame, true);
        }
    }
    pub fn flush_partial(&mut self) {
        for (direction, buffer) in [("RX", &mut self.rx), ("TX", &mut self.tx)] {
            if !buffer.is_empty() {
                (self.sink)(direction, buffer, false);
                buffer.clear();
            }
        }
    }
}
impl<T> Drop for WireTap<T> {
    fn drop(&mut self) {
        self.flush_partial();
    }
}
impl<T: AsyncRead + Unpin> AsyncRead for WireTap<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buffer.filled().len();
        let result = Pin::new(&mut this.inner).poll_read(cx, buffer);
        if let Poll::Ready(Ok(())) = &result {
            let bytes = &buffer.filled()[before..];
            if bytes.is_empty() {
                this.flush_partial();
            } else {
                this.record("RX", bytes);
            }
        } else if matches!(&result, Poll::Ready(Err(_))) {
            this.flush_partial();
        }
        result
    }
}
impl<T: AsyncWrite + Unpin> AsyncWrite for WireTap<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, bytes);
        if let Poll::Ready(Ok(n)) = &result {
            if *n > 0 && this.master && this.tx.is_empty() && !this.rx.is_empty() {
                (this.sink)("RX", &this.rx, false);
                this.rx.clear();
            }
            this.record("TX", &bytes[..*n]);
        } else if matches!(&result, Poll::Ready(Err(_))) {
            this.flush_partial();
        }
        result
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        this.flush_partial();
        Pin::new(&mut this.inner).poll_shutdown(cx)
    }
}

pub fn emit_frame(
    app: &tauri::AppHandle,
    session_id: &str,
    direction: &str,
    bytes: &[u8],
    tcp: bool,
    complete: bool,
) {
    use std::sync::atomic::{AtomicU64, Ordering};
    use tauri::Emitter;
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let crc_ok = tcp
        || (bytes.len() >= 4
            && super::rtu_router::crc16(&bytes[..bytes.len() - 2])
                == u16::from_le_bytes([bytes[bytes.len() - 2], bytes[bytes.len() - 1]]));
    let fc = bytes.get(if tcp { 7 } else { 1 }).copied().unwrap_or(0);
    let failed = !complete || !crc_ok || fc & 0x80 != 0;
    let _ = app.emit("traffic-log-entry", serde_json::json!({
        "id": format!("wire-{now}-{}", SEQ.fetch_add(1, Ordering::Relaxed)), "sessionId": session_id,
        "timestampMs": now, "direction": direction, "kind": "frame", "detail": false,
        "level": if failed { "error" } else { "info" },
        "protocol": if tcp { "TCP" } else { "RTU" }, "complete": complete,
        "message": if !complete { "Incomplete / unframed data" } else if !crc_ok { "CRC mismatch" } else if fc & 0x80 != 0 { "Modbus exception" } else { "" },
        "bytes": bytes.iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ")
    }));
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    type Events = Arc<Mutex<Vec<(String, Vec<u8>, bool)>>>;
    fn sink() -> (Events, WireSink) {
        let events: Events = Arc::new(Mutex::new(Vec::new()));
        let capture = events.clone();
        (
            events,
            Arc::new(move |direction, bytes, complete| {
                capture
                    .lock()
                    .unwrap()
                    .push((direction.into(), bytes.to_vec(), complete))
            }),
        )
    }
    #[tokio::test]
    async fn a_silent_slave_still_produces_tx_before_timeout_for_tcp_and_rtu() {
        use tokio_modbus::prelude::*;
        for tcp in [true, false] {
            let (client, mut peer) = tokio::io::duplex(128);
            let (events, sink) = sink();
            let tapped = WireTap::new(client, sink, tcp, true);
            let mut context = if tcp {
                tokio_modbus::client::tcp::attach_slave(tapped, Slave(1))
            } else {
                tokio_modbus::client::rtu::attach_slave(tapped, Slave(1))
            };
            let reading = tokio::spawn(async move {
                tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    context.read_holding_registers(0, 2),
                )
                .await
            });
            let mut request = vec![0; if tcp { 12 } else { 8 }];
            peer.read_exact(&mut request).await.unwrap();
            {
                let captured = events.lock().unwrap();
                assert_eq!(captured.len(), 1);
                assert_eq!(captured[0].0, "TX");
                assert_eq!(captured[0].1, request);
                assert!(captured[0].2);
            }
            assert!(reading.await.unwrap().is_err());
            assert_eq!(
                events.lock().unwrap().len(),
                1,
                "No RX should be fabricated on timeout"
            );
        }
    }

    #[tokio::test]
    async fn tcp_fragmented_and_coalesced_replies_preserve_every_byte_and_repeat() {
        let (client, mut peer) = tokio::io::duplex(128);
        let (events, sink) = sink();
        let mut tapped = WireTap::new(client, sink, true, true);
        let frame = [0, 7, 0, 0, 0, 5, 1, 3, 2, 0x12, 0x34];
        peer.write_all(&frame[..4]).await.unwrap();
        let mut buffer = [0; 64];
        tapped.read(&mut buffer).await.unwrap();
        assert!(events.lock().unwrap().is_empty());
        peer.write_all(&[&frame[4..], &frame[..]].concat())
            .await
            .unwrap();
        tapped.read(&mut buffer).await.unwrap();
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .all(|event| event.0 == "RX" && event.1 == frame && event.2));
    }
    #[tokio::test]
    async fn partial_writes_are_logged_only_as_accepted_and_incomplete_reads_survive_drop() {
        let (client, mut peer) = tokio::io::duplex(4);
        let (events, sink) = sink();
        let mut tapped = WireTap::new(client, sink, true, true);
        let frame = [0, 7, 0, 0, 0, 6, 1, 3, 0, 0, 0, 2];
        let accepted = tapped.write(&frame).await.unwrap();
        assert_eq!(accepted, 4);
        let mut buffer = [0; 4];
        peer.read_exact(&mut buffer).await.unwrap();
        assert_eq!(&buffer, &frame[..4]);
        peer.write_all(&[0, 7, 0]).await.unwrap();
        tapped.read(&mut buffer).await.unwrap();
        drop(tapped);
        let events = events.lock().unwrap();
        assert!(events
            .iter()
            .any(|event| event.0 == "TX" && event.1 == frame[..4] && !event.2));
        assert!(events
            .iter()
            .any(|event| event.0 == "RX" && event.1 == [0, 7, 0] && !event.2));
    }
    #[tokio::test]
    async fn rtu_records_full_frames_including_invalid_crc_and_identical_retries() {
        let (client, mut peer) = tokio::io::duplex(128);
        let (events, sink) = sink();
        let mut tapped = WireTap::new(client, sink, false, true);
        let request = [1, 3, 0, 0, 0, 2, 0xc4, 0x0b];
        tapped.write_all(&request).await.unwrap();
        tapped.write_all(&request).await.unwrap();
        let invalid = [1, 3, 2, 0, 1, 0, 0];
        peer.write_all(&invalid).await.unwrap();
        let mut buffer = [0; 32];
        tapped.read(&mut buffer).await.unwrap();
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].1, request);
        assert_eq!(events[1].1, request);
        assert_eq!(events[2].1, invalid);
    }
    #[tokio::test]
    async fn failed_write_does_not_invent_transmitted_data() {
        let (client, peer) = tokio::io::duplex(128);
        let (events, sink) = sink();
        let mut tapped = WireTap::new(client, sink, true, true);
        drop(peer);
        assert!(tapped.write_all(&[1, 2, 3]).await.is_err());
        assert!(events.lock().unwrap().is_empty());
    }
}
