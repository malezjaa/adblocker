use crate::context::Context;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::ConnectInfo;
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WsEvent {
  Test,
}

pub(super) async fn ws_handler(
  ws: WebSocketUpgrade,
  AxumState(ctx): AxumState<Context>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(ctx, socket, addr))
}

async fn handle_socket(ctx: Context, mut socket: WebSocket, who: SocketAddr) {
  let mut rx = ctx.ws_tx().subscribe();

  while let Ok(event) = rx.recv().await {
    println!("{:?}", event);
  }
}
