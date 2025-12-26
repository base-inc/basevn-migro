# Examples

This directory contains example configurations and sample data for basevn-migro.

## Configuration Files

### `job-csv-to-basevn.yaml`
Import employee data from CSV file to Base.vn.

**Features demonstrated:**
- CSV extraction with headers
- Field mapping with defaults
- Field filtering to remove sensitive data
- Incremental sync with email as key field
- Checkpointing and audit logging

**Usage:**
```bash
# Set your Base.vn access token
export BASEVN_ACCESS_TOKEN="your-token-here"

# Run migration
migro migrate --config examples/job-csv-to-basevn.yaml
```

### `job-api-to-basevn.yaml`
Import user data from REST API (JSONPlaceholder) to Base.vn.

**Features demonstrated:**
- REST API extraction
- Nested field mapping (company.name)
- Field filtering with keep mode
- Full sync mode
- Progress tracking

**Usage:**
```bash
# Set your Base.vn access token
export BASEVN_ACCESS_TOKEN="your-token-here"

# Run migration
migro migrate --config examples/job-api-to-basevn.yaml
```

## Sample Data

### `employees.csv`
Sample employee data with 5 records.

**Fields:**
- Full Name
- Email (used as key for incremental sync)
- Department
- Position
- Start Date
- Internal ID (removed by field filter)
- Password Hash (removed by field filter)

## Configuration Reference

### Source Types

#### CSV
```yaml
source:
  type: csv
  path: "./data.csv"
  options:
    delimiter: ","
    has_headers: true
    encoding: "utf-8"
    skip_rows: 0
```

#### REST API
```yaml
source:
  type: rest_api
  url: "https://api.example.com/data"
  method: "GET"
  headers:
    Authorization: "Bearer ${API_TOKEN}"
  pagination:
    type: "offset"  # or "page" or "cursor"
    limit: 100
  response_path: "data"
```

### Transformers

#### Field Mapper
```yaml
transform:
  - type: field_mapper
    fields:
      - source: "src_field"
        target: "dest_field"
        required: true
        default: "default_value"
```

#### Field Filter
```yaml
transform:
  - type: field_filter
    mode: remove  # or "keep"
    fields: ["sensitive_field1", "sensitive_field2"]
```

### Target Configuration

```yaml
target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "your_app"
  entity: "your_entity"
  auth:
    type: access_token
    token: "${BASEVN_ACCESS_TOKEN}"
  options:
    batch_size: 100
    rate_limit: 10
    timeout: 30
    max_retries: 3
```

### Operations

```yaml
operations:
  sync_mode:
    mode: incremental  # or "full"
    key_field: "email"  # required for incremental
    conflict_strategy: update  # or "skip" or "error"
  checkpoint:
    enabled: true
    path: "./checkpoint.json"
  audit_log:
    enabled: true
    path: "./audit.jsonl"
  progress:
    enabled: true
```

## Environment Variables

All configuration files support environment variable substitution using `${VARIABLE_NAME}` syntax.

**Example:**
```yaml
auth:
  token: "${BASEVN_ACCESS_TOKEN}"
```

**Set before running:**
```bash
export BASEVN_ACCESS_TOKEN="your-actual-token"
```

## Testing

You can test configurations without actually loading data by validating them:

```bash
migro validate --config examples/job-csv-to-basevn.yaml
```

This will:
- Validate YAML syntax
- Check required fields
- Verify file paths exist
- Validate field mappings

## Troubleshooting

### Common Issues

**"File not found" error:**
```bash
# Make sure you're in the project root
cd /path/to/basevn-migro

# Or use absolute paths in config
source:
  path: "/absolute/path/to/data.csv"
```

**"Missing required field" error:**
```yaml
# Ensure all required fields are mapped
fields:
  - source: "source_field"
    target: "target_field"
    required: true  # This field must exist in source
```

**"Rate limit exceeded" error:**
```yaml
# Reduce rate limit or enable adaptive adjustment
options:
  rate_limit: 5  # Lower value (requests per second)
```

**Authentication error:**
```bash
# Verify your token is set
echo $BASEVN_ACCESS_TOKEN

# Check token has correct permissions
# Token should have write access to the target app/entity
```
