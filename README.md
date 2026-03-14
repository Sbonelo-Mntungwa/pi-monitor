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

## Endpoints

| Endpoint   | Description                          |
|------------|--------------------------------------|
| `/`        | Live web dashboard (auto-refresh)    |
| `/metrics` | Prometheus exposition format         |
| `/json`    | JSON format (all metrics)            |
| `/health`  | Liveness check (`ok`)                |

## Architecture

- **Metrics collectors** — parse `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`, etc.
- **HTTP server** — tokio + hyper serving Prometheus, JSON, and HTML dashboard
- **Single static binary** — compiled with musl, no runtime dependencies
- **Background CPU sampler** — reads `/proc/stat` every 2s, computes usage via tick diffs

## Target Environment

- Raspberry Pi 3A+ running [RustPi](../rustpi/) (custom minimal Linux)
- No package manager, no glibc, no systemd
- Binary must be fully static (musl) and self-contained

## Grafana & Prometheus Setup

Pi-Monitor exposes a standard Prometheus `/metrics` endpoint, so it integrates directly with Prometheus + Grafana for historical monitoring and alerting.

### Setup

Run the setup script to install and configure everything:

```bash
./scripts/monitoring-setup.sh <your-pi-ip>
```

This installs Prometheus and Grafana via Homebrew, generates the Prometheus config pointed at your Pi, and starts both services.

Then:

1. Open Grafana at `http://localhost:3000` (login: `admin` / `admin`)
2. **Connections → Data Sources → Prometheus** → URL: `http://localhost:9090` → **Save & Test**
3. **Dashboards → Import → Upload** `monitoring/grafana-dashboard.json`
4. Set **dark theme** in Profile → Preferences for the intended Tron aesthetic

Verify Prometheus is scraping at `http://localhost:9090/targets` — status should show `UP`.

### Useful PromQL Queries

```promql
# CPU usage (excluding idle)
100 - pi_cpu_usage_percent{cpu="total", mode="idle"}

# Memory usage percentage
pi_memory_used_bytes / pi_memory_total_bytes * 100

# Disk usage percentage
pi_disk_used_bytes{mountpoint="/"} / pi_disk_total_bytes{mountpoint="/"} * 100

# Network throughput (bytes/sec)
rate(pi_network_receive_bytes_total{interface="eth0"}[1m])
rate(pi_network_transmit_bytes_total{interface="eth0"}[1m])

# Load averages
pi_load_average{period="1m"}
pi_load_average{period="5m"}
pi_load_average{period="15m"}
```

## Project Structure

```
pi-monitor/
├── .cargo/config.toml        # musl cross-compilation config
├── Cargo.toml                # dependencies & release profile
├── Vagrantfile               # build VM configuration
├── scripts/
│   ├── build.sh              # build static binary
│   ├── deploy.sh             # SCP to Pi & start
│   └── monitoring-setup.sh   # install Prometheus + Grafana
├── monitoring/
│   ├── prometheus.yml         # Prometheus scrape config
│   └── grafana-dashboard.json # pre-built Grafana dashboard
└── src/
    ├── main.rs               # async entry point
    ├── dashboard/
    │   ├── mod.rs            # compile-time HTML embedding
    │   ├── index.html
    │   ├── style.css
    │   └── script.js
    ├── metrics/
    │   ├── mod.rs
    │   ├── cpu.rs            # /proc/stat parser + background sampler
    │   ├── memory.rs         # /proc/meminfo parser
    │   ├── disk.rs           # /proc/mounts + statfs()
    │   ├── network.rs        # /proc/net/dev parser
    │   └── system.rs         # /proc/loadavg + /proc/uptime
    └── server/
        ├── mod.rs            # TCP listener + hyper setup
        ├── routes.rs         # request routing
        └── prometheus.rs     # exposition format output
```

## License

MIT License — See LICENSE file
