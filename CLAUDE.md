# CLAUDE.md

## Project

GS Store System is a full-stack monorepo for the mini app API and the admin
console.

- `backend/` is the Rust API service. The crate name remains
  `gs-store-system`.
- `frontend/` is the Vue 3 admin frontend.
- `deploy/` contains release helpers.
- `deploy.ps1` is the preferred deployment entrypoint.

## Backend Layout

- `backend/src/main.rs` starts the service, loads `.env`, initializes logging,
  connects to MySQL and Redis, runs migrations unless `SKIP_MIGRATIONS` is set,
  creates the initial admin user when configured, warms JK Pay, and serves Axum.
- `backend/src/config.rs` owns environment parsing. It builds MySQL and Redis
  URLs from split variables and requires JWT, WeChat, JK Pay, and OSS settings.
- `backend/src/state.rs` contains shared app state: MySQL pool, Redis pool,
  auth settings, WeChat settings, JK Pay credentials, and OSS settings.
- `backend/src/routes/app.rs` wires `/health`, `/auth/*`, `/api/admin/*`,
  `/api/mini/*`, and public goods routes.
- `backend/src/routes/admin/` contains admin APIs for auth, dashboard,
  categories, goods, orders, users, subscriptions, uploads, permissions, and
  logs.
- `backend/src/routes/mini_app/` contains mini app APIs for login, user info,
  categories, goods, addresses, orders, health-card balance, subscription, and
  recharge.
- `backend/src/models/` contains SQLx request/response rows and DTOs.
- `backend/src/services/` contains account, inventory, OSS, secret handling,
  and JK Pay integration.
- `backend/migrations/` is the SQL migration source of truth.
- `backend/scripts/` holds helper scripts for backend verification.

## Frontend Layout

- `frontend/package.json` defines the Vue 3 + Vite admin app.
- `frontend/src/main.js` boots Vue, Pinia, router, and Arco Design Vue.
- `frontend/src/api/` contains Axios setup and admin API clients.
- `frontend/src/router/` defines login and authenticated admin routes.
- `frontend/src/stores/` contains Pinia auth/session state.
- `frontend/src/layouts/` contains the admin shell.
- `frontend/src/views/` contains dashboard, category, goods, order, WeChat user,
  subscription, log, login, and admin-user pages.
- `frontend/src/styles/` contains shared admin styling.

## Local Commands

Run the API from `backend/`:

```powershell
cd backend
cargo run
```

Run the admin frontend from `frontend/`:

```powershell
cd frontend
npm install
npm run dev
```

Build checks:

```powershell
cd backend
cargo check
cd ..\frontend
npm run build
```

The frontend dev server defaults to `127.0.0.1:8080` and proxies `/auth` and
`/api` to `http://127.0.0.1:8081` unless `VITE_API_BASE_URL` is set.

## Environment

- Keep local secrets in ignored files only: root `env` and `backend/.env`.
- Do not commit passwords, tokens, OSS keys, WeChat secrets, JK Pay credentials,
  or SSH passwords.
- Deployment reads `DEPLOY_SSH_PASSWORD` from the environment. Do not hardcode
  SSH passwords in scripts.
- `backend/.env` should use the variable names read by `backend/src/config.rs`.

Important backend variables include:

- `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_NAME`, `DATABASE_USER`,
  `DATABASE_PASSWORD`
- `REDIS_HOST`, `REDIS_PORT`, `REDIS_USERNAME`, `REDIS_PASSWORD`, `REDIS_DB`
- `JWT_SECRET`, `JWT_EXPIRY_HOURS`
- `SERVER_HOST`, `SERVER_PORT`
- `AUTH_REQUIRE_REDIS_SESSION`
- `ADMIN_USERNAME`, `ADMIN_PASSWORD`
- `WEIXIN_APPID`, `WEIXIN_SECRET`, `DEV_WECHAT_OPENID`
- `JK_SELLER_USERNAME`, `JK_SELLER_PASSWORD`
- `OSS_ENDPOINT`, `OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`, `OSS_BUCKET`,
  `OSS_DOMAIN`

## Deployment

Use `deploy.ps1` from the repository root.

```powershell
.\deploy.ps1 -Target backend
.\deploy.ps1 -Target frontend
.\deploy.ps1 -Target all
```

Deployment defaults:

- Host: `47.103.220.84`
- User: `root`
- Backend deploy dir: `/root/workspace/gs-store-system`
- Frontend remote dir: `/root/workspace/gs-store-system/frontend`
- Backend service port: `8081`

If deployment behavior changes, update `deploy.ps1` first, then the helper under
`deploy/`.

## Housekeeping

The following are generated or local-only and may be removed/recreated:

- `backend/target/`
- `frontend/node_modules/`
- `frontend/dist/`
- `release/`
- `logs/`
- `cargo-run.*`
- `*.log`
- IDE folders such as `.idea/` and `.vscode/`

The old root `PROJECT-STRUCTURE.md` and frontend planning notes were removed
because this file now carries the canonical project layout and the current code
is the source of truth for implemented admin pages.
