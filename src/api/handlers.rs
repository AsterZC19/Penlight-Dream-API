//! Route handlers. Each handler fetches and decrypts the corresponding Garupa
//! endpoint, decodes the protobuf, maps it to a response model, and serves it
//! through the TTL cache.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::models;
use crate::api::server::resolve_server;
use crate::api::SharedState;
use crate::config::ServerConfig;
use crate::error::{AppError, AppResult};
use crate::proto::decoder::decode;
use crate::proto::garupa_schema::{
    APPLICATION_SCHEMA, AREA_LIST_SCHEMA, BAND_LIST_SCHEMA, CHARACTER_LIST_SCHEMA,
    COSTUME_LIST_SCHEMA, EVENT_TYPE_SCHEMAS, GACHA_LIST_SCHEMA, ITEM_LIST_SCHEMA,
    LOGIN_BONUS_LIST_SCHEMA, MASTER_EVENT_LIST_SCHEMA, MASTER_MONTHLY_RANKING_LIST_SCHEMA,
    MUSIC_LIST_SCHEMA, SHOP_LIST_SCHEMA, SITUATION_LIST_SCHEMA, SKILL_LIST_SCHEMA, STAMP_LIST_SCHEMA,
    USER_AREA_LIST_SCHEMA, USER_DECK_LIST_SCHEMA, USER_EPISODE_LIST_SCHEMA,
    USER_GACHA_LIST_SCHEMA, USER_ITEM_LIST_SCHEMA, USER_PRESENT_LIST_SCHEMA,
    USER_PROFILE_RESPONSE_SCHEMA, USER_SITUATION_LIST_SCHEMA, USER_STAMP_LIST_SCHEMA,
    USER_TITLE_SCHEMA, USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA,
};
use crate::proto::schema::Schema;

/// Records the process start time for the health uptime field.
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Live JP availability snapshot, cached so frequent health probes do not
/// hammer the upstream server with one `/application` call per request.
#[derive(Clone)]
struct HealthSnapshot {
    at: Instant,
    available: bool,
    client_version: String,
}

/// How long a health snapshot is reused before the upstream is re-checked.
const HEALTH_SNAPSHOT_TTL: Duration = Duration::from_secs(10);

static HEALTH_SNAPSHOT: OnceLock<Mutex<Option<HealthSnapshot>>> = OnceLock::new();

/// Returns the JP server config, or a 404 when it is not configured.
fn jp_config(state: &SharedState) -> AppResult<&ServerConfig> {
    if state.config.server.enabled() {
        Ok(&state.config.server)
    } else {
        Err(AppError::not_found("jp server is not configured"))
    }
}

// ============================================================================
// Shared fetch helpers
// ============================================================================

/// Builds a JSON response from a pre-serialized body.
fn json_response(body: String) -> Response {
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

/// Fetches a decoded protobuf response and returns its serialized JSON body
/// through the cache, so cache hits are served without re-parsing. The `map`
/// closure turns the decoded root into the response value. Concurrent cache
/// misses coalesce onto a single upstream call per cache window.
async fn cached_json(
    state: &SharedState,
    key: &str,
    ttl_secs: u64,
    url: &str,
    schema: &Schema,
    map: impl Fn(&Value, &[u8]) -> AppResult<Value>,
) -> AppResult<String> {
    if let Some(cached) = state.cache.get(key) {
        return Ok(cached);
    }
    let cfg = jp_config(state)?;
    state
        .coalescer
        .run(key, || async {
            let buf = state.client.fetch(cfg, url).await?;
            let root = decode(&buf, schema)?;
            let value = map(&root, &buf)?;
            let body = value.to_string();
            state.cache.set(key, &body, Duration::from_secs(ttl_secs));
            Ok(body)
        })
        .await
}

/// Serves the decoded list root as-is so list endpoints keep the game's
/// `entries` wrapper when present. User lists can legitimately return payloads
/// without an entries field when the account has no data, so anything the game
/// sends is passed through untouched rather than rejected.
fn wrapped_map(root: &Value, _raw: &[u8]) -> AppResult<Value> {
    Ok(root.clone())
}

/// Fetches a master list endpoint and serves its wrapped entries root, cached
/// under the master TTL.
async fn master_list(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_master_secs, url, schema, wrapped_map).await?;
    Ok(json_response(body))
}

/// Fetches a master endpoint and serves the decoded object as-is, cached under
/// the master TTL.
async fn master_fetch(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_master_secs, url, schema, |root, _| Ok(root.clone())).await?;
    Ok(json_response(body))
}

/// Fetches a user list endpoint and serves its wrapped entries root, cached
/// under the user data TTL.
async fn user_list(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_user_secs, url, schema, wrapped_map).await?;
    Ok(json_response(body))
}

/// Fetches a decoded user endpoint and serves the object as-is, cached under
/// the user data TTL.
async fn user_fetch(state: &SharedState, key: &str, url: &str, schema: &Schema) -> AppResult<Response> {
    let body = cached_json(state, key, state.config.cache_ttl_user_secs, url, schema, |root, _| Ok(root.clone())).await?;
    Ok(json_response(body))
}

// ============================================================================
// Server metadata
// ============================================================================

/// GET /servers — lists the configured server, omitting credentials and the
/// player UID so the endpoint can stay unauthenticated.
pub async fn servers(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    if !cfg.enabled() {
        return Json(Value::Array(Vec::new()));
    }
    Json(Value::Array(vec![json!({
        "index": 0,
        "name": "jp",
        "base": cfg.base,
    })]))
}

/// GET /health — process health envelope plus live JP availability check,
/// with the upstream probe cached for a short window.
pub async fn health(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    let (available, client_version) = {
        // Clone the snapshot out so the mutex guard is dropped before any
        // `.await` below; a `MutexGuard` is not `Send`.
        let snapshot = HEALTH_SNAPSHOT.get_or_init(|| Mutex::new(None)).lock().unwrap().clone();
        match snapshot {
            Some(s) if s.at.elapsed() < HEALTH_SNAPSHOT_TTL => (s.available, s.client_version),
            _ => {
                let available = if cfg.enabled() {
                    state.client.check_health(cfg).await
                } else {
                    false
                };
                let client_version = if cfg.enabled() {
                    state.client.client_version(cfg).await
                } else {
                    String::new()
                };
                if let Ok(mut guard) = HEALTH_SNAPSHOT.get_or_init(|| Mutex::new(None)).lock() {
                    *guard = Some(HealthSnapshot {
                        at: Instant::now(),
                        available,
                        client_version: client_version.clone(),
                    });
                }
                (available, client_version)
            }
        }
    };
    let uptime = START_TIME.get_or_init(Instant::now).elapsed().as_secs();
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime,
        "jp": { "available": available, "clientVersion": client_version },
    }))
}

/// GET /version — the current auto-detected client version.
pub async fn version(State(state): State<SharedState>) -> Json<Value> {
    let cfg = &state.config.server;
    let client_version = if cfg.enabled() {
        state.client.client_version(cfg).await
    } else {
        String::new()
    };
    Json(json!({ "jp": { "clientVersion": client_version } }))
}

// ============================================================================
// Monthly ranking
// ============================================================================

/// GET /api/{server}/monthly-ranking — master list of monthly ranking periods.
pub async fn monthly_ranking_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let body = cached_json(
        &state,
        "monthly-master",
        state.config.cache_ttl_master_secs,
        &state.client.monthly_ranking_master_url(cfg),
        &MASTER_MONTHLY_RANKING_LIST_SCHEMA,
        |root, _| Ok(json!({ "entries": serde_json::to_value(models::monthly_ranking_list(root))? })),
    )
    .await?;
    Ok(json_response(body))
}

/// Fetches and caches the serialized monthly ranking report for a period.
async fn fetch_monthly_body(state: &SharedState, monthly_id: i64) -> AppResult<String> {
    let cfg = jp_config(state)?;
    if monthly_id < 1 {
        return Err(AppError::bad_request("monthlyId must be >= 1"));
    }
    let key = format!("monthly:{monthly_id}");
    cached_json(
        state,
        &key,
        state.config.cache_ttl_ranking_secs,
        &state.client.monthly_ranking_url(cfg, monthly_id),
        &USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA,
        |root, _| Ok(serde_json::to_value(models::monthly_ranking_report(root))?),
    )
    .await
}

/// Fetches the monthly ranking report as a value for sub-endpoint extraction.
async fn fetch_monthly_ranking_value(state: &SharedState, monthly_id: i64) -> AppResult<Value> {
    let body = fetch_monthly_body(state, monthly_id).await?;
    Ok(serde_json::from_str(&body)?)
}

/// GET /api/{server}/monthly-ranking/{monthly_id} — near/top/border users.
pub async fn monthly_ranking_full(State(state): State<SharedState>, Path((_server, monthly_id)): Path<(String, i64)>) -> AppResult<Response> {
    let body = fetch_monthly_body(&state, monthly_id).await?;
    Ok(json_response(body))
}

/// GET /api/{server}/monthly-ranking/{monthly_id}/top — top users only.
pub async fn monthly_ranking_top(State(state): State<SharedState>, Path((_server, monthly_id)): Path<(String, i64)>) -> AppResult<Json<Value>> {
    let full = fetch_monthly_ranking_value(&state, monthly_id).await?;
    let users = full.get("monthlyRankingPointTopUsers").cloned().unwrap_or_else(|| Value::Array(Vec::new()));
    Ok(Json(json!({ "users": users })))
}

/// GET /api/{server}/monthly-ranking/{monthly_id}/border — border users only.
pub async fn monthly_ranking_border(State(state): State<SharedState>, Path((_server, monthly_id)): Path<(String, i64)>) -> AppResult<Json<Value>> {
    let full = fetch_monthly_ranking_value(&state, monthly_id).await?;
    let users = full.get("monthlyRankingPointBorderUsers").cloned().unwrap_or_else(|| Value::Array(Vec::new()));
    Ok(Json(json!({ "users": users })))
}

// ============================================================================
// Event
// ============================================================================

/// GET /api/{server}/events — master list of events.
pub async fn event_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let body = cached_json(
        &state,
        "event-master",
        state.config.cache_ttl_master_secs,
        &state.client.event_master_url(cfg),
        &MASTER_EVENT_LIST_SCHEMA,
        |root, _| Ok(json!({ "entries": serde_json::to_value(models::event_list(root))? })),
    )
    .await?;
    Ok(json_response(body))
}

/// Fetches the event master list as a bare entries array for type resolution.
async fn fetch_event_master_value(state: &SharedState) -> AppResult<Value> {
    let cfg = jp_config(state)?;
    let body = cached_json(
        state,
        "event-master",
        state.config.cache_ttl_master_secs,
        &state.client.event_master_url(cfg),
        &MASTER_EVENT_LIST_SCHEMA,
        |root, _| Ok(json!({ "entries": serde_json::to_value(models::event_list(root))? })),
    )
    .await?;
    let wrapped: Value = serde_json::from_str(&body)?;
    Ok(wrapped.get("entries").cloned().unwrap_or_else(|| Value::Array(Vec::new())))
}

#[derive(Debug, Deserialize)]
pub struct EventRankingQuery {
    /// Protobuf event type such as "medley" or "versus". When omitted, it is
    /// resolved from the event master list.
    pub r#type: Option<String>,
    /// Optional music ID for challenge/versus per-song sub-rankings.
    pub mid: Option<i64>,
}

/// GET /api/{server}/events/{event_id}/ranking — event ranking for a specific event.
pub async fn event_ranking(
    State(state): State<SharedState>,
    Path((_server, event_id)): Path<(String, i64)>,
    Query(query): Query<EventRankingQuery>,
) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    if event_id < 1 {
        return Err(AppError::bad_request("eventId must be >= 1"));
    }

    let event_type = match &query.r#type {
        Some(t) => t.clone(),
        None => {
            let events = fetch_event_master_value(&state).await?;
            let entry = events
                .as_array()
                .and_then(|arr| arr.iter().find(|e| e.get("eventId").and_then(Value::as_i64) == Some(event_id)));
            match entry.and_then(|e| e.get("eventType").and_then(Value::as_str)) {
                Some(t) => t.to_string(),
                None => return Err(AppError::not_found(format!("event {event_id} not found"))),
            }
        }
    };

    let schema = match EVENT_TYPE_SCHEMAS.iter().find(|entry| entry.0 == event_type.as_str()) {
        Some(entry) => entry.1,
        None => {
            let supported: Vec<&str> = EVENT_TYPE_SCHEMAS.iter().map(|entry| entry.0).collect();
            return Err(AppError::bad_request(format!(
                "unsupported event type \"{event_type}\", supported: {}",
                supported.join(", ")
            )));
        }
    };

    let key = format!("event-ranking:{event_id}:{event_type}:{}", query.mid.unwrap_or(0));
    let url = state.client.event_ranking_url(cfg, event_id, &event_type, query.mid);
    let body = cached_json(
        &state,
        &key,
        state.config.cache_ttl_ranking_secs,
        &url,
        schema,
        |root, _| Ok(serde_json::to_value(models::event_ranking_report(root, &event_type))?),
    )
    .await?;
    Ok(json_response(body))
}

// ============================================================================
// Application
// ============================================================================

/// GET /api/{server}/application — app version, server status, and per-platform maintenance.
pub async fn application(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_fetch(&state, "application", &state.client.application_url(cfg), &APPLICATION_SCHEMA).await
}

// ============================================================================
// Master data
// ============================================================================

/// GET /api/{server}/music — music master list.
pub async fn music_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "music-master", &state.client.music_master_url(cfg), &MUSIC_LIST_SCHEMA).await
}

/// GET /api/{server}/characters — character master list.
pub async fn character_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "character-master", &state.client.character_master_url(cfg), &CHARACTER_LIST_SCHEMA).await
}

/// GET /api/{server}/bands — band master list.
pub async fn band_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "band-master", &state.client.band_master_url(cfg), &BAND_LIST_SCHEMA).await
}

/// GET /api/{server}/areas — area master list.
pub async fn area_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "area-master", &state.client.area_master_url(cfg), &AREA_LIST_SCHEMA).await
}

/// GET /api/{server}/gacha — gacha master list.
pub async fn gacha_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "gacha-master", &state.client.gacha_master_url(cfg), &GACHA_LIST_SCHEMA).await
}

/// GET /api/{server}/items — item master list.
pub async fn item_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "item-master", &state.client.item_master_url(cfg), &ITEM_LIST_SCHEMA).await
}

/// GET /api/{server}/skills — skill master list.
pub async fn skill_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "skill-master", &state.client.skill_master_url(cfg), &SKILL_LIST_SCHEMA).await
}

/// GET /api/{server}/stamps — stamp master list.
pub async fn stamp_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "stamp-master", &state.client.stamp_master_url(cfg), &STAMP_LIST_SCHEMA).await
}

/// GET /api/{server}/login-bonuses — login bonus master list.
pub async fn login_bonus_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "loginbonus-master", &state.client.login_bonus_master_url(cfg), &LOGIN_BONUS_LIST_SCHEMA).await
}

/// GET /api/{server}/costumes — costume master list.
pub async fn costume_master(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "costume-master", &state.client.costume_master_url(cfg), &COSTUME_LIST_SCHEMA).await
}

/// GET /api/{server}/shops — shop master list.
pub async fn shops(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "shop-master", &state.client.shop_url(cfg), &SHOP_LIST_SCHEMA).await
}

/// GET /api/{server}/cards — card master list. The game serves cards under the
/// internal name `situation`, so this maps to the situation master URL.
pub async fn cards(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    master_list(&state, "situation-master", &state.client.situation_master_url(cfg), &SITUATION_LIST_SCHEMA).await
}

// ============================================================================
// User data
// ============================================================================

/// GET /api/{server}/user/profile — the configured user's profile and stats.
pub async fn user_profile(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-profile:{}", cfg.uid);
    user_fetch(&state, &key, &state.client.user_profile_url(cfg), &USER_PROFILE_RESPONSE_SCHEMA).await
}

/// GET /api/{server}/user/decks — the configured user's decks.
pub async fn user_decks(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-decks:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_deck_url(cfg), &USER_DECK_LIST_SCHEMA).await
}

/// GET /api/{server}/user/situations — the configured user's owned cards.
pub async fn user_situations(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-situations:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_situation_url(cfg), &USER_SITUATION_LIST_SCHEMA).await
}

/// GET /api/{server}/user/title — the configured user's equipped title.
pub async fn user_title(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-title:{}", cfg.uid);
    user_fetch(&state, &key, &state.client.user_title_url(cfg), &USER_TITLE_SCHEMA).await
}

/// GET /api/{server}/user/stamps — the configured user's stamps.
pub async fn user_stamps(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-stamps:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_stamp_url(cfg), &USER_STAMP_LIST_SCHEMA).await
}

/// GET /api/{server}/user/areas — the configured user's area items.
pub async fn user_areas(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-areas:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_area_url(cfg), &USER_AREA_LIST_SCHEMA).await
}

/// GET /api/{server}/user/items — the configured user's item balances.
pub async fn user_items(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-items:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_item_url(cfg), &USER_ITEM_LIST_SCHEMA).await
}

/// GET /api/{server}/user/presents — the configured user's presents.
pub async fn user_presents(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-presents:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_present_url(cfg), &USER_PRESENT_LIST_SCHEMA).await
}

/// GET /api/{server}/user/gacha — the configured user's gacha records.
pub async fn user_gacha(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-gacha:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_gacha_url(cfg), &USER_GACHA_LIST_SCHEMA).await
}

/// GET /api/{server}/user/episodes — the configured user's unlocked episodes.
pub async fn user_episodes(State(state): State<SharedState>) -> AppResult<Response> {
    let cfg = jp_config(&state)?;
    let key = format!("user-episodes:{}", cfg.uid);
    user_list(&state, &key, &state.client.user_episode_url(cfg), &USER_EPISODE_LIST_SCHEMA).await
}

// ============================================================================
// Static resources
// ============================================================================

/// GET /image/{server}/{asset_kind}/{asset_id} — placeholder for any static
/// asset. The game API exposes no static-serving endpoints, so this route never
/// fetches real bytes; it confirms the resource identity and signals that the
/// content is not served.
pub async fn image_placeholder(
    Path((server, asset_kind, asset_id)): Path<(String, String, String)>,
) -> AppResult<Json<Value>> {
    resolve_server(&server)?;
    Ok(Json(json!({
        "placeholder": true,
        "assetKind": asset_kind,
        "assetId": asset_id,
        "message": "static resources are not served",
    })))
}

// ============================================================================
// Cache
// ============================================================================

/// GET /api/{server}/cache — cache diagnostics.
pub async fn cache_stats(State(state): State<SharedState>) -> Json<Value> {
    Json(json!({ "entries": state.cache.len() }))
}

/// DELETE /api/{server}/cache — clears the response cache.
pub async fn cache_clear(State(state): State<SharedState>) -> Json<Value> {
    state.cache.clear();
    Json(json!({ "cleared": true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_map_preserves_entries_root() {
        let root = json!({ "entries": [1, 2, 3] });
        assert_eq!(wrapped_map(&root, &[]).unwrap(), root);
    }

    #[test]
    fn wrapped_map_passes_through_empty_payload() {
        assert_eq!(wrapped_map(&Value::Object(Default::default()), &[]).unwrap(), json!({}));
    }

    #[test]
    fn wrapped_map_passes_through_non_entries_payload() {
        let root = json!({ "presentBox": { "slotCount": 100 } });
        let raw = [0x12, 0x06, 0x08, 0x64];
        assert_eq!(wrapped_map(&root, &raw).unwrap(), root);
    }
}
