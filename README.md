<p align="center">
  <a href="https://github.com/etecoons">
    <img src="assets/header.jpg" alt="etecoons banner" width="100%">
  </a>
</p>

# Grid — High-Performance Kanban Board <img src="https://raw.githubusercontent.com/etecoons/unraid-apps/main/icons/grid.png" width="48" height="48" alt="grid logo" align="right">

Grid is a clean, secure, and lightning-fast self-hosted Kanban board application. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## Architecture & Stack
* **Frontend**: Yew (WASM)
* **Backend**: Axum (Rust) / Tokio
* **Deployment**: UBI container (Red Hat UBI9) on Docker Hub / Unraid / Podman / Docker Compose

---

## Key Features
* **Kanban Board**: Drag-and-drop task tracking cards across custom workflow columns.
* **Local Database Sync**: Reliable task persistence with quick backups.
* **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
* **Dynamic Themes**: Super Metroid UI themes (Crateria, Brinstar, Norfair, Wrecked Ship, Maridia, Tourian).
* **Internationalization**: Built-in multilingual translation selector support.
* **Print Optimization**: Customized print stylesheet layout and print header action button.
* **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.

---

## Deployment & Installation

### Container images (Docker Hub)

Images are **UBI9-minimal** based (Red Hat Universal Base Image). Tags:

| Tag | Meaning |
| :--- | :--- |
| `latest` | Current recommended build |
| `ubi` | Explicit UBI image (same lineage as `latest`) |
| `3.0.21` | Immutable release pin |

```bash
# Pull examples
podman pull docker.io/ghcr.io/etecoons/grid:latest
podman pull docker.io/ghcr.io/etecoons/grid:ubi
podman pull docker.io/ghcr.io/etecoons/grid:3.0.21
```

Hub: [https://hub.docker.com/r/ghcr.io/etecoons/grid](https://hub.docker.com/r/ghcr.io/etecoons/grid)

### Docker Compose
Create a `docker-compose.yml` file with the following service definition:

```yaml
services:
 grid:
 image: ghcr.io/etecoons/grid:latest
 container_name: grid
 restart: unless-stopped
 ports:
 - ${PORT:-4405}:4405
 volumes:
 - ${GRID_DATA_PATH:-./data}:/app/data
 environment:
 PORT: 4405
 SITE_TITLE: ${GRID_SITE_TITLE:-Grid}
 GRID_PIN: ${GRID_PIN:-}
 BASE_URL: ${GRID_BASE_URL:-http://localhost:4405}
 ALLOWED_ORIGINS: ${GRID_ALLOWED_ORIGINS:-*}
 TZ: ${TZ:-UTC}
 ENABLE_TRANSLATION: ${ENABLE_TRANSLATION:-false}
 ENABLE_THEMES: ${ENABLE_THEMES:-true}
 ENABLE_PRINT: ${ENABLE_PRINT:-true}
 MAX_ATTEMPTS: ${MAX_ATTEMPTS:-5}
```

### Build the UBI image locally

Requires [Podman](https://podman.io/) (or Docker) and network access to pull base images and crates.

```bash
# From the repository root
podman build --format docker -f Containerfile.ubi \
 -t docker.io/ghcr.io/etecoons/grid:3.0.21 \
 -t docker.io/ghcr.io/etecoons/grid:latest \
 -t docker.io/ghcr.io/etecoons/grid:ubi \
 .

# Optional: push all three tags
podman push docker.io/ghcr.io/etecoons/grid:3.0.21
podman push docker.io/ghcr.io/etecoons/grid:latest
podman push docker.io/ghcr.io/etecoons/grid:ubi
```

---

## Configuration Options

| Environment Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4405` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. | `Grid` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies. | `http://localhost:4405` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). | `*` |
| `GRID_PIN` | Optional 4–10 digit numerical PIN to lock access to the interface. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header. | `false` |
| `ENABLE_THEMES` | Enable the Super Metroid theme selector in the navigation header. | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header. | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before rate lockout. | `5` |
| `LOCKOUT_TIME_MINUTES` | Lockout duration in minutes for IPs exceeding `MAX_ATTEMPTS`. | `15` |
| `COOKIE_MAX_AGE_HOURS` | Duration in hours that the user's PIN session cookie remains valid. | `24` |
| `SHUTDOWN_DRAIN_SECONDS` | Seconds to wait for active connections to finish before shutting down. | `5` |
| `SHOW_VERSION` | Display the application version number in the footer. | `true` |
| `SHOW_GITHUB` | Display the GitHub repository link in the footer. | `true` |
| `TRUST_PROXY` | Set `true` if backend is hosted behind a reverse proxy. | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated IP/CIDR list of trusted upstream proxies. | None |

---

## Local Development

Ensure you have the Rust toolchain and Trunk installed.

```bash
# 1. Run workspace tests
cargo test

# 2. Run clippy workspace checks
cargo clippy --workspace --all-targets

# 3. Start frontend Yew dev server (from frontend/)
cd frontend && trunk serve

# 4. Start backend Axum server (from backend/)
cd backend && cargo run
```

---

## License
Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 etecoons.
