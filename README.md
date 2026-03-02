# Pi-Monitor

Lightweight system monitoring agent for Raspberry Pi, written in Rust.

Collects CPU, memory, disk, and network metrics from `/proc` and exposes them via a Prometheus-compatible HTTP endpoint.

## Quick Start

```bash
# 1. Start the build VM
vagrant up

# 2. Build the static binary
vagrant ssh -c "cd /vagrant && ./scripts/build.sh"

# 3. Deploy to your Pi
PI_HOST=<your-pi-ip> ./scripts/deploy.sh

# 4. Check metrics
curl http://<your-pi-ip>:9100/metrics
```

## Architecture

- **Metrics collectors** — parse `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`, etc.
- **HTTP server** — tokio + hyper serving Prometheus, JSON, and HTML dashboard
- **Single static binary** — compiled with musl, no runtime dependencies

## Target Environment

- Raspberry Pi 3A+ running [RustPi](../rustpi/) (custom minimal Linux)
- No package manager, no glibc, no systemd
- Binary must be fully static (musl) and self-contained

## Project Status

- [x] Phase 1: Project setup & build pipeline
- [x] Phase 2: CPU metrics collection
- [x] Phase 3: HTTP server & API
- [ ] Phase 4: Web dashboard
- [ ] Phase 5: Deploy & test
- [ ] Phase 6: Persistent startup (optional)