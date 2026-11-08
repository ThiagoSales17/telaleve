#[link(name = "X11")]
extern "C" {
    fn XInitThreads() -> std::os::raw::c_int;
}
pub fn init_xlib() -> Result<(), String> {
    static READY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    // Xlib must initialize its global locks before GTK or capture threads open a display.
    if *READY.get_or_init(|| unsafe { XInitThreads() != 0 }) {
        Ok(())
    } else {
        Err("XInitThreads falhou".into())
    }
}
use crate::model::Source;
use x11rb::{
    connection::Connection,
    protocol::{randr::ConnectionExt as _, xproto::*},
};
pub fn sources() -> Result<Vec<Source>, String> {
    let (c, n) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = c.setup().roots[n].root;
    let mut out = Vec::new();
    let monitors = c
        .randr_get_monitors(root, true)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    for m in monitors.monitors {
        if m.x < 0 || m.y < 0 {
            continue;
        }
        let name = c
            .get_atom_name(m.name)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        out.push(Source {
            id: format!("monitor:{}", m.name),
            name: format!("Monitor {}", String::from_utf8_lossy(&name.name)),
            xid: 0,
            x: m.x as u32,
            y: m.y as u32,
            width: m.width as u32,
            height: m.height as u32,
        });
    }
    let atom = |s: &[u8]| -> Result<u32, String> {
        Ok(c.intern_atom(false, s)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?
            .atom)
    };
    let clients = c
        .get_property(
            false,
            root,
            atom(b"_NET_CLIENT_LIST")?,
            AtomEnum::WINDOW,
            0,
            65536,
        )
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    for id in clients.value32().into_iter().flatten() {
        let Ok(g) = c.get_geometry(id).map_err(|e| e.to_string())?.reply() else {
            continue;
        };
        let Ok(attrs) = c
            .get_window_attributes(id)
            .map_err(|e| e.to_string())?
            .reply()
        else {
            continue;
        };
        if attrs.map_state != MapState::VIEWABLE {
            continue;
        }
        let title = c
            .get_property(false, id, atom(b"_NET_WM_NAME")?, AtomEnum::ANY, 0, 1024)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        let name = String::from_utf8_lossy(&title.value).to_string();
        if name.is_empty() || name == "TelaLeve" {
            continue;
        }
        out.push(Source {
            id: format!("window:{id}"),
            name,
            xid: id as u64,
            x: 0,
            y: 0,
            width: g.width as u32,
            height: g.height as u32,
        });
    }
    Ok(out)
}
