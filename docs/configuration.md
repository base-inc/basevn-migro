# Configuration Reference

Complete reference for basevn-migro configuration files.

## File Format

Configuration files use YAML or JSON format:

```yaml
version: "1"  # Required: config version
job: {...}    # Required: job metadata
source: {...} # Required: data source
transform: [...] # Optional: transformations
target: {...} # Required: load target
operations: {...} # Optional: operational settings
```

## Job Configuration

```yaml
job:
  id: "unique-job-id"      # Required: unique identifier
  name: "Job description"  # Required: human-readable name
```

## Source Configuration

### CSV Source

```yaml
source:
  type: csv
  path: "./data.csv"  # Required: file path
  options:
    delimiter: ","       # Default: ","
    has_headers: true    # Default: true
    encoding: "utf-8"    # Default: "utf-8"
    skip_rows: 0         # Default: 0
```

### REST API Source

```yaml
source:
  type: rest_api
  url: "https://api.example.com/data"  # Required
  method: "GET"  # Default: "GET"
  headers:       # Optional
    Authorization: "Bearer ${TOKEN}"
    Content-Type: "application/json"
  query_params:  # Optional
    limit: "100"
    active: "true"
  pagination:
    type: "offset"  # "offset", "page", "cursor", or "none"
    limit: 100      # Records per page
    offset_param: "offset"  # For offset pagination
    limit_param: "limit"
    # OR for cursor pagination:
    cursor_param: "cursor"
    cursor_path: "pagination.next_cursor"
  response_path: "data"  # JSONPath to records array
```

### Excel Source (Placeholder)

```yaml
source:
  type: excel
  path: "./data.xlsx"
  # Note: Convert to CSV for MVP
```

## Transform Configuration

### Field Mapper

```yaml
transform:
  - type: field_mapper
    fields:
      - source: "source_field"
        target: "target_field"
        required: true     # Fail if missing
        default: "value"   # Use if source is empty/null
```

### Field Filter

```yaml
transform:
  - type: field_filter
    mode: remove  # "remove" or "keep"
    fields:
      - "sensitive_field1"
      - "sensitive_field2"
```

## Target Configuration

### Base.vn Target

```yaml
target:
  type: basevn
  base_url: "https://api.base.vn"  # Required
  app: "app_name"                   # Required
  entity: "entity_name"             # Required
  auth:
    type: access_token  # Required
    token: "${BASEVN_ACCESS_TOKEN}"  # Required
  options:
    batch_size: 100    # Default: 100
    rate_limit: 10     # Requests/second, default: 10
    timeout: 30        # Seconds, default: 30
    max_retries: 3     # Default: 3
```

## Operations Configuration

```yaml
operations:
  sync_mode:
    mode: incremental  # "full" or "incremental"
    key_field: "email" # Required for incremental
    conflict_strategy: update  # "skip", "update", or "error"

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

Use `${VAR_NAME}` syntax for environment variable substitution:

```yaml
auth:
  token: "${BASEVN_ACCESS_TOKEN}"

headers:
  Authorization: "Bearer ${API_TOKEN}"
```

Set before running:

```bash
export BASEVN_ACCESS_TOKEN="your-token"
export API_TOKEN="api-token"
```

## Validation Rules

Configuration must satisfy:

- `version` must be "1"
- `job.id` must be unique, non-empty
- `source.type` must be "csv", "rest_api", or "excel"
- `target.type` must be "basevn"
- For incremental sync: `sync_mode.key_field` required
- All file paths must exist (for csv/excel sources)
- Batch size > 0
- Rate limit > 0

## Complete Example

```yaml
version: "1"

job:
  id: "import-customers"
  name: "Import customers from API to Base.vn"

source:
  type: rest_api
  url: "https://api.example.com/customers"
  method: "GET"
  headers:
    Authorization: "Bearer ${API_TOKEN}"
  pagination:
    type: "cursor"
    limit: 100
    cursor_param: "cursor"
    cursor_path: "pagination.next_cursor"
  response_path: "data"

transform:
  - type: field_mapper
    fields:
      - source: "customer_name"
        target: "name"
        required: true
      - source: "customer_email"
        target: "email"
        required: true
      - source: "signup_date"
        target: "created_at"
        required: false

  - type: field_filter
    mode: remove
    fields: ["internal_id", "password_hash"]

target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "crm"
  entity: "customers"
  auth:
    type: access_token
    token: "${BASEVN_ACCESS_TOKEN}"
  options:
    batch_size: 200
    rate_limit: 5
    timeout: 45
    max_retries: 5

operations:
  sync_mode:
    mode: incremental
    key_field: "email"
    conflict_strategy: update
  checkpoint:
    enabled: true
    path: "./checkpoint-customers.json"
  audit_log:
    enabled: true
    path: "./audit-customers.jsonl"
  progress:
    enabled: true
```
