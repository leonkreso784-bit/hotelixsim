//! Tipizirane postavke (pilot, dio 2 e): jedno mjesto čita varijable okoline u strukturu.
//! Produkcijski način (`HOTELIXSIM_PRODUKCIJA=1`) odbija start ako nedostaje lozinka stranice,
//! mapa weba ili baza: zaboravljena varijabla ne smije otvoriti stranicu svima.

use std::fmt;
use std::path::PathBuf;

const PRODUCTION: &str = "HOTELIXSIM_PRODUKCIJA";
const PASSWORD: &str = "HOTELIXSIM_LOZINKA";
const WEB: &str = "HOTELIXSIM_WEB";
const DATABASE_URL: &str = "DATABASE_URL";
const ADDR: &str = "HOTELIXSIM_ADDR";
const CONTENT: &str = "HOTELIXSIM_CONTENT";
const CAP_DAY: &str = "HOTELIXSIM_AI_STROP_DAN_USD";
const CAP_MONTH: &str = "HOTELIXSIM_AI_STROP_MJESEC_USD";
const API_KEY: &str = "ANTHROPIC_API_KEY";
const DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[derive(Clone)]
pub struct Settings {
    pub production: bool,
    pub addr: String,
    pub content_dir: Option<PathBuf>,
    pub database_url: Option<String>,
    pub site_password: Option<String>,
    pub web_dir: Option<PathBuf>,
    /// Dnevni i mjesečni strop troška AI-ja u dolarima (UTC); u produkciji su oba obavezna.
    pub ai_cap_day_usd: Option<f64>,
    pub ai_cap_month_usd: Option<f64>,
    pub anthropic_api_key: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// Obavezne varijable (u produkciji) koje nedostaju ili su prazne.
    Missing(Vec<&'static str>),
    /// Varijabla s vrijednošću koja se ne razumije (`value` nikad ne sadrži lozinku).
    Invalid { var: &'static str, value: String },
    /// Adresa nije lokalna, a lozinke stranice nema: stranica i AI bili bi otvoreni svima.
    OpenWithoutPassword { addr: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(vars) => write!(
                f,
                "produkcijski način ({PRODUCTION}) traži varijable koje nedostaju: {}",
                vars.join(", ")
            ),
            ConfigError::Invalid { var, value } if *var == PRODUCTION => write!(
                f,
                "varijabla {var} ima neispravnu vrijednost {value:?} (dopušteno: 1, true, 0, false)"
            ),
            ConfigError::Invalid { var, value } => {
                write!(f, "varijabla {var} je neispravna: {value}")
            }
            ConfigError::OpenWithoutPassword { addr } => write!(
                f,
                "adresa {addr} nije lokalna, a {PASSWORD} nije postavljena: stranica bi bila otvorena svima                  (postavi lozinku ili slušaj na 127.0.0.1)"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Ručni `Debug`: lozinka i adresa baze (može imati lozinku) se nikad ne ispisuju.
impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("production", &self.production)
            .field("addr", &self.addr)
            .field("content_dir", &self.content_dir)
            .field("database_url", &self.database_url.as_ref().map(|_| "***"))
            .field("site_password", &self.site_password.as_ref().map(|_| "***"))
            .field("web_dir", &self.web_dir)
            .field("ai_cap_day_usd", &self.ai_cap_day_usd)
            .field("ai_cap_month_usd", &self.ai_cap_month_usd)
            .field(
                "anthropic_api_key",
                &self.anthropic_api_key.as_ref().map(|_| "***"),
            )
            .finish()
    }
}

impl Settings {
    pub fn from_env() -> Result<Settings, ConfigError> {
        Settings::from_lookup(|k| std::env::var(k).ok())
    }

    /// Čita iz proizvoljnog izvora varijabli (testovi ne diraju stvarnu okolinu). Prazno/razmaci = nepostavljeno.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Settings, ConfigError> {
        let get = |k: &str| get(k).filter(|v| !v.trim().is_empty());
        let production = match get(PRODUCTION) {
            None => false,
            Some(v) => match v.trim().to_ascii_lowercase().as_str() {
                "1" | "true" => true,
                "0" | "false" => false,
                _ => {
                    return Err(ConfigError::Invalid {
                        var: PRODUCTION,
                        value: v,
                    });
                }
            },
        };
        let site_password = get(PASSWORD);
        if site_password.as_ref().is_some_and(|p| p.trim() != p) {
            // zalijepljena lozinka s razmakom ili novim retkom: studenti ne bi mogli ući, a nitko ne bi vidio zašto
            return Err(ConfigError::Invalid {
                var: PASSWORD,
                value: "razmak ili novi redak na početku ili kraju lozinke".to_owned(),
            });
        }
        let ai_cap_day_usd = parse_cap(CAP_DAY, get(CAP_DAY))?;
        let ai_cap_month_usd = parse_cap(CAP_MONTH, get(CAP_MONTH))?;
        let settings = Settings {
            production,
            addr: get(ADDR).unwrap_or_else(|| DEFAULT_ADDR.to_owned()),
            content_dir: get(CONTENT).map(PathBuf::from),
            database_url: get(DATABASE_URL),
            site_password,
            web_dir: get(WEB).map(PathBuf::from),
            ai_cap_day_usd,
            ai_cap_month_usd,
            anthropic_api_key: get(API_KEY),
        };
        if production {
            let mut missing = Vec::new();
            if settings.site_password.is_none() {
                missing.push(PASSWORD);
            }
            if settings.web_dir.is_none() {
                missing.push(WEB);
            }
            if settings.database_url.is_none() {
                missing.push(DATABASE_URL);
            }
            if settings.ai_cap_day_usd.is_none() {
                missing.push(CAP_DAY);
            }
            if settings.ai_cap_month_usd.is_none() {
                missing.push(CAP_MONTH);
            }
            if !missing.is_empty() {
                return Err(ConfigError::Missing(missing));
            }
        }
        // neovisno o HOTELIXSIM_PRODUKCIJA: prazna ili pogrešna zastavica ne smije otvoriti javnu adresu bez vrata
        if settings.site_password.is_none() && !is_local_addr(&settings.addr) {
            return Err(ConfigError::OpenWithoutPassword {
                addr: settings.addr,
            });
        }
        Ok(settings)
    }

    pub fn router_config(&self) -> api::RouterConfig {
        api::RouterConfig {
            secure_cookie: self.production,
            site_password: self.site_password.clone(),
            web_dir: self.web_dir.clone(),
        }
    }
}

/// Strop u dolarima: decimalni broj s točkom, konačan i >= 0; inače `Invalid`.
fn parse_cap(var: &'static str, raw: Option<String>) -> Result<Option<f64>, ConfigError> {
    let Some(raw) = raw else { return Ok(None) };
    match raw.trim().parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 0.0 => Ok(Some(v)),
        _ => Err(ConfigError::Invalid {
            var,
            value: format!("{raw:?} nije iznos u dolarima (npr. 5 ili 30.5)"),
        }),
    }
}

/// `127.0.0.1:port`, `[::1]:port`, `localhost:port` (i ostale loopback adrese) su lokalne.
fn is_local_addr(addr: &str) -> bool {
    let host = match addr.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map_or(rest, |(h, _)| h),
        None => addr.rsplit_once(':').map_or(addr, |(h, _)| h),
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}
