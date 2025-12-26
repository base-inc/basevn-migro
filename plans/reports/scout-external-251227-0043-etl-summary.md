# ETL Architecture Quick Reference

## Three Core Layers

### 1. EXTRACT LAYER (`src/extract/`)
Reads data from heterogeneous sources into async streams of `Record` objects.

| Component | File | Type | Output |
|-----------|------|------|--------|
| **CSV Extractor** | `csv.rs` | Configurable delimiters, headers, encoding | RecordStream |
| **REST API Extractor** | `rest_api.rs` | Offset/Page/Cursor pagination, JSON parsing | RecordStream |
| **Excel Extractor** | `excel.rs` | Placeholder (convert to CSV for MVP) | - |
| **Extractor Registry** | `registry.rs` | Type-based dispatch (enum → implementation) | Arc<dyn Extractor> |

**RecordStream Type**: `Pin<Box<dyn Stream<Item = Result<Record, ExtractError>> + Send>>`

---

### 2. TRANSFORM LAYER (`src/transform/`)
Applies sequential transformations to records with error propagation.

| Component | File | Operation | Behavior |
|-----------|------|-----------|----------|
| **Field Mapper** | `field_mapping.rs` | Rename, defaults, required fields | Configurable missing/unmapped handling |
| **Field Filter** | `field_filter.rs` | Keep/Remove fields | Privacy + data reduction |
| **Transformer Chain** | `traits.rs` | Sequential composition | First error stops chain |
| **Transformer Registry** | `registry.rs` | Name-based dispatch + builder | Extensible transformer lookup |

**Configuration**: 
- `MissingRequiredBehavior`: Error | SkipRecord
- `UnmappedFieldBehavior`: Ignore | Warn | Error

---

### 3. LOAD LAYER (`src/load/`)
Writes records to Base.vn API with resilience patterns.

| Component | File | Feature | Implementation |
|-----------|------|---------|-----------------|
| **Base.vn Loader** | `basevn.rs` | HTTP POST, auth, batch processing | Bearer token + JSON payload |
| **Rate Limiter** | `rate_limiter.rs` | Token bucket algorithm | Adaptive 429 response handling |
| **Retry Logic** | `basevn.rs` | Exponential backoff | 1s → 2s → 4s → 8s |
| **Response Parser** | `basevn.rs` | 3 response formats | Per-record failure tracking |
| **Loader Registry** | `registry.rs` | Type-based dispatch | Default factory pattern |

**Response Parsing Formats**:
1. Detailed: `{ "results": [{ "index", "status", "error" }] }`
2. Simple: `{ "created", "failed", "errors": [...] }`
3. Fallback: Non-JSON → assume all success

**LoadResult Struct**:
```rust
struct LoadResult {
    total: usize,
    success: usize,
    failed: usize,
    failures: Vec<RecordFailure>,
}
```

---

## Universal Data Types

### Value Enum
```rust
enum Value {
    Null, Bool(bool), Number(f64), String(String),
    Array(Vec<Value>), Object(HashMap<String, Value>)
}
```

### Record Struct
```rust
struct Record {
    fields: HashMap<String, Value>,
    metadata: HashMap<String, String>,  // Lineage tracking
}
```

**Metadata Examples**:
- CSV: `source_line`, `source_file`
- API: `source_offset`, `source_page`, `source_cursor`

---

## Registry Pattern (Unified Across Layers)

### Extract Registry
```
SourceConfig::Csv → CsvExtractor
SourceConfig::Excel → ExcelExtractor
SourceConfig::RestApi → RestApiExtractor
```

### Transform Registry
```
"field_mapper" → FieldMapper instance
"field_filter" → FieldFilter instance
```

### Load Registry
```
TargetConfig::BaseVn → BaseVnLoader instance
```

**Pattern Benefits**: Plugin architecture, runtime dispatch, type-safe mapping

---

## Data Flow Example: CSV → BaseVn

```
1. Extract (CSV) → RecordStream
   ├─ Read employees.csv
   ├─ Parse headers: "Name", "Email", "Dept"
   └─ Emit Record { fields: {...}, metadata: {source_line: "1", ...} }

2. Transform → TransformerChain
   ├─ FieldMapper: "Name" → "full_name"
   ├─ FieldMapper: "Dept" → "department" (default: "Unknown")
   ├─ FieldFilter: Remove "internal_id"
   └─ Emit Record { fields: {full_name, email, department}, metadata: {...} }

3. Load (BaseVn) → LoadResult
   ├─ RateLimiter.acquire() [10 req/s token bucket]
   ├─ POST /api/hrm/employees
   ├─ Parse response → identify per-record failures
   ├─ On 429: adjust rate to 75%, retry with exponential backoff
   └─ Return LoadResult { total: 1000, success: 995, failed: 5, failures: [...] }
```

---

## Configuration Example (job.yaml)

```yaml
version: "1"

job:
  id: "import-employees"
  name: "Import from CSV"

source:
  type: csv
  path: "./employees.csv"
  csv:
    delimiter: ","
    has_header: true

transform:
  - type: field_mapper
    fields:
      - source: "Name"
        target: "full_name"
        required: true
      - source: "Dept"
        target: "department"
        default: "Unknown"
  - type: field_filter
    mode: remove
    fields: ["internal_id", "password"]

target:
  type: basevn
  basevn:
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
```

---

## Extensibility: Adding Custom Components

### Custom Extractor
```rust
impl Extractor for MyExtractor {
    fn name(&self) -> &'static str { "my_source" }
    fn supported_types(&self) -> &[&'static str] { &["my_source"] }
    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError> { ... }
    ...
}
registry.register(Arc::new(MyExtractor));
```

### Custom Transformer
```rust
impl Transformer for MyTransformer {
    fn name(&self) -> &'static str { "my_transform" }
    fn transform(&self, record: Record) -> Result<Record, TransformError> { ... }
}
registry.register(Arc::new(MyTransformer));
```

### Custom Loader
```rust
impl Loader for MyLoader {
    fn name(&self) -> &'static str { "my_target" }
    async fn load_batch(&self, records: Vec<Record>, config: &TargetConfig) -> Result<LoadResult, LoadError> { ... }
}
registry.register(Arc::new(MyLoader));
```

---

## Error Handling

### Layer-Specific Errors
- **Extract**: FileRead, CsvParse, ApiRequest, UnsupportedType
- **Transform**: MissingRequiredField, TransformFailed, InvalidFieldMapping
- **Load**: Auth (401/403), RateLimit (429), Network, ApiRequest

### Resilience Strategies
1. **Retry with exponential backoff**: Network, RateLimit, ApiRequest errors
2. **Adaptive rate limiting**: Reduce rate to 75% on 429, min 1 req/s
3. **Per-record failure tracking**: Batch-level errors don't fail entire load
4. **Config validation**: Pre-flight checks prevent runtime surprises
5. **Metadata preservation**: Lineage tracked through entire pipeline

---

## Testing & Quality

- **81 passing tests** across all layers
- **0 clippy warnings**
- Unit tests for: CSV parsing, REST API pagination, field mapping, filtering, Base.vn response parsing, rate limiting, registry operations
- Full async/await support
- Type-safe configuration

---

## File Organization (16 Core Files)

```
src/extract/     (4 files, 1017 lines)
├─ traits.rs         Extractor interface
├─ csv.rs            CSV implementation
├─ rest_api.rs       REST API implementation
└─ registry.rs       Type-based dispatch

src/transform/   (4 files, 903 lines)
├─ traits.rs         Transformer interface + chain
├─ field_mapping.rs  Rename + defaults + required
├─ field_filter.rs   Keep/Remove modes
└─ registry.rs       Name-based dispatch + builder

src/load/        (4 files, 1029 lines)
├─ traits.rs         Loader interface + LoadResult
├─ basevn.rs         Base.vn API client
├─ rate_limiter.rs   Token bucket algorithm
└─ registry.rs       Type-based dispatch

src/core/config/ (4 files, ~360+ lines)
├─ record.rs         Value + Record types
├─ source.rs         CSV/Excel/RestApi configs
├─ target.rs         BaseVn target config
└─ mapping.rs        FieldMap + filter configs
```

---

## Key Design Principles

1. **Plugin Architecture**: Registries enable custom components
2. **Async Streaming**: Memory-efficient processing of large datasets
3. **Type Safety**: Rust prevents configuration errors at compile time
4. **Error Isolation**: Per-record failures don't fail entire batches
5. **Metadata Tracking**: Full lineage from source through transformations
6. **Config-Driven**: YAML/JSON specifications with environment variables
7. **Resilience**: Exponential backoff, rate limiting, adaptive adjustment
8. **Observability**: Comprehensive error types and structured logging

---

**Generated**: 2025-12-27 | Status: Complete
