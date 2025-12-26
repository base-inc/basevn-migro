# basevn-migro: Codebase Summary

**Version**: 1.0
**Last Updated**: December 2024
**Generated from**: Repomix analysis (55 files, 44,626 tokens)

## Overview

basevn-migro is a **Rust-based ETL tool** with a streamlined, modular architecture. The codebase is organized into six major layers: CLI, Configuration, Core Data Types, Error Handling, and three ETL layers (Extract, Transform, Load) plus an Operations layer for observability.

**Code Metrics:**
- **Total Lines**: 5,495
- **Source Files**: 40
- **Test Files**: 81 tests (100% passing)
- **Clippy Warnings**: 0
- **Test Coverage**: 70%+ for core logic
- **Edition**: Rust 2021

## Directory Structure

```
basevn-migro/
├── src/
│   ├── cli/                    # Command-line interface
│   │   ├── args.rs            # CLI argument parsing with clap
│   │   ├── mod.rs             # CLI module organization
│   │   └── commands/          # Subcommands
│   │       ├── validate.rs    # Configuration validation
│   │       ├── migrate.rs     # Run ETL pipeline
│   │       ├── status.rs      # Job status tracking
│   │       ├── update.rs      # Self-update functionality
│   │       └── mod.rs         # Command routing
│   │
│   ├── config/                # Configuration loading & validation
│   │   ├── mod.rs             # Config module root
│   │   ├── job.rs             # Job metadata
│   │   ├── source.rs          # Source configuration
│   │   ├── target.rs          # Target configuration
│   │   ├── loader.rs          # Loader-specific config
│   │   └── mapping.rs         # Field mapping configuration
│   │
│   ├── core/                  # Core data types
│   │   ├── mod.rs             # Core module exports
│   │   └── record.rs          # Record, Value, Field types
│   │
│   ├── error/                 # Error handling
│   │   ├── mod.rs             # Error module exports
│   │   └── types.rs           # Error enum definitions
│   │
│   ├── extract/               # Data extraction layer (1,017 LOC)
│   │   ├── traits.rs          # Extractor trait definition
│   │   ├── csv.rs             # CSV file extraction
│   │   ├── rest_api.rs        # REST API extraction (3,045 tokens)
│   │   ├── excel.rs           # Excel placeholder
│   │   ├── registry.rs        # Dynamic extractor dispatch
│   │   └── mod.rs             # Extract module exports
│   │
│   ├── transform/             # Data transformation layer (903 LOC)
│   │   ├── traits.rs          # Transformer trait definition
│   │   ├── field_mapping.rs   # Field mapping transformer
│   │   ├── field_filter.rs    # Field filtering transformer
│   │   ├── registry.rs        # Dynamic transformer dispatch
│   │   └── mod.rs             # Transform module exports
│   │
│   ├── load/                  # Data loading layer (1,029 LOC)
│   │   ├── traits.rs          # Loader trait definition
│   │   ├── basevn.rs          # Base.vn API loader (3,694 tokens)
│   │   ├── rate_limiter.rs    # Rate limiting implementation
│   │   ├── registry.rs        # Dynamic loader dispatch
│   │   └── mod.rs             # Load module exports
│   │
│   ├── operations/            # Operations & observability layer
│   │   ├── progress.rs        # Real-time progress tracking
│   │   ├── audit.rs           # JSONL audit logging (1,990 tokens)
│   │   ├── checkpoint.rs      # Checkpoint management
│   │   ├── sync.rs            # Sync modes & conflict resolution
│   │   └── mod.rs             # Operations module exports
│   │
│   ├── lib.rs                 # Library root, public API
│   └── main.rs                # Binary entry point
│
├── tests/                     # Integration tests (future)
│
├── examples/
│   ├── job-csv-to-basevn.yaml
│   ├── job-api-to-basevn.yaml
│   ├── employees.csv
│   ├── simple_csv/
│   │   ├── config.yaml
│   │   └── data.csv
│   └── README.md
│
├── docs/                      # Documentation
│   ├── project-overview-pdr.md
│   ├── codebase-summary.md
│   ├── code-standards.md
│   ├── system-architecture.md
│   ├── architecture.md
│   ├── getting-started.md
│   ├── configuration.md
│   └── troubleshooting.md
│
├── plans/                     # Design documents
│   ├── plan.md
│   └── reports/
│
├── Cargo.toml                 # Rust dependencies
├── Cargo.lock                 # Dependency lock file
├── rustfmt.toml               # Code formatting config
├── clippy.toml                # Linting config
├── .gitignore
├── .repomixignore
├── README.md
├── CONTRIBUTING.md
├── CLAUDE.md
└── LICENSE (Apache 2.0)
```

## Core Modules Explained

### 1. CLI Layer (`src/cli/`)

**Purpose**: Command-line interface and argument parsing.

**Key Files**:
- `args.rs`: Uses clap derive macros for command structure
  - `validate`: Validate configuration without running
  - `migrate`: Execute ETL pipeline
  - `status`: Check job status
  - `update`: Self-update mechanism

**Key Types**:
```rust
pub struct Args {
    pub command: Command,
}

pub enum Command {
    Validate { config: PathBuf },
    Migrate { config: PathBuf },
    Status { job_id: String },
    Update,
}
```

**Responsibilities**:
- Parse command-line arguments
- Dispatch to appropriate command handler
- Error reporting to user
- Environment variable loading

### 2. Configuration Layer (`src/config/`)

**Purpose**: Load, parse, and validate configuration files.

**Key Files**:
- `job.rs`: Job metadata (id, name, description)
- `source.rs`: Source configuration (type, path, options)
- `target.rs`: Target configuration (type, credentials, batch settings)
- `mapping.rs`: Field mapping rules
- `loader.rs`: Loader-specific settings

**Key Types**:
```rust
pub struct JobConfig {
    pub id: String,
    pub name: String,
    pub source: SourceConfig,
    pub transform: Vec<TransformConfig>,
    pub target: TargetConfig,
    pub operations: OperationsConfig,
}

pub struct SourceConfig {
    pub r#type: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub options: Map<String, Value>,
}
```

**Responsibilities**:
- Load YAML/JSON configuration files
- Validate required fields
- Substitute environment variables
- Resolve file paths

### 3. Core Data Types (`src/core/`)

**Purpose**: Unified data representation across all layers.

**Key Files**:
- `record.rs`: Core data structures

**Key Types**:
```rust
pub struct Record {
    fields: HashMap<Field, Value>,
    metadata: HashMap<String, String>,
}

pub type Field = String;

pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

**Responsibilities**:
- Define Record structure (field-value pairs)
- Provide metadata storage
- Implement value serialization/deserialization

### 4. Error Handling (`src/error/`)

**Purpose**: Structured error types with context.

**Key Files**:
- `types.rs`: Error enum definitions

**Error Types**:
```rust
pub enum MigroError {
    Config(ConfigError),
    Extract(ExtractError),
    Transform(TransformError),
    Load(LoadError),
    Io(std::io::Error),
}
```

**Responsibilities**:
- Define error types per layer
- Provide error context preservation
- Enable proper error reporting

### 5. Extract Layer (`src/extract/`, 1,017 LOC)

**Purpose**: Read data from various sources.

**Key Files**:
- `traits.rs`: Extractor trait definition (Send + Sync)
- `csv.rs`: CSV file extraction with configurable parsing
- `rest_api.rs`: REST API extraction with pagination (3,045 tokens)
- `excel.rs`: Excel placeholder (MVP: use CSV)
- `registry.rs`: ExtractorRegistry for dynamic dispatch

**Trait**:
```rust
pub trait Extractor: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError>;
    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError>;
    fn estimate_count(&self, config: &SourceConfig) -> Result<Option<usize>, ExtractError>;
}
```

**Key Extractors**:

1. **CsvExtractor**
   - Configurable delimiter (default: comma)
   - Header detection
   - Encoding support (UTF-8, ISO-8859-1, etc.)
   - Line tracking for error reporting

2. **RestApiExtractor**
   - GET request support
   - Three pagination modes: offset, page, cursor
   - JSONPath response parsing
   - Custom header injection
   - Bearer token support

3. **ExcelExtractor**
   - Placeholder for Phase 7
   - Currently returns error with conversion guidance

**Responsibilities**:
- Produce RecordStream from data sources
- Track extraction progress
- Validate source configuration
- Estimate record counts for planning

### 6. Transform Layer (`src/transform/`, 903 LOC)

**Purpose**: Modify records according to rules.

**Key Files**:
- `traits.rs`: Transformer trait definition
- `field_mapping.rs`: Source→target field mapping
- `field_filter.rs`: Keep/remove field modes
- `registry.rs`: TransformerRegistry for dynamic dispatch

**Trait**:
```rust
pub trait Transformer: Send + Sync {
    fn name(&self) -> &'static str;
    fn transform(&self, record: Record) -> Result<Record, TransformError>;
    fn transform_batch(&self, records: Vec<Record>) -> Result<Vec<Record>, TransformError>;
}
```

**Key Transformers**:

1. **FieldMapper**
   - Source field → target field mapping
   - Default value assignment
   - Required field validation
   - Nested field support (dot notation)

2. **FieldFilter**
   - Keep mode: include only specified fields
   - Remove mode: exclude specified fields
   - Data privacy support (filter passwords, SSNs)

3. **TransformerChain**
   - Sequential composition of multiple transformers
   - Stop-on-error behavior
   - Batch processing optimization

**Responsibilities**:
- Apply transformation rules to records
- Validate field mappings
- Track transformation errors
- Support chained transformations

### 7. Load Layer (`src/load/`, 1,029 LOC)

**Purpose**: Write records to target systems.

**Key Files**:
- `traits.rs`: Loader trait definition
- `basevn.rs`: Base.vn API loader (3,694 tokens) - largest file
- `rate_limiter.rs`: Token bucket rate limiting
- `registry.rs`: LoaderRegistry for dynamic dispatch

**Trait**:
```rust
#[async_trait]
pub trait Loader: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate_config(&self, config: &TargetConfig) -> Result<(), ConfigError>;
    async fn initialize(&self, config: &TargetConfig) -> Result<(), LoadError>;
    async fn load_batch(&self, records: Vec<Record>, config: &TargetConfig)
        -> Result<LoadResult, LoadError>;
    async fn finalize(&self) -> Result<(), LoadError>;
}
```

**Key Loaders**:

1. **BaseVnLoader**
   - Bearer token authentication
   - Batch processing (configurable batch size)
   - Exponential backoff retry (1s, 2s, 4s, 8s)
   - Rate limiting with adaptive adjustment
   - Per-record error tracking
   - Response parsing (3 format variants)

2. **RateLimiter**
   - Token bucket algorithm
   - Configurable rate (requests per second)
   - Adaptive adjustment on 429 (Too Many Requests)
   - Allows bursting within burst limit

**Responsibilities**:
- Initialize target connection
- Batch records for efficient loading
- Apply rate limiting
- Retry on transient failures
- Track individual record failures
- Finalize and clean up

### 8. Operations Layer (`src/operations/`)

**Purpose**: Cross-cutting concerns for observability and reliability.

**Key Modules**:

1. **Progress Tracker** (`progress.rs`)
   - Real-time progress bars with indicatif
   - Success rate tracking
   - ETA calculation
   - Record count per second

2. **Audit Logger** (`audit.rs`, 1,990 tokens)
   - JSONL format (one event per line)
   - Append-only writes
   - Event types: extract, transform, load, error
   - Timestamps and context preservation

3. **Checkpoint Manager** (`checkpoint.rs`)
   - Atomic writes (temp file + rename)
   - Resume capability from last offset
   - Metadata tracking (processed count, errors)
   - Crash-safe implementation

4. **Sync Manager** (`sync.rs`)
   - Full sync: delete all, insert new
   - Incremental sync: key field matching
   - Conflict resolution: skip, update, error
   - Deduplication logic

**Responsibilities**:
- Track migration progress
- Log all operations for audit trail
- Enable job resumption
- Manage sync strategies

## Dependencies

### Runtime Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.35 | Async runtime with full features |
| clap | 4.4 | CLI argument parsing (derive) |
| serde | 1.0 | Serialization framework |
| serde_yaml | 0.9 | YAML configuration parsing |
| serde_json | 1.0 | JSON parsing |
| reqwest | 0.11 | Async HTTP client (rustls) |
| csv | 1.3 | CSV file parsing |
| calamine | 0.24 | Excel parsing (placeholder) |
| thiserror | 1.0 | Error type derivation |
| anyhow | 1.0 | Error context handling |
| async-trait | 0.1 | Async trait methods |
| futures | 0.3 | Future utilities |
| chrono | 0.4 | Timestamp handling |
| uuid | 1.6 | Unique ID generation |
| indicatif | 0.17 | Progress bars |
| tracing | 0.1 | Structured logging |
| validator | 0.16 | Data validation |

### Development Dependencies

| Crate | Purpose |
|-------|---------|
| mockall | Mocking for tests |
| tempfile | Temporary file handling in tests |
| assert_cmd | CLI testing |
| predicates | Assertions for tests |

## Code Organization Principles

### 1. Layered Architecture
- Clear separation of concerns
- Each layer has defined interfaces
- Minimal coupling between layers

### 2. Plugin System via Registries
- ExtractorRegistry: Extensible extractors
- TransformerRegistry: Extensible transformers
- LoaderRegistry: Extensible loaders
- Type-safe without dynamic library loading

### 3. Streaming Data Model
- Records processed in batches
- No all-at-once caching
- Memory-efficient for large datasets

### 4. Error Handling
- Structured error types per layer
- Context preservation with anyhow
- Detailed error messages for users

### 5. Testing Strategy
```
Unit Tests (60%)     ← Individual components
  ↓
Integration Tests (30%) ← Layer interactions
  ↓
E2E Tests (10%)      ← Full pipeline with mock API
```

## Key Design Decisions

1. **Synchronous Transform, Async Load**
   - Transformations are CPU-bound and fast
   - Loading is I/O-bound and concurrent

2. **Streaming CSV Reading**
   - Single-threaded for simplicity
   - Parallel reading added in Phase 7

3. **Atomic Checkpointing**
   - Temp file + rename for crash safety
   - No partial checkpoint corruption

4. **JSONL Audit Format**
   - One event per line (grep/jq friendly)
   - Append-only for performance

5. **Bearer Token Auth**
   - Secure credential handling
   - No credential files

## Metrics & Statistics

### Code Quality
- **Warnings**: 0 (cargo clippy -D warnings clean)
- **Format**: 100% compliant (rustfmt.toml enforced)
- **Edition**: 2021 (latest features)
- **MSRV**: 1.75 (stable, proven)

### Test Coverage
- **Total Tests**: 81
- **Pass Rate**: 100%
- **Coverage Target**: 70%+ for core logic

### File Sizes (Top 5 by tokens)
1. `src/load/basevn.rs` (3,694 tokens) - Base.vn loader implementation
2. `src/extract/rest_api.rs` (3,045 tokens) - REST API extraction
3. `README.md` (2,274 tokens) - Project overview
4. `LICENSE` (2,142 tokens) - Apache 2.0 license
5. `src/operations/audit.rs` (1,990 tokens) - Audit logging

### Module Sizes
| Module | Files | LOC | Purpose |
|--------|-------|-----|---------|
| extract | 6 | 1,017 | Data source readers |
| transform | 5 | 903 | Data transformations |
| load | 5 | 1,029 | Target system writers |
| operations | 5 | 600+ | Observability features |
| config | 6 | 400+ | Configuration loading |
| cli | 4 | 300+ | Command interface |
| core | 2 | 250+ | Core data types |
| error | 2 | 150+ | Error definitions |

## Build & Development Workflow

### Building
```bash
cargo build              # Debug build
cargo build --release   # Optimized release
```

### Testing
```bash
cargo test              # Run all tests
cargo test -- --nocapture  # With output
cargo test test_name    # Specific test
```

### Code Quality
```bash
cargo fmt --all -- --check    # Check formatting
cargo clippy -- -D warnings   # Lint check
cargo clippy --fix            # Auto-fix
```

### Documentation
```bash
cargo doc --open        # Generate and open docs
```

## Extension Points

### Adding New Extractor
1. Create `src/extract/my_extractor.rs`
2. Implement `Extractor` trait
3. Register in `create_default_registry()`

### Adding New Transformer
1. Create `src/transform/my_transformer.rs`
2. Implement `Transformer` trait
3. Register in `create_default_registry()`

### Adding New Loader
1. Create `src/load/my_loader.rs`
2. Implement async `Loader` trait
3. Register in `create_default_registry()`

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| CSV Extract | O(n) | Linear file read |
| REST API Extract | O(n/p) | p = page size |
| Field Mapping | O(m) | m = mapping count |
| Field Filtering | O(f) | f = field count |
| Batch Load | O(b) | b = batch size |
| Registry Lookup | O(1) | HashMap access |
| Checkpoint Write | O(1) | Atomic rename |

## Known Limitations

1. Excel extractor is placeholder (use CSV conversion)
2. No parallel extraction (Phase 7)
3. Single-threaded transform layer
4. No database extractors (Phase 7)
5. No connection pooling (Phase 7)

---

**Generated**: December 2024
**Source**: Repomix analysis
**Next Review**: After Phase 7 completion
