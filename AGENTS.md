# welfare-store / gs-store-system

## Project Overview

This repository is a welfare store platform. The main backend service lives in
`gs-store-system-backend/` and is a Rust API for the mini app and admin system.

The backend uses:

- `axum` for HTTP routing
- `sqlx` with MySQL for persistence
- `redis` for cache/session-style state
- `jsonwebtoken` and `bcrypt` for auth
- OSS integration for file upload
- JK Pay integration for payment flows

The repo also contains a `frontend/` app. Deployment is handled by the root
`deploy.ps1` script.

## Important Paths

- `gs-store-system-backend/src/main.rs` - service entrypoint
- `gs-store-system-backend/src/config.rs` - environment configuration
- `gs-store-system-backend/src/state.rs` - shared app state
- `gs-store-system-backend/src/routes/` - API routes
- `gs-store-system-backend/src/models/` - database models
- `gs-store-system-backend/migrations/` - SQL migrations
- `deploy.ps1` - the single deployment entrypoint for backend/frontend publishing

## Deployment Notes

- Use `deploy.ps1` for all release work; there are no separate helper deployment scripts.
- `deploy.ps1 -Target backend` publishes the backend service.
- `deploy.ps1 -Target frontend` publishes the frontend dist bundle.
- `deploy.ps1 -Target all` runs backend then frontend.
- `deploy.ps1 -Target frontend -FrontendDir <path>` deploys a specific frontend project when needed.
- Default remote deployment values used by the script:
  - host: `47.103.220.84`
  - user: `root`
  - deploy dir: `/root/workspace/gs-store-system`
  - port: `8081`
- Deployment credentials and runtime values are read only from the project env file
  at the repository root (`.env` preferred, otherwise `env`). Do not hardcode SSH
  passwords in scripts or rely on machine/user environment variables for deployment.

## Housekeeping

- Generated artifacts such as `target/`, `logs/`, and `cargo-run.*` files are
  disposable.
- The old `.Codex/` helper tree has been retired in favor of this file.
- If a task touches deployment behavior, update `deploy.ps1` so the
  workflow stays in one place.


