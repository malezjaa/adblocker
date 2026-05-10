use crate::state::State;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::ConnectInfo;
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::spawn;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WsEvent {
  Test
}

pub(super) async fn ws_handler(
  ws: WebSocketUpgrade,
  AxumState(state): AxumState<State>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(state, socket, addr))
}

async fn handle_socket(
  state: State,
  mut socket: WebSocket,
  who: SocketAddr,
) {
  let mut rx = state.ws_tx().subscribe();

  while let Ok(event) = rx.recv().await {
    println!("{:?}", event);
  }
}