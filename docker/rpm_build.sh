#!/bin/bash
# docker/rpm_build.sh — Build fastlane RPM using rockylinux:8
#
# Uses a rockylinux:8 container with an rw volume mount for the
# source code. The container performs rpmbuild and outputs the
# RPM to the host via the volume mount.
#
# Rocky Linux 8 is chosen for forward glibc compatibility:
#   - Rocky 8: glibc 2.28
#   - Rocky 9: glibc 2.34
#   - Rocky 10: glibc 2.38
#   - A binary built on glibc 2.28 runs on all later versions.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
IMAGE_NAME="fastlane-rpm-builder:latest"
IMAGE_TAG="fastlane-rpm-builder:latest"
VOLUME_MOUNT="${PROJECT_DIR}:/source:rw"
OUTPUT_DIR="${PROJECT_DIR}/rpm-builds"

echo "============================================"
echo "  fastlane RPM Build (Rocky Linux 8)"
echo "============================================"
echo ""
echo "Project dir:   ${PROJECT_DIR}"
echo "Output dir:    ${OUTPUT_DIR}"
echo "Volume mount:  ${VOLUME_MOUNT}"
echo ""

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Build the RPM builder image (or use existing)
echo "--- Building RPM builder image ---"
docker build -t "${IMAGE_TAG}" -f "${PROJECT_DIR}/Dockerfile.rpm" "${PROJECT_DIR}"

# Run rpmbuild in the container with volume mount
echo ""
echo "--- Running rpmbuild (rw volume mount) ---"
docker run --rm \
    -v "${VOLUME_MOUNT}" \
    -v "${OUTPUT_DIR}:/root/rpm-output:rw" \
    "${IMAGE_TAG}" \
    bash -c '
        cd /root/rpmbuild
        rpmbuild -bb /root/rpmbuild/fastlane.spec
        cp RPMS/x86_64/fastlane*.rpm /root/rpm-output/
        cp SRPMS/fastlane*.rpm /root/rpm-output/ 2>/dev/null || true
        echo "RPM build complete!"
        ls -lh /root/rpm-output/
    '

# Show results
echo ""
echo "============================================"
echo "  RPM Build Results"
echo "============================================"
echo ""
ls -lh "${OUTPUT_DIR}/"
echo ""
echo "Installed RPMs:"
rpm -qpR "${OUTPUT_DIR}/fastlane-*.rpm" 2>/dev/null | sort | uniq || echo "  (verify with: rpm -qpR rpm-builds/fastlane-*.rpm)"
echo ""
echo "To install on Rocky/RHEL:"
echo "  rpm -ivh rpm-builds/fastlane-1.3.0-1.el8.x86_64.rpm"
echo ""
