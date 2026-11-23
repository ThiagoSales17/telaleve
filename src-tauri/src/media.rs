use crate::model::{Quality, Source};
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_webrtc as rtc;
use serde_json::json;
use std::{
    os::fd::{AsRawFd, OwnedFd},
    sync::mpsc,
    time::Duration,
};
use tokio::sync::broadcast;
pub struct Media {
    pipeline: gst::Pipeline,
    rtc: gst::Element,
    frames: gst::Element,
    source: SourceStream,
    pub codec: String,
}
fn capture(s: &Source, q: Quality, remote: Option<&OwnedFd>) -> Result<gst::Bin, String> {
    if let Some(pipewire) = &s.pipewire {
        let fd = remote.ok_or("Conexão PipeWire indisponível")?.as_raw_fd();
        let desc = format!(
            "pipewiresrc fd={fd} path={} do-timestamp=true ! {}",
            pipewire.node_id,
            normalize_portal(q)
        );
        let bin = gst::parse::bin_from_description(&desc, true).map_err(|e| e.to_string())?;
        configure_portal_scaling(&bin, q)?;
        return Ok(bin);
    }
    if s.width < 2 || s.height < 2 {
        return Err("A origem tem dimensões inválidas".into());
    }
    let (w, h, f, _, _) = q.settings();
    let (sw, sh) = fit(s.width, s.height, w as u32, h as u32);
    let location = if s.xid != 0 {
        format!("xid={}", s.xid)
    } else {
        format!(
            "startx={} starty={} endx={} endy={}",
            s.x,
            s.y,
            s.x + s.width - 1,
            s.y + s.height - 1
        )
    };
    let desc=format!("ximagesrc {location} use-damage=false show-pointer=true ! video/x-raw,framerate={f}/1 ! videoconvert ! videoscale ! video/x-raw,width={sw},height={sh},pixel-aspect-ratio=1/1 ! videobox autocrop=true ! video/x-raw,width={w},height={h} ! identity");
    gst::parse::bin_from_description(&desc, true).map_err(|e| e.to_string())
}
// The caps probe uses physical frame dimensions rather than portal logical coordinates.
fn normalize_portal(q: Quality) -> String {
    let (w, h, _, _, _) = q.settings();
    format!("videoconvert ! identity name=portal-rate ! videoscale name=portal-scale ! capsfilter name=portal-size ! videobox autocrop=true ! video/x-raw,width={w},height={h} ! identity")
}
fn configure_portal_timing(bin: &gst::Bin, quality: Quality) -> Result<(), String> {
    // Portal streams may advertise 0/1 and deliver frames only on desktop damage.
    // Rate-limit by timestamps without videorate's version-dependent 0/1 handling.
    let rate = bin
        .by_name("portal-rate")
        .ok_or("Limitador de quadros indisponível")?;
    let rate_pad = rate
        .static_pad("src")
        .ok_or("Saída do limitador indisponível")?;
    let fps = quality.settings().2;
    let interval = gst::ClockTime::from_nseconds(1_000_000_000 / fps as u64);
    let last = std::sync::Mutex::new(None::<gst::ClockTime>);
    rate_pad
        .add_probe(
            gst::PadProbeType::BUFFER | gst::PadProbeType::EVENT_DOWNSTREAM,
            move |_, info| {
                match info.data.as_mut() {
                    Some(gst::PadProbeData::Event(event)) => {
                        if let gst::EventView::Caps(caps_event) = event.view() {
                            let mut caps = caps_event.caps().to_owned();
                            if let Some(structure) = caps.make_mut().structure_mut(0) {
                                structure.set("framerate", gst::Fraction::new(fps, 1));
                            }
                            *event = gst::event::Caps::new(&caps);
                            *last.lock().unwrap() = None;
                        }
                    }
                    Some(gst::PadProbeData::Buffer(buffer)) => {
                        if let Some(pts) = buffer.pts() {
                            let mut last = last.lock().unwrap();
                            if last.is_some_and(|previous| {
                                pts >= previous && pts - previous < interval
                            }) {
                                return gst::PadProbeReturn::Drop;
                            }
                            *last = Some(pts);
                        }
                        buffer.make_mut().set_duration(interval);
                    }
                    _ => {}
                }
                gst::PadProbeReturn::Ok
            },
        )
        .ok_or("Não foi possível configurar o limite de quadros")?;
    Ok(())
}
fn configure_portal_scaling(bin: &gst::Bin, quality: Quality) -> Result<(), String> {
    configure_portal_timing(bin, quality)?;
    let scale = bin
        .by_name("portal-scale")
        .ok_or("Conversor de escala indisponível")?;
    let filter = bin
        .by_name("portal-size")
        .ok_or("Filtro de resolução indisponível")?;
    let pad = scale
        .static_pad("sink")
        .ok_or("Entrada do conversor indisponível")?;
    let (mw, mh, _, _, _) = quality.settings();
    pad.add_probe(gst::PadProbeType::EVENT_DOWNSTREAM, move |_, info| {
        if let Some(gst::PadProbeData::Event(event)) = info.data.as_ref() {
            if let gst::EventView::Caps(event) = event.view() {
                if let Some(caps) = event.caps().structure(0) {
                    if let (Ok(width), Ok(height)) =
                        (caps.get::<i32>("width"), caps.get::<i32>("height"))
                    {
                        if width > 0 && height > 0 {
                            let (width, height) =
                                fit(width as u32, height as u32, mw as u32, mh as u32);
                            let next = gst::Caps::builder("video/x-raw")
                                .field("width", width as i32)
                                .field("height", height as i32)
                                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                                .build();
                            if filter.property::<Option<gst::Caps>>("caps").as_ref() != Some(&next)
                            {
                                filter.set_property("caps", &next);
                            }
                        }
                    }
                }
            }
        }
        gst::PadProbeReturn::Ok
    })
    .ok_or("Não foi possível configurar a escala da captura")?;
    Ok(())
}
fn fit(w: u32, h: u32, mw: u32, mh: u32) -> (u32, u32) {
    let scale = (mw as f64 / w as f64).min(mh as f64 / h as f64).min(1.0);
    (
        ((w as f64 * scale) as u32 / 2 * 2).max(2),
        ((h as f64 * scale) as u32 / 2 * 2).max(2),
    )
}
fn payload(sdp: &str, codec: &str) -> Option<u32> {
    sdp.lines().find_map(|l| {
        let (pt, v) = l.trim().strip_prefix("a=rtpmap:")?.split_once(' ')?;
        if v.starts_with(&format!("{codec}/90000")) {
            pt.parse().ok()
        } else {
            None
        }
    })
}
fn hardware() -> Option<&'static str> {
    for name in ["vah264enc", "nvh264enc", "qsvh264enc"] {
        let Ok(p) = gst::parse::launch(&format!(
            "videotestsrc num-buffers=3 ! videoconvert ! {name} ! h264parse ! video/x-h264,profile=constrained-baseline ! fakesink"
        ))
        .and_then(|e| {
            e.downcast::<gst::Pipeline>()
                .map_err(|_| gst::glib::Error::new(gst::CoreError::Failed, "pipeline"))
        }) else {
            continue;
        };
        let valid = p.set_state(gst::State::Playing).is_ok()
            && p.bus()
                .unwrap()
                .timed_pop_filtered(
                    gst::ClockTime::from_seconds(3),
                    &[gst::MessageType::Eos, gst::MessageType::Error],
                )
                .is_some_and(|m| matches!(m.view(), gst::MessageView::Eos(_)));
        let _ = p.set_state(gst::State::Null);
        if valid {
            return Some(name);
        }
    }
    None
}
struct SourceStream {
    pipeline: gst::Pipeline,
    active: std::sync::Arc<std::sync::atomic::AtomicBool>,
    _remote: Option<OwnedFd>,
    _origin: Source,
}
impl SourceStream {
    fn prepare(
        s: &Source,
        q: Quality,
        frames: &gst::Element,
        remote: Option<OwnedFd>,
    ) -> Result<Self, String> {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let pipeline = gst::Pipeline::new();
        let bin = capture(s, q, remote.as_ref())?;
        let sink = gst::ElementFactory::make("appsink")
            .property("emit-signals", true)
            .property("sync", false)
            .property("max-buffers", 1u32)
            .property("drop", true)
            .build()
            .map_err(|e| e.to_string())?;
        pipeline
            .add_many([bin.upcast_ref(), &sink])
            .map_err(|e| e.to_string())?;
        bin.link(&sink).map_err(|e| e.to_string())?;
        let active = Arc::new(AtomicBool::new(false));
        let enabled = active.clone();
        let output = frames.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        sink.connect("new-sample", false, move |values| {
            let sink = values[0].get::<gst::Element>().unwrap();
            if let Some(sample) = sink.emit_by_name::<Option<gst::Sample>>("pull-sample", &[]) {
                let _ = tx.try_send(());
                if enabled.load(Ordering::Acquire) {
                    if let (Some(caps), Some(buffer)) = (sample.caps(), sample.buffer()) {
                        output.set_property("caps", caps.to_owned());
                        let mut buffer = buffer.to_owned();
                        let writable = buffer.make_mut();
                        writable.set_pts(None);
                        writable.set_dts(None);
                        let _ = output.emit_by_name::<gst::FlowReturn>("push-buffer", &[&buffer]);
                    }
                }
            }
            Some(gst::FlowReturn::Ok.to_value())
        });
        let stream = Self {
            pipeline,
            active,
            _remote: remote,
            _origin: s.clone(),
        };
        stream
            .pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(3))
            .map_err(|_| "A nova captura não produziu imagem".to_string())?;
        Ok(stream)
    }
    fn activate(&self) {
        self.active
            .store(true, std::sync::atomic::Ordering::Release);
    }
    fn deactivate(&self) {
        self.active
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
impl Drop for SourceStream {
    fn drop(&mut self) {
        self.deactivate();
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
impl Media {
    pub fn new(
        s: &Source,
        q: Quality,
        offer: &str,
        tx: broadcast::Sender<String>,
        remote: Option<OwnedFd>,
    ) -> Result<Self, String> {
        let (_, _, _, bitrate, _) = q.settings();
        let hw = if payload(offer, "H264").is_some() {
            hardware()
        } else {
            None
        };
        let (codec, encoding) = if let Some(enc) = hw {
            let pt = payload(offer, "H264").unwrap();
            ("H264",format!("{enc} name=encoder ! h264parse ! video/x-h264,profile=constrained-baseline ! rtph264pay config-interval=-1 aggregate-mode=zero-latency pt={pt} ! application/x-rtp,media=video,encoding-name=H264,payload={pt}"))
        } else {
            let pt = payload(offer, "VP8").ok_or("O receptor não oferece VP8 compatível")?;
            ("VP8",format!("vp8enc name=encoder deadline=1 cpu-used=8 lag-in-frames=0 target-bitrate={bitrate} keyframe-max-dist=30 threads=4 ! rtpvp8pay pt={pt} ! application/x-rtp,media=video,encoding-name=VP8,payload={pt}"))
        };
        let pipeline=gst::parse::launch(&format!("appsrc name=frames is-live=true format=time do-timestamp=true block=false max-buffers=1 leaky-type=downstream ! queue max-size-buffers=1 max-size-bytes=0 max-size-time=0 leaky=downstream ! videoconvert ! {encoding} ! webrtcbin name=peer bundle-policy=max-bundle")).map_err(|e|e.to_string())?.downcast::<gst::Pipeline>().map_err(|_|"pipeline")?;
        if codec == "H264" {
            let encoder = pipeline.by_name("encoder").unwrap();
            for (key, value) in [
                ("b-frames", "0"),
                ("bframes", "0"),
                ("key-int-max", "30"),
                ("gop-size", "30"),
                ("zerolatency", "true"),
            ] {
                if encoder.find_property(key).is_some() {
                    encoder.set_property_from_str(key, value);
                }
            }
        }
        let peer = pipeline.by_name("peer").unwrap();
        let frames = pipeline.by_name("frames").unwrap();
        let source = SourceStream::prepare(s, q, &frames, remote)?;
        let ice_tx = tx.clone();
        peer.connect("on-ice-candidate", false, move |args| {
            let m = args[1].get::<u32>().unwrap();
            let c = args[2].get::<String>().unwrap();
            let _ = ice_tx.send(json!({"type":"ice","candidate":c,"sdpMLineIndex":m}).to_string());
            None
        });
        let sdp =
            gstreamer_sdp::SDPMessage::parse_buffer(offer.as_bytes()).map_err(|e| e.to_string())?;
        let answer_peer = peer.clone();
        let set = gst::Promise::with_change_func(move |result| {
            if !matches!(result, Ok(Some(_)) | Ok(None))
                || result
                    .as_ref()
                    .ok()
                    .and_then(|x| *x)
                    .is_some_and(|x| x.has_field("error"))
            {
                let _ = tx
                    .send(json!({"type":"error","message":"Falha ao aceitar oferta"}).to_string());
                return;
            }
            let local_peer = answer_peer.clone();
            let answer_tx = tx.clone();
            let promise = gst::Promise::with_change_func(move |reply| {
                if let Ok(Some(reply)) = reply {
                    if let Ok(answer) = reply.get::<rtc::WebRTCSessionDescription>("answer") {
                        local_peer.emit_by_name::<()>(
                            "set-local-description",
                            &[&answer, &None::<gst::Promise>],
                        );
                        if let Ok(text) = answer.sdp().as_text() {
                            let _ = answer_tx.send(json!({"type":"answer","sdp":text}).to_string());
                        }
                    }
                }
            });
            answer_peer.emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);
        });
        pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| e.to_string())?;
        peer.emit_by_name::<()>(
            "set-remote-description",
            &[
                &rtc::WebRTCSessionDescription::new(rtc::WebRTCSDPType::Offer, sdp),
                &set,
            ],
        );
        let mut media = Self {
            pipeline,
            rtc: peer,
            frames,
            source,
            codec: codec.into(),
        };
        media
            .pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| e.to_string())?;
        media.source.activate();
        media.bitrate(bitrate);
        Ok(media)
    }
    pub fn ice(&self, m: u32, c: &str) {
        self.rtc.emit_by_name::<()>("add-ice-candidate", &[&m, &c]);
    }
    pub fn bitrate(&mut self, b: u32) {
        let e = self.pipeline.by_name("encoder").unwrap();
        if self.codec == "VP8" {
            e.set_property("target-bitrate", b as i32);
        } else if e.find_property("bitrate").is_some() {
            e.set_property("bitrate", b / 1000);
        }
    }
    pub fn switch(
        &mut self,
        s: &Source,
        q: Quality,
        remote: Option<OwnedFd>,
    ) -> Result<(), String> {
        // ximagesrc grabs the X server while reading a frame. Opening another Xlib
        // display concurrently can deadlock against libXext's global display lock.
        // Stop the old X11 reader before opening the next; retain it for rollback.
        let restart_x11 = self.source._origin.pipewire.is_none();
        if restart_x11 {
            self.source.deactivate();
            self.source
                .pipeline
                .set_state(gst::State::Ready)
                .map_err(|e| e.to_string())?;
        }
        let next = match SourceStream::prepare(s, q, &self.frames, remote) {
            Ok(next) => next,
            Err(error) => {
                if restart_x11 {
                    self.source.activate();
                    self.source
                        .pipeline
                        .set_state(gst::State::Playing)
                        .map_err(|e| format!("{error}; falha ao retomar a origem anterior: {e}"))?;
                }
                return Err(error);
            }
        };
        self.source.deactivate();
        let old = std::mem::replace(&mut self.source, next);
        drop(old);
        self.source.activate();
        Ok(())
    }
    pub fn error(&self) -> Option<String> {
        if self
            .source
            .pipeline
            .bus()?
            .pop_filtered(&[gst::MessageType::Error, gst::MessageType::Eos])
            .is_some()
        {
            return Some("A captura parou".into());
        }

        self.pipeline
            .bus()?
            .pop_filtered(&[gst::MessageType::Error])
            .map(|_| "A captura ou o encoder parou; selecione a origem novamente".into())
    }
}
impl Drop for Media {
    fn drop(&mut self) {
        self.source.deactivate();
        let _ = self.source.pipeline.set_state(gst::State::Null);
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dimensions() {
        assert_eq!(fit(640, 480, 1280, 720), (640, 480));
        assert_eq!(fit(1920, 1080, 1280, 720), (1280, 720));
    }
    #[test]
    fn portal_conversion_centers_small_frames_without_upscaling() {
        gst::init().unwrap();
        let desc = format!("videotestsrc num-buffers=1 pattern=white ! video/x-raw,format=RGB,width=320,height=240,framerate=30/1 ! {} ! video/x-raw,format=RGB ! appsink name=sample sync=false", normalize_portal(Quality::Balanced));
        let pipeline = gst::parse::launch(&desc)
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        configure_portal_scaling(pipeline.upcast_ref(), Quality::Balanced).unwrap();
        pipeline.set_state(gst::State::Playing).unwrap();
        let sink = pipeline.by_name("sample").unwrap();
        let sample =
            sink.emit_by_name::<Option<gst::Sample>>("try-pull-sample", &[&3_000_000_000u64]);
        pipeline.set_state(gst::State::Null).unwrap();
        let sample = sample.expect("A conversão não produziu um quadro");
        let caps = sample.caps().unwrap().structure(0).unwrap();
        assert_eq!(caps.get::<i32>("width").unwrap(), 1280);
        assert_eq!(caps.get::<i32>("height").unwrap(), 720);
        let buffer = sample.buffer().unwrap().map_readable().unwrap();
        assert!(buffer.as_slice()[(360 * 1280 + 640) * 3] > 250);
        assert!(
            buffer.as_slice()[(360 * 1280 + 400) * 3] < 5,
            "Uma entrada pequena foi ampliada"
        );
    }
    #[test]
    fn portal_first_live_frame_does_not_wait_for_more_desktop_damage() {
        gst::init().unwrap();
        let desc = format!("appsrc name=input is-live=true format=time ! video/x-raw,format=RGB,width=320,height=240,framerate=0/1 ! {} ! video/x-raw,format=RGB ! appsink name=sample sync=false", normalize_portal(Quality::Balanced));
        let pipeline = gst::parse::launch(&desc)
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        configure_portal_scaling(pipeline.upcast_ref(), Quality::Balanced).unwrap();
        let input = pipeline.by_name("input").unwrap();
        let sink = pipeline.by_name("sample").unwrap();
        pipeline.set_state(gst::State::Playing).unwrap();
        let mut frame = gst::Buffer::from_mut_slice(vec![255u8; 320 * 240 * 3]);
        frame.make_mut().set_pts(gst::ClockTime::ZERO);
        assert_eq!(
            input.emit_by_name::<gst::FlowReturn>("push-buffer", &[&frame]),
            gst::FlowReturn::Ok
        );
        let sample =
            sink.emit_by_name::<Option<gst::Sample>>("try-pull-sample", &[&1_000_000_000u64]);
        pipeline.set_state(gst::State::Null).unwrap();
        assert!(
            sample.is_some(),
            "A captura de um desktop estático dependeu indevidamente de um segundo quadro ou EOS"
        );
    }
    #[test]
    fn portal_variable_rate_is_limited_without_duplicate_frames() {
        gst::init().unwrap();
        let desc = format!("appsrc name=input is-live=true format=time ! video/x-raw,format=RGB,width=320,height=240,framerate=0/1 ! {} ! appsink name=sample sync=false", normalize_portal(Quality::Balanced));
        let pipeline = gst::parse::launch(&desc)
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        configure_portal_scaling(pipeline.upcast_ref(), Quality::Balanced).unwrap();
        let input = pipeline.by_name("input").unwrap();
        let sink = pipeline.by_name("sample").unwrap();
        pipeline.set_state(gst::State::Playing).unwrap();
        for i in 0..12 {
            let mut frame = gst::Buffer::from_mut_slice(vec![255u8; 320 * 240 * 3]);
            frame
                .make_mut()
                .set_pts(gst::ClockTime::from_nseconds(i * 8_333_334));
            assert_eq!(
                input.emit_by_name::<gst::FlowReturn>("push-buffer", &[&frame]),
                gst::FlowReturn::Ok
            );
        }
        assert_eq!(
            input.emit_by_name::<gst::FlowReturn>("end-of-stream", &[]),
            gst::FlowReturn::Ok
        );
        let mut count = 0;
        while let Some(sample) =
            sink.emit_by_name::<Option<gst::Sample>>("try-pull-sample", &[&1_000_000_000u64])
        {
            assert_eq!(
                sample
                    .caps()
                    .unwrap()
                    .structure(0)
                    .unwrap()
                    .get::<gst::Fraction>("framerate")
                    .unwrap(),
                gst::Fraction::new(30, 1)
            );
            assert_eq!(
                sample.buffer().unwrap().duration(),
                Some(gst::ClockTime::from_nseconds(33_333_333))
            );
            count += 1;
        }
        pipeline.set_state(gst::State::Null).unwrap();
        assert_eq!(
            count, 3,
            "O limite de 30 FPS não foi aplicado à entrada variável de 120 FPS"
        );
    }
    #[test]
    fn codec() {
        assert_eq!(payload("a=rtpmap:102 H264/90000\r\n", "H264"), Some(102));
    }
}

#[cfg(test)]
mod integration {
    use super::*;
    #[test]
    #[ignore = "Requer servidor X11 acessível e plugins GStreamer"]
    fn capture_answer_and_twenty_switches() {
        crate::capture::init_xlib().unwrap();
        gst::init().unwrap();
        let source = crate::capture::sources()
            .unwrap()
            .into_iter()
            .find(|s| s.xid == 0)
            .unwrap();
        let offer="v=0\r\no=- 1 1 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\na=recvonly\r\na=rtcp-mux\r\na=ice-ufrag:test\r\na=ice-pwd:testpassword012345678901\r\na=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\na=setup:actpass\r\na=rtpmap:96 VP8/90000\r\n";
        let (tx, mut rx) = broadcast::channel(128);
        let mut media = Media::new(&source, Quality::Smooth, offer, tx, None).unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(10);
        let mut answered = false;
        while std::time::Instant::now() < until {
            if let Ok(event) = rx.try_recv() {
                if event.contains("\"answer\"") {
                    answered = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(answered, "Não houve resposta SDP");
        let window = crate::capture::sources()
            .unwrap()
            .into_iter()
            .find(|s| s.xid != 0 && s.width >= 200 && s.height >= 200);
        eprintln!("Alternando monitor e janela: {}", window.is_some());
        for i in 0..20 {
            let chosen = if i % 2 == 0 {
                window.as_ref().unwrap_or(&source)
            } else {
                &source
            };
            let quality = if i % 2 == 0 {
                Quality::Smooth
            } else {
                Quality::Balanced
            };
            media.switch(chosen, quality, None).unwrap();
            let sink = media
                .source
                .pipeline
                .iterate_sinks()
                .into_iter()
                .next()
                .unwrap()
                .unwrap();
            let caps = sink.static_pad("sink").unwrap().current_caps().unwrap();
            assert_eq!(
                caps.structure(0)
                    .unwrap()
                    .get::<gst::Fraction>("framerate")
                    .unwrap(),
                gst::Fraction::new(quality.settings().2, 1)
            );
        }
        let mut invalid = source.clone();
        invalid.width = 0;
        assert!(media.switch(&invalid, Quality::Balanced, None).is_err());
        assert!(media
            .source
            .active
            .load(std::sync::atomic::Ordering::Acquire));
        let (result, state, _) = media.source.pipeline.state(gst::ClockTime::from_seconds(3));
        assert!(result.is_ok());
        assert_eq!(
            state,
            gst::State::Playing,
            "A origem anterior não foi retomada"
        );
        assert!(media.error().is_none());
        media
            .source
            .pipeline
            .bus()
            .unwrap()
            .post(gst::message::Eos::builder().build())
            .unwrap();
        assert!(
            media.error().is_some(),
            "O encerramento da captura deve ser detectado também sem erro do backend"
        );
    }
}
