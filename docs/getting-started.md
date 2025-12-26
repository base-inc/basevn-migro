# Getting Started with basevn-migro

This guide will walk you through your first data migration using basevn-migro.

## Prerequisites

- Rust 1.75+ and Cargo (for building from source)
- Base.vn account with API access token
- Source data (CSV file, REST API, or Excel file)

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/basevn/basevn-migro.git
cd basevn-migro

# Build the release binary
cargo build --release

# The binary will be at: target/release/migro
```

### Verify Installation

```bash
./target/release/migro --version
```

## Your First Migration

Let's migrate employee data from a CSV file to Base.vn.

### Step 1: Prepare Your Data

Create a CSV file `employees.csv`:

```csv
Full Name,Email,Department,Position
John Doe,john.doe@company.com,Engineering,Senior Engineer
Jane Smith,jane.smith@company.com,Marketing,Marketing Manager
Bob Johnson,bob.johnson@company.com,Engineering,Junior Developer
```

### Step 2: Get Your Base.vn Access Token

1. Log in to your Base.vn account
2. Navigate to **Settings** → **API Tokens**
3. Create a new token with permissions for your target app/entity
4. Copy the token

### Step 3: Create Configuration File

Create `job.yaml`:

```yaml
version: "1"

job:
  id: "import-employees"
  name: "Import employees from CSV"

source:
  type: csv
  path: "./employees.csv"
  options:
    delimiter: ","
    has_headers: true

transform:
  - type: field_mapper
    fields:
      - source: "Full Name"
        target: "full_name"
        required: true
      - source: "Email"
        target: "email"
        required: true
      - source: "Department"
        target: "department"
        required: false
      - source: "Position"
        target: "position"
        required: false

target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "hrm"           # Your Base.vn app name
  entity: "employees"   # Your entity name
  auth:
    type: access_token
    token: "${BASEVN_ACCESS_TOKEN}"
  options:
    batch_size: 100
    rate_limit: 10

operations:
  sync_mode:
    mode: full
  checkpoint:
    enabled: true
    path: "./checkpoint.json"
  audit_log:
    enabled: true
    path: "./audit.jsonl"
  progress:
    enabled: true
```

### Step 4: Set Environment Variables

```bash
export BASEVN_ACCESS_TOKEN="your-actual-token-here"
```

### Step 5: Validate Configuration

Before running the migration, validate your configuration:

```bash
./target/release/migro validate --config job.yaml
```

You should see:
```
✓ Configuration valid
✓ Source file exists: ./employees.csv
✓ Field mappings valid
✓ Target configuration valid
```

### Step 6: Run the Migration

```bash
./target/release/migro migrate --config job.yaml
```

You'll see real-time progress:

```
⠋ [00:00:05] [████████████████████████████] 3/3 (00:00:00) | Success: 100.0% | Failed: 0
Complete! Success: 3, Failed: 0, Total: 3
```

### Step 7: Verify Results

Check your Base.vn app to verify the data was imported successfully.

Review the audit log:

```bash
cat audit.jsonl | jq '.'
```

Output:
```json
{"type":"job_start","job_id":"import-employees","timestamp":"2025-12-26T23:52:00Z","source_type":"csv","target_type":"basevn"}
{"type":"record_success","job_id":"import-employees","timestamp":"2025-12-26T23:52:01Z","record_index":0,"record_id":"rec_001"}
{"type":"record_success","job_id":"import-employees","timestamp":"2025-12-26T23:52:01Z","record_index":1,"record_id":"rec_002"}
{"type":"record_success","job_id":"import-employees","timestamp":"2025-12-26T23:52:01Z","record_index":2,"record_id":"rec_003"}
{"type":"job_complete","job_id":"import-employees","timestamp":"2025-12-26T23:52:05Z","total_processed":3,"total_success":3,"total_failed":0,"duration_secs":5}
```

## Next Steps

### Incremental Sync

For subsequent runs, use incremental sync to update only changed records:

```yaml
operations:
  sync_mode:
    mode: incremental
    key_field: "email"  # Match records by email
    conflict_strategy: update  # Update existing records
```

### Field Filtering

Remove sensitive fields before loading:

```yaml
transform:
  - type: field_filter
    mode: remove
    fields: ["password", "ssn", "salary"]
```

### REST API Source

Migrate data from a REST API:

```yaml
source:
  type: rest_api
  url: "https://api.example.com/users"
  method: "GET"
  headers:
    Authorization: "Bearer ${API_TOKEN}"
  pagination:
    type: "offset"
    limit: 100
  response_path: "data"
```

### Resume on Failure

If a migration is interrupted, simply run the same command again. The checkpoint will resume from where it left off:

```bash
./target/release/migro migrate --config job.yaml
```

Output:
```
✓ Found checkpoint at offset 1500
⠋ Resuming from record 1500...
```

## Advanced Features

### Multiple Transformers

Chain multiple transformers:

```yaml
transform:
  - type: field_mapper
    fields: [...]

  - type: field_filter
    mode: keep
    fields: ["id", "name", "email"]
```

### Batch Size Tuning

Optimize batch sizes for your data:

```yaml
target:
  options:
    batch_size: 500  # Larger batches for faster throughput
```

### Rate Limiting

Control request rate to avoid overwhelming APIs:

```yaml
target:
  options:
    rate_limit: 5  # 5 requests per second
```

### Custom Retry Logic

Configure retry behavior:

```yaml
target:
  options:
    max_retries: 5
    timeout: 60
```

## Common Patterns

### Pattern 1: CSV to Base.vn with Validation

```yaml
transform:
  - type: field_mapper
    fields:
      - source: "email"
        target: "email"
        required: true  # Fail if email is missing
```

### Pattern 2: API to Base.vn with Rate Limiting

```yaml
source:
  type: rest_api
  # ... source config

target:
  options:
    rate_limit: 2  # Slow down for rate-limited APIs
    max_retries: 5
```

### Pattern 3: Full Sync with Backup

```yaml
operations:
  sync_mode:
    mode: full  # Delete all + reload
  audit_log:
    enabled: true
    path: "./audit-backup.jsonl"  # Keep audit trail
```

## Troubleshooting

See [troubleshooting.md](troubleshooting.md) for common issues and solutions.

## Further Reading

- [Configuration Reference](configuration.md)
- [Architecture Overview](architecture.md)
- [Examples Directory](../examples/README.md)
