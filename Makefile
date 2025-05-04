# Makefile for fastlane-rfc2544
.PHONY: all build test clean bench docker docker-e2e

CARGO  := cargo
CARGO_FLAGS  := --release
BUILD_DIR := target/release

all: build test

build:
	$(CARGO) build $(CARGO_FLAGS)

test:
	$(CARGO) test --all

bench:
	$(CARGO) bench -p fastlane-core

clean:
	$(CARGO) clean

# Build a static binary suitable for container deployment
build-static:
	cross build --target x86_64-unknown-linux-musl --release

# Docker images
docker-tester:
	docker build -t fastlane-tester ./docker/tester

docker-reflector:
	docker build -t fastlane-reflector ./docker/reflector

docker: docker-tester docker-reflector

# End-to-end test with docker containers
docker-e2e: docker
	docker-compose -f tests/e2e/docker-compose.yml up --abort-on-container-exit

# Generate documentation
doc:
	$(CARGO) doc --no-deps --workspace

# Run clippy
clippy:
	$(CARGO) clippy --all -- -D warnings

# Format code
fmt:
	$(CARGO) fmt --all
