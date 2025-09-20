# Modified: 2025-09-20

# FedRAMP Compliance Automation Platform

A comprehensive Rust-based platform for automating FedRAMP compliance processes, document conversion, and security control management.

## 🚀 Features

- **Document Parser & Converter**: Transform Excel/Word FedRAMP documents into OSCAL JSON
- **Compliance Dashboard**: Real-time tracking of control implementation status
- **Gap Analysis Tool**: Automated compliance gap identification and remediation planning
- **Control Mapping Engine**: Cross-framework control relationships (NIST 800-53, 800-171, CIS)
- **Risk Assessment Platform**: Automated FIPS 199 categorization and impact analysis
- **POA&M Management**: Vulnerability tracking and remediation workflows
- **SSP Generator**: Automated System Security Plan creation
- **Compliance Reporting**: Audit-ready report generation
- **CI/CD Integration**: Security pipeline integration for DevSecOps

## 🏗️ Architecture

This is a Rust monorepo with the following structure:

```
crates/
├── fedramp-core/           # Core data models and utilities
├── document-parser/        # Document parsing and OSCAL conversion
├── compliance-dashboard/   # Real-time compliance visualization
├── gap-analysis/          # Compliance gap detection and analysis
├── control-mapping/       # Cross-framework control mapping
├── risk-assessment/       # FIPS 199 and risk analysis
├── poam-management/       # POA&M lifecycle management
├── ssp-generator/         # System Security Plan generation
├── compliance-reporting/  # Report generation engine
├── inheritance-tracker/   # Control responsibility tracking
├── cicd-pipeline/         # CI/CD security integration
├── framework-converter/   # Multi-framework conversion
├── audit-trail/          # Comprehensive audit logging
├── fedramp-api/          # REST API server
├── fedramp-cli/          # Command-line interface
└── fedramp-web/          # Web frontend
```

## 🛠️ Development Setup

### Prerequisites

- Rust 1.70+ (managed via `rust-toolchain.toml`)
- PostgreSQL 15+
- Redis 7+
- Docker & Docker Compose (for development)

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Elevated-Standards/mappings.git
   cd mappings
   ```

2. **Setup development environment**:
   ```bash
   make setup
   ```

3. **Start development services**:
   ```bash
   make db-setup
   ```

4. **Run the API server**:
   ```bash
   make dev
   ```

5. **Use the CLI tool**:
   ```bash
   make cli -- --help
   ```

### Available Commands

```bash
# Development
make dev          # Start API server
make cli          # Run CLI tool
make watch        # Continuous development with auto-reload

# Building
make build        # Build all crates
make build-api    # Build API server only
make build-cli    # Build CLI tool only

# Testing
make test         # Run all tests
make clippy       # Run lints
make fmt          # Format code

# Docker
make docker       # Build Docker images
docker-compose -f ops/docker/docker-compose.yml up
```

## 📖 Usage

### CLI Examples

```bash
# Parse FedRAMP documents
fedramp parse --input documents/ --output oscal/

# Analyze compliance gaps
fedramp analyze --baseline moderate --framework nist-800-53

# Generate reports
fedramp report --type assessment --output reports/

# Generate SSP
fedramp ssp --template fedramp --system-name "My System"

# Convert between frameworks
fedramp convert --from nist-800-53 --to nist-800-171 --input controls.json
```

### API Examples

```bash
# Upload and parse document
curl -X POST http://localhost:8080/api/v1/documents/parse \
  -F "file=@poam.xlsx" \
  -F "type=poam"

# Get compliance status
curl http://localhost:8080/api/v1/compliance/status

# Generate gap analysis
curl -X POST http://localhost:8080/api/v1/analysis/gaps \
  -H "Content-Type: application/json" \
  -d '{"baseline": "moderate", "framework": "nist-800-53"}'
```

## 🔧 Configuration

The platform uses configuration files in the `config/` directory:

- `config/default.toml` - Default configuration
- `config/development.toml` - Development overrides
- `config/production.toml` - Production settings

Key configuration sections:
- Database connection settings
- API server configuration
- Document processing settings
- Framework mapping configurations
- Security and authentication settings

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p document-parser

# Run integration tests
cargo test --test '*'

# Generate test coverage
cargo tarpaulin --workspace
```

## 📊 Monitoring

The platform includes comprehensive monitoring:

- Health check endpoints (`/health`, `/metrics`)
- Structured logging with tracing
- Performance metrics collection
- Audit trail for all operations

## 🔒 Security

- Role-based access control (RBAC)
- JWT-based authentication
- Input validation and sanitization
- Audit logging for all operations
- Secure document processing

## 🚀 Deployment

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose -f ops/docker/docker-compose.yml up -d

# Or build individual containers
docker build -t fedramp-api -f ops/docker/Dockerfile.api .
```

### Production Deployment

See `ops/` directory for:
- Kubernetes manifests
- Terraform configurations
- CI/CD pipeline definitions
- Monitoring and alerting setup

## 📚 Documentation

- [API Documentation](docs/api.md)
- [CLI Reference](docs/cli.md)
- [Architecture Guide](docs/architecture.md)
- [Development Guide](docs/development.md)
- [Deployment Guide](docs/deployment.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `make test clippy fmt`
6. Submit a pull request

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🆘 Support

- [GitHub Issues](https://github.com/Elevated-Standards/mappings/issues)
- [Documentation](https://docs.rs/fedramp-compliance)
- [Discussions](https://github.com/Elevated-Standards/mappings/discussions)