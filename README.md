# Rust Coupled Fullstack

一个前后端耦合的 Rust 示例项目：Axum 后端提供 API，并直接托管 `frontend/` 静态页面。

## 运行

```powershell
cargo run
```

打开：

```text
http://127.0.0.1:3000
```

## API

- `GET /api/health`
- `GET /api/messages`
- `POST /api/messages`

`POST /api/messages` 请求体：

```json
{
  "text": "你好 Rust"
}
```
