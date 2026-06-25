# Grid - High-Performance Kanban Board

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/grid/main/logo.png" alt="Grid Logo" width="128" height="128">
</p>

Grid is a clean, secure, and lightning-fast self-hosted Kanban board application built in Rust using Yew (WebAssembly frontend) and Axum (API backend).

---

## 📦 Container Registry

The Docker image is published to the following registries:

*   **Docker Hub (Recommended)**: [ubermetroid/grid](https://hub.docker.com/r/ubermetroid/grid)
*   **GitHub Container Registry (GHCR)**: [ghcr.io/ubermetroid/grid](https://github.com/UberMetroid/grid/pkgs/container/grid)

---

## 🐳 Container Installation



1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  grid:
    image: ubermetroid/grid:latest
    container_name: grid
    restart: unless-stopped
    ports:
      - 4405:4405
    volumes:
      - ./data:/app/data
    environment:
      - PORT=4405
      - SITE_TITLE=Grid
      - BASE_URL=http://localhost:4405
      - ALLOWED_ORIGINS=*
      - GRID_PIN=1234
      - TZ=UTC
      - ENABLE_TRANSLATION=false
      - ENABLE_THEMES=true
      - ENABLE_PRINT=true
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4405`.

### Building the Image Locally

To build the Docker container locally from the source files:

```bash
docker build -t ubermetroid/grid:latest .
```


---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4405` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. *(Supports fallback `RUSTGRID_TITLE`)* | `Grid` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies to ensure redirect and websocket links are resolved correctly. | `http://localhost:4405` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `GRID_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `false` |
| `ENABLE_THEMES` | Enable the Super Metroid theme selector in the navigation header (true/false). | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header (true/false). | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |



---

*Note: This repository was forked from [DumbKan](https://github.com/DumbWareio/DumbKan).*
