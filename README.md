# ironsubst

A blazing fast, deeply configurable environment variable substitution tool written in Rust.

`ironsubst` is a ground-up Rust rewrite of `a8m/envsubst` with 100% test parity and enhanced strictness configurations for modern DevOps pipelines.

## Features

- **Full Syntax Parity**: Supports standard Bash substitutions:
  - `${VAR}` and `$VAR` (Basic substitution)
  - `${VAR:-default}` and `${VAR-default}` (Default values)
  - `${VAR:=default}` and `${VAR=default}` (Assign defaults)
  - `${VAR:+sub}` and `${VAR+sub}` (Alternate/Substitute)
- **Granular Strictness**: Control exactly what missing or empty variables are allowed using new CLI flags.
- **Multiple Inputs**: Read from files, `stdin`, or explicit string arguments.
- **Fast and Safe**: Powered by Rust, memory-safe, and standalone.

## Installation

### With Cargo (Rust)
```bash
cargo install --git https://github.com/YOUR_ORG/ironsubst.git
```

### With Docker (GHCR)
A minimal Docker image is published automatically:
```bash
docker run --rm -e USER=Guest ghcr.io/YOUR_ORG/ironsubst -- 'Hello ${USER}!'
```

## Usage

You can supply input in three ways:

1. **Inline argument** (use `--` to mark it as positional)
```bash
ironsubst -- 'Hello ${USER}!'
```

2. **File input** (using `-i` / `--input`)
```bash
ironsubst -i config.tmpl.yaml -o config.yaml
```

3. **Standard Input (stdin)**
```bash
cat config.tmpl.yaml | ironsubst > config.yaml
```

### Strictness Flags

By default, `ironsubst` will silently substitute empty strings for missing variables (just like standard bash). You can enforce strict presence with the following flags:

* `--require-explicit-values` (or `-no-unset` in `a8m/envsubst`)
  Fails if a variable is not explicitly set in the environment, even if a default value is provided in the template.
* `--require-any-values`
  Fails if a variable is not set in the environment AND there is no fallback value defined in the template.
* `--require-nonempty-values`
  Fails if a variable has no fallback in the template AND the environment variable is either entirely unset or explicitly set to an empty string.

You can also combine these with `-f` / `--fail-fast` to exit on the very first validation error instead of collecting all errors.

### Other Flags
* `--no-digit`: Do not replace variables that start with a digit (e.g. `$1`, `${12}`).

## Local Development (Mise)
If you're developing `ironsubst` locally, you can use [mise](https://mise.jdx.dev) to interact with the project:

```bash
mise run test          # Runs all tests (including parity integration tests)
mise run example       # Runs the showcase example output
mise run format:fix    # Auto-formats the code
mise run lint:fix      # Auto-fixes clippy warnings
```
