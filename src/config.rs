use std::env;

use thiserror::Error;

/// JP server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Normalized base URL, always ending in `/api/` like `https://api.garupa.jp/api/`.
    pub base: String,
    /// Player UID embedded in ranking request paths.
    pub uid: String,
    /// Value of the `X-Signature` header, the device UUID.
    pub uuid: String,
    /// Static fallback client version, used until the App Store lookup succeeds.
    pub client_version: String,
    /// `X-Unity-Version` header value.
    pub unity_version: String,
    /// `User-Agent` header value.
    pub user_agent: String,
    /// `X-ClientPlatform` header value.
    pub client_platform: String,
    /// AES-128-CBC encryption key, exactly 16 bytes.
    pub encryption_key: Vec<u8>,
    /// AES-128-CBC IV, exactly 16 bytes.
    pub encryption_iv: Vec<u8>,
    /// App Store lookup URL used to auto-detect the client version.
    pub package_url: String,
}

impl ServerConfig {
    /// The server is enabled when its base URL is configured.
    pub fn enabled(&self) -> bool {
        !self.base.is_empty()
    }

    /// A fully disabled config, used when the upstream server is not configured.
    fn disabled() -> Self {
        ServerConfig {
            base: String::new(),
            uid: String::new(),
            uuid: String::new(),
            client_version: String::new(),
            unity_version: String::new(),
            user_agent: String::new(),
            client_platform: String::new(),
            encryption_key: Vec::new(),
            encryption_iv: Vec::new(),
            package_url: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub host: String,
    pub port: u16,
    pub api_prefix: String,
    pub log_level: String,
    pub http_timeout_ms: u64,
    pub cache_ttl_ranking_secs: u64,
    pub cache_ttl_master_secs: u64,
    pub cache_ttl_user_secs: u64,
    pub version_ttl_secs: u64,
    /// When non-empty, all `/api/*` requests must send it via
    /// `X-API-Key` or `Authorization: Bearer`.
    pub api_key: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("encryption key must be exactly 16 bytes: {0}")]
    KeyLength(String),
    #[error("encryption IV must be exactly 16 bytes: {0}")]
    IvLength(String),
}

fn to_string(raw: Option<String>, fallback: &str) -> String {
    match raw {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => fallback.to_string(),
    }
}

fn to_u64(raw: Option<String>, fallback: u64) -> u64 {
    raw.and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(fallback)
}

fn to_u16(raw: Option<String>, fallback: u16) -> u16 {
    raw.and_then(|v| v.trim().parse::<u16>().ok()).unwrap_or(fallback)
}

/// Converts a raw server base into the canonical Garupa API base URL.
///
/// Accepts a full URL or a bare hostname. A bare hostname gets `https://` and
/// `/api/` prepended. An empty value yields an empty string, meaning the
/// server is disabled.
fn to_base_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if trimmed.ends_with('/') {
            trimmed.to_string()
        } else {
            format!("{trimmed}/")
        }
    } else {
        format!("https://{}/api/", trimmed.trim_end_matches('/'))
    }
}

/// Converts a string to a 16-byte cipher key or IV.
fn to_cipher_bytes(raw: &str, label: &str) -> Result<Vec<u8>, ConfigError> {
    let bytes = raw.trim().as_bytes().to_vec();
    if bytes.len() != 16 {
        return Err(ConfigError::Message(format!("{label} must be 16 bytes, got {}", bytes.len())));
    }
    Ok(bytes)
}

impl Config {
    /// Builds the configuration from environment variables, including `.env`.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load `.env.local`, then `.env`, then `.env.example`.
        let _ = dotenvy::from_path(".env.local");
        dotenvy::from_path(".env").ok();

        let base = to_base_url(&to_string(env::var("GARUPA_SERVER_BASES").ok(), ""));
        let uid = to_string(env::var("GARUPA_UIDS").ok(), "");
        let uuid = to_string(env::var("GARUPA_UUIDS").ok(), "");
        let key_raw = to_string(env::var("GARUPA_ENCRYPTION_KEYS").ok(), "");
        let iv_raw = to_string(env::var("GARUPA_ENCRYPTION_IVS").ok(), "");

        let server = if base.is_empty() {
            ServerConfig::disabled()
        } else if uid.is_empty() && uuid.is_empty() && key_raw.is_empty() && iv_raw.is_empty() {
            // Base URL configured but every credential field is empty, which is
            // the state of a freshly copied .env.example. Disable the server with
            // a warning instead of exiting, so a first `docker compose up` starts
            // and /health responds rather than crash-looping on missing fields.
            eprintln!(
                "warning: GARUPA_SERVER_BASES is set but GARUPA_UIDS, GARUPA_UUIDS, \
                 GARUPA_ENCRYPTION_KEYS and GARUPA_ENCRYPTION_IVS are empty; \
                 the JP server is disabled"
            );
            ServerConfig::disabled()
        } else {
            if uid.is_empty() {
                return Err(ConfigError::MissingField("GARUPA_UIDS"));
            }
            if uuid.is_empty() {
                return Err(ConfigError::MissingField("GARUPA_UUIDS"));
            }
            if key_raw.is_empty() {
                return Err(ConfigError::MissingField("GARUPA_ENCRYPTION_KEYS"));
            }
            if iv_raw.is_empty() {
                return Err(ConfigError::MissingField("GARUPA_ENCRYPTION_IVS"));
            }
            let encryption_key = to_cipher_bytes(&key_raw, "GARUPA_ENCRYPTION_KEYS")
                .map_err(|e| match e {
                    ConfigError::Message(m) => ConfigError::KeyLength(m),
                    other => other,
                })?;
            let encryption_iv = to_cipher_bytes(&iv_raw, "GARUPA_ENCRYPTION_IVS")
                .map_err(|e| match e {
                    ConfigError::Message(m) => ConfigError::IvLength(m),
                    other => other,
                })?;

            let client_version = to_string(env::var("GARUPA_CLIENT_VERSIONS").ok(), "10.1.3");
            let unity_version = to_string(env::var("GARUPA_UNITY_VERSIONS").ok(), "2021.3.45f2");
            let default_user_agent = format!("UnityPlayer/{unity_version} (UnityWebRequest/1.0, libcurl/8.5.0-DEV)");
            let user_agent = to_string(env::var("GARUPA_USER_AGENTS").ok(), &default_user_agent);

            ServerConfig {
                base,
                uid,
                uuid,
                client_version,
                unity_version,
                user_agent,
                client_platform: to_string(env::var("GARUPA_CLIENT_PLATFORMS").ok(), "iOS"),
                package_url: to_string(
                    env::var("GARUPA_PACKAGE_URLS").ok(),
                    "https://itunes.apple.com/jp/lookup?bundleId=jp.co.craftegg.band",
                ),
                encryption_key,
                encryption_iv,
            }
        };

        Ok(Config {
            server,
            host: to_string(env::var("HOST").ok(), "127.0.0.1"),
            port: to_u16(env::var("PORT").ok(), 8080),
            api_prefix: to_string(env::var("API_PREFIX").ok(), "/api"),
            log_level: to_string(env::var("LOG_LEVEL").ok(), "info"),
            http_timeout_ms: to_u64(env::var("GARUPA_HTTP_TIMEOUT_MS").ok(), 10_000),
            cache_ttl_ranking_secs: to_u64(env::var("GARUPA_CACHE_TTL_RANKING").ok(), 30),
            cache_ttl_master_secs: to_u64(env::var("GARUPA_CACHE_TTL_MASTER").ok(), 3600),
            cache_ttl_user_secs: to_u64(env::var("GARUPA_CACHE_TTL_USER").ok(), 300),
            version_ttl_secs: to_u64(env::var("GARUPA_VERSION_TTL_SECONDS").ok(), 3600),
            api_key: to_string(env::var("API_KEY").ok(), ""),
        })
    }
}
