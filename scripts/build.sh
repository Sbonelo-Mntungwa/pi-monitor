#!/usr/bin/env bash
# Pi-Monitor Build Script
# =======================
#
# Builds a statically-linked aarch64 binary using musl libc.
#
# Usage:
#   From inside the Vagrant VM:
#     cd /vagrant
#     ./scripts/build.sh
#
#   From the Mac (runs inside VM automatically):
#     cd ~/projects/pi-monitor
#     vagrant ssh -c "cd /vagrant && ./scripts/build.sh"
#
# What this script does:
#   1. Verifies the Rust toolchain and musl target are installed
#   2. Runs `cargo build --release --target aarch64-unknown-linux-musl`
#   3. Verifies the output binary is static and aarch64
#   4. Reports the binary size

set -euo pipefail
# set -e: Exit immediately if any command fails
# set -u: Treat unset variables as errors (catches typos)
# set -o pipefail: A pipeline fails if ANY command in it fails, not just the last one

# Colors for output (makes build output easier to scan)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TARGET="aarch64-unknown-linux-musl"
BINARY_NAME="pi-monitor"
BINARY_PATH="target/${TARGET}/release/${BINARY_NAME}"

echo -e "${YELLOW}=== Pi-Monitor Build ===${NC}"
echo ""

# Step 1: Verify toolchain
echo -e "${YELLOW}[1/4] Checking Rust toolchain...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: cargo not found. Is Rust installed?${NC}"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check that the musl target is installed
if ! rustup target list --installed | grep -q "${TARGET}"; then
    echo -e "${RED}ERROR: Target ${TARGET} not installed.${NC}"
    echo "Run: rustup target add ${TARGET}"
    exit 1
fi

echo "  rustc $(rustc --version | awk '{print $2}')"
echo "  cargo $(cargo --version | awk '{print $2}')"
echo "  target: ${TARGET} ✓"
echo ""

# Step 2: Build
echo -e "${YELLOW}[2/4] Building release binary...${NC}"
echo "  This may take a while on the first build (downloading + compiling dependencies)."
echo "  Subsequent builds will be much faster (only recompiling changed code)."
echo ""

# --release: Use the optimized release profile (our Cargo.toml has size optimizations)
# --target: Build for musl instead of the default glibc target
cargo build --release --target "${TARGET}" 2>&1

echo ""

# Step 3: Verify the binary
echo -e "${YELLOW}[3/4] Verifying binary...${NC}"

if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}ERROR: Binary not found at ${BINARY_PATH}${NC}"
    exit 1
fi

# `file` command tells us about the binary format
FILE_INFO=$(file "${BINARY_PATH}")
echo "  file: ${FILE_INFO}"

# Check it's statically linked
# A dynamically linked binary would show "dynamically linked, interpreter /lib/ld-linux..."
# A static binary shows "statically linked"
if echo "${FILE_INFO}" | grep -q "statically linked"; then
    echo -e "  ${GREEN}✓ Statically linked${NC}"
else
    echo -e "  ${RED}✗ NOT statically linked! Something is wrong with the musl config.${NC}"
    echo "  Check .cargo/config.toml and ensure musl-tools is installed."
    exit 1
fi

# Check it's aarch64
if echo "${FILE_INFO}" | grep -q "aarch64\|ARM aarch64"; then
    echo -e "  ${GREEN}✓ aarch64 architecture${NC}"
else
    echo -e "  ${RED}✗ Wrong architecture! Expected aarch64.${NC}"
    exit 1
fi

echo ""

# Step 4: Report size
echo -e "${YELLOW}[4/4] Binary size:${NC}"
BINARY_SIZE=$(ls -lh "${BINARY_PATH}" | awk '{print $5}')
BINARY_SIZE_BYTES=$(stat --format=%s "${BINARY_PATH}" 2>/dev/null || stat -f%z "${BINARY_PATH}" 2>/dev/null)
echo "  ${BINARY_PATH}"
echo "  Size: ${BINARY_SIZE} (${BINARY_SIZE_BYTES} bytes)"

# Warn if binary is unexpectedly large
# With our optimizations, the final binary should be under 5MB.
# If it's over 10MB, something might be wrong (debug symbols not stripped, etc.)
if [ "${BINARY_SIZE_BYTES}" -gt 10000000 ]; then
    echo -e "  ${YELLOW}⚠ Binary is over 10MB — consider checking for unnecessary dependencies${NC}"
elif [ "${BINARY_SIZE_BYTES}" -gt 5000000 ]; then
    echo -e "  ${YELLOW}⚠ Binary is over 5MB — acceptable but could be smaller${NC}"
else
    echo -e "  ${GREEN}✓ Good size for an embedded system${NC}"
fi

echo ""
echo -e "${GREEN}=== Build successful! ===${NC}"
echo ""
echo "Next step: deploy to your Pi with ./scripts/deploy.sh"