//! Protocol server side: accept unix connections, dispatch requests to a
//! handler, fan out events to subscribed connections.

use std::path::Path;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, mpsc};

use super::{Event, Message};

/// Sink for incremental output of a long-running command. Dropping it is
/// fine — chunks are best-effort.
#[derive(Clone)]
pub struct Streamer {
    id: u64,
    tx: mpsc::Sender<Message>,
}

impl Streamer {
    pub async fn chunk(&self, text: impl Into<String>) {
        let _ = self
            .tx
            .send(Message::Stream {
                id: self.id,
                chunk: text.into(),
            })
            .await;
    }
}

/// Command handler implemented by the supervisor and lab daemons.
#[async_trait::async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, cmd: &str, args: Value, stream: &Streamer) -> Result<Value, String>;
}

/// A running protocol server bound to a unix socket.
pub struct Server {
    pub events: broadcast::Sender<Event>,
    handle: tokio::task::JoinHandle<()>,
}

impl Server {
    /// Bind `path` (parent dirs created, stale socket file replaced) and
    /// serve until dropped/aborted.
    pub async fn bind(path: &Path, handler: Arc<dyn Handler>) -> std::io::Result<Server> {
        let (events, _) = broadcast::channel::<Event>(1024);
        Self::bind_with_events(path, handler, events).await
    }

    /// [`bind`](Self::bind) with a caller-supplied event channel, so the
    /// daemon can emit events without holding the server.
    pub async fn bind_with_events(
        path: &Path,
        handler: Arc<dyn Handler>,
        events: broadcast::Sender<Event>,
    ) -> std::io::Result<Server> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        let events_accept = events.clone();
        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let handler = handler.clone();
                        let events = events_accept.clone();
                        tokio::spawn(async move {
                            if let Err(e) = serve_conn(stream, handler, events).await {
                                tracing::debug!("connection ended: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!("accept failed: {e}");
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
        Ok(Server { events, handle })
    }

    /// Emit an event to all subscribed connections.
    pub fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn serve_conn(
    stream: UnixStream,
    handler: Arc<dyn Handler>,
    events: broadcast::Sender<Event>,
) -> anyhow::Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    // All outbound traffic for this connection funnels through one channel so
    // responses, stream chunks, and events interleave without tearing.
    let (out_tx, mut out_rx) = mpsc::channel::<Message>(256);
    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            let mut line = match serde_json::to_string(&msg) {
                Ok(l) => l,
                Err(_) => continue,
            };
            line.push('\n');
            if write_half.write_all(line.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let mut event_pump: Option<tokio::task::JoinHandle<()>> = None;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let msg: Message = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                let _ = out_tx
                    .send(Message::Resp {
                        id: 0,
                        ok: None,
                        err: Some(format!("bad message: {e}")),
                    })
                    .await;
                continue;
            }
        };
        let Message::Req { id, cmd, args } = msg else {
            continue; // clients only send requests
        };

        // `subscribe` flips this connection into event mode: events flow
        // until the client disconnects. It still gets a normal Resp.
        if cmd == "subscribe" && event_pump.is_none() {
            let mut rx = events.subscribe();
            let tx = out_tx.clone();
            event_pump = Some(tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(ev) => {
                            let data = serde_json::to_value(&ev).unwrap_or(Value::Null);
                            if tx
                                .send(Message::Event {
                                    event: ev.event.clone(),
                                    data,
                                })
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }));
            let _ = out_tx
                .send(Message::Resp {
                    id,
                    ok: Some(Value::Bool(true)),
                    err: None,
                })
                .await;
            continue;
        }

        let streamer = Streamer {
            id,
            tx: out_tx.clone(),
        };
        let handler = handler.clone();
        let out = out_tx.clone();
        // Handle each request on its own task so a long build doesn't block
        // a status query on the same connection.
        tokio::spawn(async move {
            let resp = match handler.handle(&cmd, args, &streamer).await {
                Ok(v) => Message::Resp {
                    id,
                    ok: Some(v),
                    err: None,
                },
                Err(e) => Message::Resp {
                    id,
                    ok: None,
                    err: Some(e),
                },
            };
            let _ = out.send(resp).await;
        });
    }

    if let Some(p) = event_pump {
        p.abort();
    }
    writer.abort();
    Ok(())
}
