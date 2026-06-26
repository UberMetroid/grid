# Grid - High-Performance Kanban Board

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/grid/main/frontend/Assets/favicon.png" alt="Grid Logo" width="128" height="128">
</p>

Grid is a clean, secure, and lightning-fast self-hosted Kanban board application. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## ⚡ Key Features

*   **Dynamic Themes**: Dynamic theme options.
*   **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
*   **Internationalization**: Built-in multilingual translation selector support.
*   **Print Optimization**: Customized print stylesheet layout and print header action button.
*   **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.
*   **Kanban Board**: Drag-and-drop task tracking cards across custom workflow columns.
*   **Local Database Sync**: Reliable task persistence with quick backups.

---

## 📦 Container Registry

The Docker image is built with **Nix** (no Alpine, fully reproducible) and published to Docker Hub:

*   **Docker Hub**: [ubermetroid/grid](https://hub.docker.com/r/ubermetroid/grid)

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
      - ENABLE_PRINT=false
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4405`.

### Building the Image Locally

To build the Docker container locally from the source files using Nix:

```bash
nix build .#dockerImage
docker load < result
docker tag grid-nix:latest ubermetroid/grid:latest
```

The image is Nix-built (no Alpine, no Docker daemon dependency for the build).
For development iteration, use the devShell:

```bash
nix develop
```

### APT (Debian / Ubuntu)

Grid is also distributed as a `.deb` package from the official UberMetroid APT repository:

```bash
curl -fsSL https://ubermetroid.github.io/packages/apt/install.sh | sudo bash
sudo apt install grid
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
| `ENABLE_PRINT` | Enable the print button in the navigation header (true/false). | `false` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |



---

*Note: This repository was forked from [DumbKan](https://github.com/DumbWareio/DumbKan).*
