#!/usr/bin/env bash
# Pi-Monitor Deploy Script
# ========================
#
# Copies the compiled binary to the Raspberry Pi via SCP and optionally starts it.
#
# Usage:
#   ./scripts/deploy.sh                    # Deploy and start
#   ./scripts/deploy.sh --no-start         # Deploy only, don't start
#   PI_HOST=192.168.1.50 ./scripts/deploy.sh   # Override Pi IP
#
# Prerequisites:
#   - Binary must be built first: ./scripts/build.sh
#   - Pi must be reachable via SSH (Dropbear on port 22)
#   - SSH key or password authentication configured
#
# What this script does:
#   1. Checks that the binary exists (you ran build.sh first)
#   2. Kills any running pi-monitor process on the Pi
#   3. Copies the new binary to /bin/ on the Pi via SCP
#   4. Starts the new binary on the Pi (unless --no-start)
#   5. Verifies the process is running and the HTTP endpoint responds

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# ============================================================================
# Configuration — edit these for your setup
# ============================================================================

# Pi's IP address — override with PI_HOST environment variable
PI_HOST="${PI_HOST:-192.168.1.100}"

# SSH user — RustPi runs as root (no other users on a minimal system)
PI_USER="${PI_USER:-root}"

# SSH port — Dropbear default
PI_PORT="${PI_PORT:-22}"

# Where to install the binary on the Pi
# /bin/ is on the root filesystem and is in PATH
PI_INSTALL_PATH="/bin/pi-monitor"

# Port the monitor listens on (for verification)
MONITOR_PORT="9100"

# ============================================================================
# Build paths
# ============================================================================

TARGET="aarch64-unknown-linux-musl"
BINARY_NAME="pi-monitor"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

# Parse arguments
NO_START=false
for arg in "$@"; do
    case $arg in
        --no-start) NO_START=true ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

echo -e "${YELLOW}=== Pi-Monitor Deploy ===${NC}"
echo "  Target: ${PI_USER}@${PI_HOST}:${PI_INSTALL_PATH}"
echo ""

# Step 1: Verify binary exists
echo -e "${YELLOW}[1/4] Checking binary...${NC}"
if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}ERROR: Binary not found at ${BINARY_PATH}${NC}"
    echo "Run ./scripts/build.sh first!"
    exit 1
fi
BINARY_SIZE=$(ls -lh "${BINARY_PATH}" | awk '{print $5}')
echo "  Binary: ${BINARY_PATH} (${BINARY_SIZE})"
echo ""

# Step 2: Stop any running instance on the Pi
echo -e "${YELLOW}[2/4] Stopping existing pi-monitor on Pi (if running)...${NC}"
# We use `|| true` because killall returns non-zero if the process isn't running,
# and we have set -e which would make the script exit on that "error".
# This is a common shell pattern: "try to do this, but don't fail if it doesn't work"
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "killall ${BINARY_NAME} 2>/dev/null || true"
echo "  Done (any previous instance stopped)"
echo ""

# Step 3: Copy binary to Pi
echo -e "${YELLOW}[3/4] Uploading binary to Pi...${NC}"
# SCP flags:
#   -P: SSH port (note: scp uses uppercase -P, ssh uses lowercase -p)
#   -o StrictHostKeyChecking=no: Don't prompt about unknown host keys
#     (The Pi's host key may change if you rebuild the SD card)
scp -P "${PI_PORT}" -o StrictHostKeyChecking=no \
    "${BINARY_PATH}" "${PI_USER}@${PI_HOST}:${PI_INSTALL_PATH}"

# Make sure it's executable (SCP should preserve permissions, but be safe)
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" "chmod +x ${PI_INSTALL_PATH}"
echo "  Uploaded and set executable"
echo ""

# Step 4: Start it (unless --no-start)
if [ "${NO_START}" = true ]; then
    echo -e "${GREEN}=== Deploy complete (not started — use --no-start was specified) ===${NC}"
    echo "To start manually: ssh ${PI_USER}@${PI_HOST} '${PI_INSTALL_PATH} &'"
    exit 0
fi

echo -e "${YELLOW}[4/4] Starting pi-monitor on Pi...${NC}"

# Start the process in the background on the Pi
# Explanation of this command:
#   nohup: Don't kill the process when the SSH session ends
#          (Without nohup, background processes get SIGHUP when SSH disconnects)
#   > /dev/null 2>&1: Redirect stdout and stderr to /dev/null
#          (On a minimal system, there's no log rotation — we'd fill up the disk)
#          (In a later phase, we could log to a file with size limits)
#   &: Run in the background so the SSH command can return
ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" \
    "nohup ${PI_INSTALL_PATH} > /dev/null 2>&1 &"

# Give it a moment to start
sleep 2

# Verify it's running
echo "  Checking if process is alive..."
RUNNING=$(ssh -p "${PI_PORT}" "${PI_USER}@${PI_HOST}" \
    "ps | grep ${BINARY_NAME} | grep -v grep || true")

if [ -n "${RUNNING}" ]; then
    echo -e "  ${GREEN}✓ pi-monitor is running!${NC}"
    echo "  ${RUNNING}"
else
    echo -e "  ${RED}✗ pi-monitor does not appear to be running${NC}"
    echo "  Try starting it manually to see error output:"
    echo "    ssh ${PI_USER}@${PI_HOST} '${PI_INSTALL_PATH}'"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Deploy successful! ===${NC}"
echo ""
echo "Test it:"
echo "  From Pi:  wget -qO- http://localhost:${MONITOR_PORT}/health"
echo "  From Mac: curl http://${PI_HOST}:${MONITOR_PORT}/metrics"
echo "  Dashboard: http://${PI_HOST}:${MONITOR_PORT}/"