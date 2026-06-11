//! Protocol client side, used by the CLI against both daemon tiers and by
//! the supervisor against lab daemons.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{Mutex, mpsc, oneshot};

use super::{Event, Message, ProtoError};

struct Pending {
    resp: oneshot::Sender<Result<Value, String>>,
    chunks: Option<mpsc::Sender<String>>,
}

struct Inner {
    write: Mutex<tokio::net::unix::OwnedWriteHalf>,
    pending: Mutex<HashMap<u64, Pending>>,
    events: Mutex<Option<mpsc::Sender<Event>>>,
    next_id: AtomicU64,
    reader: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// Cloneable async client for one daemon socket.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

impl Client {
    pub async fn connect(path: &Path) -> Result<Client, ProtoError> {
        let stream = UnixStream::connect(path).await?;
        let (read_half, write_half) = stream.into_split();
        let inner = Arc::new(Inner {
            write: Mutex::new(write_half),
            pending: Mutex::new(HashMap::new()),
            events: Mutex::new(None),
            next_id: AtomicU64::new(1),
            reader: Mutex::new(None),
        });
        let reader_inner = inner.clone();
        let handle = tokio::spawn(async move {
            let mut lines = BufReader::new(read_half).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(msg) = serde_json::from_str::<Message>(&line) else { continue };
                match msg {
                    Message::Resp { id, ok, err } => {
                        if let Some(p) = reader_inner.pending.lock().await.remove(&id) {
                            let result = match (ok, err) {
                                (_, Some(e)) => Err(e),
                                (Some(v), None) => Ok(v),
                                (None, None) => Ok(Value::Null),
                            };
                            let _ = p.resp.send(result);
                        }
                    }
                    Message::Stream { id, chunk } => {
                        let pending = reader_inner.pending.lock().await;
                        if let Some(Pending { chunks: Some(tx), .. }) = pending.get(&id) {
                            let _ = tx.try_send(chunk);
                        }
                    }
                    Message::Event { data, .. } => {
                        let guard = reader_inner.events.lock().await;
                        if let Some(tx) = guard.as_ref()
                            && let Ok(ev) = serde_json::from_value::<Event>(data)
                        {
                            let _ = tx.try_send(ev);
                        }
                    }
                    Message::Req { .. } => {}
                }
            }
            // Connection died: fail everything pending.
            let mut pending = reader_inner.pending.lock().await;
            for (_, p) in pending.drain() {
                let _ = p.resp.send(Err("connection closed".into()));
            }
        });
        *inner.reader.lock().await = Some(handle);
        Ok(Client { inner })
    }

    async fn send_req(
        &self,
        cmd: &str,
        args: Value,
        chunks: Option<mpsc::Sender<String>>,
    ) -> Result<oneshot::Receiver<Result<Value, String>>, ProtoError> {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, Pending { resp: tx, chunks });
        let msg = Message::Req { id, cmd: cmd.to_string(), args };
        let mut line = serde_json::to_string(&msg)
            .map_err(|e| ProtoError::Protocol(e.to_string()))?;
        line.push('\n');
        let mut w = self.inner.write.lock().await;
        w.write_all(line.as_bytes()).await?;
        Ok(rx)
    }

    /// Plain request/response.
    pub async fn call(&self, cmd: &str, args: Value) -> Result<Value, ProtoError> {
        let rx = self.send_req(cmd, args, None).await?;
        match rx.await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(ProtoError::Remote(e)),
            Err(_) => Err(ProtoError::Closed),
        }
    }

    /// Request with streamed output: `on_chunk` receives incremental text
    /// (build logs, provision output) until the final response arrives.
    pub async fn call_streaming(
        &self,
        cmd: &str,
        args: Value,
        mut on_chunk: impl FnMut(String) + Send,
    ) -> Result<Value, ProtoError> {
        let (tx, mut rx) = mpsc::channel::<String>(256);
        let resp_rx = self.send_req(cmd, args, Some(tx)).await?;
        tokio::pin!(resp_rx);
        loop {
            tokio::select! {
                chunk = rx.recv() => {
                    if let Some(c) = chunk {
                        on_chunk(c);
                    }
                }
                resp = &mut resp_rx => {
                    // Drain any chunks that raced the response.
                    while let Ok(c) = rx.try_recv() {
                        on_chunk(c);
                    }
                    return match resp {
                        Ok(Ok(v)) => Ok(v),
                        Ok(Err(e)) => Err(ProtoError::Remote(e)),
                        Err(_) => Err(ProtoError::Closed),
                    };
                }
            }
        }
    }

    /// Subscribe to the daemon's event stream.
    pub async fn subscribe(&self) -> Result<mpsc::Receiver<Event>, ProtoError> {
        let (tx, rx) = mpsc::channel(256);
        *self.inner.events.lock().await = Some(tx);
        self.call("subscribe", Value::Null).await?;
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::super::server::{Handler, Server, Streamer};
    use super::*;

    struct EchoHandler;

    #[async_trait::async_trait]
    impl Handler for EchoHandler {
        async fn handle(&self, cmd: &str, args: Value, stream: &Streamer) -> Result<Value, String> {
            match cmd {
                "echo" => Ok(args),
                "fail" => Err("nope".into()),
                "build" => {
                    for i in 0..3 {
                        stream.chunk(format!("step {i}")).await;
                    }
                    Ok(serde_json::json!({"built": true}))
                }
                "slow" => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    Ok(Value::String("slow-done".into()))
                }
                other => Err(format!("unknown command {other}")),
            }
        }
    }

    async fn start() -> (tempfile::TempDir, Server, Client) {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");
        let server = Server::bind(&sock, Arc::new(EchoHandler)).await.unwrap();
        let client = Client::connect(&sock).await.unwrap();
        (dir, server, client)
    }

    #[tokio::test]
    async fn request_response() {
        let (_dir, _server, client) = start().await;
        let v = client.call("echo", serde_json::json!({"x": 42})).await.unwrap();
        assert_eq!(v["x"], 42);
    }

    #[tokio::test]
    async fn remote_errors_propagate() {
        let (_dir, _server, client) = start().await;
        let err = client.call("fail", Value::Null).await.unwrap_err();
        assert!(matches!(err, ProtoError::Remote(ref m) if m == "nope"));
    }

    #[tokio::test]
    async fn streamed_output() {
        let (_dir, _server, client) = start().await;
        let mut chunks = Vec::new();
        let v = client
            .call_streaming("build", Value::Null, |c| chunks.push(c))
            .await
            .unwrap();
        assert_eq!(v["built"], true);
        assert_eq!(chunks, vec!["step 0", "step 1", "step 2"]);
    }

    #[tokio::test]
    async fn concurrent_requests_dont_block() {
        let (_dir, _server, client) = start().await;
        let slow = client.clone();
        let slow_task = tokio::spawn(async move { slow.call("slow", Value::Null).await });
        // The fast call completes while the slow one is still in flight.
        let fast = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            client.call("echo", serde_json::json!(1)),
        )
        .await
        .expect("fast call should not be blocked by slow one")
        .unwrap();
        assert_eq!(fast, serde_json::json!(1));
        let slow_result = slow_task.await.unwrap().unwrap();
        assert_eq!(slow_result, Value::String("slow-done".into()));
    }

    #[tokio::test]
    async fn events_flow_after_subscribe() {
        let (_dir, server, client) = start().await;
        let mut rx = client.subscribe().await.unwrap();
        server.emit(Event::new("vm.ready", "lab1", serde_json::json!({"vm": "dc01"})));
        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ev.event, "vm.ready");
        assert_eq!(ev.lab, "lab1");
        assert_eq!(ev.data["vm"], "dc01");
    }
}
