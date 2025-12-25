# Installing fastlane-rfc2544

## Prerequisites

- **Docker** with **buildx** (recommended) or legacy builder
- **Rust 1.75+** (for local build)
- **rockylinux/rockylinux:10** compatible container runtime

## Building Docker Images with buildx

### Prerequisites

Ensure you have the Docker Buildx plugin installed:

```bash
docker buildx version
# or install it:
docker plugins install buildx
```

### Build the fastlane binary (static, musl)

```bash
# Use buildx with a Rust builder container
docker buildx build \
  --target builder \
  --tag fastlane-builder \
  --load \
  .
```

### Build the main fastlane image

```bash
# Single-stage with buildx
docker buildx build \
  --tag fastlane:latest \
  --load \
  .
```

### Build all images together (multi-image)

```bash
# Build tester, reflector, and main image in parallel
docker buildx build \
  --tag fastlane:latest \
  --tag fastlane-tester:latest \
  --tag fastlane-reflector:latest \
  --progress=plain \
  --load \
  .
```

### Push to a registry

```bash
# Tag and push all images
REGISTRY="ghcr.io/wwtraveler"
docker buildx build \
  --tag ${REGISTRY}/fastlane:latest \
  --tag ${REGISTRY}/fastlane:v1.3.0 \
  --push \
  .

docker buildx build \
  --tag ${REGISTRY}/fastlane-tester:latest \
  --tag ${REGISTRY}/fastlane-tester:v1.3.0 \
  --push \
  ./docker/tester

docker buildx build \
  --tag ${REGISTRY}/fastlane-reflector:latest \
  --tag ${REGISTRY}/fastlane-reflector:v1.3.0 \
  --push \
  ./docker/reflector
```

### Push multi-arch (amd64 + arm64)

```bash
docker buildx create --use --name fastlane-builder --driver docker-container
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag ${REGISTRY}/fastlane:latest \
  --push \
  .
```

## Local Rust Build

```bash
# Install Rust 1.75+ (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build all workspace crates
cargo build --release

# Cross-compile static binary for rockylinux:10
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Verify the binary
file target/x86_64-unknown-linux-musl/release/fastlane
# Should show: statically linked
```

## Running the Tester

```bash
# Run with container-to-container test (reflector at 172.28.1.20)
docker run --rm \
  --network fastlane_net \
  fastlane-tester:latest \
  eth0 \
  --test throughput,latency \
  --size 64,128,256,512,1024,1518 \
  --jumbo \
  --num-queues 4 \
  --rate-type cbr \
  --resolution 0.5 \
  --max-iter 10 \
  --loss-tolerance 0.01 \
  --output json \
  --verbose \
  --udp-src 4200 \
  --udp-dst 4200 \
  --tx-port 0 \
  --rx-port 0
```

## Running the Reflector

```bash
# Start reflector container (multi-threaded, socat-based)
docker run --rm \
  --name fastlane-reflector \
  --network fastlane_net \
  --ip 172.28.1.20 \
  fastlane-reflector:latest
```

## End-to-End Test

```bash
# Run E2E tests with docker-compose and buildx
docker-compose -f tests/e2e/docker-compose.yml up --abort-on-container-exit

# Or with buildx directly
docker buildx build \
  --tag fastlane-e2e:latest \
  --load \
  -f tests/e2e/Dockerfile \
  .
```

## Verifying the Deployment

### Check the binary is statically linked

```bash
docker run --rm fastlane:latest file /usr/local/bin/fastlane
# Should show: statically linked
```

### Verify inter-container connectivity

```bash
# Start reflector, then tester
docker run -d --name reflector fastlane-reflector:latest
docker run --rm fastlane-tester:latest reflector --test throughput
```

### Check results

```bash
docker run --rm fastlane:latest --output json > results.json
cat results.json
```

## Image Sizes

| Image | Size (with musl) | Notes |
|---|---|---|
| `fastlane` | ~15MB | Static binary, rockylinux:10 |
| `fastlane-tester` | ~15MB | With examples directory |
| `fastlane-reflector` | ~20MB | With socat and httpd-tools |
