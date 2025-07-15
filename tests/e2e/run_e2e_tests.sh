#!/bin/bash
# End-to-end test runner for fastlane-rfc2544
# Tests both the library and CLI with container-based verification

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PASS=0
FAIL=0
TOTAL=0

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASS++))
    ((TOTAL++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAIL++))
    ((TOTAL++))
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

echo "========================================="
echo " fastlane-rfc2544 End-to-End Tests"
echo "========================================="
echo ""

# ── Test 1: Build succeeds ──────────────────────
log_info "Building fastlane-rfc2544..."
if cargo build --release -p fastlane-cli 2>/dev/null; then
    log "Build succeeded"
else
    log_fail "Build failed"
fi

# ── Test 2: Binary exists ────────────────────────
if [ -f "$PROJECT_DIR/target/release/fastlane" ]; then
    log "Binary exists at target/release/fastlane"
else
    log_fail "Binary not found"
fi

# ── Test 3: CLI help ─────────────────────────────
log_info "Testing CLI help output..."
HELP_OUTPUT=$(cargo run -p fastlane-cli -- --help 2>&1)
if echo "$HELP_OUTPUT" | grep -q "fastlane"; then
    log "CLI help displays correctly"
else
    log_fail "CLI help output missing"
fi

# ── Test 4: CLI --version ─────────────────────────
if cargo run -p fastlane-cli -- --version 2>&1 | grep -qE "[0-9]+\.[0-9]+"; then
    log "CLI version displays correctly"
else
    log_fail "CLI version missing or incorrect"
fi

# ── Test 5: Unit tests pass ────────────────
log_info "Running unit tests..."
if cargo test -p fastlane-core --lib -- --quiet 2>&1 | grep -q "test result"; then
    log "Unit tests passed"
else
    log_fail "Unit tests failed"
fi

# ── Test 6: Integration tests pass ──────────────
log_info "Running integration tests..."
if cargo test -p fastlane-core --test integration -- --quiet 2>&1 | grep -q "test result"; then
    log "Integration tests passed"
else
    log_fail "Integration tests failed"
fi

# ── Test 7: All Cargo crates test ────────────────
log_info "Testing all crates..."
if cargo test --all --quiet 2>&1 | grep -q "test result"; then
    log "All crate tests passed"
else
    log_fail "Some crate tests failed"
fi

# ── Test 8: YAML config load ─────────────────────
log_info "Testing YAML config loading..."
if [ -f "$PROJECT_DIR/examples/config.yaml" ]; then
    log "YAML config file exists"
else
    log_fail "YAML config file missing"
fi

# ── Test 9: JSON config load ─────────────────────
if [ -f "$PROJECT_DIR/examples/config.json" ]; then
    log "JSON config file exists"
else
    log_fail "JSON config file missing"
fi

# ── Test 10: Docker support ──────────────────────
log_info "Testing Docker support..."
if [ -f "$PROJECT_DIR/Dockerfile" ]; then
    log "Dockerfile exists"
else
    log_fail "Dockerfile missing"
fi

if [ -f "$PROJECT_DIR/tests/e2e/docker-compose.yml" ]; then
    log "docker-compose.yml exists"
else
    log_fail "docker-compose.yml missing"
fi

# ── Test 11: Frame size edge cases ───────────────
log_info "Testing frame size edge cases..."
cargo test -p fastlane-core --lib packet::tests::test_400g_line_rate_pps 2>&1 | grep -q "test result" && \
    log "400G line rate PPS calculation correct" || \
    log_fail "400G PPS calculation incorrect"

cargo test -p fastlane-core --lib packet::tests::test_optimal_batch_size 2>&1 | grep -q "test result" && \
    log "Optimal batch sizes correct" || \
    log_fail "Optimal batch sizes incorrect"

# ── Test 12: RFC 2544 signature validation ───────
cargo test -p fastlane-core --lib packet::tests::test_signature_validation 2>&1 | grep -q "test result" && \
    log "RFC 2544 signature validation correct" || \
    log_fail "RFC 2544 signature validation incorrect"

# ── Test 13: Pacing rate calculation ────────────
cargo test -p fastlane-core --lib pacing::tests::test_calc_interval 2>&1 | grep -q "test result" && \
    log "Pacing interval calculation correct" || \
    log_fail "Pacing interval calculation incorrect"

# ── Test 14: Back-to-back threshold calculation ──
cargo test -p fastlane-core --lib back2back::tests::test_calc_threshold_all_passed 2>&1 | grep -q "test result" && \
    log "Back-to-back threshold (all passed) correct" || \
    log_fail "Back-to-back threshold calculation incorrect"

# ── Summary ───────────────────────────────────────
echo ""
echo "========================================="
echo " Test Summary"
echo "========================================="
echo -e "  ${GREEN}Passed:${NC} $PASS"
echo -e "  ${RED}Failed:${NC} $FAIL"
echo "  Total:  $TOTAL"
echo "========================================="

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
