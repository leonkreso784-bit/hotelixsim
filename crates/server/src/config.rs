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
const GAMES_PER_GUEST: &str = "HOTELIXSIM_IGARA_PO_GOSTU";
const PILOT_GAMES_PER_GUEST: u32 = 3;
const DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[derive(Clone)]
pub struct Settings {
    pub production: bool,
    pub addr: String,
    pub content_dir: Option<PathBuf>,
    pub database_url: Option<String>,
    pub site_password: Option<String>,
    pub web_dir: Option<PathBuf>,
    /// Najveći broj igara po gostu. Zadano 3 u produkciji I na nelokalnoj adresi (zaboravljena zastavica ne gasi zaštitu), lokalno bez granice.
    pub games_per_guest: Option<u32>,
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
            .field("games_per_guest", &self.games_per_guest)
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
        let addr = get(ADDR).unwrap_or_else(|| DEFAULT_ADDR.to_owned());
        let games_per_guest = match get(GAMES_PER_GUEST) {
            Some(v) => match v.trim().parse::<u32>() {
                Ok(n) if n >= 1 => Some(n),
                _ => {
                    return Err(ConfigError::Invalid {
                        var: GAMES_PER_GUEST,
                        value: v,
                    });
                }
            },
            None => (production || !is_local_addr(&addr)).then_some(PILOT_GAMES_PER_GUEST),
        };
        let settings = Settings {
            production,
            addr,
            content_dir: get(CONTENT).map(PathBuf::from),
            database_url: get(DATABASE_URL),
            site_password,
            web_dir: get(WEB).map(PathBuf::from),
            games_per_guest,
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
