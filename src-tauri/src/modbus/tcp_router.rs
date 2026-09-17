use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::net::TcpListener;

use super::server::{start_tcp_slave, SessionMemory, TcpLogSink, TcpSlaveRoute, TcpSlaveTask};

/// Owned behind the application's async mutex. Registration and last-route
/// shutdown are serialized, including the bind/release of the listening socket.
#[derive(Default)]
pub struct TcpSlaveRegistry {
    listeners: HashMap<SocketAddr, TcpSlaveTask>,
    bindings: HashMap<String, (SocketAddr, u8)>,
    logger: Option<TcpLogSink>,
}

impl TcpSlaveRegistry {
    pub fn clients(&self, session_id: &str) -> Vec<super::server::TcpClientInfo> {
        self.bindings
            .get(session_id)
            .and_then(|(address, _)| self.listeners.get(address))
            .map(|listener| listener.clients())
            .unwrap_or_default()
    }

    pub fn set_logger(&mut self, logger: TcpLogSink) -> &mut Self {
        self.logger = Some(logger.clone());
        for listener in self.listeners.values() {
            listener.set_logger(Some(logger.clone()));
        }
        self
    }

    pub async fn register(
        &mut self,
        address: SocketAddr,
        unit_id: u8,
        session_id: String,
        session_title: String,
        memory: SessionMemory,
    ) -> Result<SocketAddr, String> {
        if let Some(&(current_address, current_unit)) = self.bindings.get(&session_id) {
            if current_address == address && current_unit == unit_id {
                return Ok(current_address);
            }
            return Err(
                "Disconnect this session before changing its TCP address or Unit ID".into(),
            );
        }

        let route = TcpSlaveRoute {
            session_id: session_id.clone(),
            session_title,
            memory,
        };
        let address = if let Some(listener) = self.listeners.get(&address) {
            listener
                .register(unit_id, route)
                .map_err(|err| format!("{address}: {err}"))?;
            address
        } else {
            let socket = TcpListener::bind(address)
                .await
                .map_err(|err| format!("Failed to listen on {address}: {err}"))?;
            let bound_address = socket.local_addr().map_err(|err| err.to_string())?;
            let listener = start_tcp_slave(socket);
            listener.set_logger(self.logger.clone());
            listener.register(unit_id, route)?;
            self.listeners.insert(bound_address, listener);
            bound_address
        };
        self.bindings.insert(session_id, (address, unit_id));
        Ok(address)
    }

    pub async fn unregister(&mut self, session_id: &str) -> Result<(), String> {
        let Some(&(address, unit_id)) = self.bindings.get(session_id) else {
            return Ok(());
        };
        if let Some(listener) = self.listeners.get(&address) {
            if listener.unregister(unit_id, session_id)? == 0 {
                if let Some(listener) = self.listeners.remove(&address) {
                    listener.stop().await;
                }
            }
        }
        self.bindings.remove(session_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    use tokio_modbus::prelude::*;

    async fn add(
        registry: &mut TcpSlaveRegistry,
        address: SocketAddr,
        id: u8,
        name: &str,
        value: u16,
    ) -> SocketAddr {
        let memory = SessionMemory::new();
        for area in ["0x01", "0x02", "0x03", "0x04"] {
            memory.set_value(area, 10, value).unwrap();
        }
        registry
            .register(address, id, name.into(), name.into(), memory)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn external_requests_emit_logs_for_the_correct_slave_session() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = events.clone();
        let mut registry = TcpSlaveRegistry::default();
        registry.set_logger(Arc::new(move |entry| captured.lock().unwrap().push(entry)));
        let address = add(
            &mut registry,
            "127.0.0.1:0".parse().unwrap(),
            1,
            "slave-A",
            11,
        )
        .await;
        add(&mut registry, address, 2, "slave-B", 22).await;
        let master_events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_master = master_events.clone();
        let sink: super::super::wire::WireSink = Arc::new(move |direction, bytes, complete| {
            captured_master.lock().unwrap().push((
                direction.to_string(),
                bytes
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                complete,
            ));
        });
        let mut client = super::super::client::connect_tcp_logged(
            &address.ip().to_string(),
            address.port(),
            2,
            1000,
            0,
            Some(sink),
        )
        .await
        .unwrap();
        client.write_single_register(10, 33).await.unwrap();
        assert_eq!(read(&mut client, 2).await.unwrap(), vec![33]);
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
        {
            let events = events.lock().unwrap();
            assert_eq!(events.len(), 6);
            let master = master_events.lock().unwrap();
            assert_eq!(master.len(), 6);
            for (index, event) in events.iter().enumerate() {
                assert_eq!(event.bytes, master[index].1);
                assert_ne!(event.direction, master[index].0);
                assert!(master[index].2);
            }
            assert!(!events[0].detail);
            assert_eq!(events[0].direction, "RX");
            assert_eq!(events[1].direction, "TX");
            assert!(events[0].bytes.contains("00 00 00 06 02 06 00 0A 00 21"));
            assert_eq!(events[0].bytes, events[1].bytes);
            assert!(!events[1].detail);
            assert!(events[..4].iter().all(|log| log.session_id == "slave-B"));
            assert!(events[4..].iter().all(|log| log.session_id == "slave-A"));
            assert!(events.iter().all(|log| log.kind == "frame"));
            assert_eq!(
                events
                    .iter()
                    .map(|log| &log.id)
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                6
            );
            let json = serde_json::to_value(&events[0]).unwrap();
            assert_eq!(json["sessionId"], "slave-B");
            assert!(json["timestampMs"].is_number());
        }
        registry.unregister("slave-A").await.unwrap();
        registry.unregister("slave-B").await.unwrap();
    }

    #[tokio::test]
    async fn client_list_tracks_idle_connections_requests_disconnects_and_shared_listeners() {
        async fn wait_count(registry: &TcpSlaveRegistry, session: &str, count: usize) {
            tokio::time::timeout(Duration::from_secs(2), async {
                while registry.clients(session).len() != count {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
        }
        let mut registry = TcpSlaveRegistry::default();
        let address = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "A", 11).await;
        add(&mut registry, address, 2, "B", 22).await;
        let other = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "C", 33).await;
        assert_ne!(address, other);
        assert!(registry.clients("A").is_empty());
        let mut active = tcp::connect_slave(address, Slave(1)).await.unwrap();
        let idle = tokio::net::TcpStream::connect(address).await.unwrap();
        let idle_address = idle.local_addr().unwrap().to_string();
        wait_count(&registry, "A", 2).await;
        let clients = registry.clients("A");
        assert!(clients
            .iter()
            .all(|client| client.connected_at > 0 && client.last_request_at.is_none()));
        assert_eq!(
            serde_json::to_value(&clients).unwrap(),
            serde_json::to_value(registry.clients("B")).unwrap()
        );
        assert!(registry.clients("C").is_empty());
        assert_eq!(read(&mut active, 1).await.unwrap(), vec![11]);
        let clients = registry.clients("B");
        assert!(clients
            .iter()
            .find(|client| client.address == idle_address)
            .unwrap()
            .last_request_at
            .is_none());
        assert_eq!(
            clients
                .iter()
                .filter(|client| client.last_request_at.is_some())
                .count(),
            1
        );
        drop(idle);
        wait_count(&registry, "B", 1).await;
        registry.unregister("A").await.unwrap();
        assert!(registry.clients("A").is_empty());
        assert_eq!(registry.clients("B").len(), 1);
        drop(active);
        wait_count(&registry, "B", 0).await;
        registry.unregister("B").await.unwrap();
        registry.unregister("C").await.unwrap();
        assert!(registry.clients("B").is_empty());
    }

    async fn read(
        client: &mut tokio_modbus::client::Context,
        id: u8,
    ) -> Result<Vec<u16>, std::io::Error> {
        client.set_slave(Slave(id));
        tokio::time::timeout(Duration::from_secs(1), client.read_holding_registers(10, 1))
            .await
            .expect("TCP request timed out")
    }

    #[tokio::test]
    async fn one_connection_routes_all_data_areas_by_unit_id() {
        let mut registry = TcpSlaveRegistry::default();
        let address = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "A", 11).await;
        add(&mut registry, address, 2, "B", 0).await;
        assert_eq!(registry.listeners.len(), 1);
        let mut client = tcp::connect_slave(address, Slave(1)).await.unwrap();
        for (id, expected) in [(1, 11), (2, 0), (1, 11)] {
            assert_eq!(read(&mut client, id).await.unwrap(), vec![expected]);
            assert_eq!(
                client.read_input_registers(10, 1).await.unwrap(),
                vec![expected]
            );
            assert_eq!(client.read_coils(10, 1).await.unwrap(), vec![expected != 0]);
            assert_eq!(
                client.read_discrete_inputs(10, 1).await.unwrap(),
                vec![expected != 0]
            );
        }
        client.set_slave(Slave(2));
        client.write_single_register(10, 22).await.unwrap();
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
        assert_eq!(read(&mut client, 2).await.unwrap(), vec![22]);
        assert!(read(&mut client, 99).await.is_err());
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
    }

    #[tokio::test]
    async fn live_add_remove_and_rejoin_keep_other_slaves_connected() {
        let mut registry = TcpSlaveRegistry::default();
        let address = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "A", 11).await;
        let mut client = tcp::connect_slave(address, Slave(1)).await.unwrap();
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
        add(&mut registry, address, 2, "B", 22).await;
        assert_eq!(read(&mut client, 2).await.unwrap(), vec![22]);
        registry.unregister("A").await.unwrap();
        registry.unregister("A").await.unwrap();
        assert!(read(&mut client, 1).await.is_err());
        assert_eq!(read(&mut client, 2).await.unwrap(), vec![22]);
        add(&mut registry, address, 1, "A", 33).await;
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![33]);
        assert_eq!(read(&mut client, 2).await.unwrap(), vec![22]);
        registry.unregister("A").await.unwrap();
        registry.unregister("B").await.unwrap();
        assert!(registry.listeners.is_empty());
        assert!(registry.bindings.is_empty());
        assert!(read(&mut client, 2).await.is_err());
        let rebound = TcpListener::bind(address).await.unwrap();
        drop(rebound);
        add(&mut registry, address, 2, "B", 44).await;
        let mut new_client = tcp::connect_slave(address, Slave(2)).await.unwrap();
        assert_eq!(read(&mut new_client, 2).await.unwrap(), vec![44]);
    }

    #[tokio::test]
    async fn duplicate_unit_is_rejected_without_changing_existing_routes() {
        let mut registry = TcpSlaveRegistry::default();
        let address = add(
            &mut registry,
            "127.0.0.1:0".parse().unwrap(),
            1,
            "Original",
            11,
        )
        .await;
        let error = registry
            .register(
                address,
                1,
                "Conflict".into(),
                "Conflict".into(),
                SessionMemory::new(),
            )
            .await
            .unwrap_err();
        assert!(error.contains("Original"));
        assert!(error.contains(&address.to_string()));
        assert_eq!(registry.bindings.len(), 1);
        let mut client = tcp::connect_slave(address, Slave(1)).await.unwrap();
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
        registry.unregister("Conflict").await.unwrap();
        assert_eq!(read(&mut client, 1).await.unwrap(), vec![11]);
    }

    #[tokio::test]
    async fn same_unit_id_on_different_ports_has_independent_memory() {
        let mut registry = TcpSlaveRegistry::default();
        let first = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "A", 11).await;
        let second = add(&mut registry, "127.0.0.1:0".parse().unwrap(), 1, "B", 22).await;
        assert_ne!(first, second);
        let mut a = tcp::connect_slave(first, Slave(1)).await.unwrap();
        let mut b = tcp::connect_slave(second, Slave(1)).await.unwrap();
        assert_eq!(read(&mut a, 1).await.unwrap(), vec![11]);
        assert_eq!(read(&mut b, 1).await.unwrap(), vec![22]);
        registry.unregister("A").await.unwrap();
        assert_eq!(read(&mut b, 1).await.unwrap(), vec![22]);
    }

    #[tokio::test]
    async fn failed_bind_leaves_no_binding_and_can_be_retried() {
        let mut registry = TcpSlaveRegistry::default();
        let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = occupied.local_addr().unwrap();
        assert!(registry
            .register(address, 1, "A".into(), "A".into(), SessionMemory::new())
            .await
            .is_err());
        assert!(registry.bindings.is_empty());
        assert!(registry.listeners.is_empty());
        drop(occupied);
        add(&mut registry, address, 1, "A", 11).await;
    }

    #[tokio::test]
    async fn concurrent_registration_supports_two_eight_and_sixteen_slaves() {
        for count in [2u8, 8, 16] {
            let socket = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = socket.local_addr().unwrap();
            drop(socket);
            let registry = Arc::new(Mutex::new(TcpSlaveRegistry::default()));
            let mut tasks = tokio::task::JoinSet::new();
            for id in 1..=count {
                let registry = registry.clone();
                tasks.spawn(async move {
                    add(
                        &mut *registry.lock().await,
                        address,
                        id,
                        &format!("Slave {id}"),
                        u16::from(id),
                    )
                    .await;
                });
            }
            while let Some(result) = tasks.join_next().await {
                result.unwrap();
            }
            assert_eq!(registry.lock().await.listeners.len(), 1);
            let mut client = tcp::connect_slave(address, Slave(1)).await.unwrap();
            for id in 1..=count {
                assert_eq!(read(&mut client, id).await.unwrap(), vec![u16::from(id)]);
            }
            for id in 1..=count {
                registry
                    .lock()
                    .await
                    .unregister(&format!("Slave {id}"))
                    .await
                    .unwrap();
            }
            assert!(registry.lock().await.listeners.is_empty());
            TcpListener::bind(address).await.unwrap();
        }
    }
}
