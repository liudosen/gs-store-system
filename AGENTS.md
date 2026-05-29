# gs-store-system

## Project Overview

This repository contains the GS Store System platform. It is a full-stack
monorepo: the Rust API service lives in `backend/`, and the admin web app lives
in `frontend/`.

The backend uses:

- `axum` for HTTP routing
- `sqlx` with MySQL for persistence
- `redis` for cache/session-style state
- `jsonwebtoken` and `bcrypt` for auth
- OSS integration for file upload
- JK Pay integration for payment flows

Deployment helpers live under `deploy/`.

## Important Paths

- `backend/src/main.rs` - service entrypoint
- `backend/src/config.rs` - environment configuration
- `backend/src/state.rs` - shared app state
- `backend/src/routes/` - API routes
- `backend/src/models/` - database models
- `backend/migrations/` - SQL migrations
- `frontend/` - admin frontend
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
