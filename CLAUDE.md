# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

HIT-measure is a **Tauri v2 desktop application** (Rust backend + Vue 3 frontend) that:
1. Subscribes to an MQTT broker for sensor data (temperature, humidity, photoelectric)
2. Streams real-time measurements to the frontend via Tauri's IPC event system (`app.emit` / `listen`)
3. Persists measurements to PostgreSQL on a configurable interval
4. Optionally captures RTSP video via ffmpeg and displays it in the UI through file-based polling + Asset Protocol

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Tauri Desktop App                                   │
│                                                      │
│  ┌─ Rust Backend (src-tauri/) ─────────────────┐    │
│  │                                              │    │
│  │  remote/mod.rs   → MQTT subscription loop   │    │
│  │  data/mod.rs     → CSV parsing + app.emit   │    │
│  │  db/mod.rs       → PostgreSQL periodic write│    │
│  │  mjpeg.rs        → ffmpeg RTSP→JPEG pipeline │    │
│  │  env.rs          → .env config (Lazy<>)     │    │
│  │  dto/mod.rs      → MeasureSample struct     │    │
│  │                                              │    │
│  └──────────────────────────────────────────────┘    │
│                      ↕ Tauri IPC                      │
│  ┌─ Vue Frontend (src/) ───────────────────────┐    │
│  │                                              │    │
│  │  App.vue  → listen("measure"), poll JPEG     │    │
│  │                                              │    │
│  └──────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

### Key Data Flows

**Real-time measurement push** (NOT HTTP SSE):
1. `remote/mod.rs`: MQTT subscribe → polls `connection.poll()` → on publish, calls `data::emit_measure_from_mqtt()`
2. `data/mod.rs`: parses CSV payload `temperature,humidity,photoelectric` → writes to `db::LAST_SAMPLE` (RwLock) → `app.emit("measure", sample)`
3. `App.vue`: `await listen("measure", handler)` receives the event and updates reactive state

**Database persistence** (separate from push):
- `db/mod.rs`: `spawn_periodic_insert()` runs a tokio interval loop that reads `LAST_SAMPLE` and INSERTs into `measure_samples` table at `MEASURE_PERSIST_INTERVAL_SECS`

**MJPEG video preview**:
- `mjpeg.rs`: ffmpeg reads RTSP → MJPEG pipe → JPEG frames parsed from stdout → written to `var/stream/last.jpg`
- `App.vue`: `convertFileSrc()` + 120ms interval polling with cache-busting `?t=...` query param

## Development Commands

### Prerequisites
- Rust toolchain (cargo, rustc)
- Node.js + npm
- ffmpeg (for RTSP preview)
- mosquitto-clients (`mosquitto_pub` for testing)
- PostgreSQL (or `docker compose up -d` via `docker-compose.yml`)

### Core Commands
```bash
# Start dev mode (Vite + Tauri window)
npm run tauri:dev

# Just start Vite dev server (no Tauri)
npm run dev

# Build frontend
npm run build

# Publish a test MQTT message (requires .env configured)
npm run test:mqtt
```

### Build & Package
```bash
# Build Tauri desktop app
npm run tauri build

# Tauri dev with direct CLI
npx tauri dev
```

### Database
```bash
# Start PostgreSQL via Docker
docker compose up -d

# The table `measure_samples` is auto-created on app startup
```

## Configuration

All config is in `.env` (loaded via `dotenvy` from `src-tauri/.env` or project root):

| Variable | Default | Purpose |
|---|---|---|
| `MQTT_BROKER_HOST` | `127.0.0.1` | MQTT broker address |
| `MQTT_BROKER_PORT` | `1883` | MQTT broker port |
| `MQTT_CLIENT_ID` | `measure-1` | Client ID for connection |
| `MQTT_MEASURE_TOPIC` | `measure/data` | Subscription topic for sensor data |
| `MQTT_USERNAME` | (empty) | MQTT auth username |
| `MQTT_PASSWORD` | (empty) | MQTT auth password |
| `MQTT_QOS` | `0` | QoS level for subscription |
| `DATABASE_URL` | (empty) | PostgreSQL connection string |
| `MEASURE_PERSIST_INTERVAL_SECS` | `5` | DB write interval |
| `RTSP_RELAY_ENABLED` | `false` | Enable RTSP→MJPEG preview |
| `RTSP_RELAY_SOURCE` | (empty) | RTSP stream URL |
| `DEV_SERVER_PORT` | `5180` | Vite dev server port |

## Important Patterns

- **`LAST_SAMPLE`** is a `Lazy<Arc<RwLock<Option<MeasureSample>>>>` — shared between MQTT handler (write) and DB writer (read)
- **MQTT runs in a dedicated `tokio::spawn`** — not inside the DB spawn block (was a bug, fixed in commit `1cf0a48`)
- **Error resilience**: MQTT connection errors trigger 1s sleep + retry; ffmpeg crashes trigger 1s sleep + restart
- **`mjpeg::start_mjpeg_feed()`** uses `Once` to ensure only one feed task runs
- **Measurement CSV format**: exactly `temperature,humidity,photoelectric` (three floats, comma-separated) — anything else is dropped with a warning
