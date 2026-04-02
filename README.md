# Inker

Rust (Axum) backend + Next.js frontend for notes and interactive streaming (YouTube captions → ML processing).

## Quickstart

### Prereqs

- **Rust** toolchain (workspace uses **edition 2024**)
- **Node.js + npm** (for `frontend/`)
- **Postgres**

![Demo](/Users/bishoy/Documents/rust/inker/media/demo.png)

### 1) Start Postgres + run migrations

Set your DB URL (default in `inker.sh`):

```bash
export DATABASE_URL="postgres://postgres@localhost:5432/postgres"
```

Run migrations:

```bash
./inker.sh migrate up
```

### 2) Run the backend (port 3000)

```bash
export DATABASE_URL="postgres://postgres@localhost:5432/postgres"
export GEMINI_KEY="YOUR_GEMINI_API_KEY"
./inker.sh run
```

Useful optional env vars:

- **`FRONTEND_URL`**: defaults to `http://localhost:3001` (used for CORS)
- **`BACKEND_URL`**: defaults to `http://localhost:3000` (used for OAuth callbacks)
- **`DEBUG`**: enable verbose logging + Swagger UI (defaults to `true` in debug builds)

### 3) Run the frontend (port 3001)

```bash
./inker.sh frontend dev
```

## Development workflows

### Watch backend changes

Requires `cargo-watch` (`cargo install cargo-watch`).

```bash
./inker.sh watch
```

### Swagger UI / OpenAPI

When `DEBUG` is enabled, Swagger UI is available at:

- `http://localhost:3000/swagger-ui`

The OpenAPI JSON is served at:

- `http://localhost:3000/api-doc/openapi.json`

### Regenerate the frontend TypeScript SDK

This fetches OpenAPI from the backend and runs `openapi-ts`.

```bash
./inker.sh frontend generate-api
```

## API overview

Backend routes are nested under:

- **`/api/v1`**

Examples:

- **Notes**: `/api/v1/notes/...`
- **Auth**: `/api/v1/auth/...`
- **Interactive YouTube WebSocket**: `/api/v1/interactive/yt/ws`

## Repo layout

- **`crates/http`**: Axum HTTP API + WebSocket endpoints (binds `0.0.0.0:3000`)
- **`crates/migration`**: SeaORM migrations (see `crates/migration/README.md`)
- **`crates/entities`**: SeaORM entities
- **`crates/ml-processing`**: ML processing client/logic (Gemini)
- **`crates/yt-processing`**: YouTube captions processing
- **`frontend/`**: Next.js app (dev server on `3001`)

## Generating SeaORM entities (optional)

If you modify the DB schema and want to regenerate `crates/entities`:

```bash
./inker.sh generate
```

## Work in progress

- convert the interactive learning to notes
- improve the markdown editor layout
- add support for another way of interactive learning (eg. pdf, book)
