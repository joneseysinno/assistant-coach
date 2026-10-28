//! T00 tracer hub binary.
//!
//!   tracer-hub              hub window (URLs + QR) with the server on its own thread
//!   tracer-hub --headless   server only, URLs printed to the console
//!   tracer-hub --port 8080  another port
//!
//! The windowed mode mirrors how T18 will run: Dioxus desktop owns the main thread; the hub
//! (tokio + axum) runs on a separate thread with its own runtime.

use tracer_hub::{serve, Hub, DEFAULT_PORT};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--headless") || !cfg!(feature = "desktop");
    let port = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let hub = Hub::new(port);
    println!("Tracer hub on port {port}");
    for url in hub.urls() {
        println!("  phone:  {url}/remote");
        println!("  TV:     {url}/board");
        println!("  wasm:   {url}/wasm");
    }
    println!("  laptop: http://localhost:{port}/   (QR codes)");

    if headless {
        run_server(hub);
    } else {
        #[cfg(feature = "desktop")]
        {
            let server_hub = hub.clone();
            std::thread::Builder::new()
                .name("hub".into())
                .spawn(move || run_server(server_hub))
                .expect("spawn hub thread");
            window::launch(hub);
        }
    }
}

fn run_server(hub: Hub) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    if let Err(e) = rt.block_on(serve(hub)) {
        eprintln!("hub server failed: {e}");
        eprintln!("Is another copy running, or is port 7878 taken? Try --port 7879.");
        std::process::exit(1);
    }
}

#[cfg(feature = "desktop")]
mod window {
    use std::sync::OnceLock;
    use std::time::Duration;

    use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
    use dioxus::prelude::*;
    use tracer_hub::{qr_svg, Hub};

    static HUB: OnceLock<Hub> = OnceLock::new();

    pub fn launch(hub: Hub) {
        let _ = HUB.set(hub);
        let window = WindowBuilder::new()
            .with_title("Assistant Coach — tracer hub (T00)")
            .with_inner_size(LogicalSize::new(760.0, 640.0));
        dioxus::LaunchBuilder::desktop()
            .with_cfg(Config::new().with_window(window).with_menu(None))
            .launch(App);
    }

    #[component]
    fn App() -> Element {
        let hub = HUB.get().expect("hub set before launch").clone();
        let urls = use_hook(|| hub.urls());
        let mut tick = use_signal(|| 0u64);
        use_future(move || async move {
            loop {
                tokio::time::sleep(Duration::from_millis(300)).await;
                tick += 1;
            }
        });
        let _ = tick();
        let score = hub.board.lock().unwrap().score;
        let clients = hub.clients();
        let first = urls
            .first()
            .cloned()
            .unwrap_or_else(|| format!("http://localhost:{}", hub.port));
        let remote_qr = qr_svg(&format!("{first}/remote"));
        let board_url = format!("http://localhost:{}/board", hub.port);

        rsx! {
            style { {CSS} }
            div { class: "wrap",
                h1 { "Tracer hub" }
                p { class: "muted", "Throwaway spike (T00). Phone and laptop must be on your router's Wi-Fi." }
                div { class: "row",
                    div { class: "qr", dangerous_inner_html: "{remote_qr}" }
                    div {
                        h2 { "Phone → scan, or type:" }
                        for u in urls.iter() {
                            p { class: "url", "{u}/remote" }
                        }
                        h2 { "TV (this laptop, second screen):" }
                        p { class: "url", "{board_url}" }
                        p { class: "muted", "Open it in a browser on the TV display, press F for full screen, click once for sound." }
                    }
                }
                div { class: "stats",
                    span { "connected devices: {clients}" }
                    span { "score: {score[0]} – {score[1]}" }
                }
            }
        }
    }

    const CSS: &str = r#"
        body { margin: 0; background: #111318; color: #eceef2; font-family: system-ui, sans-serif; }
        .wrap { padding: 20px 24px; }
        h1 { margin: 0 0 4px; font-size: 22px; }
        h2 { margin: 12px 0 4px; font-size: 15px; color: #b5b7c0; }
        .muted { color: #8b8d98; font-size: 13px; margin: 2px 0; }
        .row { display: flex; gap: 20px; align-items: flex-start; margin-top: 12px; }
        .qr { background: #fff; padding: 8px; border-radius: 8px; line-height: 0; }
        .url { font: 600 16px ui-monospace, Consolas, monospace; margin: 2px 0; user-select: text; }
        .stats { display: flex; gap: 24px; margin-top: 18px; font: 14px ui-monospace, Consolas, monospace; color: #b5b7c0; }
    "#;
}
