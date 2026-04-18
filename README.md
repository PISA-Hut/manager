# Scenario-Queue Manager

The Rust API server for the PISA Scenario-Queue project. It manages simulation scenario tasks, assigns them to executors, and tracks execution lifecycle.

## Building

```bash
cargo build
cargo build --release
```

## Running

The manager requires the following environment variables:

- `DATABASE_URL` — PostgreSQL connection string
- `AUTHENTICATOR_PASSWORD` — PostgREST role password (used by migrations)
- `MANAGER_BIND_ADDR` — bind address (default: `127.0.0.1`)
- `MANAGER_PORT` — listen port (default: `9000`)
- `MANAGER_CORS_ALLOW_ORIGINS` — optional CORS origins

```bash
RUST_LOG=debug cargo run
```

## Docker

```bash
docker build -t manager .
docker run -e DATABASE_URL=... -e AUTHENTICATOR_PASSWORD=... manager
```

## Full Stack

To run the manager together with PostgreSQL, PostgREST, Swagger UI, and nginx, see the [`infra/`](../infra/) directory.
