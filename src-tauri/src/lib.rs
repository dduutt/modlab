pub mod session;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::sync::{mpsc, Mutex};
use session::{CommandEnvelope, SessionActor, SessionCommand, SessionEvent};
pub use session::TcpClientInfo;

pub struct SessionHandle {
    pub cmd_tx: mpsc::Sender<CommandEnvelope>,
    pub clients: Arc<std::sync::RwLock<HashMap<usize, TcpClientInfo>>>,
}

#[derive(Default)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, SessionHandle>>>,
}

#[tauri::command]
async fn spawn_session_native(
    session_id: String,
    on_event: Channel<SessionEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    if let Some(old) = sessions.remove(&session_id) {
        drop(old); // Dropping cmd_tx causes old actor to exit loop and terminate
    }

    let (cmd_tx, cmd_rx) = mpsc::channel(32);
    let (evt_tx, mut evt_rx) = mpsc::channel(32);
    let clients = Arc::new(std::sync::RwLock::new(HashMap::new()));

    // 将 Actor 发出的内部事件转发到 Tauri IPC 专属通道
    tokio::spawn(async move {
        while let Some(evt) = evt_rx.recv().await {
            if on_event.send(evt).is_err() {
                break;
            }
        }
    });

    // 启动完全自治的 Actor 任务
    let actor = SessionActor::new(session_id.clone(), cmd_rx, evt_tx, clients.clone());
    tokio::spawn(actor.run());

    // 登记路由表
    sessions.insert(session_id, SessionHandle { cmd_tx, clients });
    Ok(())
}

#[tauri::command]
async fn dispatch_session_command(
    session_id: String,
    command: SessionCommand,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let sender = {
        let map = state.sessions.lock().await;
        map.get(&session_id).map(|h| h.cmd_tx.clone())
    };

    if let Some(tx) = sender {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        tx.send(CommandEnvelope {
            cmd: command,
            resp: Some(resp_tx),
        })
        .await
        .map_err(|e| e.to_string())?;

        match resp_rx.await {
            Ok(res) => res,
            Err(_) => Err("Session terminated before responding".into()),
        }
    } else {
        Err(format!("Session {} not found", session_id))
    }
}

#[tauri::command]
async fn kill_session_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _ = state.sessions.lock().await.remove(&session_id);
    Ok(())
}

#[tauri::command]
fn is_portable_installation() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            if dir.join("uninstall.exe").exists() {
                return false;
            }
        }
    }
    true
}

#[tauri::command]
async fn list_serial_ports_native() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(|| {
        let mut ports: Vec<String> = tokio_serial::available_ports()
            .map_err(|err| format!("Failed to enumerate serial ports: {err}"))?
            .into_iter()
            .map(|port| port.port_name)
            .filter(|name| !name.is_empty())
            .collect();
        ports.sort();
        ports.dedup();
        Ok(ports)
    })
    .await
    .map_err(|err| format!("Serial port discovery failed: {err}"))?
}

#[tauri::command]
async fn list_tcp_clients_native(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TcpClientInfo>, String> {
    let clients_opt = {
        let map = state.sessions.lock().await;
        map.get(&session_id).map(|h| h.clients.clone())
    };

    if let Some(clients_arc) = clients_opt {
        let guard = clients_arc.read().map_err(|e| e.to_string())?;
        let mut list: Vec<TcpClientInfo> = guard.values().cloned().collect();
        list.sort_by_key(|c| c.id);
        Ok(list)
    } else {
        Ok(vec![])
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            spawn_session_native,
            dispatch_session_command,
            kill_session_native,
            is_portable_installation,
            list_serial_ports_native,
            list_tcp_clients_native
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
