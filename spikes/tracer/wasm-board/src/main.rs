//! T00 wasm proof — the smallest Dioxus web page that exercises the whole pipeline T18/T19
//! depend on: `dx build --platform web` → copy into the hub's assets → embedded in the
//! hub exe → served at /wasm → opens the hub's /ws from inside wasm → renders live state.
//!
//! If this page shows the live score on the phone, the pipeline works.

use std::time::Duration;

use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut score = use_signal(|| [0i64; 2]);
    let mut clients = use_signal(|| 0i64);
    let mut status = use_signal(|| "connecting…".to_string());
    let mut frames = use_signal(|| 0u64);

    use_future(move || async move {
        let host = web_sys::window()
            .and_then(|w| w.location().host().ok())
            .unwrap_or_else(|| "localhost:7878".into());
        let url = format!("ws://{host}/ws");
        loop {
            match WebSocket::open(&url) {
                Ok(ws) => {
                    status.set("connected".into());
                    let (mut tx, mut rx) = ws.split();
                    let hello = r#"{"t":"hello","role":"wasm","device":"wasm"}"#;
                    let _ = tx.send(Message::Text(hello.into())).await;
                    while let Some(Ok(Message::Text(text))) = rx.next().await {
                        frames += 1;
                        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
                            continue;
                        };
                        if v["t"] == "state" {
                            score.set([
                                v["score"][0].as_i64().unwrap_or(0),
                                v["score"][1].as_i64().unwrap_or(0),
                            ]);
                            clients.set(v["clients"].as_i64().unwrap_or(0));
                        }
                    }
                    status.set("reconnecting…".into());
                }
                Err(e) => status.set(format!("cannot open {url}: {e}")),
            }
            gloo_timers::future::sleep(Duration::from_millis(1000)).await;
        }
    });

    rsx! {
        div { style: "background:#111318;color:#eceef2;font-family:system-ui,sans-serif;min-height:100vh;padding:20px;box-sizing:border-box",
            h2 { style: "margin:0 0 6px", "Dioxus wasm → hub /ws" }
            p { style: "color:#8b8d98;margin:0 0 20px", "{status} · {clients} devices · {frames} frames" }
            div { style: "display:flex;gap:40px;font-size:96px;font-weight:900;font-variant-numeric:tabular-nums",
                span { style: "color:#e5484d", "{score.read()[0]}" }
                span { style: "color:#3e63dd", "{score.read()[1]}" }
            }
            p { style: "color:#8b8d98", "If this page tracks the score as you tap on /remote, the wasm build-and-embed pipeline works." }
        }
    }
}
