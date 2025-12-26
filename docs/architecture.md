# Architecture Overview

High-level architecture and design decisions for basevn-migro.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     basevn-migro CLI                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐   ┌───────────┐   ┌──────┐   ┌────────────┐ │
│  │ Extract  │──▶│ Transform │──▶│ Load │──▶│ Operations │ │
│  └──────────┘   └───────────┘   └──────┘   └────────────┘ │
│       │              │              │              │        │
│  ┌────▼────┐    ┌───▼────┐    ┌───▼────┐    ┌────▼─────┐ │
│  │Registry │    │Registry│    │Registry│    │Checkpoint│ │
│  └─────────┘    └────────┘    └────────┘    │Audit Log │ │
│                                              │Progress  │ │
│                                              └──────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Core Layers

### 1. Extract Layer

**Responsibility**: Read data from various sources into unified Record format.

**Components**:
- `CsvExtractor`: CSV file extraction with configurable parsing
- `RestApiExtractor`: REST API with pagination (offset, page, cursor)
- `ExcelExtractor`: Placeholder (convert to CSV for MVP)
- `ExtractorRegistry`: Dynamic dispatch by source type

**Design Pattern**: Strategy pattern with registry

```rust
pub trait Extractor: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError>;
    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError>;
    fn estimate_count(&self, config: &SourceConfig) -> Result<Option<usize>, ExtractError>;
}
```

### 2. Transform Layer

**Responsibility**: Modify records according to mapping and filtering rules.

**Components**:
- `FieldMapper`: Source → target field mapping with defaults, validation
- `FieldFilter`: Keep/remove modes for field selection
- `TransformerChain`: Sequential transformer application
- `TransformerRegistry`: Dynamic dispatch by transformer type

**Design Pattern**: Chain of Responsibility with registry

```rust
pub trait Transformer: Send + Sync {
    fn name(&self) -> &'static str;
    fn transform(&self, record: Record) -> Result<Record, TransformError>;
    fn transform_batch(&self, records: Vec<Record>) -> Result<Vec<Record>, TransformError>;
}
```

### 3. Load Layer

**Responsibility**: Write records to target systems with reliability features.

**Components**:
- `BaseVnLoader`: Base.vn API client with auth, batching, retry
- `RateLimiter`: Token bucket algorithm with adaptive adjustment
- `LoaderRegistry`: Dynamic dispatch by target type

**Design Pattern**: Template Method with registry

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

### 4. Operations Layer

**Responsibility**: Operational concerns (progress, audit, checkpoint, sync).

**Components**:
- `ProgressTracker`: Real-time progress bars with stats
- `AuditLogger`: JSONL event logging
- `CheckpointManager`: Atomic checkpoint writes for resume
- `SyncStrategy`: Full/incremental sync modes with conflict resolution

**Design Pattern**: Cross-cutting concerns

## Key Design Decisions

### 1. Registry Pattern

**Reason**: Enable dynamic dispatch without dynamic library loading.

**Benefits**:
- Type-safe plugin system
- No runtime dependencies
- Easy testing and mocking
- Config-driven pipeline construction

### 2. Record-Centric Data Model

**Reason**: Unified data representation across layers.

```rust
pub struct Record {
    fields: HashMap<Field, Value>,
    metadata: HashMap<String, String>,
}
```

**Benefits**:
- Layer independence
- Flexible schema
- Metadata propagation
- Easy transformation

### 3. Async/Await Architecture

**Reason**: Efficient I/O for network requests and file operations.

**Stack**:
- `tokio`: Async runtime
- `reqwest`: Async HTTP client
- `async-trait`: Trait async methods

**Benefits**:
- Non-blocking I/O
- Scalable concurrency
- Resource efficiency

### 4. Error Handling Strategy

**Approach**: Structured errors with thiserror, granular error types.

```rust
pub enum MigroError {
    Config(ConfigError),
    Extract(ExtractError),
    Transform(TransformError),
    Load(LoadError),
    Io(std::io::Error),
}
```

**Benefits**:
- Clear error messages
- Error context preservation
- Retry decision logic
- User-friendly reporting

### 5. Checkpointing with Atomic Writes

**Implementation**: Temp file + rename pattern.

```rust
// Write to temp
write_to_file("checkpoint.json.tmp", checkpoint)?;
sync_to_disk()?;
// Atomic rename
rename("checkpoint.json.tmp", "checkpoint.json")?;
```

**Benefits**:
- Crash-safe writes
- No partial checkpoints
- Resume reliability

### 6. JSONL Audit Format

**Reason**: Streaming-friendly, tool-compatible format.

**Benefits**:
- One event per line (easy grep/jq)
- Append-only (simple, fast)
- Structured data (JSON)
- Streaming-friendly

## Data Flow

```
Source Data
    ↓
┌─────────────┐
│  Extractor  │ → RecordStream
└─────────────┘
    ↓
┌─────────────┐
│ Transformer │ → Record[] (batched)
└─────────────┘
    ↓
┌─────────────┐
│   Loader    │ → LoadResult
└─────────────┘
    ↓
Target System
```

## Concurrency Model

- **Extract**: Single-threaded streaming (no concurrent file reads)
- **Transform**: Synchronous (CPU-bound, fast operations)
- **Load**: Async batching (concurrent API requests within batch)
- **Operations**: Thread-safe (Arc/Mutex where needed)

## Memory Management

- **Streaming**: Process records in batches, not all-at-once
- **Batch Size**: Configurable (default 100 records)
- **No Record Caching**: Records released after batch processing
- **Target**: <2GB RAM for 1M+ record migrations

## Testing Strategy

```
   E2E Tests (10%)
        │
   Integration Tests (30%)
        │
   Unit Tests (60%)
```

- **Unit**: Individual components, mocked dependencies
- **Integration**: Layer interactions, fixture data
- **E2E**: Full pipeline, real/mock Base.vn API

## Extension Points

### Adding New Extractor

```rust
pub struct PostgresExtractor { /* ... */ }

impl Extractor for PostgresExtractor {
    fn name(&self) -> &'static str { "postgres" }
    // Implement trait methods...
}

// Register in create_default_registry()
registry.register(Arc::new(PostgresExtractor::new()));
```

### Adding New Transformer

```rust
pub struct DataValidator { /* ... */ }

impl Transformer for DataValidator {
    fn name(&self) -> &'static str { "validator" }
    // Implement trait methods...
}
```

### Adding New Loader

```rust
pub struct MongoDbLoader { /* ... */ }

#[async_trait]
impl Loader for MongoDbLoader {
    fn name(&self) -> &'static str { "mongodb" }
    // Implement trait methods...
}
```

## Security Considerations

1. **Environment Variables**: Secrets never in config files
2. **TLS/SSL**: All HTTPS connections verified
3. **Token Storage**: In-memory only, never persisted
4. **Audit Logs**: May contain sensitive data, secure accordingly
5. **Temp Files**: Cleaned up on exit

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| CSV Extract | O(n) | Linear file read |
| REST API Extract | O(n/p) | p = page size |
| Field Mapping | O(m) | m = mapping count |
| Field Filtering | O(f) | f = field count |
| Batch Load | O(b) | b = batch size |
| Registry Lookup | O(1) | HashMap access |

## Dependencies

**Core**:
- `tokio`: Async runtime
- `serde`: Serialization
- `thiserror`: Error handling

**Extract**:
- `csv`: CSV parsing
- `reqwest`: HTTP client

**Operations**:
- `indicatif`: Progress bars
- `chrono`: Timestamps

**Config**:
- `serde_yaml`: YAML parsing
- `serde_json`: JSON parsing

## Future Enhancements

- [ ] Streaming large file support (>10GB)
- [ ] Connection pooling for loaders
- [ ] Parallel batch processing
- [ ] Type conversion transformers
- [ ] Advanced validation rules
- [ ] PostgreSQL/MongoDB loaders
- [ ] GraphQL API extractor
