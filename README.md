# Penlight-Dream-API

**Penlight-Dream-API** 是一个 BanG Dream! Girls Band Party! 非官方第三方 API 服务器，用 Rust 实现。

## 快速开始

```bash
# 1. 复制配置模板并填写
cp .env.example .env

# 2. 编译
cargo build --release

# 3. 运行
./target/release/penlight-dream-api
```

默认监听 `http://127.0.0.1:8080`，API 前缀 `/api`。

## Docker 部署


```bash
# 方式一：docker compose，复用根目录 .env 中的凭据
docker compose up -d --build

# 方式二：docker run，凭据通过环境变量传入
docker build -t penlight-dream-api .
docker run --rm -p 8080:8080 --env-file .env penlight-dream-api

# 方式三：直接拉取 GitHub Actions 打 tag 时自动构建发布的 GHCR 镜像
docker run --rm -p 8080:8080 --env-file .env ghcr.io/asterzc19/penlight-dream-api:latest
```

容器内必须把 `HOST` 设为 `0.0.0.0` 才能被宿主机访问。compose 已自动覆盖该值；用 `docker run` 时若 `.env` 里是 `HOST=127.0.0.1`，请改为 `0.0.0.0` 或用 `-e HOST=0.0.0.0` 覆盖。

启动后可验证健康检查：

```bash
curl http://127.0.0.1:8080/health
```

## API 文档

完整的 HTTP 接口说明见 [docs/api.md](docs/api.md)。

## 免责声明

本项目仅供学习与研究，与 Craft Egg / Bushiroad 无关。请勿滥用官方 API，注意请求频率；爬取数据仅供个人研究使用。


## License

This project is licensed under the MIT License.
