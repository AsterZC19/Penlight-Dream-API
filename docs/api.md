# API 文档

**Penlight-Dream-API** 的 HTTP 接口说明。默认监听 `http://127.0.0.1:8080`，API 前缀 `/api`。

## 鉴权

在 `.env` 里设置 `API_KEY` 后，所有 `/api/{server}/*` 请求都必须携带密钥，否则返回 `401`：

```bash
curl -H "X-API-Key: your-secret-key" http://127.0.0.1:8080/api/jp/monthly-ranking
curl -H "Authorization: Bearer your-secret-key" http://127.0.0.1:8080/api/jp/monthly-ranking
```

`API_KEY` 留空则鉴权关闭，任何人都可访问。

## 服务器作用域

游戏数据路由挂在 `/api/{server}/...` 下。当前仅配置日服，`{server}` 只接受 `jp`，其它值返回 `400`：

```bash
curl http://127.0.0.1:8080/api/jp/music
# 400: {"result":"failed","status":400,"message":"unsupported server \"en\", only \"jp\" is configured"}
curl http://127.0.0.1:8080/api/en/music
```

## 响应格式

- **列表接口**返回 `{"entries": [...]}` 包裹结构。
- **单个对象接口**返回对象本身。
- **排行报告**返回结构化对象，前列/分档线用户为裸数组。
- **错误**统一为 `{"result": "failed", "status": <http状态码>, "message": "<原因>"}`。

### 服务器元信息

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/servers` | 已配置的服务器列表，不含密钥与 UID |
| GET | `/health` | 进程健康检查，含版本、运行时长、日服可用性与客户端版本 |
| GET | `/version` | 自动探测到的游戏客户端版本 |
| GET | `/image/{server}/{asset_kind}/{asset_id}` | 静态资源占位路由 |

### 月榜

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/monthly-ranking` | 月榜期次主数据 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}` | 某期月榜的完整排名 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}/top` | 仅前列用户 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}/border` | 仅分档线用户 |

### 活动

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/events` | 活动主数据列表 |
| GET | `/api/{server}/events/{event_id}/ranking?type=medley` | 某活动排名，`type` 缺省时从活动主数据解析，`mid` 可选用于按曲目子榜 |

### 主数据

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/application` | 应用版本、服务器状态、各平台维护状态 |
| GET | `/api/{server}/music` | 乐曲主数据 |
| GET | `/api/{server}/characters` | 角色主数据，含人物设定、服装季、语音、Live2D 服装 |
| GET | `/api/{server}/bands` | 乐队主数据 |
| GET | `/api/{server}/areas` | 区域主数据 |
| GET | `/api/{server}/gacha` | 卡池主数据 |
| GET | `/api/{server}/items` | 道具主数据 |
| GET | `/api/{server}/skills` | 技能主数据 |
| GET | `/api/{server}/stamps` | 表情主数据 |
| GET | `/api/{server}/login-bonuses` | 登录奖励主数据 |
| GET | `/api/{server}/costumes` | 服装主数据 |
| GET | `/api/{server}/shops` | 商店主数据 |
| GET | `/api/{server}/cards` | 卡主数据，含各等级能力值、技能引用、`episodes` 卡面剧情与 `training` 特训数据 |

### 用户数据

以下接口针对 `.env` 中配置的 UID。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/user/profile` | 用户资料与数值 |
| GET | `/api/{server}/user/decks` | 用户编队 |
| GET | `/api/{server}/user/situations` | 用户持有的卡 |
| GET | `/api/{server}/user/title` | 用户当前称号 |
| GET | `/api/{server}/user/stamps` | 用户表情 |
| GET | `/api/{server}/user/areas` | 用户区域道具 |
| GET | `/api/{server}/user/items` | 用户道具余额 |
| GET | `/api/{server}/user/presents` | 用户礼物与礼盒信息 |
| GET | `/api/{server}/user/gacha` | 用户卡池记录 |
| GET | `/api/{server}/user/episodes` | 用户剧情解锁 |

### 缓存

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/cache` | 缓存条目数 |
| DELETE | `/api/{server}/cache` | 清空缓存 |

主数据默认缓存 1 小时，用户数据默认 5 分钟，排名默认 30 秒，可在 `.env` 中通过 `GARUPA_CACHE_TTL_MASTER`、`GARUPA_CACHE_TTL_USER`、`GARUPA_CACHE_TTL_RANKING` 调整。缓存未命中的并发请求会合并为一次上游调用，避免突刺打爆官方 API。
