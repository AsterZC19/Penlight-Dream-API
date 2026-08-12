use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::auth;
use super::handlers;
use super::server::validate_server;
use super::SharedState;

/// Builds the top-level axum router.
pub fn build(state: SharedState) -> Router {
    let api_prefix = state.config.api_prefix.clone();

    let api = Router::new()
        .route("/{server}/application", get(handlers::application))
        .route("/{server}/shops", get(handlers::shops))
        .route("/{server}/cards", get(handlers::cards))
        .route("/{server}/music", get(handlers::music_master))
        .route("/{server}/characters", get(handlers::character_master))
        .route("/{server}/bands", get(handlers::band_master))
        .route("/{server}/areas", get(handlers::area_master))
        .route("/{server}/gacha", get(handlers::gacha_master))
        .route("/{server}/items", get(handlers::item_master))
        .route("/{server}/skills", get(handlers::skill_master))
        .route("/{server}/stamps", get(handlers::stamp_master))
        .route("/{server}/login-bonuses", get(handlers::login_bonus_master))
        .route("/{server}/costumes", get(handlers::costume_master))
        .route("/{server}/events", get(handlers::event_master))
        .route("/{server}/events/{event_id}/ranking", get(handlers::event_ranking))
        .route("/{server}/monthly-ranking", get(handlers::monthly_ranking_master))
        .route("/{server}/monthly-ranking/{monthly_id}", get(handlers::monthly_ranking_full))
        .route("/{server}/monthly-ranking/{monthly_id}/top", get(handlers::monthly_ranking_top))
        .route("/{server}/monthly-ranking/{monthly_id}/border", get(handlers::monthly_ranking_border))
        .route("/{server}/user/profile", get(handlers::user_profile))
        .route("/{server}/user/decks", get(handlers::user_decks))
        .route("/{server}/user/situations", get(handlers::user_situations))
        .route("/{server}/user/title", get(handlers::user_title))
        .route("/{server}/user/stamps", get(handlers::user_stamps))
        .route("/{server}/user/areas", get(handlers::user_areas))
        .route("/{server}/user/items", get(handlers::user_items))
        .route("/{server}/user/presents", get(handlers::user_presents))
        .route("/{server}/user/gacha", get(handlers::user_gacha))
        .route("/{server}/user/episodes", get(handlers::user_episodes))
        .route("/{server}/cache", get(handlers::cache_stats).delete(handlers::cache_clear))
        .with_state(state.clone())
        .layer(from_fn(validate_server))
        .layer(from_fn_with_state(state.clone(), auth::require_api_key));

    Router::new()
        .route("/servers", get(handlers::servers))
        .route("/health", get(handlers::health))
        .route("/version", get(handlers::version))
        .route("/image/{server}/{asset_kind}/{asset_id}", get(handlers::image_placeholder))
        .nest(&api_prefix, api)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
