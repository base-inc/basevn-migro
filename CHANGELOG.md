# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-12-27

### Added
- **Extract Layer**: CSV, REST API, Excel extractors with registry pattern
- **Transform Layer**: Field mapping, field filtering, transformer chain
- **Load Layer**: Base.vn API loader with rate limiting and retry
- **Operations Layer**: Progress tracking, audit logging, checkpointing, sync modes
- **CLI Commands**:
  - `validate` - Configuration validation
  - `migrate` - Full ETL pipeline orchestration with resume capability
  - `status` - Checkpoint status viewing and job listing
- **Features**:
  - Streaming architecture for memory efficiency
  - Resume capability with atomic checkpointing
  - Batch processing with configurable size
  - Real-time progress tracking
  - Audit logging in JSONL format
  - Rate limiting with adaptive adjustment
  - Exponential backoff retry
  - Record-level failure tracking
  - Sync modes: Full and Incremental
  - Conflict strategies: Skip, Update, Error
- **Quality & Release**:
  - 81 unit tests (100% passing)
  - 12 integration tests (100% passing)
  - 0 clippy warnings
  - Multi-platform release binaries (Linux, macOS Intel/ARM, Windows)
  - GitHub Actions CI/CD workflows
  - Comprehensive test coverage tracking

### Documentation
- Project overview and PDR
- Architecture documentation
- Code standards and conventions
- Configuration guide
- Getting started guide
- Troubleshooting guide
- System architecture

[Unreleased]: https://github.com/basevn/basevn-migro/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/basevn/basevn-migro/releases/tag/v0.1.0
