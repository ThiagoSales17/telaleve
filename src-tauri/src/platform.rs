/// Use the system picker on Wayland and keep direct X11 capture on legacy desktops.
pub fn portal_required(session: Option<&str>, wayland: Option<&str>) -> bool {
    match session {
        Some("wayland") => true,
        Some("x11") => false,
        _ => wayland.is_some_and(|display| !display.is_empty()),
    }
}

pub fn uses_portal() -> bool {
    portal_required(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chooses_native_display_protocol() {
        assert!(portal_required(Some("wayland"), None));
        assert!(portal_required(None, Some("wayland-0")));
        assert!(!portal_required(Some("x11"), None));
        assert!(!portal_required(Some("x11"), Some("stale-wayland")));
        assert!(!portal_required(None, Some("")));
        assert!(!portal_required(None, None));
    }
}
