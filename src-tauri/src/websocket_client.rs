use crate::commands::config::get_config;
use crate::models::config::Config;
use crate::models::status::Status;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

static CURRENT_STATUS: Lazy<Arc<Mutex<Status>>> =
  Lazy::new(|| Arc::new(Mutex::new(Status::default())));

static CURRENT_CONFIG: Lazy<Arc<Mutex<Config>>> =
  Lazy::new(|| Arc::new(Mutex::new(get_config().unwrap_or_default())));

#[derive(Serialize, Deserialize, Debug)]
enum MessageType {
  Command,
  Heartbeat,
  ControlInput,
  Status,
}

#[derive(Serialize, Deserialize, Debug)]
struct BasicMessage {
  message_type: MessageType,
  payload: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage<T> {
  message_type: MessageType,
  payload: T,
}

#[derive(Serialize, Deserialize)]
struct StatusPayload {
  water_detected: bool,
  pitch: f32,
  roll: f32,
  desired_pitch: f32,
  desired_roll: f32,
}

#[derive(Serialize, Deserialize)]
struct HeartbeatPayload {
  timestamp: Option<i64>,
}

pub fn create_control_channel() -> (Sender<[f32; 6]>, Receiver<[f32; 6]>) {
  channel(1)
}

pub async fn start_websocket_client(mut control_rx: Receiver<[f32; 6]>, app_handle: AppHandle) {
  println!("WebSocket client starting...");

  loop {
    {
      if let Ok(mut status) = CURRENT_STATUS.lock() {
        *status = Status::default();
        let _ = app_handle.emit("status_update", &*status);
      } else {
        eprintln!("Failed to lock status mutex for reset.");
        *CURRENT_STATUS.lock().unwrap() = Status::default();
      }
    }

    let config = {
      match CURRENT_CONFIG.lock() {
        Ok(guard) => guard.clone(),
        Err(e) => {
          eprintln!("Failed to lock config mutex for WS: {}. Using default.", e);
          Config::default()
        }
      }
    };

    let url = format!("ws://{}:{}", config.ip_address, config.web_socket_port);
    println!("Attempting to connect to WebSocket: {}", url);

    let last_heartbeat_time = Arc::new(Mutex::new(0i64));

    match connect_and_handle(
      &mut control_rx,
      &url,
      last_heartbeat_time.clone(),
      app_handle.clone(),
    )
    .await
    {
      Ok(_) => {
        println!("WebSocket connection closed gracefully.");
      }
      Err(e) => {
        let err_str = e.to_string();
        if !err_str.contains("Connection reset by peer")
          && !err_str.contains("Connection refused")
          && !err_str.contains("timed out")
          && !err_str.contains("No route to host")
          && !err_str.contains("Network is unreachable")
        {
          eprintln!("WebSocket error: {}", e);
        }
      }
    }

    println!("WebSocket disconnected. Retrying in 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;
  }
}

pub fn update_config(new_config: &Config) {
  if let Ok(mut config_guard) = CURRENT_CONFIG.lock() {
    *config_guard = new_config.clone();
  }
}

async fn connect_and_handle(
  control_rx: &mut Receiver<[f32; 6]>,
  url: &str,
  last_heartbeat_time: Arc<Mutex<i64>>,
  app_handle: AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
  let connect_timeout = Duration::from_secs(5);
  let (ws_stream, _) = timeout(connect_timeout, connect_async(url)).await??;
  println!("WebSocket connected successfully to {}", url);

  let connect_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
  {
    if let Ok(mut status) = CURRENT_STATUS.lock() {
      status.is_connected = true;
      status.water_detected = false;
      status.pitch = 0.0;
      status.roll = 0.0;
      status.desired_pitch = 0.0;
      status.desired_roll = 0.0;
      let _ = app_handle.emit("status_update", &*status);
    } else {
      eprintln!("Failed to lock status mutex on connect.");
      return Err("Failed to update status on connect".into());
    }
    if let Ok(mut lh_time) = last_heartbeat_time.lock() {
      *lh_time = connect_time;
    }
  }

  let (mut write, mut read) = ws_stream.split();

  let handshake = WebSocketMessage {
    message_type: MessageType::Command,
    payload: "connect",
  };
  let handshake_json = serde_json::to_string(&handshake)?;
  println!("Sending handshake: {}", handshake_json);
  write.send(Message::Text(handshake_json.into())).await?;

  let last_heartbeat_time_monitor = last_heartbeat_time.clone();
  let app_handle_monitor = app_handle.clone();
  let heartbeat_monitor = tokio::spawn(async move {
    let check_interval = Duration::from_secs(1);
    let timeout_duration: i64 = 5;

    loop {
      tokio::time::sleep(check_interval).await;
      let last_heartbeat = {
        match last_heartbeat_time_monitor.lock() {
          Ok(guard) => *guard,
          Err(_) => 0,
        }
      };
      let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(_) => 0,
      };

      if now > 0 && now.saturating_sub(last_heartbeat) > timeout_duration {
        println!("Heartbeat timeout detected ({}s)", timeout_duration);
        if let Ok(mut status) = CURRENT_STATUS.lock() {
          if status.is_connected {
            status.is_connected = false;
            let _ = app_handle_monitor.emit("status_update", &*status);
          }
        } else {
          eprintln!("Failed to lock status mutex on heartbeat timeout.");
        }
        break;
      }
    }
  });

  loop {
    tokio::select! {
        Some(message) = read.next() => {
            match message {
                Ok(msg) => {
                    if msg.is_text() {
                        let text = msg.into_text()?;
                        if let Ok(basic_msg) = serde_json::from_str::<BasicMessage>(&text) {
                            match basic_msg.message_type {
                                MessageType::Heartbeat => {
                                    if let Ok(heartbeat_msg) = serde_json::from_str::<WebSocketMessage<HeartbeatPayload>>(&text) {
                                        let received_time = heartbeat_msg.payload.timestamp.unwrap_or_else(|| {
                                            SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs() as i64
                                        });
                                        if let Ok(mut lh_time) = last_heartbeat_time.lock() {
                                            *lh_time = received_time;
                                        }
                                        if let Ok(mut status) = CURRENT_STATUS.lock() {
                                            if !status.is_connected {
                                                status.is_connected = true;
                                                let _ = app_handle.emit("status_update", &*status);
                                            }
                                        }

                                        let response = WebSocketMessage {
                                            message_type: MessageType::Heartbeat,
                                            payload: HeartbeatPayload { timestamp: None },
                                        };
                                        let response_json = serde_json::to_string(&response)?;
                                        if let Err(e) = write.send(Message::Text(response_json.into())).await {
                                            eprintln!("Failed to send heartbeat response: {}", e);
                                            break;
                                        }
                                    }
                                },
                                MessageType::Status => {
                                    if let Ok(status_msg) = serde_json::from_str::<WebSocketMessage<StatusPayload>>(&text) {
                                        if let Ok(mut status) = CURRENT_STATUS.lock() {
                                            status.water_detected = status_msg.payload.water_detected;
                                            status.pitch = status_msg.payload.pitch;
                                            status.roll = status_msg.payload.roll;
                                            status.desired_pitch = status_msg.payload.desired_pitch;
                                            status.desired_roll = status_msg.payload.desired_roll;
                                            let _ = app_handle.emit("status_update", &*status);
                                        } else {
                                            eprintln!("Failed to lock status mutex for status update.");
                                        }
                                    }
                                },
                                _ => println!("Received unhandled message type: {:?}", basic_msg.message_type),
                            }
                        } else {
                            println!("Failed to parse message as BasicMessage: {}", text);
                        }
                    } else if msg.is_close() {
                        println!("Received Close frame");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket read error: {}", e);
                    break;
                }
            }
        }

        Some(input) = control_rx.recv() => {
            let control_msg = WebSocketMessage {
                message_type: MessageType::ControlInput,
                payload: input,
            };
            let msg_json = serde_json::to_string(&control_msg)?;
            if let Err(e) = write.send(Message::Text(msg_json.into())).await {
                eprintln!("WebSocket write error: {}", e);
                break;
            }
        }
        else => {
            println!("Control channel closed, exiting WebSocket handler.");
            break;
        }
    }
  }

  heartbeat_monitor.abort();

  if let Ok(mut status) = CURRENT_STATUS.lock() {
    if status.is_connected {
      println!("Setting connection status to false in connect_and_handle exit.");
      status.is_connected = false;
      let _ = app_handle.emit("status_update", &*status);
    }
  } else {
    eprintln!("Failed to lock status mutex during disconnect handling.");
  }

  println!("Exiting connect_and_handle for {}", url);
  Ok(())
}
