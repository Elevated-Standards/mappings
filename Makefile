# Modified: 2025-01-20

.PHONY: help build test clean dev fmt clippy check docker docs

# Default target
help:
	@echo "FedRAMP Compliance Automation Platform"
	@echo ""
	@echo "Available targets:"
	@echo "  build       - Build all crates in release mode"
	@echo "  test        - Run all tests"
	@echo "  dev         - Start development API server"
	@echo "  cli         - Run CLI tool"
	@echo "  fmt         - Format all code"
	@echo "  clippy      - Run clippy lints"
	@echo "  check       - Check all crates"
	@echo "  clean       - Clean build artifacts"
	@echo "  docker      - Build Docker images"
	@echo "  docs        - Generate documentation"
	@echo "  setup       - Setup development environment"

# Build targets
build:
	cargo build --release --workspace

build-api:
	cargo build --release --bin fedramp-api

build-cli:
	cargo build --release --bin fedramp

# Development targets
dev:
	cargo run --bin fedramp-api

cli:
	cargo run --bin fedramp

# Testing targets
test:
	cargo test --workspace

test-integration:
	cargo test --workspace --test '*'

# Code quality targets
fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

check:
	cargo check --workspace

# Maintenance targets
clean:
	cargo clean

# Docker targets
docker:
	docker build -t fedramp-api -f ops/docker/Dockerfile.api .
	docker build -t fedramp-cli -f ops/docker/Dockerfile.cli .

# Documentation targets
docs:
	cargo doc --workspace --no-deps --open

# Setup targets
setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-audit cargo-outdated
	@echo "Development environment setup complete"

# Database targets
db-setup:
	docker-compose -f ops/docker/docker-compose.yml up -d postgres
	sleep 5
	sqlx database create
	sqlx migrate run

db-reset:
	sqlx database drop -y
	sqlx database create
	sqlx migrate run

# Continuous development
watch:
	cargo watch -x "check --workspace" -x "test --workspace"

watch-api:
	cargo watch -x "run --bin fedramp-api"

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update
	cargo outdated
