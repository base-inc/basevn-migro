# Contributing to basevn-migro

Thank you for considering contributing to basevn-migro! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, professional, and constructive in all interactions.

## How to Contribute

### Reporting Bugs

1. Check [existing issues](https://github.com/basevn/basevn-migro/issues) to avoid duplicates
2. Use the bug report template
3. Include:
   - Clear description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)
   - Relevant logs or error messages

### Suggesting Features

1. Check existing feature requests
2. Use the feature request template
3. Clearly describe:
   - The problem or use case
   - Proposed solution
   - Alternatives considered

### Pull Requests

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR-USERNAME/basevn-migro.git
   cd basevn-migro
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b bugfix/issue-description
   ```

3. **Make Changes**
   - Follow coding conventions (see below)
   - Add tests for new functionality
   - Update documentation if needed

4. **Run Tests**
   ```bash
   cargo fmt --all -- --check
   cargo clippy -- -D warnings
   cargo test
   ```

5. **Commit Changes**
   ```bash
   git commit -m "feat(extract): add Excel extractor"
   ```

   Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat`: New feature
   - `fix`: Bug fix
   - `docs`: Documentation
   - `style`: Code style (formatting)
   - `refactor`: Code restructuring
   - `perf`: Performance improvement
   - `test`: Tests
   - `chore`: Build/tooling

6. **Push and Create PR**
   ```bash
   git push origin your-branch-name
   ```

   Then create a pull request on GitHub.

## Coding Conventions

### Rust Style

- **Format**: Run `cargo fmt` before committing
- **Linting**: Fix all `cargo clippy` warnings
- **Max line width**: 100 characters
- **Naming**:
  - Modules: `snake_case`
  - Types/Structs: `PascalCase`
  - Functions: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`

### Error Handling

```rust
// ✅ Good: Explicit error context
let config = load_config(&path)
    .with_context(|| format!("Failed to load config from {}", path.display()))?;

// ❌ Bad: Panic in production code
let config = load_config(&path).unwrap();
```

### Documentation

All public APIs must have documentation:

```rust
/// Extracts records from a CSV file.
///
/// # Arguments
///
/// * `path` - Path to the CSV file
/// * `options` - Extraction options
///
/// # Returns
///
/// A stream of `Record` items or an `ExtractError`.
///
/// # Errors
///
/// Returns `ExtractError::FileReadError` if the file cannot be opened.
pub fn extract_csv(path: &Path, options: &CsvOptions) -> Result<RecordStream, ExtractError> {
    // implementation
}
```

### Testing

- **Unit tests**: In the same file as the code (`#[cfg(test)] mod tests`)
- **Integration tests**: In `tests/` directory
- **Coverage target**: 70%+ for core logic

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapping_basic() {
        let mapper = FieldMapper::new(vec![
            FieldMap { source: "name".into(), target: "full_name".into(), required: true },
        ]);

        let record = Record::from([("name", "John Doe")]);
        let result = mapper.transform(record).unwrap();

        assert_eq!(result.get("full_name"), Some(&Value::String("John Doe".into())));
    }
}
```

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git

### Setup

```bash
# Clone repository
git clone https://github.com/basevn/basevn-migro.git
cd basevn-migro

# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
basevn-migro/
├── src/
│   ├── cli/          # CLI interface
│   ├── core/         # Core types (Record, etc.)
│   ├── extract/      # Data extraction
│   ├── transform/    # Data transformation
│   ├── load/         # Data loading
│   ├── config/       # Configuration
│   ├── error/        # Error types
│   └── main.rs
├── tests/            # Integration tests
├── plans/            # Design documents
└── Cargo.toml
```

## Pull Request Checklist

Before submitting, ensure:

- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] All clippy warnings fixed (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] New functionality has tests
- [ ] Documentation updated if needed
- [ ] Commit messages follow conventional commits
- [ ] PR description explains the change
- [ ] Linked to related issue (if applicable)

## Getting Help

- Read [plans/plan.md](plans/plan.md) for architecture details
- Check existing [issues](https://github.com/basevn/basevn-migro/issues) and [discussions](https://github.com/basevn/basevn-migro/discussions)
- Ask questions in issues or discussions

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

---

Thank you for contributing to basevn-migro! 🎉
