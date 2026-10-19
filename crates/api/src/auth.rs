//! Kolačić gosta: `httpOnly`, token nikad u JavaScriptu (ARHITEKTURA §12).

use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, HeaderValue, header, request::Parts};

pub const COOKIE: &str = "hotelixsim_guest";

/// Token gosta iz kolačića; pokvaren ili nepostojeći = `None` (anoniman pozivatelj, odluku donosi `app`).
pub struct Token(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Token {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Token(read(&parts.headers)))
    }
}

fn read(h: &HeaderMap) -> Option<String> {
    h.get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE)
        .map(|(_, v)| v.to_owned())
        .filter(|v| v.len() == 32 && v.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Trajanje kolačića: 120 dana (Leonova odluka 19.10.), jedina konstanta; klizni rok je u `me_hotel`.
pub const COOKIE_MAX_AGE_SECS: u32 = 120 * 24 * 60 * 60;

/// Je li kolačić `Secure` (samo iza HTTPS-a, produkcija); lokalno je http pa je isključen.
#[derive(Clone, Copy, Debug)]
pub struct CookieMode {
    pub secure: bool,
}

pub fn set_cookie(token: &str, mode: CookieMode) -> HeaderValue {
    let secure = if mode.secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={COOKIE_MAX_AGE_SECS}{secure}"
    ))
    .expect("token je heksadecimalan")
}
