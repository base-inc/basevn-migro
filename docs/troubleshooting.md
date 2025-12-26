# Troubleshooting Guide

Common issues and solutions for basevn-migro.

## Configuration Errors

### "File not found" Error

**Problem:**
```
Error: Failed to load config: File not found: job.yaml
```

**Solutions:**
1. Check current directory:
   ```bash
   pwd  # Make sure you're in the right directory
   ls job.yaml  # Verify file exists
   ```

2. Use absolute path:
   ```yaml
   source:
     path: "/home/user/data/employees.csv"
   ```

3. Check file permissions:
   ```bash
   ls -la job.yaml
   chmod 644 job.yaml
   ```

### "Invalid YAML" Error

**Problem:**
```
Error: Invalid YAML config: expected mapping
```

**Solutions:**
1. Check indentation (use spaces, not tabs):
   ```yaml
   source:     # 2 spaces
     type: csv # 4 spaces
   ```

2. Validate YAML syntax:
   ```bash
   # Using yq or yamllint
   yamllint job.yaml
   ```

3. Check for special characters:
   ```yaml
   # Quote strings with special chars
   password: "p@ssw0rd!"
   ```

## Source Errors

### CSV "No such file" Error

**Problem:**
```
Error: Failed to read source file: employees.csv
```

**Solutions:**
1. Verify file path:
   ```bash
   ls -la employees.csv
   ```

2. Use absolute path:
   ```yaml
   source:
     path: "/absolute/path/to/employees.csv"
   ```

3. Check file encoding:
   ```bash
   file employees.csv
   # Should be: ASCII text or UTF-8 Unicode text
   ```

### REST API Connection Error

**Problem:**
```
Error: API request failed: connection refused
```

**Solutions:**
1. Test API endpoint:
   ```bash
   curl -I https://api.example.com/data
   ```

2. Check network/firewall:
   ```bash
   ping api.example.com
   telnet api.example.com 443
   ```

3. Verify SSL certificates:
   ```bash
   curl -v https://api.example.com/data
   ```

### API Authentication Error

**Problem:**
```
Error: Authentication failed: 401 Unauthorized
```

**Solutions:**
1. Verify token is set:
   ```bash
   echo $BASEVN_ACCESS_TOKEN
   ```

2. Check token format:
   ```yaml
   auth:
     token: "${BASEVN_ACCESS_TOKEN}"  # Correct
     # NOT: token: "BASEVN_ACCESS_TOKEN"
   ```

3. Test token manually:
   ```bash
   curl -H "Authorization: Bearer $BASEVN_ACCESS_TOKEN" \
        https://api.base.vn/api/your-app/your-entity
   ```

## Transform Errors

### "Missing required field" Error

**Problem:**
```
Error: Missing required field: email
```

**Solutions:**
1. Check source data has the field:
   ```bash
   head -1 employees.csv  # Check headers
   ```

2. Verify field mapping:
   ```yaml
   fields:
     - source: "Email"  # Must match CSV header exactly
       target: "email"
       required: true
   ```

3. Add default value:
   ```yaml
   fields:
     - source: "Email"
       target: "email"
       required: false
       default: "unknown@company.com"
   ```

### Field Mapping Not Working

**Problem:**
```
Target field is empty after mapping
```

**Solutions:**
1. Check field name case sensitivity:
   ```yaml
   source: "Full Name"  # Exact match required
   ```

2. Verify source data:
   ```bash
   cat employees.csv | head -5
   ```

3. Test with minimal config:
   ```yaml
   transform:
     - type: field_mapper
       fields:
         - source: "name"
           target: "name"  # 1:1 mapping to test
   ```

## Load Errors

### Rate Limit Exceeded

**Problem:**
```
Error: Rate limit exceeded (429 Too Many Requests)
```

**Solutions:**
1. Reduce rate limit:
   ```yaml
   options:
     rate_limit: 5  # Lower from 10 to 5 req/s
   ```

2. Enable adaptive rate limiting (already enabled):
   ```yaml
   options:
     max_retries: 5  # More retries
   ```

3. Increase batch size (fewer requests):
   ```yaml
   options:
     batch_size: 200  # Up from 100
   ```

### Batch Upload Failed

**Problem:**
```
Error: Batch load attempt 3 failed
```

**Solutions:**
1. Reduce batch size:
   ```yaml
   options:
     batch_size: 50  # Smaller batches
   ```

2. Increase timeout:
   ```yaml
   options:
     timeout: 60  # Up from 30 seconds
   ```

3. Check audit log for details:
   ```bash
   cat audit.jsonl | jq 'select(.type == "record_failure")'
   ```

## Resume & Checkpoint Errors

### Checkpoint Corrupted

**Problem:**
```
Error: Failed to load checkpoint: invalid JSON
```

**Solutions:**
1. Delete checkpoint and restart:
   ```bash
   rm checkpoint.json
   migro migrate --config job.yaml
   ```

2. Manually edit checkpoint:
   ```bash
   cat checkpoint.json | jq '.'
   # Fix JSON syntax errors
   ```

3. Backup before deleting:
   ```bash
   mv checkpoint.json checkpoint.json.bak
   ```

### Resume from Wrong Offset

**Problem:**
```
Checkpoint shows offset 5000, but only 100 records exist
```

**Solutions:**
1. Delete checkpoint:
   ```bash
   rm checkpoint.json
   ```

2. Check source data didn't change:
   ```bash
   wc -l employees.csv  # Count lines
   ```

## Performance Issues

### Slow Extraction

**Problem:**
Migration taking too long to extract data.

**Solutions:**
1. Check file size:
   ```bash
   du -h employees.csv
   ```

2. For large files, use streaming (already enabled in CSV extractor)

3. For REST API, increase pagination limit:
   ```yaml
   pagination:
     limit: 500  # Up from 100
   ```

### Slow Loading

**Problem:**
Upload to Base.vn is slow.

**Solutions:**
1. Increase batch size:
   ```yaml
   options:
     batch_size: 500  # Up from 100
   ```

2. Increase rate limit (if API allows):
   ```yaml
   options:
     rate_limit: 20  # Up from 10
   ```

3. Check network latency:
   ```bash
   ping api.base.vn
   traceroute api.base.vn
   ```

## Common Patterns

### Debugging Failed Records

```bash
# View all failures
cat audit.jsonl | jq 'select(.type == "record_failure")'

# Count failures by error type
cat audit.jsonl | jq -r 'select(.type == "record_failure") | .error' | sort | uniq -c

# Get failed record IDs
cat audit.jsonl | jq -r 'select(.type == "record_failure") | .record_id' > failed_ids.txt
```

### Testing Configuration

```bash
# Validate before running
migro validate --config job.yaml

# Dry run (if supported)
# Phase 6 feature - not yet implemented
```

### Monitoring Progress

```bash
# Watch checkpoint file
watch -n 1 cat checkpoint.json

# Monitor audit log
tail -f audit.jsonl | jq '.'
```

## Getting Help

1. **Check logs**: Review audit.jsonl for detailed error messages
2. **Enable debug logging**: Set `RUST_LOG=debug`
3. **Search issues**: [GitHub Issues](https://github.com/basevn/basevn-migro/issues)
4. **Report bug**: Include config, error message, and audit log excerpt
