use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub xid: u64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Economy,
    #[default]
    Balanced,
    Sharp,
}
impl Quality {
    pub fn settings(self) -> (i32, i32, i32, u32, u32) {
        match self {
            Self::Economy => (1280, 720, 20, 1_500_000, 2_500_000),
            Self::Balanced => (1280, 720, 30, 2_500_000, 4_000_000),
            Self::Sharp => (1920, 1080, 30, 4_000_000, 6_000_000),
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
