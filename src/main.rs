mod api;
mod cache;
mod client;
mod config;
mod crypto;
mod error;
mod proto;

use std::net::SocketAddr;
use std::sync::Arc;

use api::{routes, AppState};
use cache::{Cache, Coalescer};
use client::GarupaClient;
use config::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("configuration error: {e}");
            std::process::exit(1);
        }
    };

    init_tracing(&config.log_level);

    let client = match GarupaClient::new(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("client error: {e}");
            std::process::exit(1);
        }
    };

    let state = Arc::new(AppState { config, client, cache: Cache::new(), coalescer: Coalescer::new() });

    if state.config.server.enabled() {
        info!("JP server configured");
    } else {
        info!("JP server not configured");
    }

    let addr: SocketAddr = format!("{}:{}", state.config.host, state.config.port)
        .parse()
        .expect("invalid bind address");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    info!(%addr, "penlight-dream-api listening");
    info!(prefix = %state.config.api_prefix, "API prefix");

    axum::serve(listener, routes::build(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutting down");
}

/// Live discovery probe for Garupa API endpoints. Ignored by default; run with
/// `cargo test -- --ignored probe_endpoints`. Requires a configured `.env`.
#[cfg(test)]
mod live_probe {
    use crate::client::GarupaClient;
    use crate::config::Config;
    use crate::proto::decoder::{dump_raw, entries_containing_fields, first_entry_dump, top_field_union};

    const MASTER_CANDIDATES: &[&str] = &[
        "application",
        "information",
        "announcement",
        "music",
        "card",
        "character",
        "band",
        "area",
        "gacha",
        "live",
        "story",
        "course",
        "item",
        "skill",
        "stamp",
        "title",
        "login",
        "loginbonus",
        "musicDifficulty",
        "characterSituation",
        "characterEpisode",
        "honor",
        "avatar",
        "frame",
    ];

    const MASTER_WAVE2: &[&str] = &[
        "cardList",
        "cards",
        "musicList",
        "musicTag",
        "episode",
        "mainStory",
        "subStory",
        "specialStory",
        "eventStory",
        "storyMain",
        "storySub",
        "storySpecial",
        "mainStoryEpisode",
        "subStoryEpisode",
        "gachaRate",
        "gachaPickup",
        "gachaItem",
        "booth",
        "present",
        "campaign",
        "banner",
        "boundary",
        "boost",
        "recoverItem",
        "areaItem",
        "areaObject",
        "costume",
        "galaxy",
        "liveSetting",
        "liveSchedule",
        "placet",
        "battle",
        "rankMatch",
        "arena",
        "tournament",
        "challengeLive",
        "album",
        "friendGift",
        "room",
        "roomItem",
        "characterRank",
        "live2d",
        "skillList",
        "itemList",
        "titleList",
        "plate",
    ];

    const USER_WAVE2: &[&str] = &[
        "user/{uid}",
        "user/{uid}/profile",
        "user/{uid}/userProfile",
        "user/{uid}/deck",
        "user/{uid}/deck/1",
        "user/{uid}/situation",
        "user/{uid}/situation/1",
        "user/{uid}/title",
        "user/{uid}/titleList",
        "user/{uid}/stamp",
        "user/{uid}/stampList",
        "user/{uid}/card",
        "user/{uid}/music",
        "user/{uid}/live",
        "user/{uid}/episode",
        "user/{uid}/friend",
        "user/{uid}/boost",
        "user/{uid}/area",
        "user/{uid}/band",
        "user/{uid}/item",
        "user/{uid}/gacha",
        "user/{uid}/present",
        "user/{uid}/booth",
        "user/{uid}/placet",
        "user/{uid}/battle",
        "user/{uid}/rankMatch",
    ];

    const USER_WAVE3: &[&str] = &[
        "user/{uid}/login",
        "user/{uid}/monthlyranking",
        "user/{uid}/event",
        "user/{uid}/friendList",
        "user/{uid}/situationList",
        "user/{uid}/boostItem",
        "user/{uid}/galaxy",
        "user/{uid}/music",
        "user/{uid}/stamp/1",
        "user/{uid}/card/1",
    ];

    /// Structural endpoints probed to understand the API's shape, such as a
    /// master-data manifest or resource listing. Mostly expected to 404.
    const META_ENDPOINTS: &[&str] = &[
        "master",
        "masterdata",
        "masterData",
        "masterVersion",
        "master-version",
        "masterVersionInfo",
        "resource",
        "resources",
        "static",
        "staticData",
        "asset",
        "assets",
        "assetBundle",
        "assetbundles",
        "cdn",
        "content",
        "contents",
        "booth",
        "shop",
        "placet",
        "sticker",
        "favorite",
        "gift",
        "mission",
        "banner",
        "campaign",
    ];

    /// Every plausible master table name. Confirmed tables are annotated.
    const MASTER_TABLES: &[&str] = &[
        "application",
        "music",
        "character",
        "band",
        "area",
        "gacha",
        "item",
        "skill",
        "stamp",
        "loginbonus",
        "costume",
        "event",
        "monthlyranking",
        "card",
        "cardRarity",
        "cardEpisode",
        "cardList",
        "cards",
        "musicTag",
        "musicDifficulty",
        "musicInstrument",
        "musicCategory",
        "musicVideo",
        "musicList",
        "characterProfile",
        "characterEpisode",
        "characterSituation",
        "characterRank",
        "characterCostume",
        "characterList",
        "situation",
        "bandMember",
        "bandLevel",
        "bandList",
        "areaItem",
        "areaItemLevel",
        "areaObject",
        "areaList",
        "eventStory",
        "eventPointReward",
        "eventRankingReward",
        "eventExchange",
        "eventList",
        "gachaRate",
        "gachaType",
        "gachaPickup",
        "gachaCeilItem",
        "gachaList",
        "loginBonusReward",
        "loginbonuslist",
        "loginBonusList",
        "costume3D",
        "costumeList",
        "title",
        "titleRelease",
        "titleList",
        "honor",
        "honorGroup",
        "honorList",
        "present",
        "presentType",
        "presentList",
        "live",
        "liveSetting",
        "liveSchedule",
        "liveList",
        "story",
        "storyMain",
        "storySub",
        "storySpecial",
        "storyEpisode",
        "storyList",
        "course",
        "courseList",
        "boost",
        "boostItem",
        "recoverItem",
        "room",
        "roomItem",
        "challengeLive",
        "rankMatch",
        "arena",
        "album",
        "galaxy",
        "plate",
        "mission",
        "missionReward",
        "friendGift",
        "sticker",
        "live2d",
        "monthlyRankingReward",
        "monthlyRankingGrade",
    ];

    /// Plausible per-user sub-endpoints not yet covered by the earlier waves.
    const USER_ENDPOINTS: &[&str] = &[
        "user/{uid}",
        "user/{uid}/profile",
        "user/{uid}/userProfile",
        "user/{uid}/status",
        "user/{uid}/home",
        "user/{uid}/deck",
        "user/{uid}/deckList",
        "user/{uid}/situation",
        "user/{uid}/situationList",
        "user/{uid}/title",
        "user/{uid}/titleList",
        "user/{uid}/stamp",
        "user/{uid}/stampList",
        "user/{uid}/card",
        "user/{uid}/cardList",
        "user/{uid}/music",
        "user/{uid}/live",
        "user/{uid}/episode",
        "user/{uid}/friend",
        "user/{uid}/friendList",
        "user/{uid}/boost",
        "user/{uid}/boostItem",
        "user/{uid}/recoverItem",
        "user/{uid}/area",
        "user/{uid}/band",
        "user/{uid}/item",
        "user/{uid}/itemList",
        "user/{uid}/gacha",
        "user/{uid}/gachaRate",
        "user/{uid}/present",
        "user/{uid}/presentType",
        "user/{uid}/booth",
        "user/{uid}/placet",
        "user/{uid}/battle",
        "user/{uid}/rankMatch",
        "user/{uid}/arena",
        "user/{uid}/challengeLive",
        "user/{uid}/room",
        "user/{uid}/galaxy",
        "user/{uid}/honor",
        "user/{uid}/honorList",
        "user/{uid}/sticker",
        "user/{uid}/login",
        "user/{uid}/monthlyranking",
        "user/{uid}/event",
    ];

    /// Endpoints that would serve static content such as images or assets.
    /// These are probed only for existence; their bodies are never persisted.
    const STATIC_ENDPOINTS: &[&str] = &[
        "image",
        "image/character",
        "image/music",
        "image/jacket",
        "static",
        "static/character",
        "asset",
        "asset/music",
        "assets",
        "resource",
        "resources",
        "cdn",
        "cdn/music",
        "content",
        "contents",
        "download",
        "download/music",
        "masterdata",
    ];

    /// Card-related master names probed after the main scan, because no confirmed
    /// endpoint returns card ability values or skill mappings.
    const CARD_NAMES: &[&str] = &[
        "card",
        "cards",
        "cardList",
        "cardRarity",
        "cardEpisode",
        "cardSkill",
        "cardTag",
        "cardType",
        "cardMaster",
        "allCard",
        "situation",
        "situationList",
        "situationMaster",
        "cardSituation",
        "member",
        "memberCard",
        "memberSituation",
        "cardAppendParameter",
        "situationAppendParameter",
    ];

    #[tokio::test]
    #[ignore = "live network probe"]
    async fn probe_endpoints() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            eprintln!("server disabled, skipping probe");
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        for path in MASTER_CANDIDATES.iter().chain(MASTER_WAVE2.iter()) {
            let url = format!("{}{}", cfg.base, path);
            probe_one(&client, cfg, path, &url).await;
        }

        for path in USER_WAVE2.iter().chain(USER_WAVE3.iter()) {
            let url = format!("{}{}", cfg.base, path.replace("{uid}", &cfg.uid));
            probe_one(&client, cfg, path, &url).await;
        }
    }

    async fn probe_one(client: &GarupaClient, cfg: &crate::config::ServerConfig, label: &str, url: &str) {
        match client.fetch(cfg, url).await {
            Ok(buf) => {
                eprintln!("=== OK {label} ({url}) — {} bytes ===", buf.len());
                let full = dump_raw(&buf).to_string();
                if full.len() > 4000 {
                    if let Some(record) = first_entry_dump(&buf) {
                        eprintln!("--- first record ---");
                        eprintln!("{record}");
                    } else {
                        eprintln!("{full}");
                    }
                    eprintln!("--- top field union over 3 records ---");
                    eprintln!("{}", top_field_union(&buf, 3));
                } else {
                    eprintln!("{full}");
                }
            }
            Err(e) => {
                eprintln!("=== FAIL {label} ({url}): {e} ===");
            }
        }
    }

    /// Prints one machine-parseable scan line for an endpoint probe.
    async fn scan_one(client: &GarupaClient, cfg: &crate::config::ServerConfig, group: &str, label: &str, url: &str) {
        match client.fetch(cfg, url).await {
            Ok(buf) => {
                eprintln!(
                    "SCAN {group} OK {label} bytes={} union={}",
                    buf.len(),
                    if buf.len() > 64 { top_field_union(&buf, 3).to_string() } else { dump_raw(&buf).to_string() }
                );
            }
            Err(e) => {
                eprintln!("SCAN {group} FAIL {label} err={e}");
            }
        }
    }

    /// Probes every candidate endpoint and records which exist. Static-resource
    /// bodies are never written to disk; only their existence and field shape
    /// are reported so the API can placeholder them later.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn comprehensive_scan() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            eprintln!("server disabled, skipping probe");
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        eprintln!("--- master meta ---");
        for path in META_ENDPOINTS {
            scan_one(&client, cfg, "META", path, &format!("{}{}", cfg.base, path)).await;
        }
        eprintln!("--- master tables ---");
        for path in MASTER_TABLES {
            scan_one(&client, cfg, "MASTER", path, &format!("{}{}", cfg.base, path)).await;
        }
        eprintln!("--- user endpoints ---");
        for path in USER_ENDPOINTS {
            let url = format!("{}{}", cfg.base, path.replace("{uid}", &cfg.uid));
            scan_one(&client, cfg, "USER", path, &url).await;
        }
        eprintln!("--- static endpoints ---");
        for path in STATIC_ENDPOINTS {
            let url = format!("{}{}", cfg.base, path.replace("{uid}", &cfg.uid));
            scan_one(&client, cfg, "STATIC", path, &url).await;
        }
        eprintln!("--- scan complete ---");
    }

    /// Dumps the first card master entry from the `situation` master endpoint
    /// so its ability and skill fields can be mapped.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn dump_situation_master() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;
        let url = format!("{}situation", cfg.base);
        match client.fetch(cfg, &url).await {
            Ok(buf) => {
                eprintln!("situation master bytes={}", buf.len());
                if let Some(record) = first_entry_dump(&buf) {
                    eprintln!("--- first card ---");
                    eprintln!("{record}");
                }
                eprintln!("--- top field union over 3 records ---");
                eprintln!("{}", top_field_union(&buf, 3));
            }
            Err(e) => eprintln!("FAIL: {e}"),
        }
    }

    /// Dumps raw entries exposing fields missing from the ported schemas:
    /// band field 12, gacha fields 12/42, and the rare situation fields 14/15
    /// for episodes and training.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn dump_missing_fields() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            eprintln!("server disabled, skipping");
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        if let Ok(buf) = client.fetch(cfg, &format!("{}band", cfg.base)).await {
            eprintln!("--- band first entry (field 12) ---");
            if let Some(e) = first_entry_dump(&buf) {
                eprintln!("{e}");
            }
        }

        if let Ok(buf) = client.fetch(cfg, &format!("{}gacha", cfg.base)).await {
            eprintln!("--- gacha first entry (fields 12/42 only) ---");
            if let Some(e) = first_entry_dump(&buf) {
                for f in e.as_array().map(|a| a.iter().filter(|f| {
                    f.get("field").and_then(|v| v.as_u64()).map(|n| n == 12 || n == 42).unwrap_or(false)
                }).collect::<Vec<_>>()).unwrap_or_default() {
                    eprintln!("{f}");
                }
            }
        }

        if let Ok(buf) = client.fetch(cfg, &format!("{}situation", cfg.base)).await {
            eprintln!("--- situation entries with rare fields 14/15 ---");
            for e in entries_containing_fields(&buf, &[14, 15], 4) {
                eprintln!("{e}");
            }
        }
    }

    /// Probes card-related master names to check whether card data is exposed
    /// under any naming variant.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn probe_card_names() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;
        for path in CARD_NAMES {
            let url = format!("{}{}", cfg.base, path);
            scan_one(&client, cfg, "CARD", path, &url).await;
        }
        eprintln!("--- card probe complete ---");
    }

    /// Decodes the situation master cards and reports how many cards have a
    /// second skill reference that diverges from the primary skillId.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn probe_card_skills() {
        use crate::proto::decoder::decode;
        use crate::proto::garupa_schema::SITUATION_LIST_SCHEMA;
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        let buf = client.fetch(cfg, &format!("{}situation", cfg.base)).await.unwrap();
        let root = decode(&buf, &SITUATION_LIST_SCHEMA).unwrap();
        let entries = root["entries"].as_array().cloned().unwrap_or_default();
        eprintln!("total cards={}", entries.len());
        let mut diverged = 0;
        for entry in &entries {
            let f7 = entry["skillId"].as_i64().unwrap_or(-1);
            let f18 = entry["skillId2"].as_i64().unwrap_or(-1);
            if f7 != f18 {
                diverged += 1;
                eprintln!(
                    "DIVERGE card {} f7={} f18={} name={}",
                    entry["situationId"], f7, f18, entry["cardName"]
                );
            }
        }
        eprintln!("diverged count={diverged}");
    }

    /// Full raw dump of the newly discovered `shop` and user episode endpoints.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn dump_new_endpoints() {
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;
        for (label, url) in [
            ("shop", format!("{}shop", cfg.base)),
            ("user/episode", format!("{}user/{}/episode", cfg.base, cfg.uid)),
            ("user/title", format!("{}user/{}/title", cfg.base, cfg.uid)),
            ("user/present", format!("{}user/{}/present", cfg.base, cfg.uid)),
        ] {
            probe_one(&client, cfg, label, &url).await;
        }
    }

    /// Decodes every implemented endpoint with its production schema and
    /// prints a compact summary, validating the field mappings against live data.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn decode_check() {
        use crate::proto::garupa_schema::{
            APPLICATION_SCHEMA, AREA_LIST_SCHEMA, BAND_LIST_SCHEMA, CHARACTER_LIST_SCHEMA,
            COSTUME_LIST_SCHEMA, GACHA_LIST_SCHEMA, ITEM_LIST_SCHEMA, LOGIN_BONUS_LIST_SCHEMA,
            MUSIC_LIST_SCHEMA, SHOP_LIST_SCHEMA, SITUATION_LIST_SCHEMA, SKILL_LIST_SCHEMA,
            STAMP_LIST_SCHEMA,
            USER_AREA_LIST_SCHEMA, USER_DECK_LIST_SCHEMA, USER_EPISODE_LIST_SCHEMA,
            USER_GACHA_LIST_SCHEMA, USER_ITEM_LIST_SCHEMA, USER_PRESENT_LIST_SCHEMA,
            USER_PROFILE_RESPONSE_SCHEMA, USER_SITUATION_LIST_SCHEMA, USER_STAMP_LIST_SCHEMA,
            USER_TITLE_SCHEMA,
        };
        use crate::proto::schema::Schema;

        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            eprintln!("server disabled, skipping decode check");
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        let cases: Vec<(&str, String, &Schema)> = vec![
            ("application", client.application_url(cfg), &APPLICATION_SCHEMA),
            ("music", client.music_master_url(cfg), &MUSIC_LIST_SCHEMA),
            ("characters", client.character_master_url(cfg), &CHARACTER_LIST_SCHEMA),
            ("bands", client.band_master_url(cfg), &BAND_LIST_SCHEMA),
            ("areas", client.area_master_url(cfg), &AREA_LIST_SCHEMA),
            ("gacha", client.gacha_master_url(cfg), &GACHA_LIST_SCHEMA),
            ("items", client.item_master_url(cfg), &ITEM_LIST_SCHEMA),
            ("skills", client.skill_master_url(cfg), &SKILL_LIST_SCHEMA),
            ("stamps", client.stamp_master_url(cfg), &STAMP_LIST_SCHEMA),
            ("loginbonus", client.login_bonus_master_url(cfg), &LOGIN_BONUS_LIST_SCHEMA),
            ("costumes", client.costume_master_url(cfg), &COSTUME_LIST_SCHEMA),
            ("shops", client.shop_url(cfg), &SHOP_LIST_SCHEMA),
            ("cards", client.situation_master_url(cfg), &SITUATION_LIST_SCHEMA),
            ("user/profile", client.user_profile_url(cfg), &USER_PROFILE_RESPONSE_SCHEMA),
            ("user/decks", client.user_deck_url(cfg), &USER_DECK_LIST_SCHEMA),
            ("user/situations", client.user_situation_url(cfg), &USER_SITUATION_LIST_SCHEMA),
            ("user/title", client.user_title_url(cfg), &USER_TITLE_SCHEMA),
            ("user/stamps", client.user_stamp_url(cfg), &USER_STAMP_LIST_SCHEMA),
            ("user/areas", client.user_area_url(cfg), &USER_AREA_LIST_SCHEMA),
            ("user/items", client.user_item_url(cfg), &USER_ITEM_LIST_SCHEMA),
            ("user/presents", client.user_present_url(cfg), &USER_PRESENT_LIST_SCHEMA),
            ("user/gacha", client.user_gacha_url(cfg), &USER_GACHA_LIST_SCHEMA),
            ("user/episodes", client.user_episode_url(cfg), &USER_EPISODE_LIST_SCHEMA),
        ];

        for (label, url, schema) in cases {
            match client.fetch(cfg, &url).await {
                Ok(buf) => match crate::proto::decoder::decode(&buf, schema) {
                    Ok(value) => eprintln!("=== DECODED {label}: {}", summarize(&value, 5)),
                    Err(e) => eprintln!("=== DECODE-ERR {label}: {e}"),
                },
                Err(e) => eprintln!("=== FETCH-ERR {label}: {e}"),
            }
        }
    }

    /// Checks whether the situations endpoint returns the full card list or is
    /// paginated or capped. Probes query-parameter variants and counts entries.
    #[tokio::test]
    #[ignore = "live network probe"]
    async fn probe_situations() {
        use crate::proto::garupa_schema::USER_SITUATION_LIST_SCHEMA;
        dotenvy::from_path(".env").ok();
        let config = Config::from_env().expect("config");
        if !config.server.enabled() {
            eprintln!("server disabled, skipping");
            return;
        }
        let client = GarupaClient::new(&config).expect("client");
        let cfg = &config.server;

        let base = format!("{}user/{}/situation", cfg.base, cfg.uid);
        let variants = [
            ("plain", base.clone()),
            ("count=100", format!("{base}?count=100")),
            ("limit=100", format!("{base}?limit=100")),
            ("page=1", format!("{base}?page=1")),
            ("page=2", format!("{base}?page=2")),
            ("offset=5", format!("{base}?offset=5")),
            ("per_page=100", format!("{base}?per_page=100")),
            ("situation/max", format!("{}user/{}/situation/max", cfg.base, cfg.uid)),
        ];
        for (label, url) in variants {
            match client.fetch(cfg, &url).await {
                Ok(buf) => match crate::proto::decoder::decode(&buf, &USER_SITUATION_LIST_SCHEMA) {
                    Ok(root) => {
                        let entries = root["entries"].as_array().cloned().unwrap_or_default();
                        let ids: Vec<&serde_json::Value> = entries
                            .iter()
                            .filter_map(|e| e.get("situationId"))
                            .take(12)
                            .collect();
                        eprintln!(
                            "=== {label}: OK, {} entries, ids=[{}]",
                            entries.len(),
                            ids.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                    }
                    Err(e) => eprintln!("=== {label}: decode err {e}"),
                },
                Err(e) => eprintln!("=== {label}: FAIL {e}"),
            }
        }
    }

    /// Renders a compact one-line summary of a decoded value.
    fn summarize(v: &serde_json::Value, depth: usize) -> String {        fn walk(v: &serde_json::Value, depth: usize, out: &mut Vec<String>) {
            match v {
                serde_json::Value::Object(map) if depth > 0 => {
                    for (k, val) in map {
                        match val {
                            serde_json::Value::Array(arr) if !arr.is_empty() => {
                                let mut inner = Vec::new();
                                walk(&arr[0], depth - 1, &mut inner);
                                let joined = inner.join(",");
                                out.push(format!("{k}=[{joined}]"));
                            }
                            serde_json::Value::Object(_) => {
                                let mut inner = Vec::new();
                                walk(val, depth - 1, &mut inner);
                                let joined = inner.join(",");
                                out.push(format!("{k}={{{joined}}}"));
                            }
                            other => out.push(format!("{k}={other}")),
                        }
                    }
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        walk(v, depth, &mut out);
        if out.is_empty() {
            v.to_string()
        } else {
            out.join(",")
        }
    }
}
