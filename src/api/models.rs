//! API response models and the mapping from decoded protobuf values into them.
//! Field shapes mirror GarupaSpeedTracker's `RankingUserRaw` and the
//! `MonthlyRankingBandoriRaw` and `EventRankingBandoriRaw` reports.

use serde::Serialize;
use serde_json::Value;

// ============================================================================
// Ranking user
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingUser {
    /// Player UID from the `userId` field.
    pub uid: i64,
    /// Player name.
    pub name: String,
    /// Player introduction.
    pub introduction: String,
    /// Player level from the `rankLevel` field.
    pub rank: i64,
    /// Displayed card ID, resolved from profile situation or deck leader.
    pub sid: i64,
    /// 1 when the displayed card is the after-training illustration.
    pub strained: i64,
    /// Player's equipped profile degree IDs.
    pub degrees: Vec<i64>,
    /// Ranking position from the `rank` field.
    pub tier: i64,
    /// Ranking points from the `point` field.
    pub point: i64,
}

fn i64_of(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn str_of(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Normalizes a decoded ranking user, resolving the displayed card exactly as
/// the game client does by mirroring its `resolveDisplayCard` logic.
fn parse_user(v: &Value) -> RankingUser {
    let profile = v.get("userProfileSituation");
    let view_status = profile.and_then(|p| p.get("viewProfileSituationStatus")).and_then(Value::as_str);

    let (sid, strained) = if view_status == Some("profile_situation") {
        let p = profile.unwrap_or(&Value::Null);
        let sid = i64_of(p, "situationId");
        let strained = if str_of(p, "illust") == "after_training" { 1 } else { 0 };
        (sid, strained)
    } else {
        let deck_leader = v.get("userDeck").and_then(|d| d.get("leader")).and_then(Value::as_i64).unwrap_or(0);
        if deck_leader > 0 {
            let entries = v
                .get("userSituationList")
                .and_then(|l| l.get("entries"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let leader_card = entries.iter().find(|e| i64_of(e, "situationId") == deck_leader);
            let strained = if leader_card.map(|c| str_of(c, "illust") == "after_training").unwrap_or(false) { 1 } else { 0 };
            (deck_leader, strained)
        } else {
            (1, 0)
        }
    };

    let degrees = v
        .get("userProfileDegreeMap")
        .and_then(|m| m.get("entries"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(|e| i64_of(e.get("value").unwrap_or(&Value::Null), "degreeId")).collect())
        .unwrap_or_default();

    RankingUser {
        uid: i64_of(v, "userId"),
        name: str_of(v, "name"),
        introduction: str_of(v, "introduction"),
        rank: i64_of(v, "rankLevel"),
        sid,
        strained,
        degrees,
        tier: i64_of(v, "rank"),
        point: i64_of(v, "point"),
    }
}

/// Converts a ranking container with an `entries` array into users.
fn build_users(v: Option<&Value>) -> Vec<RankingUser> {
    v.and_then(|c| c.get("entries"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(parse_user).collect())
        .unwrap_or_default()
}

// ============================================================================
// Monthly ranking
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyRankingReport {
    pub monthly_ranking_point_near_users: Vec<RankingUser>,
    pub monthly_ranking_point_top_users: Vec<RankingUser>,
    pub monthly_ranking_point_border_users: Vec<RankingUser>,
}

pub fn monthly_ranking_report(root: &Value) -> MonthlyRankingReport {
    MonthlyRankingReport {
        monthly_ranking_point_near_users: build_users(root.get("monthlyRankingPointNearUsers")),
        monthly_ranking_point_top_users: build_users(root.get("monthlyRankingPointTopUsers")),
        monthly_ranking_point_border_users: build_users(root.get("monthlyRankingPointBorderUsers")),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyRankingReward {
    pub id: i64,
    pub monthly_ranking_id: i64,
    pub from_rank: i64,
    pub to_rank: i64,
    pub reward_type: String,
    pub reward_id: i64,
    pub reward_quantity: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyRankingGrade {
    pub id: i64,
    pub monthly_ranking_id: i64,
    pub grade_ahead_type: String,
    pub pt: i64,
    pub reward_type: String,
    pub reward_id: i64,
    pub reward_quantity: i64,
    pub ranking_threshold_flg: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyRankingInfo {
    pub monthly_ranking_id: i64,
    pub monthly_ranking_name: String,
    pub asset_bundle_name: String,
    pub bgm_asset_bundle_name: String,
    pub bgm_file_name: String,
    pub start_at: i64,
    pub end_at: i64,
    pub enable_flg: bool,
    pub public_start_at: i64,
    pub public_end_at: i64,
    pub distribution_start_at: i64,
    pub distribution_end_at: i64,
    pub reception_end_at: i64,
    pub aggregate_end_at: i64,
    pub rewards: Vec<MonthlyRankingReward>,
    pub grades: Vec<MonthlyRankingGrade>,
}

pub fn monthly_ranking_list(root: &Value) -> Vec<MonthlyRankingInfo> {
    root.get("entries")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|e| MonthlyRankingInfo {
                    monthly_ranking_id: i64_of(e, "monthlyRankingId"),
                    monthly_ranking_name: str_of(e, "monthlyRankingName"),
                    asset_bundle_name: str_of(e, "assetBundleName"),
                    bgm_asset_bundle_name: str_of(e, "bgmAssetBundleName"),
                    bgm_file_name: str_of(e, "bgmFileName"),
                    start_at: i64_of(e, "startAt"),
                    end_at: i64_of(e, "endAt"),
                    enable_flg: e.get("enableFlg").and_then(Value::as_bool).unwrap_or(false),
                    public_start_at: i64_of(e, "publicStartAt"),
                    public_end_at: i64_of(e, "publicEndAt"),
                    distribution_start_at: i64_of(e, "distributionStartAt"),
                    distribution_end_at: i64_of(e, "distributionEndAt"),
                    reception_end_at: i64_of(e, "receptionEndAt"),
                    aggregate_end_at: i64_of(e, "aggregateEndAt"),
                    rewards: e.get("rewards").and_then(Value::as_array).map(|arr| parse_rewards(arr)).unwrap_or_default(),
                    grades: e.get("grades").and_then(Value::as_array).map(|arr| parse_grades(arr)).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_rewards(arr: &[Value]) -> Vec<MonthlyRankingReward> {
    arr.iter()
        .map(|r| MonthlyRankingReward {
            id: i64_of(r, "id"),
            monthly_ranking_id: i64_of(r, "monthlyRankingId"),
            from_rank: i64_of(r, "fromRank"),
            to_rank: i64_of(r, "toRank"),
            reward_type: str_of(r, "rewardType"),
            reward_id: i64_of(r, "rewardId"),
            reward_quantity: i64_of(r, "rewardQuantity"),
        })
        .collect()
}

fn parse_grades(arr: &[Value]) -> Vec<MonthlyRankingGrade> {
    arr.iter()
        .map(|g| MonthlyRankingGrade {
            id: i64_of(g, "id"),
            monthly_ranking_id: i64_of(g, "monthlyRankingId"),
            grade_ahead_type: str_of(g, "gradeAheadType"),
            pt: i64_of(g, "pt"),
            reward_type: str_of(g, "rewardType"),
            reward_id: i64_of(g, "rewardId"),
            reward_quantity: i64_of(g, "rewardQuantity"),
            ranking_threshold_flg: g.get("rankingThresholdFlg").and_then(Value::as_bool).unwrap_or(false),
        })
        .collect()
}

// ============================================================================
// Event ranking
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicRanking {
    pub music_id: i64,
    pub score_top_users: Vec<RankingUser>,
    pub score_border_users: Vec<RankingUser>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRankingReport {
    pub event_type: String,
    pub event_point_top_users: Vec<RankingUser>,
    pub event_point_border_users: Vec<RankingUser>,
    pub music_rankings: Vec<MusicRanking>,
}

fn build_music_rankings(v: Option<&Value>) -> Vec<MusicRanking> {
    v.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|m| MusicRanking {
                    music_id: i64_of(m, "musicId"),
                    score_top_users: build_users(m.get("scoreTopUsers")),
                    score_border_users: build_users(m.get("scoreBorderUsers")),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn event_ranking_report(root: &Value, event_type: &str) -> EventRankingReport {
    let music_rankings = match event_type {
        "medley" => vec![MusicRanking {
            music_id: 1,
            score_top_users: build_users(root.get("scoreTopUsers")),
            score_border_users: build_users(root.get("scoreBorderUsers")),
        }],
        "challenge" => build_music_rankings(root.get("challengeMusicRankings")),
        "versus" => build_music_rankings(root.get("versusMusicRankings")),
        _ => Vec::new(),
    };

    let (top, border) = match event_type {
        "medley" | "challenge" | "versus" => (root.get("eventPointTopUsers"), root.get("eventPointBorderUsers")),
        "live_try" | "team_live_festival" => (root.get("topUsers"), root.get("eventPointBorderUsers")),
        "mission_live" => (root.get("topUsers"), root.get("borderUsers")),
        "story" => (root.get("topUsers"), None),
        _ => (root.get("topUsers"), root.get("eventPointBorderUsers")),
    };

    EventRankingReport {
        event_type: event_type.to_string(),
        event_point_top_users: build_users(top),
        event_point_border_users: build_users(border),
        music_rankings,
    }
}

// ============================================================================
// Event master
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPointReward {
    pub id: i64,
    pub event_id: i64,
    pub point: i64,
    pub reward_type: String,
    pub reward_id: i64,
    pub reward_quantity: i64,
    pub recommend_flg: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRankingReward {
    pub id: i64,
    pub event_id: i64,
    pub from_rank: i64,
    pub to_rank: i64,
    pub reward_type: String,
    pub reward_id: i64,
    pub reward_quantity: i64,
    pub recommend_flg: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInfo {
    pub event_id: i64,
    pub event_type: String,
    pub event_name: String,
    pub asset_bundle_name: String,
    pub start_at: i64,
    pub end_at: i64,
    pub enable_flg: bool,
    pub public_start_at: i64,
    pub public_end_at: i64,
    pub distribution_start_at: i64,
    pub distribution_end_at: i64,
    pub bgm_asset_bundle_name: String,
    pub bgm_file_name: String,
    pub aggregate_end_at: i64,
    pub event_exchanges_end_at: i64,
    pub reception_end_at: i64,
    pub previous_event_id: i64,
    pub point_rewards: Vec<EventPointReward>,
    pub ranking_rewards: Vec<EventRankingReward>,
}

pub fn event_list(root: &Value) -> Vec<EventInfo> {
    root.get("entries")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|e| EventInfo {
                    event_id: i64_of(e, "eventId"),
                    event_type: str_of(e, "eventType"),
                    event_name: str_of(e, "eventName"),
                    asset_bundle_name: str_of(e, "assetBundleName"),
                    start_at: i64_of(e, "startAt"),
                    end_at: i64_of(e, "endAt"),
                    enable_flg: e.get("enableFlg").and_then(Value::as_bool).unwrap_or(false),
                    public_start_at: i64_of(e, "publicStartAt"),
                    public_end_at: i64_of(e, "publicEndAt"),
                    distribution_start_at: i64_of(e, "distributionStartAt"),
                    distribution_end_at: i64_of(e, "distributionEndAt"),
                    bgm_asset_bundle_name: str_of(e, "bgmAssetBundleName"),
                    bgm_file_name: str_of(e, "bgmFileName"),
                    aggregate_end_at: i64_of(e, "aggregateEndAt"),
                    event_exchanges_end_at: i64_of(e, "eventExchangesEndAt"),
                    reception_end_at: i64_of(e, "receptionEndAt"),
                    previous_event_id: i64_of(e, "previousEventId"),
                    point_rewards: e.get("pointRewards").and_then(Value::as_array).map(|arr| parse_point_rewards(arr)).unwrap_or_default(),
                    ranking_rewards: e.get("rankingRewards").and_then(Value::as_array).map(|arr| parse_ranking_rewards(arr)).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_point_rewards(arr: &[Value]) -> Vec<EventPointReward> {
    arr.iter()
        .map(|r| EventPointReward {
            id: i64_of(r, "id"),
            event_id: i64_of(r, "eventId"),
            point: i64_of(r, "point"),
            reward_type: str_of(r, "rewardType"),
            reward_id: i64_of(r, "rewardId"),
            reward_quantity: i64_of(r, "rewardQuantity"),
            recommend_flg: r.get("recommendFlg").and_then(Value::as_bool).unwrap_or(false),
        })
        .collect()
}

fn parse_ranking_rewards(arr: &[Value]) -> Vec<EventRankingReward> {
    arr.iter()
        .map(|r| EventRankingReward {
            id: i64_of(r, "id"),
            event_id: i64_of(r, "eventId"),
            from_rank: i64_of(r, "fromRank"),
            to_rank: i64_of(r, "toRank"),
            reward_type: str_of(r, "rewardType"),
            reward_id: i64_of(r, "rewardId"),
            reward_quantity: i64_of(r, "rewardQuantity"),
            recommend_flg: r.get("recommendFlg").and_then(Value::as_bool).unwrap_or(false),
        })
        .collect()
}
