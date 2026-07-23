<h1 align="center">
  <img src="assets/icon.png?v=1.0.31" width="48" height="48" valign="middle"> Grid
</h1>

<p align="center">
  <b>Fast, secure self-hosted visual kanban board and task manager written in Rust.</b>
</p>

---

### Instant One-Line Install (Docker Container)

Run the official zero-dependency container on port 4401:

```bash
docker run -d --name grid -p 4401:4401 -v /mnt/user/appdata/grid:/config ghcr.io/studio2201/grid:latest
```

Open your browser to `http://localhost:4401` to start managing boards immediately.

---

### Environment Configuration

The backend service can be customized using the following environment variables:

| Variable | Description | Default |
| :--- | :--- | :---: |
| `PORT` | Network port the web server binds to | `4401` |
| `GRID_PIN` | Security PIN required for application access | *(Disabled)* |
| `GRID_DATA_DIR` | Directory path for persistent data and boards | `/config` |
| `GRID_ALLOWED_ORIGINS` | CORS allowed origins list (comma-separated) | `*` |
| `TRUST_PROXY` | Honor reverse proxy headers (`X-Forwarded-For`) | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated CIDR list of trusted reverse proxies | *(None)* |
| `LOG_LEVEL` | Tracing filter (`error`, `warn`, `info`, `debug`) | `info` |

---

### Administration CLI & TUI Dashboard

Every container and package includes a built-in administration utility (`grid`).

Launch interactive TUI dashboard:
```bash
docker exec -it grid grid tui
```

System diagnostics and self-healing check:
```bash
docker exec -it grid grid doctor
```

CLI Command Reference:
- `grid tui` — Interactive terminal user interface.
- `grid doctor` — Diagnoses storage permissions, ports, and database health.
- `grid status` — Displays network configuration and security parameters.
- `grid data stats` — Shows storage utilization and entry metrics.
- `grid data list` — Lists kanban boards and task items.

---

### Architecture & Security

- **Axum Web Backend**: High-concurrency async HTTP/JSON runtime built on Tokio.
- **Yew WebAssembly Frontend**: Type-safe client bundle running natively in browser WASM runtime.
- **Strict Input & Path Sanitization**: Path canonicalization guards preventing directory traversal escapes.
- **Fail-Closed Security PIN Authentication**: Rate-limited brute force protection with automatic lockout timers.

---

### License

Distributed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <a href="https://github.com/studio2201/grid">
    <img src="assets/corgi-footer.jpg" alt="studio2201 banner" width="100%">
  </a>
</p>
