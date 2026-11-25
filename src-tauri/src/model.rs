use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    os::fd::OwnedFd,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

/// A portal session capable of opening an independent PipeWire connection.
pub trait PipeWireRemote: std::fmt::Debug + Send + Sync {
    fn open(&self) -> Pin<Box<dyn Future<Output = Result<OwnedFd, String>> + Send + '_>>;
}

#[derive(Clone, Debug)]
pub struct PipeWireSource {
    pub node_id: u32,
    pub remote: Arc<dyn PipeWireRemote>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub xid: u64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    #[serde(skip)]
    pub pipewire: Option<PipeWireSource>,
}
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Economy,
    #[default]
    Balanced,
    Sharp,
    Smooth,
}
impl Quality {
    pub fn settings(self) -> (i32, i32, i32, u32, u32) {
        match self {
            Self::Economy => (1280, 720, 20, 1_500_000, 2_500_000),
            Self::Balanced => (1280, 720, 30, 2_500_000, 4_000_000),
            Self::Sharp => (1920, 1080, 30, 4_000_000, 6_000_000),
            Self::Smooth => (1280, 720, 60, 4_000_000, 6_000_000),
        }
    }
}
pub struct Access {
    pub token: String,
    pub expires: Instant,
    pub used: bool,
    pub resume: Option<String>,
    pub disconnected: Option<Instant>,
    pub connected: bool,
}
impl Access {
    pub fn new() -> Self {
        Self {
            token: uuid::Uuid::new_v4().to_string(),
            expires: Instant::now() + Duration::from_secs(300),
            used: false,
            resume: None,
            disconnected: None,
            connected: false,
        }
    }
    pub fn claim(&mut self, token: &str, resume: bool, now: Instant) -> bool {
        if self.connected {
            return false;
        }
        let valid = if resume {
            self.resume.as_deref() == Some(token)
                && self
                    .disconnected
                    .is_some_and(|t| now.duration_since(t) < Duration::from_secs(30))
        } else {
            !self.used && now < self.expires && self.token == token
        };
        if valid {
            self.connected = true;
            if !resume {
                self.used = true;
            }
            self.disconnected = None;
        }
        valid
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct PrivateRemote;
    impl PipeWireRemote for PrivateRemote {
        fn open(&self) -> Pin<Box<dyn Future<Output = Result<OwnedFd, String>> + Send + '_>> {
            Box::pin(async { Err("Não usado na serialização".into()) })
        }
    }
    #[test]
    fn source_serialization_does_not_expose_portal_access() {
        let source = Source {
            id: "portal".into(),
            name: "Tela".into(),
            xid: 0,
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
            pipewire: Some(PipeWireSource {
                node_id: 42,
                remote: Arc::new(PrivateRemote),
            }),
        };
        let value = serde_json::to_value(&source).unwrap();
        assert!(value.get("pipewire").is_none());
        assert!(value.get("remote").is_none());
        let restored: Source = serde_json::from_value(value).unwrap();
        assert!(restored.pipewire.is_none());
    }
    #[test]
    fn token_lifecycle() {
        let mut a = Access::new();
        let t = a.token.clone();
        let now = Instant::now();
        assert!(!a.claim("wrong", false, now));
        assert!(a.claim(&t, false, now));
        assert!(!a.claim(&t, false, now));
        a.connected = false;
        assert!(!a.claim(&t, false, now));
    }
    #[test]
    fn expiry() {
        let mut a = Access::new();
        let t = a.token.clone();
        assert!(!a.claim(&t, false, a.expires));
    }
    #[test]
    fn reconnect() {
        let mut a = Access::new();
        a.resume = Some("resume".into());
        let now = Instant::now();
        a.disconnected = Some(now);
        assert!(a.claim("resume", true, now + Duration::from_secs(29)));
        a.connected = false;
        a.disconnected = Some(now);
        assert!(!a.claim("resume", true, now + Duration::from_secs(30)));
    }
}
