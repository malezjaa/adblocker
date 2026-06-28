use crate::context::Context;
use crate::dashboard::auth::AuthGuard;
use crate::database::stats::{HourStat, Stats, TopDomain};
use axum::body::Bytes;
use axum::extract::ConnectInfo;
use axum::extract::State as AxumState;
use axum::extract::ws::close_code::NORMAL;
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tracing::{debug, error, info, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WsEvent {
  DNSRequest,
}

pub(super) async fn ws_handler(
  _guard: AuthGuard,
  ws: WebSocketUpgrade,
  AxumState(ctx): AxumState<Context>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(ctx, socket, addr))
}

async fn handle_socket(ctx: Context, mut socket: WebSocket, who: SocketAddr) {
  if socket.send(Message::Ping(Bytes::from_static(&[1, 2, 3]))).await.is_ok() {
    debug!("Pinged {who}...");
  } else {
    warn!("Could not send ping {who}!");
    return;
  }

  let mut rx = ctx.ws_tx().subscribe();
  let (mut sender, mut receiver) = socket.split();

  #[derive(serde::Serialize)]
  struct RealtimePayload {
    stats: Stats,
    top_blocked: Vec<TopDomain>,
    hours: Vec<HourStat>,
  }

  let mut send_task = tokio::spawn(async move {
    let mut count = 0;
    while let Ok(event) = rx.recv().await {
      count += 1;
      match event {
        WsEvent::DNSRequest => {
          let stats = ctx.db().stats(None, None).await?;
          let top = ctx.db().top_blocked(Some(10)).await?;
          let hours = ctx.db().stats_by_hour_today().await?;

          let payload = RealtimePayload { stats, top_blocked: top, hours };
          sender.send(Message::Text(serde_json::to_string(&payload)?.into())).await?;
        }
      }
    }

    info!("Sending close to {who}...");
    if let Err(e) = sender
      .send(Message::Close(Some(CloseFrame {
        code: NORMAL,
        reason: Utf8Bytes::from_static("Goodbye"),
      })))
      .await
    {
      warn!("Could not send Close due to {e}");
    }

    Ok::<_, anyhow::Error>(count)
  });

  let mut recv_task = tokio::spawn(async move {
    let mut count = 0;
    while let Some(Ok(msg)) = receiver.next().await {
      count += 1;
      if process_message(msg, who).is_break() {
        break;
      }
    }
    Ok::<_, anyhow::Error>(count)
  });

  tokio::select! {
    rv_a = (&mut send_task) => {
      match rv_a {
        Ok(Ok(count)) => debug!("{count} messages sent to {who}"),
        Ok(Err(e)) => error!("Send task failed: {e}"),
        Err(e) => error!("Send task panicked: {e:?}"),
      }
      recv_task.abort();
    },
    rv_b = (&mut recv_task) => {
      match rv_b {
        Ok(Ok(count)) => debug!("Received {count} messages from {who}"),
        Ok(Err(e)) => error!("Recv task failed: {e}"),
        Err(e) => error!("Recv task panicked: {e:?}"),
      }
      send_task.abort();
    }
  }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
  if let Message::Close(c) = msg {
    if let Some(cf) = c {
      info!("{who} sent close with code {} and reason `{}`", cf.code, cf.reason);
    }
    return ControlFlow::Break(());
  }
  ControlFlow::Continue(())
}
