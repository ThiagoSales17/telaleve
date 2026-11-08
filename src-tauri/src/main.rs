#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod capture;
mod media;
mod model;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use model::{Access, Quality, Source};
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::Manager;
use tokio::sync::broadcast;
type Shared = Arc<Mutex<App>>;
struct App {
    access: Option<Access>,
    source: Option<Source>,
    quality: Quality,
    media: Option<media::Media>,
    offer: Option<String>,
    ice: Vec<(u32, String)>,
    approved: bool,
    status: String,
    tx: broadcast::Sender<String>,
    url: String,
    bitrate: u32,
    server_ready: bool,
}
impl App {
    fn stop(&mut self) {
        self.media = None;
        self.access = None;
        self.approved = false;
        self.offer = None;
        self.ice.clear();
        self.status = "encerrado".into();
        let _ = self.tx.send(json!({"type":"ended"}).to_string());
    }
    fn start_media(&mut self) -> Result<(), String> {
        let offer = self.offer.as_ref().ok_or("Aguardando oferta do celular")?;
        self.media = None;
        let m = media::Media::new(
            self.source.as_ref().ok_or("Escolha uma origem")?,
            self.quality,
            offer,
            self.tx.clone(),
        )?;
        for (i, c) in self.ice.drain(..) {
            m.ice(i, &c)
        }
        self.media = Some(m);
        self.status = "conectando".into();
        Ok(())
    }
}
#[tauri::command]
fn list_sources() -> Result<Vec<Source>, String> {
    capture::sources()
}
#[tauri::command]
fn networks() -> Vec<Value> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| match i.ip() {
            std::net::IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => {
                Some(json!({"name":i.name,"ip":ip.to_string()}))
            }
            _ => None,
        })
        .collect()
}
#[tauri::command]
fn begin(
    state: tauri::State<Shared>,
    source: String,
    ip: String,
    quality: Quality,
) -> Result<Value, String> {
    let ip: std::net::Ipv4Addr = ip.parse().map_err(|_| "IP inválido")?;
    if !networks().iter().any(|n| n["ip"] == ip.to_string()) {
        return Err("Selecione uma interface local".into());
    }
    let s = capture::sources()?
        .into_iter()
        .find(|s| s.id == source)
        .ok_or("Origem indisponível")?;
    let mut a = state.lock().unwrap();
    if !a.server_ready {
        return Err("O servidor local não está disponível; confira a porta 9876".into());
    }
    a.stop();
    a.source = Some(s);
    a.quality = quality;
    a.bitrate = quality.settings().3;
    a.access = Some(Access::new());
    a.url = format!("http://{ip}:9876/#{}", a.access.as_ref().unwrap().token);
    a.status = "aguardando".into();
    let code = qrcode::QrCode::new(a.url.as_bytes())
        .map_err(|e| e.to_string())?
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .build();
    Ok(json!({"url":a.url,"qr":code}))
}
#[tauri::command]
fn approve(state: tauri::State<Shared>, allow: bool) -> Result<(), String> {
    let mut a = state.lock().unwrap();
    if !allow {
        a.stop();
        return Ok(());
    }
    if a.status != "solicitando_aprovacao" {
        return Err("Não há solicitação pendente".into());
    }
    a.approved = true;
    let resume = uuid::Uuid::new_v4().to_string();
    a.access.as_mut().ok_or("Sessão encerrada")?.resume = Some(resume.clone());
    let _ =
        a.tx.send(json!({"type":"approved","resume":resume}).to_string());
    if let Err(e) = a.start_media() {
        a.stop();
        return Err(e);
    }
    Ok(())
}
#[tauri::command]
fn end(state: tauri::State<Shared>) {
    state.lock().unwrap().stop()
}
#[tauri::command]
fn change_source(
    state: tauri::State<Shared>,
    source: String,
    quality: Quality,
) -> Result<(), String> {
    let s = capture::sources()?
        .into_iter()
        .find(|s| s.id == source)
        .ok_or("Origem indisponível")?;
    let mut a = state.lock().unwrap();
    if let Some(m) = a.media.as_mut() {
        m.switch(&s, quality)?;
        m.bitrate(quality.settings().3);
    }
    a.source = Some(s);
    a.quality = quality;
    a.bitrate = quality.settings().3;
    if a.approved && a.media.is_none() {
        let _ = a.tx.send(json!({"type":"restart"}).to_string());
    }
    Ok(())
}
#[tauri::command]
fn status(state: tauri::State<Shared>) -> Value {
    let a = state.lock().unwrap();
    json!({"state":a.status,"codec":a.media.as_ref().map(|m|&m.codec),"bitrate":a.bitrate,"source":a.source.as_ref().map(|s|&s.id)})
}
async fn upgrade(State(a): State<Shared>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.max_message_size(128 * 1024)
        .on_upgrade(move |s| socket(s, a))
}
async fn socket(mut socket: WebSocket, a: Shared) {
    let first = tokio::time::timeout(std::time::Duration::from_secs(10), socket.recv()).await;
    let Ok(Some(Ok(Message::Text(text)))) = first else {
        return;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return;
    };
    let (valid, approved, generation, mut rx, retry) = {
        let mut app = a.lock().unwrap();
        let retry = v["resume"].as_bool() == Some(true)
            && app
                .access
                .as_ref()
                .is_some_and(|x| x.connected && x.resume.as_deref() == v["token"].as_str());
        let valid = app.access.as_mut().is_some_and(|x| {
            x.claim(
                v["token"].as_str().unwrap_or(""),
                v["resume"].as_bool().unwrap_or(false),
                Instant::now(),
            )
        });
        (
            valid,
            app.approved,
            app.access.as_ref().map(|x| x.token.clone()),
            app.tx.subscribe(),
            retry,
        )
    };
    if !valid {
        let _ = socket
            .send(Message::Text(
                json!({"type":if retry{"retry"}else{"error"},"message":"Código expirado, já usado ou sessão ocupada"})
                    .to_string()
                    .into(),
            ))
            .await;
        return;
    }
    let _ = socket
        .send(Message::Text(
            json!({"type":"ready","approved":approved})
                .to_string()
                .into(),
        ))
        .await;
    let (mut sink, mut stream) = socket.split();
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(5));
    let mut seen = Instant::now();
    loop {
        tokio::select! {
        _=heartbeat.tick()=>{if seen.elapsed().as_secs()>15{break}if sink.send(Message::Ping(Vec::new().into())).await.is_err(){break}},
        event=rx.recv()=>{match event{Ok(text)=>{let ended=text.contains("\"ended\"");if sink.send(Message::Text(text.into())).await.is_err()||ended{break}},Err(_)=>break}},
        msg=stream.next()=>{let text=match msg{Some(Ok(Message::Text(text)))=>text,Some(Ok(Message::Pong(_)))=>{seen=Instant::now();continue},_=>break};seen=Instant::now();let Ok(v)=serde_json::from_str::<Value>(&text)else{continue};let mut app=a.lock().unwrap();if app.access.as_ref().map(|x|x.token.clone()) != generation {break}
        match v["type"].as_str().unwrap_or(""){
        "offer"=>{if let Some(sdp)=v["sdp"].as_str(){app.offer=Some(sdp.into());if app.approved{if let Err(e)=app.start_media(){let _=app.tx.send(json!({"type":"error","message":e}).to_string());}}else{app.status="solicitando_aprovacao".into();let _=app.tx.send(json!({"type":"waiting"}).to_string());}}},
        "ice"=>{if let(Some(c),Some(i))=(v["candidate"].as_str(),v["sdpMLineIndex"].as_u64()){if let Some(m)=app.media.as_ref(){m.ice(i as u32,c)}else if app.ice.len()<128{app.ice.push((i as u32,c.into()));}}},
        "connected"=>if app.approved{app.status="transmitindo".into()},
        "feedback"=>if app.approved{let loss=v["loss"].as_f64().unwrap_or(0.0);let jitter=v["jitter"].as_f64().unwrap_or(0.0);let max=app.quality.settings().4;let b=if loss>0.03||jitter>0.08{(app.bitrate as f64*0.8)as u32}else{app.bitrate.saturating_add(100_000)}.clamp(400_000,max);app.bitrate=b;if let Some(m)=app.media.as_mut(){m.bitrate(b)}},
        _=>{}
        }
        }
        }
    }
    let mut app = a.lock().unwrap();
    if app.access.as_ref().map(|x| x.token.clone()) != generation {
        return;
    }
    if let Some(access) = app.access.as_mut() {
        access.connected = false;
        access.disconnected = Some(Instant::now());
        app.media = None;
        app.offer = None;
        app.ice.clear();
        if app.approved {
            app.status = "reconectando".into()
        } else {
            app.stop();
        }
    }
}
fn main() {
    if let Err(e) = capture::init_xlib() {
        eprintln!("{e}");
        return;
    }
    if let Err(e) = gstreamer::init() {
        eprintln!("GStreamer indisponível: {e}");
        return;
    }
    let (tx, _) = broadcast::channel(128);
    let app: Shared = Arc::new(Mutex::new(App {
        access: None,
        source: None,
        quality: Quality::Balanced,
        media: None,
        offer: None,
        ice: Vec::new(),
        approved: false,
        status: "encerrado".into(),
        tx,
        url: String::new(),
        bitrate: 2_500_000,
        server_ready: false,
    }));
    let server = app.clone();
    let monitor = app.clone();
    tauri::Builder::default()
        .manage(app)
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                window.state::<Shared>().lock().unwrap().stop();
            }
        })
        .setup(move |_| {
            tauri::async_runtime::spawn(async move {
                let router = Router::new()
                    .route(
                        "/",
                        get(|| async { Html(include_str!("../../web/viewer.html")) }),
                    )
                    .route(
                        "/viewer.js",
                        get(|| async {
                            (
                                [("content-type", "text/javascript")],
                                include_str!("../../web/viewer.js"),
                            )
                        }),
                    )
                    .route(
                        "/style.css",
                        get(|| async {
                            (
                                [("content-type", "text/css")],
                                include_str!("../../web/style.css"),
                            )
                        }),
                    )
                    .route("/ws", get(upgrade))
                    .with_state(server.clone());
                match tokio::net::TcpListener::bind("0.0.0.0:9876").await {
                    Ok(listener) => {
                        server.lock().unwrap().server_ready = true;
                        if axum::serve(listener, router).await.is_err() {
                            server.lock().unwrap().status = "erro_servidor".into();
                        }
                    }
                    Err(_) => {
                        server.lock().unwrap().status = "erro_porta_9876".into();
                    }
                }
            });
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let sources = capture::sources();
                let mut a = monitor.lock().unwrap();
                let expired = a
                    .access
                    .as_ref()
                    .is_some_and(|x| x.disconnected.is_some_and(|t| t.elapsed().as_secs() >= 30));
                if expired {
                    a.stop();
                    continue;
                }
                if a.media.is_some() {
                    let lost = match (&sources, &a.source) {
                        (Ok(all), Some(s)) => !all.iter().any(|n| {
                            n.id == s.id
                                && n.width == s.width
                                && n.height == s.height
                                && n.x == s.x
                                && n.y == s.y
                        }),
                        _ => true,
                    };
                    let err = a.media.as_ref().and_then(|m| m.error());
                    if lost || err.is_some() {
                        a.media = None;
                        a.status = "origem_perdida".into();
                        let _ = a.tx.send(json!({"type":"source_lost"}).to_string());
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_sources,
            networks,
            begin,
            approve,
            end,
            change_source,
            status
        ])
        .run(tauri::generate_context!())
        .expect("Falha ao abrir TelaLeve");
}
