#!/bin/bash
# RFC 2544 reflector for test verification
# Runs on rockylinux:10 with SELinux compatibility
set -euo pipefail

# Start HTTP server for monitoring
httpd -DFOREGROUND -p 42 --cors &

# Echo RFC2544 test packets back
while true; do
    echo "reflector ready" | nc -ul -p 42 > /dev/null 2>&1 || true
done
