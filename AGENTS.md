# gs-store-system

## Project Overview

This repository contains the GS Store System platform. The main backend service
lives in `gs-store-system/` and is a Rust API for the mini app and admin system.

The backend uses:

- `axum` for HTTP routing
- `sqlx` with MySQL for persistence
- `redis` for cache/session-style state
- `jsonwebtoken` and `bcrypt` for auth
- OSS integration for file upload
- JK Pay integration for payment flows

The admin frontend lives in `backend/`. Deployment helpers live under `deploy/`.

## Important Paths

- `gs-store-system/src/main.rs` - service entrypoint
- `gs-store-system/src/config.rs` - environment configuration
- `gs-store-system/src/state.rs` - shared app state
- `gs-store-system/src/routes/` - API routes
- `gs-store-system/src/models/` - database models
- `gs-store-system/migrations/` - SQL migrations
- `backend/` - admin frontend
- `deploy/` - release scripts
- `deploy.ps1` - single entrypoint for backend/frontend publishing

## Deployment Notes

- Prefer `deploy.ps1` for release work.
- `deploy.ps1 -Target backend` publishes the backend service.
- `deploy.ps1 -Target frontend` publishes the frontend dist bundle.
- `deploy.ps1 -Target all` runs backend then frontend.
- Default remote deployment values used by the script:
  - host: `47.103.220.84`
  - user: `root`
  - deploy dir: `/root/workspace/gs-store-system`
  - port: `8081`
- SSH passwords must not be hardcoded in scripts; deployment helpers read them
  from the environment variable `DEPLOY_SSH_PASSWORD` only.

## Housekeeping

- Generated artifacts such as `target/`, `logs/`, and `cargo-run.*` files are
  disposable.
- If a task touches deployment behavior, update `deploy.ps1` first so the
  workflow stays in one place.
