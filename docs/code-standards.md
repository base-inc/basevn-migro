# basevn-migro: Code Standards & Guidelines

**Version**: 1.0
**Last Updated**: December 2024
**Status**: Production-Ready
**Enforcement**: Pre-commit checks required

## Overview

This document establishes the coding standards and best practices for basevn-migro development. All code must comply with these standards before being accepted into the codebase.

**Current Status**: 100% Compliant
- 0 clippy warnings
- 100% rustfmt compliant
- 81/81 tests passing

## Rust Coding Conventions

### Formatting

All code must be formatted according to `rustfmt.toml` configuration.

**Configuration** (`rustfmt.toml`):
```toml
edition = "2021"
max_width = 100           # Maximum line width
tab_spaces = 4            # Indentation size
use_small_heuristics = "Default"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

**Enforcement**:
```bash
# Check formatting before committing
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all
```

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Modules | snake_case | `extract`, `field_mapper` |
| Types/Structs | PascalCase | `RecordStream`, `ExtractError` |
| Traits | PascalCase | `Extractor`, `Transformer` |
| Functions | snake_case | `extract_csv`, `map_field` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRIES`, `DEFAULT_BATCH_SIZE` |
| Variables | snake_case | `batch_size`, `error_count` |
| Lifetimes | lowercase | `'a`, `'s` |
| Type Parameters | UPPERCASE | `T`, `E` |

**Examples**:
```rust
// Good: Consistent with conventions
pub struct CsvExtractor { }
pub async fn extract_records() -> Result<RecordStream, ExtractError> { }
const DEFAULT_DELIMITER: char = ',';
let batch_size = 100;

// Bad: Inconsistent conventions
pub struct csv_extractor { }  // Should be PascalCase
pub async fn ExtractRecords() { }  // Should be snake_case
const default_delimiter: char = ',';  // Should be SCREAMING_SNAKE_CASE
```

### Line Length

**Maximum**: 100 characters (enforced by rustfmt.toml)

**Reason**:
- Fits on standard editor splits
- Improves readability
- Matches typical terminal widths

**Exception**: Long strings (URLs, long error messages) can exceed 100 characters.

```rust
// Good: Broken into multiple lines
pub async fn load_batch(
    &self,
    records: Vec<Record>,
    config: &TargetConfig,
) -> Result<LoadResult, LoadError> { }

// Bad: Too long on one line
pub async fn load_batch(&self, records: Vec<Record>, config: &TargetConfig) -> Result<LoadResult, LoadError> { }
```

### Imports Organization

**Order**: Standard library → External crates → Internal crates

```rust
// Good: Organized imports
use std::fs;
use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::{Record, Value};
use crate::error::ExtractError;

// Bad: Mixed order
use crate::error::ExtractError;
use serde::{Deserialize, Serialize};
use std::fs;
```

### Visibility Modifiers

**Rules**:
- Mark public API with `pub`
- Mark internal items as private (default)
- Use `pub(crate)` for internal-only exports
- Document why items are public

```rust
// Good: Clear visibility
pub struct Record { /* ... */ }  // Public API
pub(crate) fn validate_config() { }  // Internal only
fn process_record() { }  // Private helper

// Bad: Missing visibility
struct Record { }  // Should be pub
```

## Error Handling Patterns

### Structured Error Types

All errors must use the thiserror crate for structured error types.

**Pattern**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid CSV format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Error Context

Always provide context with `anyhow::Context`:

```rust
// Good: With context
let config = load_config(&path)
    .context("Failed to load configuration")?;

let records = extract_csv(&path)
    .context("Failed to extract CSV records")?;

// Bad: No context
let config = load_config(&path)?;
let records = extract_csv(&path)?;
```

### Error Propagation

**Use `?` operator** for propagating errors up the call stack:

```rust
// Good: Using ?
pub fn process() -> Result<Output, Error> {
    let config = load_config()?;
    let data = extract_data(config)?;
    Ok(process_data(data))
}

// Bad: Using unwrap (panics on error)
pub fn process() -> Result<Output, Error> {
    let config = load_config().unwrap();
    let data = extract_data(config).unwrap();
    Ok(process_data(data))
}

// Bad: Using expect (still panics)
pub fn process() -> Result<Output, Error> {
    let config = load_config().expect("config failed");
    // ...
}
```

### Handling Errors in Production Code

**Never panic in production code**:

```rust
// Good: Handle errors gracefully
match parse_number(value) {
    Ok(n) => process(n),
    Err(e) => log_error(e),
}

// Bad: Panics in production
let n = parse_number(value).unwrap();

// Bad: Using unwrap_or with wrong default
let n = parse_number(value).unwrap_or(0);  // Silent default is bad
```

### Recoverable vs Unrecoverable

**Recoverable** (use Result):
- File not found
- Invalid configuration
- Network timeout
- Rate limit exceeded

**Unrecoverable** (panic is acceptable):
- Internal logic error in tests
- Violated invariant (should be prevented at compile time)
- Truly exceptional condition that requires process halt

```rust
// Good: Unrecoverable error
fn extract_internal_id() -> u64 {
    // This should never happen if our code is correct
    debug_assert!(self.id > 0, "ID must be positive");
    self.id
}

// Good: Recoverable error
pub fn extract_config(path: &Path) -> Result<Config, Error> {
    std::fs::read_to_string(path)
        .context("Failed to read config file")?
}
```

## Documentation Standards

### Public API Documentation

All public functions, structs, and modules must have documentation comments.

**Format**: Use `///` for documentation comments (not `//`).

```rust
/// Extracts records from a CSV file.
///
/// This function reads a CSV file and yields records as a stream.
/// It validates headers and handles various encoding formats.
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
/// Returns `ExtractError::FileNotFound` if the file doesn't exist.
/// Returns `ExtractError::InvalidFormat` if CSV format is invalid.
///
/// # Examples
///
/// ```no_run
/// use basevn_migro::extract::CsvExtractor;
///
/// let extractor = CsvExtractor::new();
/// let records = extractor.extract_csv("data.csv", Default::default())?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub async fn extract_csv(
    path: &Path,
    options: &CsvOptions,
) -> Result<RecordStream, ExtractError> {
    // implementation
}
```

### Documentation Sections

For complex items, include these sections:

1. **Summary** (1 line): What does it do?
2. **Details** (optional): How does it work?
3. **Arguments** (if applicable): Parameter descriptions
4. **Returns** (if applicable): Return value description
5. **Errors** (if applicable): Error conditions
6. **Examples** (optional): Usage examples
7. **Panics** (if applicable): When it might panic

### Code Comments

Use regular comments (`//`) for implementation notes.

```rust
// Extract only non-empty records
let records: Vec<_> = stream
    .filter(|r| !r.is_empty())
    .collect();

// Batch records for efficient loading
for batch in records.chunks(batch_size) {
    loader.load_batch(batch)?;
}
```

### Module Documentation

Document module purpose at the top of `mod.rs`:

```rust
//! CSV extraction module.
//!
//! This module provides CSV file extraction with support for:
//! - Custom delimiters
//! - Header detection
//! - Multiple encodings
//! - Line number tracking for error reporting
//!
//! # Examples
//!
//! ```no_run
//! use basevn_migro::extract::CsvExtractor;
//!
//! let extractor = CsvExtractor::new();
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

pub mod csv;
pub mod rest_api;
pub mod traits;
pub mod registry;
```

## Testing Standards

### Test Location

Place unit tests in the same file as the code being tested using `#[cfg(test)]` modules:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative_numbers() {
        assert_eq!(add(-2, -3), -5);
    }
}
```

### Test Naming

Name tests clearly with `test_<function>_<scenario>` pattern:

```rust
#[test]
fn test_field_mapping_basic() { }

#[test]
fn test_field_mapping_with_defaults() { }

#[test]
fn test_field_mapping_missing_required_field() { }

#[test]
fn test_csv_extraction_with_custom_delimiter() { }
```

### Test Organization

Group related tests in test modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod field_mapping {
        use super::*;

        #[test]
        fn test_basic() { }

        #[test]
        fn test_with_defaults() { }
    }

    mod validation {
        use super::*;

        #[test]
        fn test_required_field() { }
    }
}
```

### Test Coverage Requirements

- **Core logic**: Minimum 70% coverage
- **Happy path**: Always test successful case first
- **Error cases**: Test each error condition
- **Edge cases**: Empty inputs, nulls, very large values
- **Integration**: Test layer interactions

### Testing Async Code

Use `#[tokio::test]` for async tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_extract() {
        let result = extract_csv(path).await;
        assert!(result.is_ok());
    }
}
```

### Mocking

Use `mockall` crate for mocking traits:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_with_mock_loader() {
        let mut mock = MockLoader::new();
        mock.expect_load_batch()
            .with(always())
            .times(1)
            .returning(|_| Ok(LoadResult::default()));

        // Test using mock
    }
}
```

## Commit Message Conventions

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/) format.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

| Type | Purpose | Example |
|------|---------|---------|
| feat | New feature | `feat(extract): add PostgreSQL extractor` |
| fix | Bug fix | `fix(load): handle 429 rate limit response` |
| docs | Documentation | `docs: update README with examples` |
| style | Code style (fmt, lint) | `style: reformat with rustfmt` |
| refactor | Code restructuring | `refactor(transform): extract field mapper` |
| perf | Performance improvement | `perf(load): optimize batch processing` |
| test | Tests | `test(extract): add CSV edge case tests` |
| chore | Build/tooling | `chore: upgrade tokio dependency` |

### Scope

Use scope to indicate which module is affected:

- `extract`: Extract layer changes
- `transform`: Transform layer changes
- `load`: Load layer changes
- `config`: Configuration changes
- `cli`: CLI interface changes
- `operations`: Progress, audit, checkpoint changes
- `core`: Core data types changes
- `error`: Error handling changes

### Subject

- Imperative mood ("add" not "added" or "adds")
- Don't capitalize first letter
- No period at the end
- Maximum 50 characters

### Body

- Explain what and why, not how
- Wrap at 72 characters
- Separate from subject with blank line

### Footer

Reference related issues:

```
feat(load): add retry exponential backoff

Implement exponential backoff for transient failures:
- Start with 1 second
- Double on each retry
- Maximum 8 seconds
- Configurable max attempts

Closes #42
Related: #41
```

## Code Review Checklist

Before submitting a pull request, verify:

### Functionality
- [ ] Code solves the stated problem
- [ ] All acceptance tests pass
- [ ] Error cases handled appropriately
- [ ] No panics in production code

### Code Quality
- [ ] Passes `cargo fmt --all -- --check`
- [ ] Passes `cargo clippy -- -D warnings`
- [ ] Follows naming conventions
- [ ] Appropriate error types used

### Testing
- [ ] New functionality has tests
- [ ] All 81 tests pass: `cargo test`
- [ ] Tests are meaningful and thorough
- [ ] No commented-out tests

### Documentation
- [ ] Public API documented with `///`
- [ ] Complex logic explained with comments
- [ ] README updated if needed
- [ ] CONTRIBUTING.md updated if needed

### Performance
- [ ] No unnecessary allocations
- [ ] No unnecessary cloning
- [ ] Appropriate data structures used
- [ ] No O(n²) algorithms for n > 1000

### Security
- [ ] No unwrap/expect in production code
- [ ] Credentials not hardcoded
- [ ] No SQL injection vulnerabilities
- [ ] Input validation applied

## Common Patterns

### Error Handling Pattern

```rust
pub fn process_file(path: &Path) -> Result<Output, Error> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read file")?;

    let parsed = parse(&content)
        .context("Failed to parse content")?;

    validate(&parsed)
        .context("Validation failed")?;

    Ok(processed)
}
```

### Registry Pattern

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
}

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }
}
```

### Builder Pattern

```rust
pub struct Config {
    batch_size: usize,
    timeout: Duration,
    retry_count: u32,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            batch_size: 100,
            timeout: Duration::from_secs(30),
            retry_count: 3,
        }
    }

    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn build(self) -> Config {
        Config { /* ... */ }
    }
}
```

## Linting Rules (clippy.toml)

**MSRV**: 1.75 (Minimum Supported Rust Version)

**Configuration** (`clippy.toml`):
```toml
msrv = "1.75"
```

**Enforced**:
```bash
cargo clippy -- -D warnings
```

**Common Warnings to Fix**:

| Warning | Solution |
|---------|----------|
| `needless_borrow` | Remove unnecessary `&` |
| `clone_on_copy` | Use copy instead of clone |
| `single_match` | Use if-let for single match arms |
| `match_like_matches_macro` | Use matches! macro |
| `manual_ok_or` | Use ok_or instead of manual |

## Dependency Management

### Adding Dependencies

**Rules**:
1. Prefer existing dependencies
2. Prefer popular, well-maintained crates
3. Check license compatibility (Apache 2.0)
4. Minimize transitive dependencies
5. Keep dependencies up to date

**Process**:
```bash
# Add dependency
cargo add dependency_name

# Update all
cargo update

# Check for vulnerabilities
cargo audit
```

### Pinning Versions

Use semantic versioning:

```toml
# Latest patch version only
dependency = "1.0"

# Latest minor version
dependency = "1.*"

# Specific version (for build tools)
build-tool = "2.5.0"
```

## Performance Guidelines

### Memory
- Process records in batches, not all-at-once
- Use references where possible
- Avoid unnecessary cloning
- Use `Vec::with_capacity` for known sizes

### Time
- O(1) registry lookups (HashMap)
- O(n) extraction and loading (linear pass)
- O(m) transformation (field count)
- Stream processing, not batch loading

### I/O
- Buffer reads/writes
- Batch API requests
- Use connection pooling (Phase 7)
- Compression for large transfers

## Security Best Practices

### Credentials
- Never hardcode credentials
- Use environment variables
- Clear sensitive data from memory
- Log only non-sensitive errors

### Input Validation
- Validate all user input
- Sanitize file paths
- Check field existence before access
- Limit string lengths

### Error Messages
- Don't expose system paths in user messages
- Don't leak sensitive data in logs
- Provide helpful context for debugging

## Anti-Patterns to Avoid

| Anti-Pattern | Why | Better |
|--------------|-----|--------|
| `unwrap()` in production | Panics on error | Use `?` operator |
| `clone()` everywhere | Inefficient memory | Use references |
| Very long functions | Hard to understand | Break into smaller functions |
| Global state | Hard to test | Pass parameters |
| Too many parameters | Readability issues | Use struct builders |
| Comments explaining "what" | Obvious from code | Comment "why" |
| Deeply nested code | Hard to follow | Extract to functions |

---

**Effective Date**: December 2024
**Last Updated**: December 2024
**Review Cycle**: Annual or after major release
