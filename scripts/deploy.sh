#!/usr/bin/env bash
set -euo pipefail

PI_HOST="${PI_HOST:-192.168.1.100}"
PI_USER="${PI_USER:-root}"
PI_PORT="${PI_PORT:-22}"
PI_PATH="/bin/pi-monitor"
TARGET="aarch64-unknown-linux-musl"
BINARY="target/${TARGET}/release/pi-monitor"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

NO_START=false
[[ "${1:-}" == "--no-start" ]] && NO_START=true

echo -e "${YELLOW}=== Deploying to ${PI_USER}@${PI_HOST} ===${NC}"

[ -f "${BINARY}" ] || { echo -e "${RED}Binary not found — run build.sh first${NC}"; exit 1; }

# Stop existing instance
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "killall pi-monitor 2>/dev/null || true"

# Upload
scp -P "${PI_PORT}" -o StrictHostKeyChecking=no "${BINARY}" "${PI_USER}@${PI_HOST}:${PI_PATH}"
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "chmod +x ${PI_PATH}"
echo -e "  ${GREEN}✓ Uploaded${NC}"

if [ "${NO_START}" = true ]; then
    echo -e "${GREEN}=== Deployed (not started) ===${NC}"
    exit 0
fi

# Start
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "nohup ${PI_PATH} > /dev/null 2>&1 &"
sleep 2

RUNNING=$(ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "ps | grep pi-monitor | grep -v grep || true")
if [ -n "${RUNNING}" ]; then
    echo -e "  ${GREEN}✓ Running${NC}"
else
    echo -e "  ${RED}✗ Not running — try manually: ssh ${PI_USER}@${PI_HOST} '${PI_PATH}'${NC}"
    exit 1
fi

echo -e "${GREEN}=== Deploy successful ===${NC}"
echo "  curl http://${PI_HOST}:9100/metrics"

# Ensure auto-start entry exists in /etc/profile (idempotent)
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" '
  if ! grep -q pi-monitor /etc/profile 2>/dev/null; then
    cat >> /etc/profile << "STARTUP"

# Auto-start pi-monitor if not already running
if ! ps | grep -v grep | grep -q pi-monitor; then
  /bin/pi-monitor > /dev/null 2>&1 &
fi
STARTUP
    echo "  Auto-start added to /etc/profile"
  fi
'