//! Garupa official-game API client: request signing and AES decryption.
//! Ported from GarupaSpeedTracker's `backend/src/api/garupa.ts`, keeping only
//! the non-CN path since this project targets the JP server.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::warn;

use crate::config::{Config, ServerConfig};
use crate::crypto::decrypt_aes_128_cbc;
use crate::error::{AppError, AppResult};

/// Builds the request headers required by the Garupa API.
fn build_headers(cfg: &ServerConfig, client_version: &str, extra: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();

    fn hv(s: &str) -> HeaderValue {
        HeaderValue::from_str(s).unwrap_or_else(|_| HeaderValue::from_static(""))
    }

    headers.insert("User-Agent", hv(&cfg.user_agent));
    headers.insert("X-Unity-Version", hv(&cfg.unity_version));
    headers.insert("X-ClientPlatform", hv(&cfg.client_platform));
    headers.insert("X-ClientVersion", hv(client_version));
    headers.insert("X-Signature", hv(&cfg.uuid));
    headers.insert("Accept-Encoding", hv("deflate, gzip"));
    headers.insert("Content-Type", hv("application/octet-stream"));
    headers.insert("Accept", hv("application/octet-stream"));

    for (name, value) in extra {
        if let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) {
            headers.insert(header_name, hv(value));
        }
    }

    headers
}

/// Caches the auto-detected client version, refreshed from the App Store lookup.
pub struct VersionCache {
    inner: Mutex<Option<(String, Instant)>>,
    ttl: Duration,
    http: reqwest::Client,
}

impl VersionCache {
    pub fn new(ttl: Duration, http: reqwest::Client) -> Self {
        Self { inner: Mutex::new(None), ttl, http }
    }

    /// Returns the current client version, refreshing from the App Store lookup
    /// URL when the cache is stale. Falls back to the configured static version
    /// on any lookup failure.
    pub async fn get(&self, cfg: &ServerConfig) -> String {
        if let Some((version, at)) = self.inner.lock().unwrap().clone() {
            if at.elapsed() < self.ttl {
                return version;
            }
        }

        match self.fetch_from_store(cfg).await {
            Ok(version) => {
                self.inner.lock().unwrap().replace((version.clone(), Instant::now()));
                version
            }
            Err(e) => {
                warn!("client version lookup failed: {e}");
                if let Some((version, _)) = self.inner.lock().unwrap().clone() {
                    version
                } else {
                    cfg.client_version.clone()
                }
            }
        }
    }

    /// Drops the cached version so the next read refetches from the store.
    /// Used when the game API reports the client version is out of date.
    pub fn drop_cache(&self) {
        self.inner.lock().unwrap().take();
    }

    async fn fetch_from_store(&self, cfg: &ServerConfig) -> Result<String, AppError> {
        if cfg.package_url.is_empty() {
            return Ok(cfg.client_version.clone());
        }
        let resp = self
            .http
            .get(&cfg.package_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        let json: serde_json::Value = resp.json().await?;
        json["results"][0]["version"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Internal("client version not found in App Store response".to_string()))
    }
}

/// The Garupa game API client.
pub struct GarupaClient {
    http: reqwest::Client,
    versions: VersionCache,
}

struct FetchRaw {
    status: u16,
    body: Vec<u8>,
}

impl GarupaClient {
    pub fn new(config: &Config) -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.http_timeout_ms))
            .build()
            .map_err(|e| AppError::Internal(format!("failed to build HTTP client: {e}")))?;

        let versions = VersionCache::new(Duration::from_secs(config.version_ttl_secs), http.clone());

        Ok(Self { http, versions })
    }

    /// Returns the current client version, auto-detected with a static fallback.
    pub async fn client_version(&self, cfg: &ServerConfig) -> String {
        self.versions.get(cfg).await
    }

    /// Fetches a URL with the given headers and returns the status and raw body.
    async fn fetch_raw(&self, url: &str, headers: HeaderMap) -> AppResult<FetchRaw> {
        let resp = self.http.get(url).headers(headers).send().await?;
        let status = resp.status().as_u16();
        let body = resp.bytes().await.map_err(|e| AppError::UpstreamError(format!("failed to read upstream body: {e}")))?;
        Ok(FetchRaw { status, body: body.to_vec() })
    }

    /// Decrypts a response body with the server's AES-128-CBC key/IV.
    fn decrypt(&self, cfg: &ServerConfig, body: &[u8]) -> AppResult<Vec<u8>> {
        decrypt_aes_128_cbc(&cfg.encryption_key, &cfg.encryption_iv, body).map_err(AppError::from)
    }

    fn expect_success(&self, raw: &FetchRaw) -> AppResult<()> {
        if (200..300).contains(&raw.status) {
            Ok(())
        } else {
            Err(AppError::Upstream(raw.status))
        }
    }

    /// Fetches a Garupa endpoint and returns the decrypted protobuf bytes.
    ///
    /// A 426 response means the game updated and the current client version is
    /// stale; the version cache is dropped, refetched from the store, and the
    /// request retried once before the error is surfaced.
    pub async fn fetch(&self, cfg: &ServerConfig, url: &str) -> AppResult<Vec<u8>> {
        let client_version = self.client_version(cfg).await;
        let headers = build_headers(cfg, &client_version, &[]);
        let raw = self.fetch_raw(url, headers).await?;
        if raw.status == 426 {
            self.versions.drop_cache();
            let client_version = self.client_version(cfg).await;
            let headers = build_headers(cfg, &client_version, &[]);
            let raw = self.fetch_raw(url, headers).await?;
            self.expect_success(&raw)?;
            return self.decrypt(cfg, &raw.body);
        }
        self.expect_success(&raw)?;
        self.decrypt(cfg, &raw.body)
    }

    /// Checks whether the server is reachable by hitting its `/application` endpoint.
    pub async fn check_health(&self, cfg: &ServerConfig) -> bool {
        let url = format!("{}application", cfg.base);
        let headers = build_headers(cfg, &cfg.client_version, &[]);
        match self.http.get(url).headers(headers).timeout(Duration::from_secs(3)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

// ============================================================================
// URL builders used by the API handlers
// ============================================================================

impl GarupaClient {
    pub fn monthly_ranking_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}monthlyranking", cfg.base)
    }

    pub fn monthly_ranking_url(&self, cfg: &ServerConfig, monthly_id: i64) -> String {
        format!("{}user/{}/monthlyranking/{}/ranking", cfg.base, cfg.uid, monthly_id)
    }

    pub fn event_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}event", cfg.base)
    }

    pub fn application_url(&self, cfg: &ServerConfig) -> String {
        format!("{}application", cfg.base)
    }

    pub fn music_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}music", cfg.base)
    }

    pub fn character_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}character", cfg.base)
    }

    pub fn band_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}band", cfg.base)
    }

    pub fn area_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}area", cfg.base)
    }

    pub fn gacha_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}gacha", cfg.base)
    }

    pub fn item_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}item", cfg.base)
    }

    pub fn skill_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}skill", cfg.base)
    }

    pub fn stamp_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}stamp", cfg.base)
    }

    pub fn login_bonus_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}loginbonus", cfg.base)
    }

    pub fn costume_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}costume", cfg.base)
    }

    pub fn user_profile_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}", cfg.base, cfg.uid)
    }

    pub fn user_deck_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/deck", cfg.base, cfg.uid)
    }

    pub fn user_situation_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/situation", cfg.base, cfg.uid)
    }

    pub fn user_title_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/title", cfg.base, cfg.uid)
    }

    pub fn user_stamp_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/stamp", cfg.base, cfg.uid)
    }

    pub fn user_area_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/area", cfg.base, cfg.uid)
    }

    pub fn user_item_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/item", cfg.base, cfg.uid)
    }

    pub fn user_present_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/present", cfg.base, cfg.uid)
    }

    pub fn user_gacha_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/gacha", cfg.base, cfg.uid)
    }

    pub fn shop_url(&self, cfg: &ServerConfig) -> String {
        format!("{}shop", cfg.base)
    }

    pub fn situation_master_url(&self, cfg: &ServerConfig) -> String {
        format!("{}situation", cfg.base)
    }

    pub fn user_episode_url(&self, cfg: &ServerConfig) -> String {
        format!("{}user/{}/episode", cfg.base, cfg.uid)
    }

    pub fn event_ranking_url(&self, cfg: &ServerConfig, event_id: i64, event_type: &str, mid: Option<i64>) -> String {
        let segment = event_type_to_url_segment(event_type);
        let mut url = format!("{}user/{}/event/{}/{}/ranking", cfg.base, cfg.uid, event_id, segment);
        if let Some(m) = mid {
            url.push_str(&format!("?mid={m}"));
        }
        url
    }
}

/// Maps a protobuf event type string to its API URL path segment.
fn event_type_to_url_segment(event_type: &str) -> &str {
    match event_type {
        "challenge" => "challenge",
        "live_try" => "livetry",
        "medley" => "medley",
        "mission_live" => "mission",
        "story" => "story",
        "team_live_festival" => "festival",
        "versus" => "versus",
        other => other,
    }
}
