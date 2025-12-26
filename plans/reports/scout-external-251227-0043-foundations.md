# Scout Report: Core Foundations & Configuration
**Date:** 2025-12-27 | **CWD:** /home/hardy/workspace/BASE/basevn-migro

## Executive Summary

Basevn-migro is an open-source ETL (Extract-Transform-Load) tool built in Rust for migrating data to the Base.vn platform. MVP complete (Phases 1-5) with production-ready features: streaming architecture, plugin-based design, comprehensive error handling, and operational capabilities (checkpointing, audit logging, progress tracking).

**Stats:**
- 40 Rust source files | 5495 total LOC
- 81 passing unit tests | 0 clippy warnings
- Rust 1.75+ | Apache 2.0 license

---

## 1. Core Data Structures

### File: `/src/core/record.rs`

**Universal Record Type** - Central data structure flowing through ETL pipeline.

#### `Value` Enum
Supports all JSON-compatible data types:
```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

Key methods:
- `is_empty()` - Check null/empty values
- `as_string()` - Convert to string representation
- Implements `From<T>` for String, &str, f64, bool

#### `Record` Struct
Represents a single data unit (CSV row, API response object):
```rust
pub struct Record {
    fields: HashMap<Field, Value>,
    pub metadata: HashMap<String, String>,
}
```

Key API:
- `new()` / `default()` - Create empty record
- `insert(key, value)` - Add field-value pair
- `get(field)` - Retrieve value by name
- `remove(field)` - Remove and return field
- `contains(field)` - Check field existence
- `fields()` - Get all field names
- `iter()` - Iterate over pairs
- `len()` / `is_empty()` - Size queries
- `add_metadata(key, value)` - Add metadata

**Type Aliases:**
- `Field = String` - Field name type

---

## 2. Configuration Architecture

### File: `/src/config/mod.rs`

Central config module with layered structure:

```
config/
├── job.rs        → JobConfig, sync, retry, logging, audit, checkpoint
├── source.rs     → SourceConfig variants (CSV, Excel, REST API)
├── target.rs     → TargetConfig (Base.vn only)
├── mapping.rs    → Field mapping rules
└── loader.rs     → Config file loading & validation
```

### 2.1 Job Configuration (`job.rs`)

**`JobConfig`** - Complete job specification:
```rust
pub struct JobConfig {
    pub version: String,          // Schema version (default: "1")
    pub job: JobMetadata,
    pub source: SourceConfig,
    pub target: TargetConfig,
    pub mapping: MappingConfig,
    pub sync: SyncConfig,
    pub retry: RetryConfig,
    pub logging: LoggingConfig,
    pub audit: AuditConfig,
    pub checkpoint: CheckpointConfig,
}
```

**`JobMetadata`**:
```rust
pub struct JobMetadata {
    pub id: String,               // Unique identifier
    pub name: String,             // Human-readable name
    pub description: Option<String>,
}
```

#### Sync Configuration
**`SyncMode`**: Full | Incremental

**`SyncConfig`**:
```rust
pub struct SyncConfig {
    pub mode: SyncMode,           // Default: Full
    pub incremental: Option<IncrementalConfig>,
}
```

**`IncrementalConfig`**:
```rust
pub struct IncrementalConfig {
    pub key_field: String,        // Field for matching existing records
    pub conflict: ConflictStrategy,
    pub state_file: Option<String>,
}
```

**`ConflictStrategy`**: Skip | Update | Error

#### Retry Configuration
**`RetryConfig`**:
```rust
pub struct RetryConfig {
    pub max_attempts: usize,      // Default: 3
    pub strategy: RetryStrategy,  // Default: Exponential
    pub initial_delay_ms: u64,    // Default: 1000
    pub max_delay_ms: u64,        // Default: 30000
    pub multiplier: f64,          // Default: 2.0
}
```

**`RetryStrategy`**: Fixed | Exponential

#### Logging Configuration
**`LoggingConfig`**:
```rust
pub struct LoggingConfig {
    pub level: LogLevel,          // Default: Info
    pub format: LogFormat,        // Default: Text
    pub file: Option<String>,
}
```

**Enums:**
- `LogLevel`: Debug | Info | Warn | Error
- `LogFormat`: Text | Json

#### Audit Configuration
**`AuditConfig`**:
```rust
pub struct AuditConfig {
    pub enabled: bool,            // Default: true
    pub file: String,             // Default: "./logs/audit.jsonl"
    pub include_record_data: bool, // Default: false
}
```

#### Checkpoint Configuration
**`CheckpointConfig`**:
```rust
pub struct CheckpointConfig {
    pub enabled: bool,            // Default: true
    pub file: String,             // Default: "./.migro/checkpoint.json"
    pub interval: usize,          // Default: 100 records
}
```

### 2.2 Source Configuration (`source.rs`)

**`SourceConfig`** - Tagged enum for source type selection:

#### CSV Source
```rust
SourceConfig::Csv {
    path: String,
    csv: Option<CsvOptions>,
}
```

**`CsvOptions`**:
```rust
pub struct CsvOptions {
    pub delimiter: String,        // Default: ","
    pub quote_char: String,       // Default: "\""
    pub has_header: bool,         // Default: true
    pub skip_rows: usize,         // Default: 0
    pub encoding: String,         // Default: "utf-8"
}
```

#### Excel Source
```rust
SourceConfig::Excel {
    path: String,
    excel: Option<ExcelOptions>,
}
```

**`ExcelOptions`**:
```rust
pub struct ExcelOptions {
    pub sheet: String,            // Default: "Sheet1"
    pub header_row: usize,        // Default: 1 (1-based)
    pub skip_rows: usize,         // Default: 0
    pub range: Option<String>,    // Optional cell range
}
```

#### REST API Source
```rust
SourceConfig::RestApi {
    rest_api: Box<RestApiOptions>,
}
```

**`RestApiOptions`**:
```rust
pub struct RestApiOptions {
    pub url: String,
    pub method: String,           // Default: "GET"
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<String>,     // For POST requests
    pub pagination: Option<PaginationConfig>,
}
```

**`PaginationConfig`**:
```rust
pub struct PaginationConfig {
    pub page_type: PaginationType,
    pub limit_param: String,
    pub offset_param: Option<String>,
    pub cursor_param: Option<String>,
    pub page_param: Option<String>,
    pub limit: usize,             // Default: 100
    pub response_path: String,    // JSONPath to records array
    pub next_cursor_path: Option<String>,
}
```

**`PaginationType`**: Offset | Cursor | Page

### 2.3 Target Configuration (`target.rs`)

**`TargetConfig`** - Tagged enum (currently Base.vn only):
```rust
TargetConfig::BaseVn {
    basevn: BaseVnConfig,
}
```

**`BaseVnConfig`**:
```rust
pub struct BaseVnConfig {
    pub base_url: String,         // Default: "https://api.base.vn"
    pub app: String,              // Application name
    pub entity: String,           // Entity type
    pub auth: AuthConfig,
    pub options: BaseVnOptions,
}
```

**`AuthConfig`**:
```rust
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub token: Option<String>,    // Bearer token
}
```

**`AuthType`**: AccessToken (enum for extensibility)

**`BaseVnOptions`**:
```rust
pub struct BaseVnOptions {
    pub batch_size: usize,        // Default: 100 records/request
    pub rate_limit: usize,        // Default: 10 requests/sec
    pub timeout: u64,             // Default: 30 seconds
    pub max_retries: usize,       // Default: 3
}
```

### 2.4 Field Mapping Configuration (`mapping.rs`)

**`MappingConfig`**:
```rust
pub struct MappingConfig {
    pub fields: Vec<FieldMap>,
    pub unmapped_fields: UnmappedFieldBehavior,  // Default: Ignore
    pub missing_required: MissingRequiredBehavior, // Default: Error
}
```

**`FieldMap`**:
```rust
pub struct FieldMap {
    pub source: String,           // Source field name
    pub target: String,           // Target field name
    pub required: bool,           // Default: false
    pub default: Option<String>,  // Default if null/empty
}
```

**Enums:**
- `UnmappedFieldBehavior`: Ignore | Warn | Error
- `MissingRequiredBehavior`: Error | SkipRecord

### 2.5 Config Loading (`loader.rs`)

**`load_config<P: AsRef<Path>>(path) -> Result<JobConfig, ConfigError>`**

Features:
- Supports YAML and JSON formats
- Automatic format detection via serde_yaml
- Environment variable substitution (via `${VAR}` syntax)
- Basic validation:
  - Job ID not empty
  - Job name not empty
  - At least one field mapping

---

## 3. Error Handling Architecture

### File: `/src/error/types.rs`

**Centralized error types using `thiserror` crate.**

#### Main Error Type
**`MigroError`** - Wraps all subsystem errors:
```rust
pub enum MigroError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Extract error: {0}")]
    Extract(#[from] ExtractError),
    #[error("Transform error: {0}")]
    Transform(#[from] TransformError),
    #[error("Load error: {0}")]
    Load(#[from] LoadError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}
```

#### `ConfigError` - Configuration validation errors
```rust
pub enum ConfigError {
    FileRead { path: PathBuf, source: std::io::Error },
    YamlParse(#[from] serde_yaml::Error),
    JsonParse(#[from] serde_json::Error),
    Validation(String),
    MissingField { field: String },
}
```

#### `ExtractError` - Data extraction failures
```rust
pub enum ExtractError {
    FileRead { path: PathBuf, source: std::io::Error },
    CsvParse { line: usize, message: String },
    ExcelParse(String),
    ApiRequest(String),
    UnsupportedType(String),
}
```

#### `TransformError` - Data transformation failures
```rust
pub enum TransformError {
    MissingRequiredField { field: String },
    InvalidValue { field: String, message: String },
    TransformFailed(String),
}
```

#### `LoadError` - Data loading failures
```rust
pub enum LoadError {
    Connection(String),
    Auth(String),
    ApiRequest(String),
    RateLimit,
    Validation(String),
    Network(#[from] reqwest::Error),
}
```

#### Record-Level Error Tracking
**`RecordError`** - Individual record failure context:
```rust
pub struct RecordError {
    pub offset: usize,           // Position in stream
    pub key: Option<String>,     // Record identifier
    pub error: String,           // Error message
}
```

---

## 4. Project Dependencies

### File: `Cargo.toml`

#### Async & Runtime
- **tokio 1.35** - Async runtime (full features)
- **async-trait 0.1** - Async trait support
- **futures 0.3** - Async utilities

#### CLI & Serialization
- **clap 4.4** - CLI framework (derive + env features)
- **serde 1.0** - Serialization framework (derive)
- **serde_yaml 0.9** - YAML support
- **serde_json 1.0** - JSON support

#### HTTP & Data
- **reqwest 0.11** - HTTP client (json + rustls-tls)
- **csv 1.3** - CSV parsing
- **calamine 0.24** - Excel support

#### Logging & Progress
- **tracing 0.1** - Structured logging
- **tracing-subscriber 0.3** - Logging subscriber (json + env-filter)
- **indicatif 0.17** - Progress bars

#### Error Handling
- **thiserror 1.0** - Error types with derive
- **anyhow 1.0** - Error context

#### Utilities
- **chrono 0.4** - Date/time (with serde)
- **uuid 1.6** - UUID generation (v4 + serde)
- **validator 0.16** - Field validation (derive)

#### Dev Dependencies
- **mockall 0.12** - Mocking framework
- **tempfile 3.9** - Temporary files for tests
- **assert_cmd 2.0** - CLI testing
- **predicates 3.0** - Assertion predicates

---

## 5. Project Overview

### Purpose
Open-source ETL tool for seamless data migration to Base.vn platform with:
- High-performance streaming architecture
- Plugin-based extensibility
- Production-ready reliability features
- Comprehensive observability

### Status: MVP Complete (Phases 1-5)
- Core ETL pipeline fully functional
- All layers implemented: Extract, Transform, Load, Operations
- 81 unit tests (100% passing)
- Production-ready error handling

### Architecture
```
Extract Layer          Transform Layer        Load Layer
├── CSV                ├── Field Mapper       ├── Base.vn API
├── Excel              ├── Field Filter       ├── Batch Processing
├── REST API           ├── Validation         ├── Rate Limiting
└── Registry           └── Chain Registry     └── Retry Logic
                                                   │
                                    ┌───────────────┘
                                    ▼
                          Operations Layer
                    ├── Progress Tracking
                    ├── Audit Logging (JSONL)
                    ├── Checkpointing
                    └── Sync Modes (Full/Incremental)
```

### Key Features Implemented
✅ CSV extraction with configurable delimiters/encoding
✅ REST API extraction with pagination (offset/page/cursor)
✅ Excel placeholder (MVP: convert to CSV)
✅ Field mapping with required fields, defaults, validation
✅ Field filtering (keep/remove modes)
✅ Base.vn API loader with Bearer token auth
✅ Rate limiting with adaptive adjustment (429 responses)
✅ Exponential backoff retry (configurable)
✅ Real-time progress tracking
✅ Atomic checkpointing for resume capability
✅ Audit logging in JSONL format
✅ Sync modes: Full (delete + insert) & Incremental (key matching)
✅ Conflict strategies: Skip, Update, Error

### Roadmap Status
- [x] Phase 1 - Foundation (core types, config, errors)
- [x] Phase 2 - Extract Layer (CSV, REST API, registry)
- [x] Phase 3 - Transform Layer (mapping, filtering, chain)
- [x] Phase 4 - Load Layer (Base.vn loader, rate limit, retry)
- [x] Phase 5 - Operations (progress, audit, checkpoint, sync)
- [ ] Phase 6 - Polish & Release (integration tests, docs, binaries)

---

## 6. Configuration Example

```yaml
version: "1"

job:
  id: "import-employees"
  name: "Import employee data from CSV"
  description: "Migrate employee records from CSV to Base.vn"

source:
  type: csv
  path: "./examples/employees.csv"
  csv:
    delimiter: ","
    has_header: true
    encoding: "utf-8"

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
    rate_limit: 10
    timeout: 30
    max_retries: 3

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
      required: false
      default: "Unknown"
  unmapped_fields: ignore
  missing_required: error

sync:
  mode: incremental
  incremental:
    key_field: "email"
    conflict: update
    state_file: "./sync-state.json"

retry:
  max_attempts: 3
  strategy: exponential
  initial_delay_ms: 1000
  max_delay_ms: 30000
  multiplier: 2.0

logging:
  level: info
  format: json
  file: "./logs/migro.log"

audit:
  enabled: true
  file: "./logs/audit.jsonl"
  include_record_data: false

checkpoint:
  enabled: true
  file: "./.migro/checkpoint.json"
  interval: 100
```

---

## 7. File Structure

```
basevn-migro/
├── src/
│   ├── cli/              # CLI interface (commands: validate, migrate, status)
│   ├── config/           # Configuration module (6 files)
│   │   ├── mod.rs        # Public API exports
│   │   ├── job.rs        # Job config (327 lines)
│   │   ├── source.rs     # Source config (170 lines)
│   │   ├── target.rs     # Target config (104 lines)
│   │   ├── mapping.rs    # Field mapping (61 lines)
│   │   └── loader.rs     # Config loading (145 lines)
│   ├── core/             # Core types (2 files)
│   │   ├── mod.rs        # Module exports
│   │   └── record.rs     # Record, Value types (207 lines)
│   ├── error/            # Error handling (2 files)
│   │   ├── mod.rs        # Module exports
│   │   └── types.rs      # Error types (114 lines)
│   ├── extract/          # Extraction layer (4+ files)
│   ├── transform/        # Transformation layer (3+ files)
│   ├── load/             # Loading layer (3+ files)
│   ├── operations/       # Operations layer (4 files)
│   └── lib.rs
├── Cargo.toml            # Dependencies & metadata
├── README.md             # Project documentation
└── examples/
    └── job.yaml          # Example configuration
```

---

## 8. Key Design Patterns

### 1. Registry Pattern
Used for extensibility:
- **Extractor Registry** - Dynamic dispatch for CSV, Excel, REST API
- **Transformer Registry** - Config-driven pipeline composition
- **Loader Registry** - Support for multiple target systems

### 2. Error Hierarchy
- Domain-specific error types (`ConfigError`, `ExtractError`, etc.)
- Automatic conversion via `#[from]` derive
- Record-level error tracking with context (offset, key)

### 3. Type Safety
- Tagged enums for config variants (`SourceConfig::Csv { ... }`)
- Sealed field visibility (private with public API methods)
- Serde integration for YAML/JSON serialization

### 4. Async/Streaming
- Tokio for async runtime
- Streaming extraction for memory efficiency
- Batch processing at loader for throughput

### 5. Composability
- Field mapping as composable transform
- Transform chains for sequential processing
- Checkpoint intervals for fault tolerance

---

## Key Code Paths

**Config Loading Flow:**
```
load_config(path)
  → read_file(path)
  → serde_yaml::from_str()
  → validate_config()
    ├─ Check job.id not empty
    ├─ Check job.name not empty
    └─ Check mapping.fields not empty
  → Ok(JobConfig)
```

**Record Flow Through Pipeline:**
```
Extractor → Record
  ↓
Transformer(s) → Record (fields mapped/filtered)
  ↓
Loader → Base.vn API
  ├─ Rate Limiter (track requests/sec)
  ├─ Retry Logic (exponential backoff)
  └─ Response Parser (individual record tracking)
  ↓
Operations Layer:
  ├─ Progress Bar (indicatif)
  ├─ Audit Log (JSONL)
  ├─ Checkpoint (offset tracking)
  └─ Sync Mode (incremental matching)
```

---

## Unresolved Questions

None at this stage - foundational exploration complete.

