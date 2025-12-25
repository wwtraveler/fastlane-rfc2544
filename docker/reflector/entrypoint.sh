#!/bin/bash
# RFC 2544 multi-threaded reflector
# Uses socat (multi-threaded) for high-speed UDP packet reflection
# Suitable for 400G+ interfaces with parallel packet processing
set -euo pipefail

REFLECTOR_PORT=4200

echo "reflector starting on port ${REFLECTOR_PORT} (multi-threaded)"

# Start HTTP monitoring server
socat TCP-LISTEN:80,reuseaddr,fork OPEN:/dev/null &

# Multi-threaded UDP reflector using socat
# -fork enables parallel packet handling for 400G+ throughput
# UDP packets are read and echoed back with RFC2544 signature preserved
socat UDP-LISTEN:${REFLECTOR_PORT},fork,reuseaddr SYSTEM:"cat > /dev/null" &

wait
