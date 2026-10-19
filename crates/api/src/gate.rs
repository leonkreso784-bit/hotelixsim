//! Vrata stranice (pilot, dio 2 a): HTTP Basic Auth za cijeli poslužitelj. Provjerava se samo lozinka
//! (korisničko ime je bilo koje), usporedba je u stalnom vremenu, a `/v1/health` je uvijek otvoren.

use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

/// Putanja koja ostaje otvorena (ping i provjera Railwaya).
const OPEN_PATH: &str = "/v1/health";

#[derive(Clone)]
pub struct Gate {
    digest: [u8; 32],
}

impl Gate {
    /// `None` za nepostavljenu ili praznu lozinku: tada vrata ne postoje.
    pub fn new(password: Option<&str>) -> Option<Self> {
        let p = password?;
        if p.trim().is_empty() {
            return None;
        }
        Some(Self { digest: hash(p) })
    }

    fn accepts(&self, supplied: &str) -> bool {
        // Usporedba SHA-256 sažetaka istog trajanja: ne odaje ni duljinu lozinke.
        let other = hash(supplied);
        let mut diff = 0u8;
        for (a, b) in self.digest.iter().zip(other.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }
}

fn hash(s: &str) -> [u8; 32] {
    Sha256::digest(s.as_bytes()).into()
}

pub async fn require_password(State(gate): State<Gate>, req: Request, next: Next) -> Response {
    if req.uri().path() == OPEN_PATH {
        return next.run(req).await;
    }
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(password_from_basic)
        .is_some_and(|p| gate.accepts(&p));
    if ok {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"HotelixSim\", charset=\"UTF-8\""),
            )],
            "Potrebna je lozinka.",
        )
            .into_response()
    }
}

/// `Basic base64(korisnik:lozinka)` daje lozinku (dijeli se na prvoj `:`).
fn password_from_basic(header: &str) -> Option<String> {
    let (scheme, rest) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    let bytes = STANDARD.decode(rest.trim()).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    let (_, password) = text.split_once(':')?;
    Some(password.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_basic_credentials() {
        assert_eq!(
            password_from_basic("Basic c3R1ZGVudDp0YWpuYTEyMw==").as_deref(),
            Some("tajna123")
        );
        assert_eq!(
            password_from_basic("Basic dTp0YTpqbmE=").as_deref(),
            Some("ta:jna")
        );
        assert_eq!(password_from_basic("Bearer abc"), None);
        assert_eq!(password_from_basic("Basic !!!"), None);
    }

    #[test]
    fn blank_password_means_no_gate() {
        assert!(Gate::new(None).is_none());
        assert!(Gate::new(Some("")).is_none());
        assert!(Gate::new(Some("   ")).is_none());
        assert!(Gate::new(Some("x")).unwrap().accepts("x"));
        assert!(!Gate::new(Some("x")).unwrap().accepts("y"));
    }
}
