//! T00 tracer bullet hub — THROWAWAY (deleted at T18).
//!
//! One in-memory scoreboard: two scores and one countdown clock, shared by every connected
//! Board and Remote over `/ws` (JSON). It exists to answer transport questions only:
//! does the phone reach the laptop, how long does a tap take, does the clock stay in step
//! on the TV, do taps survive a Wi-Fi drop. It has no domain model on purpose.
//!
//! Shapes borrowed from the real specs so the lessons transfer:
//! - clock = duration + banked + started_at (D-012); clients never count down on their own;
//! - every client message carries an id; the hub acks it and ignores repeats (S11 §4);
//! - undo is per device and is itself a message, not a delete (D-013 in miniature).

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

pub const DEFAULT_PORT: u16 = 7878;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------------------

/// D-012 shape: the clock is data (duration, time banked before the last start, start
/// instant), never a decrementing counter.
#[derive(Debug, Clone, Default)]
pub struct Clock {
    pub duration_ms: u64,
    pub banked_ms: u64,
    pub started_at_ms: Option<u64>,
}

impl Clock {
    pub fn elapsed_ms(&self, now: u64) -> u64 {
        self.banked_ms + self.started_at_ms.map_or(0, |s| now.saturating_sub(s))
    }
    /// Negative once the block runs over (the Board shows overtime in red).
    pub fn remaining_ms(&self, now: u64) -> i64 {
        self.duration_ms as i64 - self.elapsed_ms(now) as i64
    }
    pub fn running(&self) -> bool {
        self.started_at_ms.is_some()
    }
}

#[derive(Debug, Clone)]
struct TapRecord {
    device: String,
    team: usize,
    pts: i32,
    undone: bool,
}

#[derive(Debug)]
pub struct Scoreboard {
    pub score: [i32; 2],
    pub clock: Clock,
    pub seq: u64,
    taps: Vec<TapRecord>,
    seen: HashSet<String>,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            score: [0, 0],
            clock: Clock {
                duration_ms: 8 * 60_000,
                ..Clock::default()
            },
            seq: 0,
            taps: Vec::new(),
            seen: HashSet::new(),
        }
    }
}

/// Messages from Board / Remote.
#[derive(Debug, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum In {
    Hello {
        role: String,
        device: String,
    },
    Tap {
        id: String,
        device: String,
        team: usize,
        pts: i32,
    },
    Undo {
        id: String,
        device: String,
    },
    Clock {
        id: String,
        cmd: String,
        #[serde(default)]
        secs: Option<u64>,
    },
    Ping {
        c: f64,
    },
}

#[derive(Debug, Serialize)]
pub struct ClockView {
    pub running: bool,
    pub remaining_ms: i64,
    pub duration_ms: u64,
}

/// Messages to Board / Remote.
#[derive(Debug, Serialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Out<'a> {
    State {
        seq: u64,
        score: [i32; 2],
        clock: ClockView,
        /// Hub time when this snapshot was taken; clients add their clock offset to it.
        server_ms: u64,
        clients: usize,
    },
    Ack {
        id: &'a str,
        dup: bool,
    },
    Pong {
        c: f64,
        s: u64,
    },
    Error {
        id: Option<&'a str>,
        msg: String,
    },
}

/// The outcome of applying one message.
#[derive(Debug, PartialEq, Eq)]
pub enum Applied {
    /// New and applied; state changed.
    Changed,
    /// Seen before; acked again, state untouched (idempotency).
    Duplicate,
    /// Valid but nothing to do (e.g. undo with no taps).
    NoOp,
}

impl Scoreboard {
    /// Apply one id-carrying message. Pure apart from `now`, so it's unit-testable.
    pub fn apply(&mut self, msg: &In, now: u64) -> Result<Applied, String> {
        let id = match msg {
            In::Tap { id, .. } | In::Undo { id, .. } | In::Clock { id, .. } => id,
            _ => return Ok(Applied::NoOp),
        };
        if self.seen.contains(id) {
            return Ok(Applied::Duplicate);
        }
        let changed = match msg {
            In::Tap {
                device, team, pts, ..
            } => {
                if *team > 1 || !(-3..=3).contains(pts) {
                    return Err(format!("bad tap: team {team}, pts {pts}"));
                }
                self.score[*team] += pts;
                self.taps.push(TapRecord {
                    device: device.clone(),
                    team: *team,
                    pts: *pts,
                    undone: false,
                });
                true
            }
            In::Undo { device, .. } => {
                match self
                    .taps
                    .iter_mut()
                    .rev()
                    .find(|t| !t.undone && &t.device == device)
                {
                    Some(t) => {
                        t.undone = true;
                        self.score[t.team] -= t.pts;
                        true
                    }
                    None => false,
                }
            }
            In::Clock { cmd, secs, .. } => {
                let c = &mut self.clock;
                match cmd.as_str() {
                    "start" if !c.running() => {
                        c.started_at_ms = Some(now);
                        true
                    }
                    "pause" if c.running() => {
                        c.banked_ms = c.elapsed_ms(now);
                        c.started_at_ms = None;
                        true
                    }
                    "reset" => {
                        c.banked_ms = 0;
                        c.started_at_ms = None;
                        true
                    }
                    "set" => {
                        let s = secs.ok_or("set needs secs")?;
                        *c = Clock {
                            duration_ms: s * 1000,
                            ..Clock::default()
                        };
                        true
                    }
                    "start" | "pause" => false,
                    other => return Err(format!("unknown clock cmd {other}")),
                }
            }
            _ => unreachable!(),
        };
        self.seen.insert(id.clone());
        if changed {
            self.seq += 1;
            Ok(Applied::Changed)
        } else {
            Ok(Applied::NoOp)
        }
    }
}

// ---------------------------------------------------------------------------------------
// Hub (shared state + fan-out)
// ---------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Hub {
    pub board: Arc<Mutex<Scoreboard>>,
    clients: Arc<AtomicUsize>,
    tx: broadcast::Sender<String>,
    pub port: u16,
}

impl Hub {
    pub fn new(port: u16) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            board: Arc::new(Mutex::new(Scoreboard::default())),
            clients: Arc::new(AtomicUsize::new(0)),
            tx,
            port,
        }
    }

    pub fn clients(&self) -> usize {
        self.clients.load(Ordering::Relaxed)
    }

    pub fn snapshot_json(&self) -> String {
        let now = now_ms();
        let b = self.board.lock().unwrap();
        serde_json::to_string(&Out::State {
            seq: b.seq,
            score: b.score,
            clock: ClockView {
                running: b.clock.running(),
                remaining_ms: b.clock.remaining_ms(now),
                duration_ms: b.clock.duration_ms,
            },
            server_ms: now,
            clients: self.clients(),
        })
        .unwrap()
    }

    fn broadcast_state(&self) {
        let _ = self.tx.send(self.snapshot_json());
    }

    /// Every LAN address the phone could use, best guess first.
    pub fn urls(&self) -> Vec<String> {
        let mut ips: Vec<String> = Vec::new();
        if let Ok(ip) = local_ip_address::local_ip() {
            ips.push(ip.to_string());
        }
        if let Ok(list) = local_ip_address::list_afinet_netifas() {
            for (_, ip) in list {
                if ip.is_ipv4() && !ip.is_loopback() && !ips.contains(&ip.to_string()) {
                    ips.push(ip.to_string());
                }
            }
        }
        ips.into_iter()
            .map(|ip| format!("http://{ip}:{}", self.port))
            .collect()
    }
}

// ---------------------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------------------

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

fn asset(path: &str) -> Response {
    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [
                    (header::CONTENT_TYPE, mime.as_ref().to_string()),
                    (header::CACHE_CONTROL, "no-cache".to_string()),
                ],
                file.data.into_owned(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, format!("not found: {path}")).into_response(),
    }
}

pub fn router(hub: Hub) -> Router {
    Router::new()
        .route("/", get(|| async { asset("index.html") }))
        .route("/board", get(|| async { asset("board.html") }))
        .route("/remote", get(|| async { asset("remote.html") }))
        .route("/wasm", get(|| async { asset("wasm/index.html") }))
        .route("/wasm/", get(|| async { asset("wasm/index.html") }))
        .route("/info", get(info))
        .route("/qr.svg", get(qr))
        .route("/ws", get(ws_upgrade))
        .route(
            "/{*path}",
            get(|axum::extract::Path(p): axum::extract::Path<String>| async move { asset(&p) }),
        )
        .with_state(hub)
}

#[derive(Serialize)]
struct Info {
    urls: Vec<String>,
    port: u16,
    clients: usize,
    score: [i32; 2],
    wasm_built: bool,
}

async fn info(State(hub): State<Hub>) -> Json<Info> {
    let score = hub.board.lock().unwrap().score;
    Json(Info {
        urls: hub.urls(),
        port: hub.port,
        clients: hub.clients(),
        score,
        wasm_built: Assets::iter().any(|p| p.ends_with(".wasm")),
    })
}

#[derive(Deserialize)]
struct QrQuery {
    text: String,
}

pub fn qr_svg(text: &str) -> String {
    match qrcode::QrCode::new(text.as_bytes()) {
        Ok(code) => code
            .render::<qrcode::render::svg::Color>()
            .min_dimensions(220, 220)
            .quiet_zone(true)
            .build(),
        Err(e) => format!("<svg xmlns='http://www.w3.org/2000/svg'><text y='20'>{e}</text></svg>"),
    }
}

async fn qr(Query(q): Query<QrQuery>) -> Response {
    ([(header::CONTENT_TYPE, "image/svg+xml")], qr_svg(&q.text)).into_response()
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(hub): State<Hub>) -> Response {
    ws.on_upgrade(move |socket| session(socket, hub))
}

async fn session(socket: WebSocket, hub: Hub) {
    hub.clients.fetch_add(1, Ordering::Relaxed);
    let mut rx = hub.tx.subscribe();
    let (mut tx, mut inbound) = socket.split();
    hub.broadcast_state();
    if tx
        .send(Message::Text(hub.snapshot_json().into()))
        .await
        .is_err()
    {
        hub.clients.fetch_sub(1, Ordering::Relaxed);
        return;
    }

    loop {
        tokio::select! {
            msg = inbound.next() => {
                let Some(Ok(msg)) = msg else { break };
                let Message::Text(text) = msg else { continue };
                let replies = handle(&hub, &text);
                let mut failed = false;
                for r in replies {
                    if tx.send(Message::Text(r.into())).await.is_err() { failed = true; break; }
                }
                if failed { break; }
            }
            out = rx.recv() => {
                match out {
                    Ok(s) => if tx.send(Message::Text(s.into())).await.is_err() { break },
                    // fell behind: send a fresh snapshot instead of the missed ones
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        if tx.send(Message::Text(hub.snapshot_json().into())).await.is_err() { break }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
    hub.clients.fetch_sub(1, Ordering::Relaxed);
    hub.broadcast_state();
}

/// Handle one inbound text frame; returns the direct replies (ack/pong/error).
/// State changes are broadcast to everyone, including the sender.
fn handle(hub: &Hub, text: &str) -> Vec<String> {
    let msg: In = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            return vec![serde_json::to_string(&Out::Error {
                id: None,
                msg: e.to_string(),
            })
            .unwrap()]
        }
    };
    match &msg {
        In::Ping { c } => vec![serde_json::to_string(&Out::Pong { c: *c, s: now_ms() }).unwrap()],
        In::Hello { role, device } => {
            println!("[{}] hello {role} {device}", now_ms());
            vec![]
        }
        In::Tap { id, .. } | In::Undo { id, .. } | In::Clock { id, .. } => {
            let result = hub.board.lock().unwrap().apply(&msg, now_ms());
            match result {
                Ok(applied) => {
                    if applied == Applied::Changed {
                        hub.broadcast_state();
                    }
                    vec![serde_json::to_string(&Out::Ack {
                        id,
                        dup: applied == Applied::Duplicate,
                    })
                    .unwrap()]
                }
                Err(e) => vec![serde_json::to_string(&Out::Error {
                    id: Some(id),
                    msg: e,
                })
                .unwrap()],
            }
        }
    }
}

/// Bind `0.0.0.0:port` and serve until the process exits.
pub async fn serve(hub: Hub) -> std::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], hub.port));
    let listener = TcpListener::bind(addr).await?;
    serve_on(listener, hub).await
}

pub async fn serve_on(listener: TcpListener, hub: Hub) -> std::io::Result<()> {
    axum::serve(listener, router(hub)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tap(id: &str, dev: &str, team: usize, pts: i32) -> In {
        In::Tap {
            id: id.into(),
            device: dev.into(),
            team,
            pts,
        }
    }

    #[test]
    fn repeated_ids_are_applied_once() {
        let mut b = Scoreboard::default();
        assert_eq!(b.apply(&tap("a1", "a", 0, 2), 0), Ok(Applied::Changed));
        assert_eq!(b.apply(&tap("a1", "a", 0, 2), 0), Ok(Applied::Duplicate));
        assert_eq!(b.score, [2, 0]);
    }

    #[test]
    fn undo_is_per_device() {
        let mut b = Scoreboard::default();
        b.apply(&tap("a1", "a", 0, 3), 0).unwrap();
        b.apply(&tap("b1", "b", 1, 1), 0).unwrap();
        let undo = In::Undo {
            id: "a2".into(),
            device: "a".into(),
        };
        b.apply(&undo, 0).unwrap();
        assert_eq!(
            b.score,
            [0, 1],
            "device a's undo leaves device b's tap alone"
        );
    }

    #[test]
    fn clock_is_start_banked_duration() {
        let mut b = Scoreboard::default();
        let c = |id: &str, cmd: &str| In::Clock {
            id: id.into(),
            cmd: cmd.into(),
            secs: Some(60),
        };
        b.apply(&c("1", "set"), 0).unwrap();
        b.apply(&c("2", "start"), 1_000).unwrap();
        b.apply(&c("3", "pause"), 11_000).unwrap();
        assert_eq!(
            b.clock.remaining_ms(99_000),
            50_000,
            "paused clock doesn't move"
        );
        b.apply(&c("4", "start"), 100_000).unwrap();
        assert_eq!(
            b.clock.remaining_ms(160_000),
            -10_000,
            "overtime is negative"
        );
    }

    #[test]
    fn bad_taps_are_rejected() {
        let mut b = Scoreboard::default();
        assert!(b.apply(&tap("x", "a", 2, 1), 0).is_err());
        assert!(b.apply(&tap("y", "a", 0, 9), 0).is_err());
    }
}
