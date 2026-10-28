//! T00 integration: two WebSocket clients score at the same time, one of them re-sends
//! every tap (as a Remote does after a reconnect). The final score must count each tap
//! exactly once. This is the shape of T18's acceptance test.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracer_hub::{serve_on, Hub};

async fn start() -> (String, Hub) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let hub = Hub::new(port);
    tokio::spawn(serve_on(listener, hub.clone()));
    (format!("ws://127.0.0.1:{port}/ws"), hub)
}

/// Send `n` taps from `device`, each `repeat` times, and wait for every ack.
async fn scorer(url: String, device: &'static str, team: usize, n: usize, repeat: usize) -> usize {
    let (mut ws, _) = connect_async(&url).await.unwrap();
    for i in 0..n {
        let msg = format!(
            r#"{{"t":"tap","id":"{device}-{i}","device":"{device}","team":{team},"pts":2}}"#
        );
        for _ in 0..repeat {
            ws.send(Message::Text(msg.clone().into())).await.unwrap();
        }
    }
    let mut acks = 0;
    while acks < n * repeat {
        match tokio::time::timeout(Duration::from_secs(5), ws.next()).await {
            Ok(Some(Ok(Message::Text(t)))) if t.contains(r#""t":"ack""#) => acks += 1,
            Ok(Some(Ok(_))) => {}
            other => panic!("{device}: stream ended early: {other:?}"),
        }
    }
    acks
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_clients_scoring_at_once_count_each_tap_once() {
    let (url, hub) = start().await;
    let a = tokio::spawn(scorer(url.clone(), "phone", 0, 200, 1));
    let b = tokio::spawn(scorer(url.clone(), "tablet", 1, 150, 3)); // every tap re-sent twice
    assert_eq!(a.await.unwrap(), 200);
    assert_eq!(b.await.unwrap(), 450);
    assert_eq!(hub.board.lock().unwrap().score, [400, 300]);
}

#[tokio::test]
async fn http_routes_serve_the_pages() {
    let (url, hub) = start().await;
    let base = url.replace("ws://", "").replace("/ws", "");
    for path in [
        "/",
        "/board",
        "/remote",
        "/wasm",
        "/info",
        "/qr.svg?text=hi",
    ] {
        let mut stream = tokio::net::TcpStream::connect(&base).await.unwrap();
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let req = format!("GET {path} HTTP/1.1\r\nHost: {base}\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).await.unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "{path}: {}",
            resp.lines().next().unwrap_or("")
        );
    }
    let _ = hub;
}
