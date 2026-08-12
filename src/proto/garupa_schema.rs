//! Garupa protobuf message schemas, ported field-for-field from
//! GarupaSpeedTracker's `src/types/garupaSchema/*`.

use super::schema::{field, ProtoType, Schema};

// ============================================================================
// Ranking user, shared by monthly and event rankings
// ============================================================================

pub static USER_DECK_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("deckId", ProtoType::Int, false)),
        (2, field("deckName", ProtoType::String, false)),
        (3, field("leader", ProtoType::Int, false)),
        (4, field("member1", ProtoType::Int, false)),
        (5, field("member2", ProtoType::Int, false)),
        (6, field("member3", ProtoType::Int, false)),
        (7, field("member4", ProtoType::Int, false)),
        (8, field("bondsEffectIds", ProtoType::Int, true)),
        (10, field("deckType", ProtoType::String, false)),
    ],
};

pub static USER_APPEND_PARAMETER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("performance", ProtoType::Int, false)),
        (4, field("technique", ProtoType::Int, false)),
        (5, field("visual", ProtoType::Int, false)),
        (6, field("characterPotentialPerformance", ProtoType::Int, false)),
        (7, field("characterPotentialTechnique", ProtoType::Int, false)),
        (8, field("characterPotentialVisual", ProtoType::Int, false)),
        (9, field("characterBonusPerformance", ProtoType::Int, false)),
        (10, field("characterBonusTechnique", ProtoType::Int, false)),
        (11, field("characterBonusVisual", ProtoType::Int, false)),
    ],
};

pub static USER_SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("exp", ProtoType::Int, false)),
        (5, field("createdAt", ProtoType::Long, false)),
        (6, field("addExp", ProtoType::Int, false)),
        (7, field("trainingStatus", ProtoType::String, false)),
        (8, field("duplicateCount", ProtoType::Int, false)),
        (9, field("illust", ProtoType::String, false)),
        (10, field("skillExp", ProtoType::Int, false)),
        (11, field("skillLevel", ProtoType::Int, false)),
        (12, field("userAppendParameter", ProtoType::Message(&USER_APPEND_PARAMETER_SCHEMA), false)),
        (13, field("limitBreakRank", ProtoType::Int, false)),
    ],
};

pub static USER_SITUATION_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_SITUATION_SCHEMA), true))],
};

pub static USER_PROFILE_SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("illust", ProtoType::String, false)),
        (4, field("viewProfileSituationStatus", ProtoType::String, false)),
    ],
};

pub static USER_PROFILE_DEGREE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("profileDegreeType", ProtoType::String, false)),
        (3, field("degreeId", ProtoType::Int, false)),
    ],
};

pub static USER_PROFILE_DEGREE_MAP_ENTRY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("key", ProtoType::String, false)),
        (2, field("value", ProtoType::Message(&USER_PROFILE_DEGREE_SCHEMA), false)),
    ],
};

/// Map fields are encoded as repeated nested K-V messages.
pub static USER_PROFILE_DEGREE_MAP_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_PROFILE_DEGREE_MAP_ENTRY_SCHEMA), true))],
};

pub static RANKING_USER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("name", ProtoType::String, false)),
        (2, field("ownFlg", ProtoType::Bool, false)),
        (3, field("rankLevel", ProtoType::Int, false)),
        (4, field("introduction", ProtoType::String, false)),
        (5, field("rank", ProtoType::Int, false)),
        (6, field("point", ProtoType::Int, false)),
        (7, field("userId", ProtoType::Int, false)),
        (8, field("degreeId", ProtoType::Int, false)),
        (9, field("userDeck", ProtoType::Message(&USER_DECK_SCHEMA), false)),
        (10, field("userSituationList", ProtoType::Message(&USER_SITUATION_LIST_SCHEMA), false)),
        (11, field("userProfileSituation", ProtoType::Message(&USER_PROFILE_SITUATION_SCHEMA), false)),
        (12, field("userProfileDegreeMap", ProtoType::Message(&USER_PROFILE_DEGREE_MAP_SCHEMA), false)),
    ],
};

pub static RANKING_USER_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&RANKING_USER_SCHEMA), true))],
};

// ============================================================================
// Monthly ranking
// ============================================================================

pub static USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("monthlyRankingPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("monthlyRankingPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("monthlyRankingPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static MASTER_MONTHLY_RANKING_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("monthlyRankingId", ProtoType::Int, false)),
        (3, field("fromRank", ProtoType::Int, false)),
        (4, field("toRank", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
    ],
};

pub static MASTER_MONTHLY_RANKING_GRADE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("monthlyRankingId", ProtoType::Int, false)),
        (3, field("gradeAheadType", ProtoType::String, false)),
        (4, field("pt", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
        (8, field("rankingThresholdFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_MONTHLY_RANKING_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("monthlyRankingId", ProtoType::Int, false)),
        (2, field("monthlyRankingName", ProtoType::String, false)),
        (3, field("assetBundleName", ProtoType::String, false)),
        (4, field("bgmAssetBundleName", ProtoType::String, false)),
        (5, field("bgmFileName", ProtoType::String, false)),
        (6, field("startAt", ProtoType::Long, false)),
        (7, field("endAt", ProtoType::Long, false)),
        (8, field("enableFlg", ProtoType::Bool, false)),
        (9, field("publicStartAt", ProtoType::Long, false)),
        (10, field("publicEndAt", ProtoType::Long, false)),
        (11, field("distributionStartAt", ProtoType::Long, false)),
        (12, field("distributionEndAt", ProtoType::Long, false)),
        (13, field("receptionEndAt", ProtoType::Long, false)),
        (14, field("aggregateEndAt", ProtoType::Long, false)),
        (101, field("rewards", ProtoType::Message(&MASTER_MONTHLY_RANKING_REWARD_SCHEMA), true)),
        (102, field("grades", ProtoType::Message(&MASTER_MONTHLY_RANKING_GRADE_SCHEMA), true)),
    ],
};

pub static MASTER_MONTHLY_RANKING_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&MASTER_MONTHLY_RANKING_SCHEMA), true))],
};

// ============================================================================
// Event ranking, one response schema per event type
// ============================================================================

pub static USER_MEDLEY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (5, field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (6, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_LIVE_TRY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_STORY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_CHALLENGE_MUSIC_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_CHALLENGE_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (101, field("challengeMusicRankings", ProtoType::Message(&USER_CHALLENGE_MUSIC_RANKING_RESPONSE_SCHEMA), true)),
    ],
};

pub static USER_MISSION_LIVE_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("borderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_TEAM_LIVE_FESTIVAL_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_VERSUS_MUSIC_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_VERSUS_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("versusMusicRankings", ProtoType::Message(&USER_VERSUS_MUSIC_RANKING_RESPONSE_SCHEMA), true)),
        (4, field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

/// Maps a protobuf event type string to its ranking response schema.
pub static EVENT_TYPE_SCHEMAS: &[(&str, &Schema)] = &[
    ("medley", &USER_MEDLEY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("challenge", &USER_CHALLENGE_EVENT_RANKING_RESPONSE_SCHEMA),
    ("versus", &USER_VERSUS_EVENT_RANKING_RESPONSE_SCHEMA),
    ("live_try", &USER_LIVE_TRY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("story", &USER_STORY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("mission_live", &USER_MISSION_LIVE_EVENT_RANKING_RESPONSE_SCHEMA),
    ("team_live_festival", &USER_TEAM_LIVE_FESTIVAL_EVENT_RANKING_RESPONSE_SCHEMA),
];

// ============================================================================
// Event master
// ============================================================================

pub static MASTER_EVENT_POINT_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("eventId", ProtoType::Int, false)),
        (3, field("point", ProtoType::Long, false)),
        (4, field("rewardType", ProtoType::String, false)),
        (5, field("rewardId", ProtoType::Int, false)),
        (6, field("rewardQuantity", ProtoType::Int, false)),
        (7, field("recommendFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_EVENT_RANKING_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("eventId", ProtoType::Int, false)),
        (3, field("fromRank", ProtoType::Int, false)),
        (4, field("toRank", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
        (8, field("recommendFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_EVENT_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("eventId", ProtoType::Int, false)),
        (2, field("eventType", ProtoType::String, false)),
        (3, field("eventName", ProtoType::String, false)),
        (4, field("assetBundleName", ProtoType::String, false)),
        (5, field("startAt", ProtoType::Long, false)),
        (6, field("endAt", ProtoType::Long, false)),
        (7, field("enableFlg", ProtoType::Bool, false)),
        (8, field("publicStartAt", ProtoType::Long, false)),
        (9, field("publicEndAt", ProtoType::Long, false)),
        (10, field("distributionStartAt", ProtoType::Long, false)),
        (11, field("distributionEndAt", ProtoType::Long, false)),
        (12, field("bgmAssetBundleName", ProtoType::String, false)),
        (13, field("bgmFileName", ProtoType::String, false)),
        (14, field("aggregateEndAt", ProtoType::Long, false)),
        (15, field("eventExchangesEndAt", ProtoType::Long, false)),
        (16, field("receptionEndAt", ProtoType::Long, false)),
        (18, field("previousEventId", ProtoType::Int, false)),
        (101, field("pointRewards", ProtoType::Message(&MASTER_EVENT_POINT_REWARD_SCHEMA), true)),
        (102, field("rankingRewards", ProtoType::Message(&MASTER_EVENT_RANKING_REWARD_SCHEMA), true)),
    ],
};

pub static MASTER_EVENT_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&MASTER_EVENT_SCHEMA), true))],
};

// ============================================================================
// Application
// ============================================================================

pub static APPLICATION_PLATFORM_STATUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("platformId", ProtoType::Int, false)),
        (2, field("status", ProtoType::String, false)),
    ],
};

/// Flat application health/version response, not entries-wrapped.
pub static APPLICATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("clientVersion", ProtoType::String, false)),
        (2, field("dataVersion", ProtoType::String, false)),
        (3, field("appStatus", ProtoType::String, false)),
        (4, field("appType", ProtoType::String, false)),
        (5, field("serverName", ProtoType::String, false)),
        (7, field("liveStatus", ProtoType::String, false)),
        (8, field("gachaStatus", ProtoType::String, false)),
        (9, field("shopStatus", ProtoType::String, false)),
        (10, field("masterVersion", ProtoType::String, false)),
        (11, field("checksum", ProtoType::String, false)),
        (12, field("platformMaintenance", ProtoType::Message(&APPLICATION_PLATFORM_STATUS_SCHEMA), true)),
    ],
};

// ============================================================================
// Music
// ============================================================================

pub static MUSIC_MISSION_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("missionType", ProtoType::String, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
    ],
};

pub static MUSIC_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("musicName", ProtoType::String, false)),
        (3, field("bgmAssetBundleName", ProtoType::String, false)),
        (4, field("jacketAssetBundleName", ProtoType::String, false)),
        (5, field("lyricist", ProtoType::String, false)),
        (6, field("composer", ProtoType::String, false)),
        (7, field("musicType", ProtoType::String, false)),
        (9, field("arranger", ProtoType::String, false)),
        (10, field("keyword", ProtoType::String, false)),
        (11, field("releaseFlg", ProtoType::Int, false)),
        (12, field("releaseCondition", ProtoType::String, false)),
        (13, field("missionRewards", ProtoType::Message(&MUSIC_MISSION_REWARD_SCHEMA), true)),
        (14, field("assetBundleName", ProtoType::String, false)),
        (15, field("sortOrder", ProtoType::Int, false)),
        (16, field("startAt", ProtoType::Long, false)),
        (17, field("endAt", ProtoType::Long, false)),
        (20, field("musicNameKana", ProtoType::String, false)),
        (21, field("category", ProtoType::String, false)),
        (22, field("seq", ProtoType::Int, false)),
    ],
};

pub static MUSIC_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&MUSIC_SCHEMA), true))],
};

// ============================================================================
// Character
// ============================================================================

pub static CHARACTER_PROFILE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("profileType", ProtoType::String, false)),
        (3, field("voiceActorName", ProtoType::String, false)),
        (4, field("school", ProtoType::String, false)),
        (5, field("grade", ProtoType::String, false)),
        (6, field("birthday", ProtoType::Long, false)),
        (7, field("zodiacSign", ProtoType::String, false)),
        (8, field("favoriteFood", ProtoType::String, false)),
        (9, field("dislikedFood", ProtoType::String, false)),
        (10, field("hobby", ProtoType::String, false)),
        (11, field("introduction", ProtoType::String, false)),
        (12, field("height", ProtoType::Int, false)),
        (13, field("class", ProtoType::String, false)),
        (14, field("schoolType", ProtoType::String, false)),
    ],
};

pub static CHARACTER_COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("costumeVariation", ProtoType::Int, false)),
        (3, field("costumeType", ProtoType::String, false)),
        (4, field("costumeSubType", ProtoType::String, false)),
        (5, field("assetName", ProtoType::String, false)),
        (6, field("assetBundleName", ProtoType::String, false)),
        (7, field("season", ProtoType::String, false)),
    ],
};

pub static CHARACTER_COSTUME_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&CHARACTER_COSTUME_SCHEMA), true))],
};

pub static CHARACTER_COSTUME_SEASON_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("season", ProtoType::String, false)),
        (2, field("costumes", ProtoType::Message(&CHARACTER_COSTUME_LIST_SCHEMA), false)),
    ],
};

pub static CHARACTER_COSTUME_SEASON_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("season", ProtoType::Message(&CHARACTER_COSTUME_SEASON_SCHEMA), false))],
};

pub static CHARACTER_EPISODE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("episodeId", ProtoType::Int, false)),
        (2, field("episodeType", ProtoType::String, false)),
        (4, field("assetBundleName", ProtoType::String, false)),
        (11, field("episodeName", ProtoType::String, false)),
        (12, field("releaseFlg", ProtoType::Int, false)),
        (13, field("episodeNumber", ProtoType::Int, false)),
    ],
};

pub static CHARACTER_EPISODE_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("episodes", ProtoType::Message(&CHARACTER_EPISODE_SCHEMA), true))],
};

pub static CHARACTER_LIVE2D_COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("costumeVariation", ProtoType::Int, false)),
        (3, field("costumeId", ProtoType::Int, false)),
        (4, field("assetName", ProtoType::String, false)),
        (5, field("assetBundleName", ProtoType::String, false)),
    ],
};

pub static CHARACTER_LIVE2D_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("costumes", ProtoType::Message(&CHARACTER_LIVE2D_COSTUME_SCHEMA), true))],
};

pub static CHARACTER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("characterType", ProtoType::String, false)),
        (3, field("characterName", ProtoType::String, false)),
        (4, field("characterNameKana", ProtoType::String, false)),
        (7, field("givenName", ProtoType::String, false)),
        (8, field("familyName", ProtoType::String, false)),
        (9, field("bandId", ProtoType::Int, false)),
        (10, field("resourceName", ProtoType::String, false)),
        (11, field("index", ProtoType::Int, false)),
        (12, field("profile", ProtoType::Message(&CHARACTER_PROFILE_SCHEMA), false)),
        (13, field("costumeSeasons", ProtoType::Message(&CHARACTER_COSTUME_SEASON_ENTRY_SCHEMA), true)),
        (14, field("episodes", ProtoType::Message(&CHARACTER_EPISODE_ENTRY_SCHEMA), true)),
        (16, field("color", ProtoType::String, false)),
        (17, field("live2dCostumes", ProtoType::Message(&CHARACTER_LIVE2D_ENTRY_SCHEMA), true)),
        (20, field("attribute", ProtoType::String, false)),
    ],
};

pub static CHARACTER_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&CHARACTER_SCHEMA), true))],
};

// ============================================================================
// Band
// ============================================================================

pub static BAND_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("bandId", ProtoType::Int, false)),
        (2, field("bandName", ProtoType::String, false)),
        (3, field("enableFlg", ProtoType::Int, false)),
        (4, field("color", ProtoType::String, false)),
        (5, field("member1", ProtoType::Int, false)),
        (6, field("member2", ProtoType::Int, false)),
        (7, field("member3", ProtoType::Int, false)),
        (8, field("member4", ProtoType::Int, false)),
        (9, field("member5", ProtoType::Int, false)),
        (10, field("seq", ProtoType::Int, false)),
        (11, field("bandType", ProtoType::String, false)),
        (13, field("bandNameKana", ProtoType::String, false)),
        (14, field("sortOrder", ProtoType::Int, false)),
    ],
};

pub static BAND_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&BAND_SCHEMA), true))],
};

// ============================================================================
// Area
// ============================================================================

pub static AREA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaId", ProtoType::Int, false)),
        (2, field("areaType", ProtoType::String, false)),
        (3, field("areaName", ProtoType::String, false)),
        (4, field("description", ProtoType::String, false)),
        (8, field("seq", ProtoType::Int, false)),
        (13, field("fromSeason", ProtoType::String, false)),
        (14, field("toSeason", ProtoType::String, false)),
    ],
};

pub static AREA_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&AREA_SCHEMA), true))],
};

// ============================================================================
// Gacha
// ============================================================================

pub static GACHA_RATE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("rarity", ProtoType::Int, false)),
        (3, field("rate", ProtoType::Float, false)),
        (4, field("rateType", ProtoType::Int, false)),
    ],
};

pub static GACHA_TYPE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("costType", ProtoType::String, false)),
        (3, field("cost", ProtoType::Int, false)),
        (4, field("sortOrder", ProtoType::Int, false)),
        (5, field("enableFlg", ProtoType::Int, false)),
        (6, field("type", ProtoType::String, false)),
        (7, field("paidFlg", ProtoType::Int, false)),
        (9, field("costTotal", ProtoType::Int, false)),
        (12, field("discountFlg", ProtoType::Int, false)),
        (13, field("freeFlg", ProtoType::Int, false)),
    ],
};

pub static GACHA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("gachaName", ProtoType::String, false)),
        (3, field("gachaType", ProtoType::String, false)),
        (4, field("startAt", ProtoType::Long, false)),
        (5, field("endAt", ProtoType::Long, false)),
        (7, field("rates", ProtoType::Message(&GACHA_RATE_SCHEMA), true)),
        (8, field("types", ProtoType::Message(&GACHA_TYPE_SCHEMA), true)),
        (9, field("description", ProtoType::String, false)),
        (10, field("sortOrder", ProtoType::Int, false)),
        (11, field("assetBundleName", ProtoType::String, false)),
        (13, field("endText", ProtoType::String, false)),
        (19, field("gachaCategory", ProtoType::String, false)),
        (35, field("pickupText", ProtoType::String, false)),
        (38, field("limitFlg", ProtoType::Int, false)),
        (40, field("stampFlg", ProtoType::Int, false)),
        (43, field("bonusFlg", ProtoType::Int, false)),
    ],
};

pub static GACHA_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&GACHA_SCHEMA), true))],
};

// ============================================================================
// Item
// ============================================================================

pub static ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("itemId", ProtoType::Int, false)),
        (2, field("itemName", ProtoType::String, false)),
        (3, field("itemType", ProtoType::Int, false)),
        (4, field("description", ProtoType::String, false)),
        (5, field("seq", ProtoType::Int, false)),
    ],
};

pub static ITEM_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&ITEM_SCHEMA), true))],
};

// ============================================================================
// Skill
// ============================================================================

pub static SKILL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("skillId", ProtoType::Int, false)),
        (2, field("skillLevel", ProtoType::Int, false)),
        (3, field("effectValue", ProtoType::Float, false)),
        (4, field("skillName", ProtoType::String, false)),
        (5, field("description", ProtoType::String, false)),
        (6, field("skillType", ProtoType::String, false)),
    ],
};

pub static SKILL_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&SKILL_SCHEMA), true))],
};

// ============================================================================
// Stamp
// ============================================================================

pub static STAMP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("stampId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("assetName", ProtoType::String, false)),
        (4, field("stampType", ProtoType::String, false)),
        (7, field("startAt", ProtoType::Long, false)),
        (8, field("endAt", ProtoType::Long, false)),
    ],
};

pub static STAMP_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&STAMP_SCHEMA), true))],
};

// ============================================================================
// Login bonus
// ============================================================================

pub static LOGIN_BONUS_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("loginBonusId", ProtoType::Int, false)),
        (2, field("day", ProtoType::Int, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
        (7, field("seq", ProtoType::Int, false)),
        (8, field("presentType", ProtoType::String, false)),
    ],
};

pub static LOGIN_BONUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("loginBonusId", ProtoType::Int, false)),
        (2, field("loginBonusType", ProtoType::String, false)),
        (3, field("loginBonusName", ProtoType::String, false)),
        (4, field("campaignName", ProtoType::String, false)),
        (5, field("receiveFlg", ProtoType::Int, false)),
        (6, field("startAt", ProtoType::Long, false)),
        (7, field("rewards", ProtoType::Message(&LOGIN_BONUS_REWARD_SCHEMA), true)),
        (8, field("imageAssetBundleName", ProtoType::String, false)),
    ],
};

pub static LOGIN_BONUS_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&LOGIN_BONUS_SCHEMA), true))],
};

// ============================================================================
// Costume
// ============================================================================

pub static COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("costumeId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("assetBundleName", ProtoType::String, false)),
        (5, field("costumeName", ProtoType::String, false)),
        (6, field("sortOrder", ProtoType::Int, false)),
        (7, field("sdAssetBundleName", ProtoType::String, false)),
        (9, field("startAt", ProtoType::Long, false)),
        (10, field("characterId", ProtoType::Int, false)),
    ],
};

pub static COSTUME_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&COSTUME_SCHEMA), true))],
};

// ============================================================================
// Situation master, the game's name for cards
// ============================================================================

pub static SITUATION_APPEND_PARAMETER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("situationId", ProtoType::Int, false)),
        (2, field("level", ProtoType::Int, false)),
        (3, field("performance", ProtoType::Int, false)),
        (4, field("technique", ProtoType::Int, false)),
        (5, field("visual", ProtoType::Int, false)),
    ],
};

pub static SITUATION_LEVEL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("level", ProtoType::Int, false)),
        (2, field("appendParameter", ProtoType::Message(&SITUATION_APPEND_PARAMETER_SCHEMA), false)),
    ],
};

pub static SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("situationId", ProtoType::Int, false)),
        (2, field("situationType", ProtoType::Int, false)),
        (3, field("rarity", ProtoType::Int, false)),
        (5, field("attribute", ProtoType::String, false)),
        (7, field("skillId", ProtoType::Int, false)),
        (8, field("levels", ProtoType::Message(&SITUATION_LEVEL_SCHEMA), true)),
        (10, field("cardName", ProtoType::String, false)),
        (11, field("maxLevel", ProtoType::Int, false)),
        (12, field("resourceName", ProtoType::String, false)),
        (13, field("sdAssetName", ProtoType::String, false)),
        (16, field("characterIndex", ProtoType::Int, false)),
        (17, field("releaseAt", ProtoType::Long, false)),
        (18, field("skillId2", ProtoType::Int, false)),
        (19, field("flag2", ProtoType::Int, false)),
        (20, field("illustType", ProtoType::String, false)),
        (24, field("extra", ProtoType::String, false)),
        (25, field("seq", ProtoType::Int, false)),
    ],
};

pub static SITUATION_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&SITUATION_SCHEMA), true))],
};

// ============================================================================
// Shop
// ============================================================================

/// Shop where area items and music are exchanged. Field 2 is a per-type order
/// hint discovered from a live dump; its exact meaning is unconfirmed.
pub static SHOP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("shopId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("shopName", ProtoType::String, false)),
        (4, field("description", ProtoType::String, false)),
        (5, field("shopType", ProtoType::String, false)),
    ],
};

pub static SHOP_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&SHOP_SCHEMA), true))],
};

// ============================================================================
// User profile, deck, situation
// ============================================================================

pub static USER_PROFILE_STATS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("rank", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("exp", ProtoType::Int, false)),
        (5, field("liveCount", ProtoType::Int, false)),
        (7, field("stamina", ProtoType::Int, false)),
        (9, field("staminaMax", ProtoType::Int, false)),
        (16, field("friendCount", ProtoType::Int, false)),
        (17, field("playCount", ProtoType::Int, false)),
    ],
};

pub static USER_PROFILE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("uuid", ProtoType::String, false)),
        (3, field("userName", ProtoType::String, false)),
        (4, field("clientVersion", ProtoType::String, false)),
        (5, field("platform", ProtoType::String, false)),
        (6, field("device", ProtoType::String, false)),
        (7, field("osVersion", ProtoType::String, false)),
        (9, field("userCode", ProtoType::String, false)),
        (10, field("tutorialStatus", ProtoType::String, false)),
        (11, field("comment", ProtoType::String, false)),
        (12, field("profileFrame", ProtoType::String, false)),
        (13, field("lastLoginAt", ProtoType::Long, false)),
    ],
};

pub static USER_PROFILE_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("profile", ProtoType::Message(&USER_PROFILE_SCHEMA), false)),
        (2, field("stats", ProtoType::Message(&USER_PROFILE_STATS_SCHEMA), false)),
    ],
};

pub static USER_DECK_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_DECK_SCHEMA), true))],
};

// ============================================================================
// User title, stamps, areas, items, presents, gacha
// ============================================================================

/// Flat equipped-title response, not entries-wrapped.
pub static USER_TITLE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("titleId", ProtoType::Int, false)),
        (2, field("assetName", ProtoType::String, false)),
        (3, field("startAt", ProtoType::Long, false)),
        (4, field("endAt", ProtoType::Long, false)),
        (6, field("seq", ProtoType::Int, false)),
        (7, field("colorType", ProtoType::String, false)),
        (8, field("frameColor", ProtoType::String, false)),
        (9, field("copyrightAssetName", ProtoType::String, false)),
        (10, field("rightsAssetName", ProtoType::String, false)),
        (11, field("enableFlg", ProtoType::Int, false)),
    ],
};

pub static USER_STAMP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("stampId", ProtoType::Int, false)),
        (3, field("seq", ProtoType::Int, false)),
    ],
};

pub static USER_STAMP_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_STAMP_SCHEMA), true))],
};

pub static USER_AREA_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaItemId", ProtoType::Int, false)),
        (2, field("status", ProtoType::String, false)),
    ],
};

pub static USER_AREA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaId", ProtoType::Int, false)),
        (2, field("areaItems", ProtoType::Message(&USER_AREA_ITEM_SCHEMA), true)),
    ],
};

pub static USER_AREA_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_AREA_SCHEMA), true))],
};

pub static USER_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("itemId", ProtoType::Int, false)),
        (3, field("count", ProtoType::Int, false)),
    ],
};

pub static USER_ITEM_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_ITEM_SCHEMA), true))],
};

pub static USER_PRESENT_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("presentId", ProtoType::Int, false)),
        (2, field("userId", ProtoType::Long, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
        (6, field("message", ProtoType::String, false)),
        (7, field("receivedAt", ProtoType::Long, false)),
        (8, field("createdAt", ProtoType::Long, false)),
    ],
};

/// Present box summary. Field 1 is a max-slot sentinel that is i64::MAX on a
/// live dump, field 2 the current slot count, field 3 an unread flag. Names are
/// best guesses from a single empty-box response.
pub static USER_PRESENT_BOX_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("maxSlot", ProtoType::Long, false)),
        (2, field("slotCount", ProtoType::Int, false)),
        (3, field("unreadCount", ProtoType::Int, false)),
    ],
};

pub static USER_PRESENT_LIST_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("entries", ProtoType::Message(&USER_PRESENT_SCHEMA), true)),
        (2, field("presentBox", ProtoType::Message(&USER_PRESENT_BOX_SCHEMA), false)),
    ],
};

pub static USER_GACHA_ENTRY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("gachaId", ProtoType::Int, false)),
        (3, field("count", ProtoType::Int, false)),
    ],
};

pub static USER_GACHA_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_GACHA_ENTRY_SCHEMA), true))],
};

// ============================================================================
// User episodes
// ============================================================================

/// Provisional episode unlock schema. A live probe of the configured user
/// returned an empty payload, so these fields await a user with episodes.
pub static USER_EPISODE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("episodeId", ProtoType::Int, false)),
        (3, field("status", ProtoType::String, false)),
    ],
};

pub static USER_EPISODE_LIST_SCHEMA: Schema = Schema {
    fields: &[(1, field("entries", ProtoType::Message(&USER_EPISODE_SCHEMA), true))],
};
