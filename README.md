# basevn-migro

> **Open-source ETL tool for migrating data to Base.vn platform**

[![CI](https://github.com/basevn/basevn-migro/workflows/CI/badge.svg)](https://github.com/basevn/basevn-migro/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A high-performance, plugin-based ETL (Extract-Transform-Load) CLI tool built in Rust for seamless data migration to Base.vn platform. Features streaming architecture, incremental sync, resume capability, and comprehensive audit logging.

## Status

🚧 **Phase 1 - Foundation (In Progress)**

Currently implementing core architecture and CLI foundation. See [plan.md](plans/plan.md) for full roadmap.

### What Works Now

- ✅ CLI interface with subcommands (migrate, validate, status, update)
- ✅ Configuration loading and validation (YAML/JSON)
- ✅ Core data types (Record, Field, Value)
- ✅ Error handling framework
- ✅ Logging infrastructure
- ✅ Testing framework

### Coming Soon (Phase 2-6)

- 🔄 CSV/Excel/REST API extractors
- 🔄 Field mapping transformer
- 🔄 Base.vn API loader with rate limiting
- 🔄 Progress tracking and checkpointing
- 🔄 Audit logging
- 🔄 Full ETL pipeline orchestration

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/basevn/basevn-migro.git
cd basevn-migro

# Build from source
cargo build --release

# Install binary
cargo install --path .
```

### Usage

```bash
# Validate configuration
migro validate --config job.yaml

# Run migration (placeholder - Phase 2+)
migro migrate --config job.yaml

# Check job status (placeholder - Phase 2+)
migro status --job-id import-employees

# Self-update (placeholder - Phase 6)
migro update
```

## Configuration Example

```yaml
version: "1"

job:
  id: "import-employees"
  name: "Import employee data from CSV"

source:
  type: csv
  path: "./employees.csv"

target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "hrm"
  entity: "employees"
  auth:
    type: access_token
    token: "${BASEVN_ACCESS_TOKEN}"

mapping:
  fields:
    - source: "Họ và tên"
      target: "full_name"
      required: true
    - source: "Email"
      target: "email"
      required: true
    - source: "Phòng ban"
      target: "department"
```

## Architecture

```
Extract → Transform → Load
  ↓         ↓         ↓
CSV      Field     Base.vn
Excel    Mapping    API
REST API
```

See [plans/plan.md](plans/plan.md) for detailed architecture documentation.

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
cargo test
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

### Development Phases

1. **Phase 1 - Foundation** (Days 1-2) ✅ In Progress
2. **Phase 2 - Extract Layer** (Days 2-3)
3. **Phase 3 - Transform Layer** (Days 3-4)
4. **Phase 4 - Load Layer** (Days 4-5)
5. **Phase 5 - Operations** (Days 5-6)
6. **Phase 6 - Polish & Release** (Days 6-7)

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under Apache 2.0 - see [LICENSE](LICENSE) file for details.

## Support

- Documentation: [plans/plan.md](plans/plan.md)
- Issues: [GitHub Issues](https://github.com/basevn/basevn-migro/issues)
- Base.vn Developer Docs: https://developers.base.vn

---

Built with ❤️ by the Base.vn Engineering Team
