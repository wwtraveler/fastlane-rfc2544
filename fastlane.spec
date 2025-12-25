# fastlane.spec — RPM spec file for fastlane RFC 2544 benchmarking tool
#
# Builds on Rocky Linux 8 for forward glibc compatibility (glibc 2.28+).
# A binary built on rockylinux:8 (glibc 2.28) will run on:
#   - Rocky 8, 9, 10 (glibc 2.28 → 2.38)
#   - RHEL 8, 9, 10 (glibc 2.28 → 2.38)
#   - CentOS 8, 9
#
# This is because glibc is FORWARD compatible: a binary that links
# against glibc 2.28 will run on any later glibc without issues.

Name:           fastlane
Version:        1.3.0
Release:        1%{?dist}
Summary:        High-performance RFC 2544 network benchmarking tool
License:        MIT
URL:            https://github.com/wwtraveler/fastlane-rfc2544
BuildArch:      x86_64

# Sources
Source0:        .

# Build dependencies
BuildRequires:  cargo >= 1.75
BuildRequires:  rust >= 1.75
BuildRequires:  make >= 4.0
BuildRequires:  gcc >= 8.0

# Runtime dependencies
Requires:       libpcap >= 1.8
Requires:       glibc >= 2.28
Requires:       libaio >= 0.3
Requires:       libnuma >= 2.0

# Description
%description
fastlane is a blazing-fast RFC 2544 network benchmarking tool
written in Rust. Optimized for 400G+ interfaces with AF_XDP
zero-copy, multi-queue, lock-free ring buffers, and pre-allocated
memory pools. Implements all four core RFC 2544 tests plus extended
tests (Y.1564, RFC 2889, RFC 6349, Y.1731, MEF, TSN).

%build
cargo build --release --target x86_64-unknown-linux-gnu

%install
mkdir -p %{buildroot}/usr/bin
cp target/x86_64-unknown-linux-gnu/release/fastlane \
   %{buildroot}/usr/bin/fastlane

mkdir -p %{buildroot}/usr/share/doc/fastlane
cp README.md %{buildroot}/usr/share/doc/fastlane/
cp LICENSE %{buildroot}/usr/share/doc/fastlane/

mkdir -p %{buildroot}/usr/share/fastlane/config
for cfg in config/*.yaml; do
    cp "$cfg" %{buildroot}/usr/share/fastlane/config/ 2>/dev/null || true
done

mkdir -p %{buildroot}/etc/fastlane
cat > %{buildroot}/etc/fastlane/config.yaml <<'EOF'
# Default fastlane configuration
interface: eth0
test:
  - throughput
  - latency
  - frame_loss
  - back_to_back
frame_sizes: [64, 128, 256, 512, 1024, 1280, 1518, 9000]
include_jumbo: true
throughput:
  initial_rate_pct: 100.0
  resolution_pct: 0.1
  max_iterations: 25
  tolerated_frame_loss: 0.001
output:
  format: text
EOF

%files
/usr/bin/fastlane
/usr/share/doc/fastlane/README.md
/usr/share/doc/fastlane/LICENSE
/usr/share/fastlane/config/*.yaml
/etc/fastlane/config.yaml

%post
echo "fastlane RFC 2544 installed successfully!"
echo "  Run 'fastlane --help' for available options"
echo "  Config: /etc/fastlane/config.yaml"
echo "  Docs: /usr/share/doc/fastlane/README.md"

%changelog
* Thu Dec 12 2025 - wwtraveler <wwtraveler@users.noreply.github.com>
- Version 1.3.0
- AF_XDP zero-copy with 400G+ ring sizing
- Multi-queue parallel processing with N queues
- Pre-allocated memory pool (zero malloc during tests)
- Cache-line aligned ring buffers (64-byte padding)
- Lock-free SPSC/MPSC ring buffers
- Hugepage-backed buffers for zero page faults
- NUMA-aware thread binding
- All RFC 2544 core tests: throughput, latency, frame_loss, back_to_back
- Extended tests: Y.1564 (config/perf/full), Y.1731 (delay/loss/slm/loopback)
- Extended tests: RFC 2889 (forwarding/caching/learning/broadcast/congestion)
- Extended tests: RFC 6349 (throughput/path), MEF (config/perf/full), TSN (timing/isolation/latency/full)
- Rate limiting: CBR (accurate), Poison (fast), Hardware (NIC-level)
- Frame sizes: 64, 128, 256, 512, 1024, 1280, 1518, 9000 (jumbo)
- Output: text, JSON, CSV
- Hardware support: X540 (HW rate limit), I210/I350 (SW rate limit)
- Container-ready: rockylinux:8 glibc 2.28 (forward compatible with Rocky 9/10, RHEL 8/9/10)
- Systemd service included
- Default config at /etc/fastlane/config.yaml
