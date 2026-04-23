# ironsubst

A blazing fast, deeply configurable environment variable substitution tool written in Rust.

`ironsubst` is a ground-up Rust rewrite of [`a8m/envsubst`](https://github.com/a8m/envsubst) with enhanced strictness configurations for modern DevOps pipelines. It follows POSIX semantics for all operators (including the POSIX-correct distinction between `${VAR:+alt}` and `${VAR+alt}`).

**~25% faster and ~62% smaller** than the Go reference implementation (`envsubst`): ironsubst ships as a ~760 KB stripped binary vs ~2.4 MB for Go envsubst — ideal for container images and embedded use cases.

## Features

- **Full Syntax Parity**: Supports standard Bash substitutions:
  - `${VAR}` and `$VAR` (Basic substitution)
  - `${VAR:-default}` and `${VAR-default}` (Default values)
  - `${VAR:=default}` and `${VAR=default}` (Assign defaults — note: unlike real bash, the value is *not* written back to the environment; only the output is affected)
  - `${VAR:+sub}` and `${VAR+sub}` (Alternate/Substitute — POSIX-correct: `:+` fires only when VAR is set *and* non-empty)
  - `${VAR:?msg}` and `${VAR?msg}` (Error — exit with custom message if VAR is unset/empty; `a8m/envsubst` doesn't support this)
- **Granular Strictness**: Control exactly what missing or empty variables are allowed using new CLI flags.
- **Multiple Inputs**: Read from files, `stdin`, or explicit string arguments.
- **Fast and Safe**: Powered by Rust, memory-safe, zero `unsafe` code, standalone binary.
- **Atomic output writes**: When writing to a file with `-o`, the file is written atomically (temp-file + rename), so a crash mid-write cannot leave a corrupt output file.

## Installation

### With Cargo (Rust)
```bash
cargo install --git https://github.com/melonswork/ironsubst.git
```

### With Docker (GHCR)
A minimal Docker image is published automatically:
```bash
docker run --rm -e USER=Guest ghcr.io/melonswork/ironsubst -- 'Hello ${USER}!'
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

* `--require-explicit-values` (equivalent to `-no-unset` in `a8m/envsubst`)  
  Fails if a variable is not explicitly set in the environment. When a default/fallback
  operator fires (e.g. `${NOTSET:-fallback}`), that expression succeeds because the
  fallback provides a value; only a bare unset variable triggers the error.
* `--require-any-values`  
  Fails if a variable is not set in the environment AND there is no fallback operator
  in the template expression (i.e. the variable appears as a bare `$VAR` or `${VAR}`).
* `--require-nonempty-values`  
  Fails if a variable is set to an empty string (or is unset and no fallback fires).

You can also combine these with `-f` / `--fail-fast` to exit on the very first validation error instead of collecting all errors.

### Other Flags
* `--no-digit`: Do not replace variables that start with a digit (e.g. `$1`, `${12}`).

### Shell Completions

```bash
# Bash
ironsubst completions bash > /etc/bash_completion.d/ironsubst
# Zsh
ironsubst completions zsh  > "${fpath[1]}/_ironsubst"
# Fish
ironsubst completions fish > ~/.config/fish/completions/ironsubst.fish
```

### Man Page

The binary can self-generate a man page (useful for `cargo install` users who have no package manager to install it for them):

```bash
# User-local (no sudo required):
mkdir -p ~/.local/share/man/man1
ironsubst --generate-man-page > ~/.local/share/man/man1/ironsubst.1

# System-wide:
ironsubst --generate-man-page | sudo tee /usr/local/share/man/man1/ironsubst.1 > /dev/null
```

Then `man ironsubst` works immediately.

## Local Development (Mise)
If you're developing `ironsubst` locally, you can use [mise](https://mise.jdx.dev) to interact with the project:

```bash
mise run test          # Runs all tests (including parity integration tests)
mise run example       # Runs the showcase example output
mise run format:fix    # Auto-formats the code
mise run lint:fix      # Auto-fixes clippy warnings
mise run bench         # Benchmarks ironsubst vs Go envsubst
mise run demo          # Regenerates demo.gif via VHS
```
