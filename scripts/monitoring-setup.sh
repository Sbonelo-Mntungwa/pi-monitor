#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PI_HOST="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MONITORING_DIR="${PROJECT_DIR}/monitoring"

if [ -z "${PI_HOST}" ]; then
    echo -e "${RED}Usage: $0 <pi-ip-address>${NC}"
    echo "  Example: $0 10.0.0.111"
    exit 1
fi

echo -e "${YELLOW}=== Pi-Monitor: Prometheus + Grafana Setup ===${NC}"
echo "  Pi target: ${PI_HOST}:9100"
echo ""

# Step 1: Check/install Homebrew dependencies
echo -e "${YELLOW}[1/5] Checking dependencies...${NC}"
if ! command -v brew &>/dev/null; then
    echo -e "${RED}Homebrew not found. Install it from https://brew.sh${NC}"
    exit 1
fi

for pkg in prometheus grafana; do
    if brew list "${pkg}" &>/dev/null; then
        echo "  ${pkg} ✓ (already installed)"
    else
        echo "  Installing ${pkg}..."
        brew install "${pkg}"
        echo "  ${pkg} ✓"
    fi
done
echo ""

# Step 2: Generate Prometheus config
echo -e "${YELLOW}[2/5] Generating Prometheus config...${NC}"
mkdir -p "${MONITORING_DIR}"
cat > "${MONITORING_DIR}/prometheus.yml" << EOF
global:
  scrape_interval: 5s
  evaluation_interval: 5s

scrape_configs:
  - job_name: 'pi-monitor'
    static_configs:
      - targets: ['${PI_HOST}:9100']
        labels:
          instance: 'rustpi'
EOF
echo "  Written to ${MONITORING_DIR}/prometheus.yml"
echo ""

# Step 3: Verify Pi is reachable
echo -e "${YELLOW}[3/5] Checking Pi connectivity...${NC}"
if curl -s --connect-timeout 3 "http://${PI_HOST}:9100/health" | grep -q "ok"; then
    echo -e "  ${GREEN}✓ Pi-Monitor is responding at ${PI_HOST}:9100${NC}"
else
    echo -e "  ${YELLOW}⚠ Could not reach Pi-Monitor at ${PI_HOST}:9100${NC}"
    echo "  Make sure pi-monitor is running on the Pi."
    echo "  Continuing setup anyway..."
fi
echo ""

# Step 4: Start Prometheus
echo -e "${YELLOW}[4/5] Starting Prometheus...${NC}"
if pgrep -x prometheus &>/dev/null; then
    echo "  Prometheus already running, restarting..."
    pkill -x prometheus || true
    sleep 1
fi

nohup prometheus \
    --config.file="${MONITORING_DIR}/prometheus.yml" \
    --storage.tsdb.path="${MONITORING_DIR}/prometheus-data" \
    --storage.tsdb.retention.time=30d \
    --web.listen-address="0.0.0.0:9090" \
    > "${MONITORING_DIR}/prometheus.log" 2>&1 &

sleep 2
if pgrep -x prometheus &>/dev/null; then
    echo -e "  ${GREEN}✓ Prometheus running on http://localhost:9090${NC}"
else
    echo -e "  ${RED}✗ Prometheus failed to start. Check ${MONITORING_DIR}/prometheus.log${NC}"
    exit 1
fi
echo ""

# Step 5: Start Grafana
echo -e "${YELLOW}[5/5] Starting Grafana...${NC}"
brew services start grafana 2>/dev/null || true
sleep 2
if curl -s --connect-timeout 3 "http://localhost:3000/api/health" | grep -q "ok"; then
    echo -e "  ${GREEN}✓ Grafana running on http://localhost:3000${NC}"
else
    echo -e "  ${YELLOW}⚠ Grafana may still be starting. Give it a few seconds.${NC}"
fi

echo ""
echo -e "${GREEN}=== Setup complete ===${NC}"
echo ""
echo "  Prometheus:  http://localhost:9090"
echo "  Grafana:     http://localhost:3000  (login: admin / admin)"
echo "  Pi-Monitor:  http://${PI_HOST}:9100"
echo ""
echo "Next steps:"
echo "  1. Open Grafana at http://localhost:3000"
echo "  2. Add data source: Connections → Data Sources → Prometheus → URL: http://localhost:9090"
echo "  3. Import dashboard: Dashboards → Import → Upload ${MONITORING_DIR}/grafana-dashboard.json"