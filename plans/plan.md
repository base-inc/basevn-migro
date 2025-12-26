# plan.md - basevn-migro

> **Single Source of Truth** for the basevn-migro project
> Open Source ETL Tool for migrating data to Base.vn platform

---

## Validation Summary

**Validated:** 2025-12-26
**Questions asked:** 8

### Confirmed Decisions

1. **Async Strategy:** Full streaming architecture with tokio - Proceed with RecordStream design for 1M+ record support
2. **Checkpoint Reliability:** Atomic writes (temp + rename) with checksum validation - Critical for resume capability
3. **Rate Limiting:** Adaptive rate limiting - Start 10 rps but auto-adjust on 429 responses (enhancement over static limit)
4. **Plugin System:** Built-in plugins only - No dynamic library loading, simpler and safer for MVP
5. **Audit Logs:** JSONL append-only format - Simple, works with jq/grep, good for MVP
6. **Sync Strategy:** Single key_field matching - Sufficient for MVP use cases
7. **Error Handling:** Log and continue - Failed records logged, job continues (matches LoadResult design)
8. **Transformations:** Field mapping only - No type conversion in MVP, users prepare data

### Action Items

- [ ] Implement adaptive rate limiting (enhancement: auto-adjust on 429 instead of static 10 rps)
- [ ] Document user expectation: data must be pre-formatted (no type conversion in MVP)
- [ ] Ensure atomic checkpoint writes with temp file + rename pattern
- [ ] Design error tolerance: track failed records but continue processing

### Recommendation

**Proceed to implementation.** All architectural decisions confirmed. One enhancement identified (adaptive rate limiting) but doesn't block MVP. Plan is validated and ready for execution.

---

## 1. Project Context

### 1.1 Problem Statement

Organizations using Base.vn platform face significant challenges when migrating data from various sources:

- **No standardized tooling**: Manual migration processes are error-prone and time-consuming
- **Complex field mapping**: Different data schemas require careful transformation
- **Lack of validation**: No pre-migration checks lead to data quality issues
- **No rollback capability**: Failed migrations are difficult to recover from
- **Poor visibility**: No progress tracking or audit trails

### 1.2 Solution

**basevn-migro** is an open-source, plugin-based ETL (Extract-Transform-Load) CLI tool that enables:

- Seamless data migration from multiple sources (CSV, Excel, REST APIs) to Base.vn
- Flexible field mapping with transformation capabilities
- Incremental sync with conflict resolution
- Resume capability for interrupted jobs
- Comprehensive audit logging

### 1.3 Target Users

| User Type | Use Case |
|-----------|----------|
| **Base.vn Engineers** | Build and maintain migration pipelines for customers |
| **Customer Developers** | Integrate their systems with Base.vn |
| **Data Engineers** | Large-scale data migration projects |
| **Technical Consultants** | One-time migration during onboarding |

### 1.4 Success Metrics

| Metric | Target |
|--------|--------|
| Migration throughput | 10,000+ records/minute |
| Memory efficiency | Process 1M records with <2GB RAM |
| Reliability | 99.9% success rate for valid data |
| Developer onboarding | <30 minutes to first successful migration |
| Resume capability | 100% data integrity on interrupted jobs |

---

## 2. Tech Stack & Coding Rules

### 2.1 Core Technology

| Category | Choice | Rationale |
|----------|--------|-----------|
| **Language** | Rust 1.75+ | Cross-platform, single binary, memory safety, excellent performance |
| **Async Runtime** | tokio | Industry standard, mature ecosystem |
| **CLI Framework** | clap v4 | Derive macros, auto-completion, excellent UX |
| **Serialization** | serde | Universal serialization framework |
| **HTTP Client** | reqwest | Async, full-featured, well-maintained |
| **Configuration** | YAML (primary), JSON (supported) | Human-readable, widely adopted |

### 2.2 Dependencies

```toml
# Cargo.toml - Core dependencies

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# CLI
clap = { version = "4.4", features = ["derive", "env"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# HTTP
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Data parsing
csv = "1.3"
calamine = "0.24"  # Excel support

# Logging & Progress
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
indicatif = "0.17"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
async-trait = "0.1"
futures = "0.3"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
validator = { version = "0.16", features = ["derive"] }

# Self-update
self_update = "0.39"

[dev-dependencies]
mockall = "0.12"
wiremock = "0.5"
tempfile = "3.9"
assert_cmd = "2.0"
predicates = "3.0"
```

### 2.3 Coding Conventions

#### Formatting (rustfmt.toml)

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

#### Linting (clippy.toml)

```toml
# Strict mode for production quality
msrv = "1.75"
```

#### Clippy Configuration

```bash
# CI command
cargo clippy -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -W clippy::pedantic
```

#### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `field_mapping`, `rest_api` |
| Types/Structs | PascalCase | `JobConfig`, `MigrationRecord` |
| Traits | PascalCase (verb-ish) | `Extractable`, `Loadable` |
| Functions | snake_case | `load_config`, `process_batch` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_BATCH_SIZE` |
| Enum Variants | PascalCase | `SyncMode::Incremental` |

#### Error Handling Rules

```rust
// ✅ Good: Explicit error context
let config = load_config(&path)
    .with_context(|| format!("Failed to load config from {}", path.display()))?;

// ❌ Bad: Panic in production code
let config = load_config(&path).unwrap();

// ✅ Good: Custom error types for library code
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("Failed to read source file: {path}")]
    FileReadError { path: PathBuf, source: std::io::Error },
    
    #[error("Invalid CSV format at line {line}: {message}")]
    CsvParseError { line: usize, message: String },
}
```

#### Documentation Requirements

```rust
//! Module-level documentation explaining purpose and usage.
//!
//! # Examples
//!
//! ```rust
//! use basevn_migro::extract::CsvExtractor;
//! let extractor = CsvExtractor::new();
//! ```

/// Extracts records from a CSV file.
///
/// # Arguments
///
/// * `path` - Path to the CSV file
/// * `options` - Extraction options (delimiter, encoding, etc.)
///
/// # Returns
///
/// A stream of `Record` items or an `ExtractError`.
///
/// # Errors
///
/// Returns `ExtractError::FileReadError` if the file cannot be opened.
pub fn extract_csv(path: &Path, options: &CsvOptions) -> Result<RecordStream, ExtractError> {
    // implementation
}
```

### 2.4 Git Workflow

#### Branch Naming

```
main              # Production-ready code
├── develop       # Integration branch
├── feature/*     # New features (feature/csv-extractor)
├── bugfix/*      # Bug fixes (bugfix/encoding-issue)
├── hotfix/*      # Production hotfixes (hotfix/auth-token-refresh)
└── release/*     # Release preparation (release/v0.1.0)
```

#### Commit Message Format (Conventional Commits)

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

Examples:
```
feat(extract): add Excel file support with calamine
fix(load): handle rate limit errors with exponential backoff
docs(readme): add configuration examples
refactor(core): extract batch processing into separate module
```

#### Pull Request Requirements

- Minimum 1 approval from maintainer
- All CI checks passing
- No merge conflicts
- Linked issue (if applicable)
- Updated CHANGELOG.md for user-facing changes

---

## 3. Architecture

### 3.1 Directory Structure

```
basevn-migro/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                 # Lint, test, build on PR
│   │   ├── release.yml            # Build & publish binaries
│   │   └── audit.yml              # Weekly security audit
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── CODEOWNERS
│
├── src/
│   ├── main.rs                    # Entry point
│   │
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── args.rs                # CLI argument definitions
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── migrate.rs         # migrate command handler
│   │       ├── validate.rs        # validate command handler
│   │       ├── status.rs          # status command handler
│   │       └── update.rs          # self-update handler
│   │
│   ├── core/
│   │   ├── mod.rs
│   │   ├── pipeline.rs            # ETL pipeline orchestrator
│   │   ├── record.rs              # Universal record type
│   │   ├── batch.rs               # Batch processing
│   │   └── state.rs               # Job state & checkpointing
│   │
│   ├── extract/
│   │   ├── mod.rs
│   │   ├── traits.rs              # Extractor trait definition
│   │   ├── csv.rs                 # CSV extractor
│   │   ├── excel.rs               # Excel extractor (calamine)
│   │   └── rest_api.rs            # Generic REST API extractor
│   │
│   ├── transform/
│   │   ├── mod.rs
│   │   ├── traits.rs              # Transformer trait definition
│   │   └── field_mapping.rs       # Field mapping transformer
│   │
│   ├── load/
│   │   ├── mod.rs
│   │   ├── traits.rs              # Loader trait definition
│   │   └── basevn/
│   │       ├── mod.rs
│   │       ├── client.rs          # Base.vn API client
│   │       ├── auth.rs            # Authentication handler
│   │       ├── rate_limiter.rs    # Rate limiting
│   │       └── models.rs          # API models
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   ├── job.rs                 # Job configuration struct
│   │   ├── source.rs              # Source configuration
│   │   ├── target.rs              # Target configuration
│   │   ├── mapping.rs             # Field mapping configuration
│   │   └── loader.rs              # Config file loader
│   │
│   ├── audit/
│   │   ├── mod.rs
│   │   ├── logger.rs              # Audit event logger
│   │   └── events.rs              # Audit event types
│   │
│   ├── progress/
│   │   ├── mod.rs
│   │   ├── tracker.rs             # Progress tracking
│   │   └── reporter.rs            # CLI progress display
│   │
│   ├── error/
│   │   ├── mod.rs
│   │   └── types.rs               # Centralized error types
│   │
│   └── utils/
│       ├── mod.rs
│       ├── retry.rs               # Retry with backoff
│       └── validation.rs          # Common validators
│
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── csv_migration_test.rs
│   │   ├── excel_migration_test.rs
│   │   └── api_migration_test.rs
│   └── fixtures/
│       ├── sample.csv
│       ├── sample.xlsx
│       ├── sample_config.yaml
│       └── mock_responses/
│           └── basevn_success.json
│
├── examples/
│   ├── simple_csv/
│   │   ├── config.yaml
│   │   └── data.csv
│   ├── excel_employees/
│   │   ├── config.yaml
│   │   └── employees.xlsx
│   └── rest_api_sync/
│       └── config.yaml
│
├── docs/
│   ├── getting-started.md
│   ├── configuration.md
│   ├── extending.md               # Plugin development guide
│   ├── troubleshooting.md
│   └── api-reference.md
│
├── Cargo.toml
├── Cargo.lock
├── rustfmt.toml
├── clippy.toml
├── README.md
├── LICENSE                        # Apache 2.0
├── CONTRIBUTING.md
├── CHANGELOG.md
├── SECURITY.md
└── plan.md
```

### 3.2 Module Responsibilities

| Module | Responsibility | Key Types |
|--------|----------------|-----------|
| `cli` | Command-line interface, argument parsing, command dispatch | `Cli`, `Commands` |
| `core::pipeline` | Orchestrate ETL flow, manage execution | `Pipeline`, `PipelineBuilder` |
| `core::record` | Universal data representation | `Record`, `Field`, `Value` |
| `core::batch` | Batch collection and processing | `BatchProcessor`, `Batch` |
| `core::state` | Job state persistence, checkpointing | `JobState`, `Checkpoint` |
| `extract` | Data extraction from sources | `trait Extractor`, `CsvExtractor`, `ExcelExtractor` |
| `transform` | Data transformation | `trait Transformer`, `FieldMapper` |
| `load` | Data loading to targets | `trait Loader` |
| `load::basevn` | Base.vn API integration | `BaseVnClient`, `BaseVnAuth` |
| `config` | Configuration parsing and validation | `JobConfig`, `SourceConfig`, `TargetConfig` |
| `audit` | Audit trail logging | `AuditLogger`, `AuditEvent` |
| `progress` | Progress tracking and reporting | `ProgressTracker`, `ProgressReporter` |
| `error` | Error type definitions | `MigroError`, `ExtractError`, `LoadError` |

### 3.3 Core Traits (Plugin System)

```rust
// === extract/traits.rs ===

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub type RecordStream = Pin<Box<dyn Stream<Item = Result<Record, ExtractError>> + Send>>;

#[async_trait]
pub trait Extractor: Send + Sync {
    /// Unique identifier for this extractor
    fn name(&self) -> &'static str;
    
    /// Supported source types (e.g., ["csv", "tsv"])
    fn supported_types(&self) -> &[&'static str];
    
    /// Validate source configuration
    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError>;
    
    /// Extract records as an async stream
    fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError>;
    
    /// Estimate total record count (for progress tracking)
    async fn estimate_count(&self, config: &SourceConfig) -> Result<Option<u64>, ExtractError>;
}

// === transform/traits.rs ===

pub trait Transformer: Send + Sync {
    /// Unique identifier for this transformer
    fn name(&self) -> &'static str;
    
    /// Transform a single record
    fn transform(&self, record: Record) -> Result<Record, TransformError>;
    
    /// Batch transform (default: iterate single transform)
    fn transform_batch(&self, records: Vec<Record>) -> Result<Vec<Record>, TransformError> {
        records.into_iter().map(|r| self.transform(r)).collect()
    }
}

// === load/traits.rs ===

#[async_trait]
pub trait Loader: Send + Sync {
    /// Unique identifier for this loader
    fn name(&self) -> &'static str;
    
    /// Validate target configuration
    fn validate_config(&self, config: &TargetConfig) -> Result<(), ConfigError>;
    
    /// Initialize connection/authentication
    async fn connect(&mut self, config: &TargetConfig) -> Result<(), LoadError>;
    
    /// Load a batch of records
    async fn load_batch(&self, records: Vec<Record>) -> Result<LoadResult, LoadError>;
    
    /// Verify a record was loaded successfully (optional)
    async fn verify(&self, record: &Record) -> Result<bool, LoadError> {
        Ok(true) // Default: assume success
    }
    
    /// Cleanup/disconnect
    async fn disconnect(&mut self) -> Result<(), LoadError>;
}

#[derive(Debug)]
pub struct LoadResult {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<RecordError>,
}
```

### 3.4 Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MIGRATION PIPELINE                              │
└─────────────────────────────────────────────────────────────────────────────┘

     ┌──────────────────────────────────────────────────────────────────┐
     │                         Configuration                            │
     │  ┌─────────────┐                                                 │
     │  │ job.yaml    │──> Parse ──> Validate ──> JobConfig            │
     │  └─────────────┘                                                 │
     └────────────────────────────────┬─────────────────────────────────┘
                                      │
                                      ▼
     ┌──────────────────────────────────────────────────────────────────┐
     │                         EXTRACT PHASE                            │
     │                                                                  │
     │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐        │
     │  │   CSV       │     │   Excel     │     │  REST API   │        │
     │  │  Extractor  │     │  Extractor  │     │  Extractor  │        │
     │  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘        │
     │         │                   │                   │                │
     │         └───────────────────┴───────────────────┘                │
     │                             │                                    │
     │                             ▼                                    │
     │                    RecordStream (async)                          │
     │                    [Record, Record, ...]                         │
     └────────────────────────────┬─────────────────────────────────────┘
                                  │
                                  ▼
     ┌──────────────────────────────────────────────────────────────────┐
     │                       TRANSFORM PHASE                            │
     │                                                                  │
     │  ┌─────────────────────────────────────────────────────────┐    │
     │  │                    FieldMapper                           │    │
     │  │                                                          │    │
     │  │  source: "Họ và tên"  ──────────>  target: "full_name"  │    │
     │  │  source: "Email"      ──────────>  target: "email"      │    │
     │  │  source: "Phòng ban"  ──────────>  target: "dept_id"    │    │
     │  └─────────────────────────────────────────────────────────┘    │
     │                             │                                    │
     │                             ▼                                    │
     │                    Transformed Records                           │
     └────────────────────────────┬─────────────────────────────────────┘
                                  │
                                  ▼
     ┌──────────────────────────────────────────────────────────────────┐
     │                        BATCH PHASE                               │
     │                                                                  │
     │  Records ──> Collect(batch_size=100) ──> [Batch1, Batch2, ...]  │
     │                                                                  │
     └────────────────────────────┬─────────────────────────────────────┘
                                  │
                                  ▼
     ┌──────────────────────────────────────────────────────────────────┐
     │                         LOAD PHASE                               │
     │                                                                  │
     │  ┌─────────────────────────────────────────────────────────┐    │
     │  │                  BaseVnLoader                            │    │
     │  │                                                          │    │
     │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │    │
     │  │  │ Rate Limit  │─>│ POST /api/  │─>│   Retry     │      │    │
     │  │  │  (10 rps)   │  │   entity    │  │  (3x exp)   │      │    │
     │  │  └─────────────┘  └─────────────┘  └─────────────┘      │    │
     │  │                          │                               │    │
     │  │                          ▼                               │    │
     │  │                   LoadResult                             │    │
     │  │              { succeeded, failed }                       │    │
     │  └─────────────────────────────────────────────────────────┘    │
     │                             │                                    │
     └────────────────────────────┬─────────────────────────────────────┘
                                  │
                                  ▼
     ┌──────────────────────────────────────────────────────────────────┐
     │                     STATE & AUDIT                                │
     │                                                                  │
     │  ┌─────────────────┐           ┌─────────────────┐              │
     │  │   Checkpoint    │           │   Audit Log     │              │
     │  │  checkpoint.json│           │  audit.jsonl    │              │
     │  │                 │           │                 │              │
     │  │  - last_offset  │           │  - timestamp    │              │
     │  │  - processed    │           │  - event_type   │              │
     │  │  - failed_ids   │           │  - record_id    │              │
     │  └─────────────────┘           │  - status       │              │
     │                                └─────────────────┘              │
     └─────────────────────────────────────────────────────────────────┘
```

### 3.5 Configuration Schema

```yaml
# job.yaml - Complete configuration reference

# Schema version for forward compatibility
version: "1"

# Job metadata
job:
  id: "unique-job-identifier"      # Required: Unique ID for this job
  name: "Human readable name"      # Required: Display name
  description: "Optional details"  # Optional: Long description

# Source configuration
source:
  type: csv | excel | rest_api     # Required: Source type
  
  # For file-based sources (csv, excel)
  path: "./data/file.csv"          # Path to source file
  
  # CSV-specific options
  csv:
    delimiter: ","                 # Default: comma
    quote_char: "\""               # Default: double quote
    has_header: true               # Default: true
    skip_rows: 0                   # Skip N rows before header
    encoding: "utf-8"              # Default: utf-8
  
  # Excel-specific options
  excel:
    sheet: "Sheet1"                # Sheet name or index (0-based)
    header_row: 1                  # Row number for headers (1-based)
    skip_rows: 0                   # Skip N rows after header
    range: "A1:Z1000"              # Optional: specific range
  
  # REST API-specific options
  rest_api:
    url: "https://api.example.com/data"
    method: GET                    # GET or POST
    headers:
      Authorization: "Bearer ${API_TOKEN}"
      Content-Type: "application/json"
    query_params:                  # For GET requests
      status: "active"
    body: |                        # For POST requests
      {"filter": "active"}
    pagination:
      type: offset | cursor | page # Pagination strategy
      limit_param: "limit"
      offset_param: "offset"       # For offset-based
      cursor_param: "cursor"       # For cursor-based
      page_param: "page"           # For page-based
      limit: 100
      response_path: "data"        # JSONPath to records array
      next_cursor_path: "next"     # JSONPath to next cursor

# Target configuration
target:
  type: basevn                     # Currently only basevn supported
  
  # Base.vn specific
  basevn:
    base_url: "https://api.base.vn"
    app: "hrm"                     # Base.vn application
    entity: "employees"            # Entity type
    
    auth:
      type: access_token           # access_token | oauth (future)
      token: "${BASEVN_ACCESS_TOKEN}"
    
    options:
      batch_size: 100              # Records per API call
      rate_limit: 10               # Requests per second
      timeout: 30                  # Request timeout (seconds)
      max_retries: 3               # Max retry attempts

# Field mapping configuration
mapping:
  fields:
    - source: "Source Field Name"
      target: "target_field_name"
      required: true | false       # Fail if source field missing
      default: "default value"     # Use if source is null/empty
    
    # More field mappings...
  
  # How to handle unmapped source fields
  unmapped_fields: ignore | warn | error
  
  # How to handle missing required fields
  missing_required: error | skip_record

# Sync behavior
sync:
  mode: full | incremental
  
  # For incremental sync
  incremental:
    key_field: "email"             # Field to identify existing records
    conflict: skip | update | error
    
    # Track sync state (for delta detection)
    state_file: "./.migro/sync_state.json"

# Retry configuration
retry:
  max_attempts: 3
  strategy: fixed | exponential
  initial_delay_ms: 1000
  max_delay_ms: 30000
  multiplier: 2.0                  # For exponential backoff

# Logging configuration
logging:
  level: debug | info | warn | error
  format: text | json
  file: "./logs/migro.log"         # Optional: log to file

# Audit trail
audit:
  enabled: true
  file: "./logs/audit.jsonl"
  include_record_data: false       # Include full record in audit log

# Checkpoint for resume capability
checkpoint:
  enabled: true
  file: "./.migro/checkpoint.json"
  interval: 100                    # Save checkpoint every N records
```

### 3.6 Integration Points

| Integration | Protocol | Authentication | Notes |
|-------------|----------|----------------|-------|
| **Base.vn API** | REST/HTTPS | Access Token (header) | Primary target, rate limited |
| **Local Files** | Filesystem | N/A | CSV, Excel, JSON |
| **REST APIs** | REST/HTTPS | Configurable (header/query) | Generic source connector |
| **Google Sheets** | REST/OAuth | OAuth 2.0 | Phase 2 |
| **Databases** | Native protocols | Connection string | Phase 2 (MySQL, PostgreSQL) |

### 3.7 High-Risk Areas

| Area | Risk Level | Complexity | Mitigation Strategy |
|------|------------|------------|---------------------|
| **Pipeline Orchestration** | High | High | Extensive testing, clear state machine |
| **Base.vn API Client** | High | Medium | Mock testing, comprehensive error handling |
| **Checkpoint/Resume** | High | High | Atomic writes, corruption detection |
| **Rate Limiting** | Medium | Medium | Token bucket, configurable limits |
| **Large File Streaming** | Medium | Medium | Streaming iterators, memory monitoring |
| **Encoding Detection** | Low | Medium | Force UTF-8, validation on read |

---

## 4. Feature Breakdown

### 4.1 MVP Features (Week 1)

#### Source Connectors
- **CSV Parser**: Read CSV files with configurable delimiter, encoding, header detection
- **Excel Parser**: Read .xlsx/.xls files using calamine, support sheet selection
- **REST API Connector**: Generic HTTP client with pagination support

#### Transformation
- **Field Mapping**: Map source fields to target fields with renaming
- **Required Fields**: Mark fields as required, skip/fail on missing

#### Loading
- **Base.vn API Client**: POST records to Base.vn API with access token auth
- **Batch Processing**: Configurable batch sizes for API efficiency
- **Incremental Sync**: Detect existing records by key field, update or skip

#### Operations
- **CLI Interface**: `migrate`, `validate`, `status`, `update` commands
- **Progress Tracking**: Real-time progress bar with ETA
- **Error Handling**: Retry with exponential backoff, detailed error logs
- **Audit Logging**: JSONL audit trail for all operations

### 4.2 Phase 2 Features

- **Google Sheets Connector**: OAuth authentication, sheet selection
- **Database Connectors**: MySQL, PostgreSQL source support
- **Data Type Conversion**: Date parsing, number formatting, boolean conversion
- **Data Validation Rules**: Regex patterns, value ranges, custom validators

### 4.3 Future Features

- **Custom Transformation Scripts**: Lua or WASM-based custom logic
- **Rollback Mechanism**: Undo failed migrations
- **Web UI**: Browser-based configuration and monitoring
- **Job Scheduling**: Cron-like scheduled migrations
- **Dry-Run Mode**: Preview changes without executing

---

## 5. Development Phases

### Phase 1 — Foundation (Days 1-2)

#### Project Setup
- [ ] Initialize Cargo workspace with proper structure
- [ ] Configure rustfmt.toml and clippy.toml
- [ ] Set up GitHub repository with branch protection
- [ ] Create CI workflow (lint, test, build)
- [ ] Add LICENSE (Apache 2.0), README, CONTRIBUTING.md

#### Core Types
- [ ] Define `Record` type with `Field` and `Value` enums
- [ ] Implement `JobConfig` and config loader (YAML/JSON)
- [ ] Create error types with thiserror
- [ ] Set up tracing-based logging infrastructure

#### CLI Skeleton
- [ ] Implement clap-based CLI with subcommands
- [ ] Add `--config`, `--verbose`, `--version` global options
- [ ] Create placeholder handlers for all commands

### Phase 2 — Extract Layer (Days 2-3)

#### Extractor Trait
- [ ] Define `Extractor` trait with async stream return
- [ ] Implement extractor registry for dynamic dispatch

#### CSV Extractor
- [ ] Implement CSV parsing with `csv` crate
- [ ] Support configurable delimiter, quote char, encoding
- [ ] Handle header row detection and field naming
- [ ] Implement streaming (no full file load)
- [ ] Add record count estimation for progress

#### Excel Extractor
- [ ] Implement Excel reading with `calamine`
- [ ] Support sheet selection by name or index
- [ ] Handle header row and skip rows
- [ ] Implement streaming row iteration

#### REST API Extractor
- [ ] Implement generic HTTP GET/POST with reqwest
- [ ] Support header and query param configuration
- [ ] Implement pagination (offset, cursor, page-based)
- [ ] Handle response parsing with JSONPath

### Phase 3 — Transform Layer (Day 3-4)

#### Transformer Trait
- [ ] Define `Transformer` trait
- [ ] Implement transformer chain (multiple transformers)

#### Field Mapping
- [ ] Implement `FieldMapper` with source→target mapping
- [ ] Handle required fields with validation
- [ ] Support default values for missing fields
- [ ] Configure unmapped field handling (ignore/warn/error)

### Phase 4 — Load Layer (Days 4-5)

#### Loader Trait
- [ ] Define `Loader` trait with batch loading
- [ ] Implement `LoadResult` with success/failure tracking

#### Base.vn Client
- [ ] Implement HTTP client for Base.vn API
- [ ] Add access token authentication
- [ ] Implement rate limiter (token bucket)
- [ ] Add configurable timeouts

#### Batch Processor
- [ ] Collect records into configurable batch sizes
- [ ] Process batches with concurrent execution
- [ ] Track per-batch results

#### Retry Logic
- [ ] Implement exponential backoff retry
- [ ] Configure max attempts and delays
- [ ] Handle retryable vs non-retryable errors

### Phase 5 — Operations (Days 5-6)

#### Progress Tracking
- [ ] Integrate indicatif progress bars
- [ ] Show records processed, success rate, ETA
- [ ] Update progress on each batch completion

#### Audit Logging
- [ ] Define audit event schema
- [ ] Implement JSONL writer
- [ ] Log job start, record success/failure, job completion
- [ ] Include timestamps, record IDs, error details

#### Checkpointing
- [ ] Implement checkpoint file (JSON)
- [ ] Save last processed offset, failed record IDs
- [ ] Add resume capability from checkpoint
- [ ] Atomic checkpoint writes (temp file + rename)

#### Sync Modes
- [ ] Implement full sync (delete + insert all)
- [ ] Implement incremental sync with key field matching
- [ ] Handle conflicts (skip, update, error)

### Phase 6 — Polish & Release (Days 6-7)

#### Self-Update
- [ ] Integrate self_update crate
- [ ] Check GitHub releases for new version
- [ ] Implement `update` command

#### Documentation
- [ ] Write comprehensive README
- [ ] Create getting-started guide with examples
- [ ] Document configuration options
- [ ] Add troubleshooting guide

#### Testing
- [ ] Unit tests for all core modules (70%+ coverage)
- [ ] Integration tests with fixture files
- [ ] Mock Base.vn API responses with wiremock
- [ ] End-to-end test with sample migration

#### Release
- [ ] Create release workflow for multi-platform binaries
- [ ] Build for Linux, macOS, Windows (x64 + ARM64)
- [ ] Create GitHub release with changelog
- [ ] Publish to GitHub Releases

---

## 6. Testing Strategy

### 6.1 Testing Pyramid

```
                    ┌───────────────┐
                    │     E2E       │  ← 10% (full migration scenarios)
                    │    Tests      │
                   ─┴───────────────┴─
                  ┌───────────────────┐
                  │   Integration     │  ← 30% (module interactions)
                  │     Tests         │
                 ─┴───────────────────┴─
                ┌───────────────────────┐
                │      Unit Tests       │  ← 60% (individual functions)
                │                       │
                └───────────────────────┘
```

### 6.2 Unit Testing

**Location**: Same file as implementation (`#[cfg(test)] mod tests`)

**Coverage Target**: 70%+ for core logic

**Key Areas**:
- Config parsing and validation
- Record transformation
- Field mapping logic
- Error handling paths
- Retry logic

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapping_basic() {
        let mapper = FieldMapper::new(vec![
            FieldMap { source: "name".into(), target: "full_name".into(), required: true },
        ]);
        
        let record = Record::from([("name", "John Doe")]);
        let result = mapper.transform(record).unwrap();
        
        assert_eq!(result.get("full_name"), Some(&Value::String("John Doe".into())));
    }

    #[test]
    fn test_field_mapping_missing_required() {
        let mapper = FieldMapper::new(vec![
            FieldMap { source: "email".into(), target: "email".into(), required: true },
        ]);
        
        let record = Record::from([("name", "John")]);
        let result = mapper.transform(record);
        
        assert!(matches!(result, Err(TransformError::MissingRequiredField { .. })));
    }
}
```

### 6.3 Integration Testing

**Location**: `tests/integration/`

**Focus Areas**:
- Full ETL pipeline with mock API
- File parsing with real fixtures
- Checkpoint save/restore
- Rate limiter behavior

**Example**:
```rust
// tests/integration/csv_migration_test.rs

#[tokio::test]
async fn test_csv_to_basevn_migration() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/api/v1/hrm/employees"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "id": "emp_123"
        })))
        .mount(&mock_server)
        .await;
    
    // Create test config
    let config = JobConfig {
        source: SourceConfig::Csv { path: "tests/fixtures/employees.csv".into() },
        target: TargetConfig::BaseVn { base_url: mock_server.uri(), .. },
        ..Default::default()
    };
    
    // Run migration
    let result = Pipeline::new(config).run().await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().succeeded, 10);
}
```

### 6.4 End-to-End Testing

**Approach**: Test CLI commands with real (or mocked) endpoints

**Tools**: `assert_cmd`, `predicates`

**Example**:
```rust
#[test]
fn test_migrate_command_success() {
    let mut cmd = Command::cargo_bin("migro").unwrap();
    
    cmd.arg("migrate")
       .arg("--config")
       .arg("tests/fixtures/sample_config.yaml")
       .env("BASEVN_ACCESS_TOKEN", "test_token");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("Migration completed"));
}

#[test]
fn test_validate_command_invalid_config() {
    let mut cmd = Command::cargo_bin("migro").unwrap();
    
    cmd.arg("validate")
       .arg("--config")
       .arg("tests/fixtures/invalid_config.yaml");
    
    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("Invalid configuration"));
}
```

### 6.5 Critical Test Cases

| Scenario | Test Type | Expected Behavior |
|----------|-----------|-------------------|
| Valid CSV migration | E2E | All records loaded successfully |
| Missing required field | Unit | TransformError with field name |
| API rate limit exceeded | Integration | Retry with backoff, eventually succeed |
| Network timeout | Integration | Retry, then fail with clear error |
| Invalid config YAML | Unit | Parse error with line number |
| Resume from checkpoint | Integration | Continue from last position |
| Unicode in CSV | Unit | Correctly parse Vietnamese characters |
| Large file (1M rows) | Integration | Complete without OOM, stream processing |
| Concurrent batch loading | Integration | All batches complete, no data loss |
| Auth token expired | Integration | Clear error message, suggest refresh |

### 6.6 Test Fixtures

```
tests/fixtures/
├── sample.csv                    # 10 rows, basic employee data
├── sample_large.csv              # 10,000 rows for perf testing
├── sample.xlsx                   # Excel version
├── sample_unicode.csv            # Vietnamese characters
├── sample_config.yaml            # Valid config
├── invalid_config.yaml           # Config with errors
└── mock_responses/
    ├── success.json              # 200 OK response
    ├── rate_limited.json         # 429 response
    ├── validation_error.json     # 400 with field errors
    └── server_error.json         # 500 response
```

---

## 7. Deployment & DevOps

### 7.1 CI/CD Pipeline

#### Continuous Integration (ci.yml)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --all-features
      - name: Run doc tests
        run: cargo test --doc

  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
```

#### Release Pipeline (release.yml)

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build-release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: migro-linux-x64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: migro-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: migro-macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: migro-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: migro-windows-x64.exe
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/migro*

  create-release:
    needs: build-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          generate_release_notes: true
```

#### Security Audit (audit.yml)

```yaml
name: Security Audit

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  push:
    paths:
      - 'Cargo.lock'

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run audit
        run: cargo audit
```

### 7.2 Environments

| Environment | Purpose | Deployment |
|-------------|---------|------------|
| **Local Dev** | Development and testing | Manual, cargo run |
| **CI** | Automated testing | GitHub Actions on every PR |
| **Release** | Production binaries | GitHub Releases on tag push |

### 7.3 Binary Distribution

| Platform | Architecture | Binary Name | Size (est.) |
|----------|--------------|-------------|-------------|
| Linux | x86_64 | migro-linux-x64 | ~10 MB |
| Linux | aarch64 | migro-linux-arm64 | ~10 MB |
| macOS | x86_64 | migro-macos-x64 | ~10 MB |
| macOS | aarch64 (M1/M2) | migro-macos-arm64 | ~10 MB |
| Windows | x86_64 | migro-windows-x64.exe | ~12 MB |

### 7.4 Self-Update Mechanism

```rust
// src/cli/commands/update.rs

pub async fn run_update() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("basevn")
        .repo_name("basevn-migro")
        .bin_name("migro")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    
    match status {
        self_update::Status::UpToDate(v) => {
            println!("Already up to date: v{}", v);
        }
        self_update::Status::Updated(v) => {
            println!("Updated to v{}", v);
        }
    }
    
    Ok(())
}
```

### 7.5 Logging & Monitoring

#### Log Levels

| Level | Usage |
|-------|-------|
| ERROR | Failures that stop processing |
| WARN | Issues that don't stop processing but need attention |
| INFO | Key milestones (job start, batch complete, job finish) |
| DEBUG | Detailed operation info (individual records) |
| TRACE | Very detailed debugging (HTTP requests/responses) |

#### Log Output Formats

**Text (default, human-readable)**:
```
2024-01-15T10:30:00Z INFO  [basevn_migro::core::pipeline] Starting migration job: import-employees
2024-01-15T10:30:01Z INFO  [basevn_migro::extract::csv] Extracted 1000 records from employees.csv
2024-01-15T10:30:02Z INFO  [basevn_migro::load::basevn] Loaded batch 1/10 (100 records)
```

**JSON (for log aggregation)**:
```json
{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","target":"basevn_migro::core::pipeline","message":"Starting migration job","job_id":"import-employees"}
```

#### Audit Log Format (JSONL)

```json
{"timestamp":"2024-01-15T10:30:00Z","event":"job_started","job_id":"import-employees","config_path":"./job.yaml"}
{"timestamp":"2024-01-15T10:30:01Z","event":"record_loaded","job_id":"import-employees","record_id":"1","source_key":"john@example.com","target_id":"emp_123"}
{"timestamp":"2024-01-15T10:30:01Z","event":"record_failed","job_id":"import-employees","record_id":"2","source_key":"invalid@","error":"API validation failed: invalid email format"}
{"timestamp":"2024-01-15T10:30:05Z","event":"job_completed","job_id":"import-employees","total":1000,"succeeded":998,"failed":2,"duration_ms":5000}
```

### 7.6 Backup & Recovery

#### Checkpoint File Structure

```json
{
  "version": 1,
  "job_id": "import-employees",
  "started_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:35:00Z",
  "source": {
    "type": "csv",
    "path": "./employees.csv",
    "last_offset": 5000,
    "total_estimated": 10000
  },
  "progress": {
    "extracted": 5000,
    "transformed": 5000,
    "loaded": 4900,
    "failed": 100
  },
  "failed_records": [
    {"offset": 123, "key": "invalid@", "error": "validation failed"},
    {"offset": 456, "key": "duplicate@example.com", "error": "duplicate key"}
  ]
}
```

#### Resume Process

1. Check for existing checkpoint file
2. Validate checkpoint matches current job config
3. Seek source to last processed offset
4. Continue processing from checkpoint
5. Retry failed records (optional flag)

---

## 8. Risk Register

| ID | Risk | Likelihood | Impact | Mitigation | Owner |
|----|------|------------|--------|------------|-------|
| R1 | Base.vn API changes break integration | Medium | High | Version API client, abstract interface, integration tests | Core team |
| R2 | Large files cause OOM | Medium | High | Streaming iterators, batch processing, memory limits | Core team |
| R3 | Rate limiting causes job timeout | High | Medium | Configurable rate limits, token bucket, progress resume | Core team |
| R4 | Checkpoint corruption on crash | Low | High | Atomic writes, checksum validation, backup checkpoints | Core team |
| R5 | Unicode/encoding issues with Vietnamese data | Medium | Medium | Force UTF-8, encoding detection, validation | Core team |
| R6 | Excel format variations fail parsing | Medium | Low | Calamine handles major formats, fallback CSV export | Core team |
| R7 | Slow adoption due to learning curve | Medium | Medium | Comprehensive docs, example configs, video tutorials | Core team |
| R8 | Security vulnerabilities in dependencies | Low | High | Weekly cargo-audit, dependabot alerts | Core team |
| R9 | MVP scope creep delays release | Medium | Medium | Strict MVP definition, defer non-essential features | Project lead |
| R10 | Network instability during migration | Medium | Medium | Retry with backoff, checkpoint resume | Core team |

---

## 9. Appendix

### 9.1 Glossary

| Term | Definition |
|------|------------|
| **ETL** | Extract-Transform-Load: Data pipeline pattern |
| **Record** | Single data unit (row in CSV, object from API) |
| **Batch** | Collection of records processed together |
| **Checkpoint** | Saved state for resuming interrupted jobs |
| **Extractor** | Component that reads from data sources |
| **Transformer** | Component that modifies records |
| **Loader** | Component that writes to targets |
| **Field Mapping** | Configuration linking source fields to target fields |
| **Incremental Sync** | Only process new/changed records |
| **Rate Limiting** | Controlling request frequency to avoid API throttling |

### 9.2 CLI Reference

```bash
# Show help
migro --help
migro <command> --help

# Migrate data
migro migrate --config job.yaml
migro migrate --config job.yaml --verbose
migro migrate --config job.yaml --resume  # Resume from checkpoint

# Validate configuration
migro validate --config job.yaml

# Check job status (from checkpoint)
migro status --job-id import-employees
migro status --checkpoint .migro/checkpoint.json

# Self-update
migro update
migro update --check  # Check only, don't install
```

### 9.3 Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `BASEVN_ACCESS_TOKEN` | Base.vn API authentication | `bv_abc123...` |
| `MIGRO_LOG_LEVEL` | Override log level | `debug`, `info`, `warn` |
| `MIGRO_LOG_FORMAT` | Log output format | `text`, `json` |
| `MIGRO_CHECKPOINT_DIR` | Default checkpoint directory | `./.migro` |
| `NO_COLOR` | Disable colored output | `1` |

### 9.4 Example Configurations

#### Simple CSV to Base.vn

```yaml
version: "1"
job:
  id: "simple-import"
  name: "Import employees from CSV"

source:
  type: csv
  path: "./employees.csv"

target:
  type: basevn
  basevn:
    base_url: "https://api.base.vn"
    app: "hrm"
    entity: "employees"
    auth:
      type: access_token
      token: "${BASEVN_ACCESS_TOKEN}"

mapping:
  fields:
    - source: "name"
      target: "full_name"
      required: true
    - source: "email"
      target: "email"
      required: true
```

#### REST API with Pagination

```yaml
version: "1"
job:
  id: "api-sync"
  name: "Sync from external API"

source:
  type: rest_api
  rest_api:
    url: "https://external-api.com/users"
    method: GET
    headers:
      Authorization: "Bearer ${EXTERNAL_API_TOKEN}"
    pagination:
      type: offset
      limit_param: "limit"
      offset_param: "offset"
      limit: 100
      response_path: "data"

target:
  type: basevn
  basevn:
    base_url: "https://api.base.vn"
    app: "crm"
    entity: "contacts"
    auth:
      type: access_token
      token: "${BASEVN_ACCESS_TOKEN}"
    options:
      batch_size: 50
      rate_limit: 5

mapping:
  fields:
    - source: "full_name"
      target: "name"
    - source: "email_address"
      target: "email"
    - source: "phone"
      target: "phone_number"

sync:
  mode: incremental
  incremental:
    key_field: "email"
    conflict: update
```

### 9.5 Related Resources

- [Base.vn Developer Documentation](https://developers.base.vn)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Serde Documentation](https://serde.rs/)

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-15  
**Authors**: Base.vn Engineering Team  
**License**: Apache 2.0
