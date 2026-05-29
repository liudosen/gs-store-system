# gs-store-system Project Structure

## Root

- `backend/` - admin frontend built with Vue and Vite.
- `gs-store-system/` - Rust API service for the admin system and mini app.
- `deploy/` - backend/frontend release and remote deployment helpers.
- `deploy.ps1` - unified deployment entrypoint.
- `AGENTS.md` - project guidance for local coding agents.

## Backend Service

- `gs-store-system/src/main.rs` - service entrypoint.
- `gs-store-system/src/config.rs` - environment configuration.
- `gs-store-system/src/state.rs` - shared application state.
- `gs-store-system/src/routes/` - HTTP route handlers.
- `gs-store-system/src/models/` - database models.
- `gs-store-system/migrations/` - SQL migrations.

## Admin Frontend

- `backend/src/api/` - API clients.
- `backend/src/layouts/` - admin shell layout.
- `backend/src/router/` - Vue Router configuration.
- `backend/src/stores/` - Pinia stores.
- `backend/src/views/` - admin pages.
- `backend/src/styles/` - shared styles.

## Generated Artifacts

Generated build outputs and logs are disposable and should not be treated as
source of truth:

- `target/`
- `dist/`
- `release/`
- `logs/`
- `cargo-run.*`
- `*.log`
