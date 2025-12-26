# basevn-migro: System Architecture & Deployment Guide

**Version**: 1.0
**Last Updated**: December 2024
**Status**: Production-Ready
**For Implementation Details**: See [docs/architecture.md](architecture.md)

## High-Level System Design

### System Context Diagram

```
┌─────────────────────────────────────────────────┐
│           basevn-migro CLI                      │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │      Configuration File (YAML/JSON)      │  │
│  │  • Job metadata                          │  │
│  │  • Source configuration                  │  │
│  │  • Transformation rules                  │  │
│  │  • Target configuration                  │  │
│  │  • Operations settings                   │  │
│  └──────────────────────────────────────────┘  │
│                      │                         │
│                      ▼                         │
│  ┌──────────────────────────────────────────┐  │
│  │         Migration Engine                 │  │
│  │  • Extract → Transform → Load Pipeline   │  │
│  │  • Progress Tracking                     │  │
│  │  • Audit Logging                         │  │
│  │  • Checkpoint Management                 │  │
│  └──────────────────────────────────────────┘  │
│                      │                         │
└──────────────────────┼─────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
    ┌────────┐    ┌────────┐    ┌─────────┐
    │  CSV   │    │ REST   │    │ Target  │
    │ Files  │    │  API   │    │ Systems │
    │        │    │        │    │         │
    │Source  │    │Source  │    │ Base.vn │
    └────────┘    └────────┘    └─────────┘
```

### Data Flow Architecture

```
Source System → [Extract] → [Transform] → [Load] → Target System
                    │            │           │
                    └─ Registry ─┴─ Registry ┘
                           │
                    [Operations Layer]
                    • Progress tracking
                    • Audit logging
                    • Checkpointing
                    • Sync management
```

## Deployment Architecture

### Standalone CLI Deployment

basevn-migro is designed as a **standalone CLI tool** with no server component.

**Deployment Model:**
```
┌─────────────────────────────────────┐
│       Developer/DevOps Machine      │
│                                     │
│  ┌───────────────────────────────┐ │
│  │ basevn-migro CLI Binary       │ │
│  │                               │ │
│  │ • Single executable           │ │
│  │ • No dependencies (vendored)  │ │
│  │ • ~20-30 MB binary size       │ │
│  └───────────────────────────────┘ │
│             │                       │
│             ├─→ Config File         │
│             ├─→ CSV/Data Files      │
│             └─→ Output Files        │
│                 (checkpoint, audit) │
└─────────────────────────────────────┘
         │                    │
         ▼                    ▼
    Source Systems      Base.vn API
```

### Execution Modes

#### Mode 1: Interactive CLI (Development)
```bash
./migro migrate --config job.yaml
./migro validate --config job.yaml
./migro status --job-id import-employees
```

**Use Case**: Manual data migrations, testing, debugging

#### Mode 2: Scheduled Job (Production)
```bash
# Cron job for periodic syncs
0 2 * * * /usr/local/bin/migro migrate --config /etc/migro/daily-sync.yaml
```

**Use Case**: Daily/weekly incremental syncs

#### Mode 3: CI/CD Pipeline
```yaml
# GitHub Actions or GitLab CI
migrate:
  script:
    - migro migrate --config config/production.yaml
  environment:
    BASEVN_ACCESS_TOKEN: $BASEVN_ACCESS_TOKEN
```

**Use Case**: Automated deployments, integration tests

### System Requirements

**Minimum Requirements:**
- CPU: 1 core
- Memory: 512 MB RAM
- Disk: 100 MB (binary + configs)
- Network: HTTPS (TLS 1.2+)

**Recommended:**
- CPU: 2+ cores (for concurrent operations in Phase 7)
- Memory: 2 GB RAM (for large migrations)
- Disk: 1 GB (for logs and checkpoints)

**Supported Platforms:**
- Linux (x86_64, ARM)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

### Network Architecture

```
┌─────────────────────────────┐
│   basevn-migro CLI          │
│                             │
│  Outbound:                  │
│  • HTTPS to Source APIs     │  ─→ Firewall: Allow outbound HTTPS
│  • HTTPS to Base.vn API     │  ─→ Firewall: Allow to api.base.vn:443
│                             │
│  Inbound:                   │
│  • None required            │  ← CLI is outbound-only
└─────────────────────────────┘
```

**Security Considerations:**
- All connections use TLS 1.2+ (enforced by reqwest)
- No listening ports exposed
- Credentials via environment variables only
- Audit logs stored locally

## Scalability Architecture

### Processing Capacity

**Single Instance Throughput:**
```
CSV Processing:    1,000-5,000 records/sec
API Processing:    100-1,000 records/sec (depends on API)
Loading:           10-100 records/sec (depends on rate limit)

Limiting Factor:   Target API rate limit
```

### Scaling Strategies

#### Strategy 1: Horizontal Scaling (Multiple Instances)
```
Job Queue (external tool)
    ├─→ Instance 1: Job #1
    ├─→ Instance 2: Job #2
    ├─→ Instance 3: Job #3
    └─→ Instance N: Job #N

Each instance:
• Reads separate input file/query
• Writes to separate key space in target
• Maintains independent checkpoint
```

**Implementation**:
```bash
# Run multiple jobs in parallel on different machines
instance1: migro migrate --config job-users.yaml
instance2: migro migrate --config job-products.yaml
instance3: migro migrate --config job-orders.yaml
```

#### Strategy 2: Batch Partitioning
```
Large Dataset → Partition by Range
    ├─→ Batch 1: Records 1-10,000
    ├─→ Batch 2: Records 10,001-20,000
    ├─→ Batch 3: Records 20,001-30,000
    └─→ Batch N: Records (N-1)*10,000 to N*10,000

Sequential or parallel processing with coordinator
```

**Configuration**:
```yaml
source:
  type: rest_api
  url: "https://api.example.com/users"
  pagination:
    type: offset
    limit: 10000
    offset: 0  # Adjust per batch
```

#### Strategy 3: Incremental Syncing
```
Large Initial Import
    ↓
Daily Incremental Updates (small delta)
    ↓
Balanced resource usage
```

**Configuration**:
```yaml
operations:
  sync_mode:
    mode: incremental
    key_field: id
    conflict_strategy: update
```

### Memory Scaling

**Memory Usage Formula:**
```
Memory = Base (50MB) + RecordCount × AverageRecordSize

Example:
• 1M records × 1KB average = ~1GB
• 10M records × 1KB average = ~10GB
```

**Optimization Strategies:**
1. Reduce batch size if memory constrained
2. Use field filtering to remove unnecessary data
3. Stream processing instead of caching (already implemented)
4. Run on machine with more RAM

## Reliability Architecture

### Fault Tolerance

#### Transient Failures
```
Network timeout / Rate limit exceeded
        │
        ▼
Exponential backoff retry
    1s → 2s → 4s → 8s
        │
        ├─ Success: Continue
        └─ Failure: Log & move to next
```

**Configuration:**
```yaml
target:
  options:
    max_retries: 3
    timeout: 30  # seconds
```

#### Checkpoint-Based Recovery
```
Processing interrupted (crash/power loss)
        │
        ▼
Read last checkpoint (atomic file)
        │
        ▼
Resume from last successful offset
        │
        ▼
Continue processing
```

**Benefits:**
- Zero data loss on process restart
- Efficient resume (skip already processed)
- Atomic writes prevent corruption

#### Data Validation
```
Each record processed through:
    ├─→ Field presence validation
    ├─→ Field mapping validation
    ├─→ Required field checks
    └─→ Error tracking per record

Failed records:
    ├─→ Logged to audit trail
    ├─→ Included in final report
    └─→ Available for retry
```

### Monitoring & Observability

#### Progress Tracking
```
Real-time metrics:
• Records processed: 1,234 / 5,000
• Success rate: 98.5%
• Speed: 245 records/sec
• ETA: 2m 34s remaining
```

#### Audit Logging (JSONL Format)
```json
{"timestamp":"2024-12-27T10:30:45Z","type":"extract_start","job_id":"import-users"}
{"timestamp":"2024-12-27T10:30:50Z","type":"record_extracted","count":1000}
{"timestamp":"2024-12-27T10:31:00Z","type":"transform_complete","count":1000}
{"timestamp":"2024-12-27T10:31:05Z","type":"load_batch","batch_size":100,"success":true}
{"timestamp":"2024-12-27T10:31:10Z","type":"load_error","record_id":"user_123","error":"invalid_email"}
{"timestamp":"2024-12-27T10:32:00Z","type":"migration_complete","total":1000,"success":990,"failed":10}
```

**Usage:**
```bash
# Search for errors
grep '"type":"load_error"' audit.jsonl | jq .

# Count by type
jq -s 'group_by(.type) | map({type: .[0].type, count: length})' audit.jsonl

# Filter by timestamp
jq 'select(.timestamp > "2024-12-27T10:30:00Z")' audit.jsonl
```

#### Status Tracking
```
Status file: checkpoint.json
{
  "job_id": "import-users",
  "total_processed": 5000,
  "total_success": 4950,
  "total_failed": 50,
  "last_offset": 5000,
  "last_timestamp": "2024-12-27T10:31:50Z",
  "status": "completed"
}
```

### Error Handling Strategy

```
Recoverable Errors (Retry):
• Network timeouts
• Rate limit (429 Too Many Requests)
• Temporary API unavailability (5xx)
→ Exponential backoff with max retries

Record-Level Errors (Log & Continue):
• Invalid data format
• Missing required field
• Duplicate key (with conflict strategy)
→ Log to audit trail, move to next record

Configuration Errors (Fail Fast):
• Invalid YAML syntax
• Missing required fields
• Invalid field mappings
→ Exit with clear error message

Unrecoverable Errors (Halt):
• File not found
• Authentication failure
• Out of memory
→ Exit with detailed error
```

## Integration Points

### Source Systems

#### CSV Files
```
Source: Local/mounted file
Format: RFC 4180 CSV
Access: File read permissions
Typical: Employee lists, product catalogs

Configuration:
source:
  type: csv
  path: "./employees.csv"
  options:
    delimiter: ","
    encoding: "utf-8"
    has_headers: true
```

#### REST APIs
```
Source: HTTP/HTTPS endpoint
Format: JSON (with JSONPath support)
Access: API key / Bearer token
Typical: Salesforce, HubSpot, custom APIs

Configuration:
source:
  type: rest_api
  url: "https://api.example.com/contacts"
  pagination:
    type: "offset"  # or "page" or "cursor"
    limit: 100
```

### Target System: Base.vn

```
Endpoint: https://api.base.vn
Authentication: Bearer token (environment variable)
Format: JSON
Methods: POST (create), PUT (update)
Batch Size: Configurable (default: 100)
Rate Limit: Configurable (default: 10 req/sec)

Configuration:
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
```

### External Configuration

Environment variables for credentials:
```bash
BASEVN_ACCESS_TOKEN=abc123xyz...
CSV_FILE_PATH=/data/employees.csv
API_KEY=key123...
```

Used in config files:
```yaml
auth:
  token: "${BASEVN_ACCESS_TOKEN}"

source:
  path: "${CSV_FILE_PATH}"
```

## Performance Characteristics

### Throughput Analysis

**CSV Extraction:**
```
File I/O: ~10-50 MB/s (depends on storage)
Typical: 1,000-5,000 records/sec
Limiting: Disk I/O, record size
```

**REST API Extraction:**
```
Network I/O: ~1-10 MB/s
Typical: 100-1,000 records/sec
Limiting: API rate limit, pagination
```

**Transformation:**
```
CPU-bound: Very fast
Typical: 10,000+ records/sec
Limiting: Number of rules, field complexity
```

**Loading:**
```
Network I/O + Base.vn API
Typical: 10-100 records/sec (per rate limit)
Limiting: Target API rate limit
Batch Size: Optimize for Base.vn rate limit
```

**Full Pipeline Example:**
```
Scenario: Import 10,000 employee records from CSV to Base.vn
Extract:     10s (1,000 rec/sec)
Transform:   0.1s (100,000 rec/sec)
Load:        100s (100 rec/sec with rate limit)
Total:       ~110 seconds
```

### Resource Usage

**CPU:**
- Extract: Low (I/O bound)
- Transform: Medium (field operations)
- Load: Low (I/O bound, async)

**Memory:**
- Baseline: 50-100 MB
- Per 1,000 records: ~1-10 MB (depending on record size)
- Peak usage: Batch size × average record size

**Disk:**
- Checkpoint file: ~1 KB
- Audit log: 50-200 bytes per record
- Total growth: ~1% of processed data volume

## Production Deployment Checklist

### Pre-Deployment
- [ ] Configuration validated with `migro validate --config job.yaml`
- [ ] Environment variables set securely (not in config)
- [ ] Credentials rotated / access verified
- [ ] Network connectivity tested to target API
- [ ] Source data validated (sample records tested)
- [ ] Backup of existing target data created
- [ ] Rollback plan documented

### During Deployment
- [ ] Monitor progress with real-time progress bars
- [ ] Watch audit logs for errors
- [ ] Check rate limit not being exceeded
- [ ] Verify record sample in target system

### Post-Deployment
- [ ] Record count verification
- [ ] Data quality spot checks
- [ ] Audit log review for errors
- [ ] Checkpoint cleanup (if successful)
- [ ] Performance metrics captured
- [ ] Lessons learned documented

## Disaster Recovery

### Scenarios & Recovery

**Scenario: Process Interrupted**
```
Solution: Checkpoint-based resume
./migro migrate --config job.yaml --resume
→ Starts from last successful checkpoint
```

**Scenario: Partial Load Failure**
```
Solution: Retry with filtering
./migro migrate --config job.yaml --skip-ids 123,456
→ Skips already-loaded records
```

**Scenario: Bad Data Detected**
```
Solution: Review audit log, fix, re-run
grep '"type":"load_error"' audit.jsonl
→ Fix data in source
→ Re-run with --resume
```

**Scenario: Rate Limit Exceeded**
```
Solution: Reduce rate limit and retry
Adjust config: rate_limit: 5  # from 10
./migro migrate --config job.yaml --resume
```

### Backup Strategy

**Before Large Migration:**
```bash
# Backup target data (Base.vn specific)
# Typically done via Base.vn backup mechanism

# Keep source data available for re-migration
cp employees.csv employees.csv.backup
```

**Checkpoint Files:**
```bash
# Automatically created by basevn-migro
checkpoint-employees.json  # Automatically preserved

# For recovery:
./migro migrate --config job.yaml --checkpoint checkpoint-employees.json
```

## Security Architecture

### Credential Management

**Rules:**
```
1. Never store credentials in config files
2. Use environment variables only
3. Access tokens cleared after migration
4. Audit logs sanitized (no token logging)
```

**Implementation:**
```bash
# Set before running
export BASEVN_ACCESS_TOKEN="your-actual-token"

# Use in config
auth:
  token: "${BASEVN_ACCESS_TOKEN}"

# Token loaded at runtime, never persisted
```

### Audit Trail

```
All operations logged to JSONL file:
✓ User/job identification
✓ Timestamp of every action
✓ Success/failure status
✓ Error messages (sanitized)
✓ Record counts
✗ Credentials (never logged)
✗ Personal data (filtered by field_filter)
```

### Data Privacy

**Field Filtering:**
```yaml
transform:
  - type: field_filter
    mode: remove
    fields: ["password", "ssn", "credit_card"]
```

**Field Mapping:**
```yaml
- type: field_mapper
  fields:
    - source: "email"
      target: "contact_email"  # Rename for privacy
```

## Capacity Planning

### For Different Load Sizes

| Records | Time (CSV) | Storage | Memory | Notes |
|---------|-----------|---------|--------|-------|
| 1,000 | <1 min | <10 MB | <200 MB | Single machine |
| 10,000 | ~2 min | 100 MB | 500 MB | Single machine |
| 100,000 | ~20 min | 1 GB | 1-2 GB | Single machine |
| 1,000,000 | 3-5 hrs | 10 GB | 2-4 GB | Consider partitioning |
| 10,000,000 | 30+ hrs | 100 GB | Partition | Use batching strategy |

### Recommended Configuration by Scale

**Small (< 10K records):**
```yaml
batch_size: 50
rate_limit: 5
checkpoint: enabled  # For safety
```

**Medium (10K - 100K records):**
```yaml
batch_size: 100
rate_limit: 10
checkpoint: enabled
audit_log: enabled  # Full audit trail
```

**Large (> 100K records):**
```yaml
batch_size: 200
rate_limit: 5-10
checkpoint: enabled
# Consider partitioning
```

---

**Version**: 1.0
**Status**: Production-Ready
**Last Updated**: December 2024
**For Implementation Details**: See [architecture.md](architecture.md)
