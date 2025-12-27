# basevn-migro

> **Open-source ETL tool for migrating data to Base.vn platform**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A high-performance, plugin-based ETL (Extract-Transform-Load) CLI tool built in Rust for seamless data migration to Base.vn platform. Features streaming architecture, progress tracking, audit logging, checkpointing for resume capability, and comprehensive error handling.

## Status

🎉 **Phase 6 Complete** - Production Ready!

Full ETL pipeline with multi-platform releases, comprehensive testing, and CI/CD automation. See [roadmap](#roadmap) for details.

### ✅ What Works Now

**Extract Layer**
- ✅ CSV extraction with configurable delimiters, encoding, headers
- ✅ REST API extraction with pagination (offset, page, cursor)
- ✅ Excel placeholder (convert to CSV for MVP)
- ✅ Extractor registry for dynamic dispatch

**Transform Layer**
- ✅ Field mapping with required fields, defaults, validation
- ✅ Field filtering (keep/remove modes) for privacy & data reduction
- ✅ Transformer chain with sequential processing
- ✅ Transformer registry for config-driven pipelines

**Load Layer**
- ✅ Base.vn API loader with Bearer token authentication
- ✅ Rate limiting with adaptive adjustment on 429 responses
- ✅ Exponential backoff retry (configurable max attempts)
- ✅ Response parsing for individual record failure tracking
- ✅ Loader registry for extensibility

**Operations Layer**
- ✅ Real-time progress tracking with indicatif
- ✅ Audit logging in JSONL format (jq/grep compatible)
- ✅ Atomic checkpointing for resume capability
- ✅ Sync modes: Full (delete + insert) & Incremental (key field matching)
- ✅ Conflict strategies: Skip, Update, Error

**Quality & Release**
- ✅ 81 unit tests + 12 integration tests (100% passing)
- ✅ 0 clippy warnings
- ✅ Multi-platform binaries (Linux, macOS Intel/ARM, Windows)
- ✅ CI/CD automation with GitHub Actions
- ✅ Comprehensive documentation suite

## Quick Start

### Installation

**Option 1: Download Binary (Recommended)**

Download pre-built binaries from [GitHub Releases](https://github.com/basevn/basevn-migro/releases):

```bash
# Linux / macOS
wget https://github.com/basevn/basevn-migro/releases/latest/download/migro-<VERSION>-<TARGET>.tar.gz
tar -xzf migro-*.tar.gz
sudo mv migro /usr/local/bin/

# Verify installation
migro --version
```

**Option 2: Build from Source**

```bash
# Clone repository
git clone https://github.com/basevn/basevn-migro.git
cd basevn-migro

# Build release
cargo build --release

# Binary at: target/release/migro
```

**Option 3: Cargo Install**

```bash
cargo install basevn-migro
```

### Usage

```bash
# Validate configuration
migro validate --config examples/job.yaml

# Run migration
migro migrate --config examples/job.yaml

# Check job status
migro status --job-id import-employees

# List all checkpoints
migro status

# Resume interrupted migration
migro migrate --config examples/job.yaml --resume
```

## Configuration Example

```yaml
version: "1"

job:
  id: "import-employees"
  name: "Import employee data from CSV"

source:
  type: csv
  path: "./examples/employees.csv"
  options:
    delimiter: ","
    has_headers: true
    encoding: "utf-8"

transform:
  - type: field_mapper
    fields:
      - source: "Họ và tên"
        target: "full_name"
        required: true
      - source: "Email"
        target: "email"
        required: true
      - source: "Phòng ban"
        target: "department"
        required: false
        default: "Unknown"
  - type: field_filter
    mode: remove
    fields: ["password", "ssn"]

target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "hrm"
  entity: "employees"
  auth:
    type: access_token
    token: "${BASEVN_ACCESS_TOKEN}"
  options:
    batch_size: 100
    rate_limit: 10  # requests per second
    timeout: 30     # seconds
    max_retries: 3

operations:
  sync_mode:
    mode: incremental
    key_field: "email"
    conflict_strategy: update
  checkpoint:
    enabled: true
    path: "./checkpoint.json"
  audit_log:
    enabled: true
    path: "./audit.jsonl"
  progress:
    enabled: true
```

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Extract   │────▶│  Transform   │────▶│    Load     │
│             │     │              │     │             │
│ • CSV       │     │ • Field Map  │     │ • Base.vn   │
│ • Excel     │     │ • Filter     │     │   API       │
│ • REST API  │     │ • Validate   │     │ • Batch     │
│ • Registry  │     │ • Chain      │     │ • Retry     │
└─────────────┘     └──────────────┘     └─────────────┘
                                                │
                         ┌──────────────────────┘
                         ▼
                  ┌─────────────┐
                  │ Operations  │
                  │             │
                  │ • Progress  │
                  │ • Audit     │
                  │ • Checkpoint│
                  │ • Sync Mode │
                  └─────────────┘
```

See [docs/architecture.md](docs/architecture.md) for detailed design.

## Features

### 🚀 Performance
- **Streaming architecture**: Process millions of records with minimal memory
- **Batch processing**: Configurable batch sizes for optimal throughput
- **Concurrent extraction**: Parallel processing where applicable

### 🔒 Reliability
- **Atomic checkpointing**: Resume from last successful offset on interruption
- **Exponential backoff**: Intelligent retry with configurable max attempts
- **Error tracking**: Individual record failure tracking with detailed errors
- **Audit logging**: JSONL format for compliance and debugging

### 🎯 Flexibility
- **Plugin architecture**: Registry pattern for extractors, transformers, loaders
- **Config-driven**: YAML/JSON configuration with environment variable support
- **Sync modes**: Full sync (wipe + reload) or incremental (smart updates)
- **Conflict resolution**: Configurable strategies (Skip, Update, Error)

### 📊 Observability
- **Real-time progress**: Live progress bars with success rate and ETA
- **Comprehensive audit trail**: Every operation logged in structured format
- **Status tracking**: Resume capability with checkpoint metadata
- **Error context**: Detailed error messages with record IDs and indexes

## Development

### Prerequisites

- Rust 1.75+
- Cargo

### Build

```bash
cargo build
```

### Test

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_checkpoint_manager

# Check code quality
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

### Project Structure

```
basevn-migro/
├── src/
│   ├── cli/           # Command-line interface
│   ├── config/        # Configuration loading and validation
│   ├── core/          # Core data types (Record, Value)
│   ├── error/         # Error types and handling
│   ├── extract/       # Data extraction layer
│   │   ├── csv.rs
│   │   ├── excel.rs
│   │   ├── rest_api.rs
│   │   └── registry.rs
│   ├── transform/     # Data transformation layer
│   │   ├── field_filter.rs
│   │   ├── field_mapping.rs
│   │   ├── traits.rs
│   │   └── registry.rs
│   ├── load/          # Data loading layer
│   │   ├── basevn.rs
│   │   ├── rate_limiter.rs
│   │   ├── traits.rs
│   │   └── registry.rs
│   ├── operations/    # Operations layer
│   │   ├── progress.rs
│   │   ├── audit.rs
│   │   ├── checkpoint.rs
│   │   └── sync.rs
│   └── lib.rs
├── examples/          # Example configurations and data
├── plans/             # Planning documents and reports
├── docs/              # Comprehensive documentation
└── Cargo.toml
```

## Roadmap

- [x] **Phase 1 - Foundation** (Complete)
  - Core data types, config, error handling
- [x] **Phase 2 - Extract Layer** (Complete)
  - CSV, REST API, Excel placeholder, registry
- [x] **Phase 3 - Transform Layer** (Complete)
  - Field mapping, filtering, chain, registry
- [x] **Phase 4 - Load Layer** (Complete)
  - Base.vn loader, rate limiting, retry, registry
- [x] **Phase 5 - Operations** (Complete)
  - Progress, audit, checkpoint, sync modes
- [x] **Phase 6 - Polish & Release** (Complete)
  - Integration tests, documentation, multi-platform releases, CI/CD automation

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under Apache 2.0 - see [LICENSE](LICENSE) file for details.

## Documentation

- **Project Overview & PDR**: [docs/project-overview-pdr.md](docs/project-overview-pdr.md) - Vision, goals, requirements, and roadmap
- **Codebase Summary**: [docs/codebase-summary.md](docs/codebase-summary.md) - Architecture, modules, and code organization
- **Code Standards**: [docs/code-standards.md](docs/code-standards.md) - Rust conventions, testing, and contribution guidelines
- **System Architecture**: [docs/system-architecture.md](docs/system-architecture.md) - Deployment, scaling, and reliability
- **Architecture Details**: [docs/architecture.md](docs/architecture.md) - Implementation details and design decisions
- **Getting Started**: [docs/getting-started.md](docs/getting-started.md) - Quick start guide and tutorials
- **Configuration Guide**: [docs/configuration.md](docs/configuration.md) - Configuration reference and options
- **Troubleshooting**: [docs/troubleshooting.md](docs/troubleshooting.md) - Common issues and solutions

## Support

- **Planning**: [plans/plan.md](plans/plan.md)
- **Issues**: [GitHub Issues](https://github.com/basevn/basevn-migro/issues)
- **Base.vn Docs**: https://developers.base.vn

## Acknowledgments

Built with ❤️ using:
- [Rust](https://www.rust-lang.org) - Systems programming language
- [tokio](https://tokio.rs) - Async runtime
- [clap](https://clap.rs) - CLI framework
- [serde](https://serde.rs) - Serialization framework
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

---

**Status**: Production Ready | **Tests**: 93/93 Passing | **Platforms**: Linux, macOS, Windows | **License**: Apache 2.0
