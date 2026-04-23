## [unreleased]

### 🚀 Features

- Complete Rust rewrite
- Shell `completions` subcommand, `--generate-man-page` hidden flag
- Add --prefix filter to restrict substitution to matching variable names
- Add --env-file flag to load environment from .env files
- Add support for the POSIX `?` and `:?` error operators

### 💼 Security

- Snyk tool and security test task
- Deny unsafe code; write output atomically via tempfile

### ⚡ Performance

- Add bench task to mise.toml comparing ironsubst and envsubst