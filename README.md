# ![ironsubst](https://capsule-render.vercel.app/api?type=blur&height=300&color=0:B7410E,100:a8a8a8&text=ironsubst&textBg=false&fontColor=faf8f8&descSize=12&descAlignY=62&fontSize=72&stroke=faf0f0&strokeWidth=0)

A fast, configurable env**iron**ment variable **subst**itution tool written in Rust.

![generated with `mise run demo`](demo.gif)

`ironsubst` is a ground-up Rust rewrite of [`a8m/envsubst`](https://github.com/a8m/envsubst) with enhanced strictness configurations for modern DevOps pipelines. It follows POSIX semantics for all operators (including the POSIX-correct distinction between `${VAR:+alt}` and `${VAR+alt}`).

**~33% faster[^1] and ~65% smaller[^2]** than the Go reference implementation (`envsubst`) — ideal for container images and embedded use cases:

| Command            | Mean [ms] | Min [ms] | Max [ms] |    Relative |
| :----------------- | --------: | -------: | -------: | ----------: |
| `ironsubst (Rust)` | 2.4 ± 0.3 |      2.0 |      8.2 |        1.00 |
| `envsubst (Go)`    | 3.2 ± 0.3 |      2.7 |     12.8 | 1.33 ± 0.23 |

[^1]: as per `mise run bench`

[^2]: as per `mise run size-compare`

## Features

- **Full Syntax Parity**: Supports standard POSIX / Bash substitutions:
  - `${VAR}` and `$VAR` (Basic substitution)
  - `${VAR:-default}` and `${VAR-default}` (Default values)
  - `${VAR:=default}` and `${VAR=default}` (Assign defaults — note: unlike real bash, the value is _not_ written back to the environment; only the output is affected)
  - `${VAR:+sub}` and `${VAR+sub}` (Alternate/Substitute — POSIX-correct: `:+` fires only when VAR is set _and_ non-empty)
  - `${VAR:?msg}` and `${VAR?msg}` (Error — exit with custom message if VAR is unset/empty; `a8m/envsubst` doesn't support this)
  - `${#VAR}` (String length)
  - `${VAR#pattern}` and `${VAR##pattern}` (Prefix strip — shortest and longest match; supports `*`, `?`, `[abc]` globs)
  - `${VAR%pattern}` and `${VAR%%pattern}` (Suffix strip — shortest and longest match; supports `*`, `?`, `[abc]` globs)
  - `${VAR:offset}` and `${VAR:offset:length}` (Substring — counts Unicode characters, not bytes)
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

### With mise-en-place

```bash
mise use ironsubst
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

- `--require-values` (equivalent to `-no-unset` in `a8m/envsubst`)  
  Fails if a variable is not explicitly set in the environment. When a default/fallback
  operator fires (e.g. `${NOTSET:-fallback}`), that expression succeeds because the
  fallback provides a value; only a bare unset variable triggers the error.
  Fails if a variable is not set in the environment AND there is no fallback operator
  in the template expression (i.e. the variable appears as a bare `$VAR` or `${VAR}`).
- `--require-nonempty-values`  
  Fails if a variable is set to an empty string (or is unset and no fallback fires).

You can also combine these with `-f` / `--fail-fast` to exit on the very first validation error instead of collecting all errors.

### Environment Files

Load variables from one or more `.env` files with `--env-file`:

```bash
ironsubst --env-file prod.env -i config.tmpl.yaml -o config.yaml
# Multiple files: later files override earlier ones
ironsubst --env-file base.env --env-file override.env -i config.tmpl.yaml
```

To ignore the caller's shell environment entirely (only use file-sourced variables):

```bash
ironsubst --env-file prod.env --ignore-env -i config.tmpl.yaml
```

`.env` file syntax supported: `KEY=VALUE`, `export KEY=VALUE`, double- and single-quoted values, and `#` comments.

### Prefix Filter

With `--prefix`, only variables whose names start with the given prefix are substituted. Variables that don't match are left verbatim in the output:

```bash
# Substitutes $MYAPP_HOST and $MYAPP_PORT but leaves $OTHER unchanged
ironsubst --prefix MYAPP_ -i config.tmpl.yaml
```

### Other Flags

- `--no-digit`: Do not replace variables that start with a digit (e.g. `$1`, `${12}`).

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
```

## Parity with other env substitution tools

As generated with `mise run compare`

### Variable state: UNSET

| Tool                                    | `${UNSET_VAR}`<br>Bare                | `${UNSET_VAR-fb}`<br>Def (-) | `${UNSET_VAR:-fb}`<br>Def (:-) | `${UNSET_VAR=fb}`<br>Assign (=) | `${UNSET_VAR:=fb}`<br>Assign (:=) | `${UNSET_VAR:=}`<br>Empty Assign (:=) | `${UNSET_VAR+alt}`<br>Subst (+) | `${UNSET_VAR:+alt}`<br>Subst (:+) | `${UNSET_VAR?err}`<br>Error (?)       | `${UNSET_VAR:?err}`<br>Error (:?)     |
| --------------------------------------- | ------------------------------------- | ---------------------------- | ------------------------------ | ------------------------------- | --------------------------------- | ------------------------------------- | ------------------------------- | --------------------------------- | ------------------------------------- | ------------------------------------- |
| **ironsubst (default)**                 | `""`                                  | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | ERR(1): `UNSET_VAR: err`              | ERR(1): `UNSET_VAR: err`              |
| **ironsubst --require-values**          | ERR(1): `variable ${UNSET_VAR} ...`   | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | ERR(1): `UNSET_VAR: err`              | ERR(1): `UNSET_VAR: err`              |
| **ironsubst --require-nonempty-values** | `""`                                  | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | ERR(1): `UNSET_VAR: err`              | ERR(1): `UNSET_VAR: err`              |
| **a8m/envsubst (default)**              | `""`                                  | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | `""`                                  | `""`                                  |
| **a8m/envsubst -no-unset**              | ERR(1): `variable ${UNSET_VAR} ...`   | `fb`                         | `fb`                           | `fb`                            | `fb`                              | ERR(1): `variable ${UNSET_VAR} ...`   | `""`                            | `""`                              | ERR(1): `variable ${UNSET_VAR} ...`   | ERR(1): `variable ${UNSET_VAR} ...`   |
| **a8m/envsubst -no-empty**              | `""`                                  | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | `""`                                  | `""`                                  |
| **gettext envsubst**                    | `""`                                  | `${UNSET_VAR-fb}`            | `${UNSET_VAR:-fb}`             | `${UNSET_VAR=fb}`               | `${UNSET_VAR:=fb}`                | `${UNSET_VAR:=}`                      | `${UNSET_VAR+alt}`              | `${UNSET_VAR:+alt}`               | `${UNSET_VAR?err}`                    | `${UNSET_VAR:?err}`                   |
| **bash (default)**                      | `""`                                  | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | ERR(127): `bash: line 1: UNSET_VA...` | ERR(127): `bash: line 1: UNSET_VA...` |
| **bash (set -u)**                       | ERR(127): `bash: line 1: UNSET_VA...` | `fb`                         | `fb`                           | `fb`                            | `fb`                              | `""`                                  | `""`                            | `""`                              | ERR(127): `bash: line 1: UNSET_VA...` | ERR(127): `bash: line 1: UNSET_VA...` |

### Variable state: EMPTY

| Tool                                    | `${EMPTY_VAR}`<br>Bare              | `${EMPTY_VAR-fb}`<br>Def (-)        | `${EMPTY_VAR:-fb}`<br>Def (:-) | `${EMPTY_VAR=fb}`<br>Assign (=)     | `${EMPTY_VAR:=fb}`<br>Assign (:=) | `${EMPTY_VAR:=}`<br>Empty Assign (:=) | `${EMPTY_VAR+alt}`<br>Subst (+) | `${EMPTY_VAR:+alt}`<br>Subst (:+) | `${EMPTY_VAR?err}`<br>Error (?)     | `${EMPTY_VAR:?err}`<br>Error (:?)     |
| --------------------------------------- | ----------------------------------- | ----------------------------------- | ------------------------------ | ----------------------------------- | --------------------------------- | ------------------------------------- | ------------------------------- | --------------------------------- | ----------------------------------- | ------------------------------------- |
| **ironsubst (default)**                 | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `""`                              | `""`                                | ERR(1): `EMPTY_VAR: err`              |
| **ironsubst --require-values**          | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `""`                              | `""`                                | ERR(1): `EMPTY_VAR: err`              |
| **ironsubst --require-nonempty-values** | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb`                           | ERR(1): `variable ${EMPTY_VAR} ...` | `fb`                              | `""`                                  | `alt`                           | `""`                              | `""`                                | ERR(1): `EMPTY_VAR: err`              |
| **a8m/envsubst (default)**              | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `alt`                             | `""`                                | `""`                                  |
| **a8m/envsubst -no-unset**              | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `alt`                             | `""`                                | `""`                                  |
| **a8m/envsubst -no-empty**              | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb`                           | ERR(1): `variable ${EMPTY_VAR} ...` | `fb`                              | ERR(1): `variable ${EMPTY_VAR} ...`   | `alt`                           | `alt`                             | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...`   |
| **gettext envsubst**                    | `""`                                | `${EMPTY_VAR-fb}`                   | `${EMPTY_VAR:-fb}`             | `${EMPTY_VAR=fb}`                   | `${EMPTY_VAR:=fb}`                | `${EMPTY_VAR:=}`                      | `${EMPTY_VAR+alt}`              | `${EMPTY_VAR:+alt}`               | `${EMPTY_VAR?err}`                  | `${EMPTY_VAR:?err}`                   |
| **bash (default)**                      | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `""`                              | `""`                                | ERR(127): `bash: line 1: EMPTY_VA...` |
| **bash (set -u)**                       | `""`                                | `""`                                | `fb`                           | `""`                                | `fb`                              | `""`                                  | `alt`                           | `""`                              | `""`                                | ERR(127): `bash: line 1: EMPTY_VA...` |

### Variable state: SET

| Tool                                    | `${SET_VAR}`<br>Bare | `${SET_VAR-fb}`<br>Def (-) | `${SET_VAR:-fb}`<br>Def (:-) | `${SET_VAR=fb}`<br>Assign (=) | `${SET_VAR:=fb}`<br>Assign (:=) | `${SET_VAR:=}`<br>Empty Assign (:=) | `${SET_VAR+alt}`<br>Subst (+) | `${SET_VAR:+alt}`<br>Subst (:+) | `${SET_VAR?err}`<br>Error (?) | `${SET_VAR:?err}`<br>Error (:?) |
| --------------------------------------- | -------------------- | -------------------------- | ---------------------------- | ----------------------------- | ------------------------------- | ----------------------------------- | ----------------------------- | ------------------------------- | ----------------------------- | ------------------------------- |
| **ironsubst (default)**                 | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **ironsubst --require-values**          | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **ironsubst --require-nonempty-values** | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **a8m/envsubst (default)**              | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **a8m/envsubst -no-unset**              | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **a8m/envsubst -no-empty**              | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **gettext envsubst**                    | `val`                | `${SET_VAR-fb}`            | `${SET_VAR:-fb}`             | `${SET_VAR=fb}`               | `${SET_VAR:=fb}`                | `${SET_VAR:=}`                      | `${SET_VAR+alt}`              | `${SET_VAR:+alt}`               | `${SET_VAR?err}`              | `${SET_VAR:?err}`               |
| **bash (default)**                      | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
| **bash (set -u)**                       | `val`                | `val`                      | `val`                        | `val`                         | `val`                           | `val`                               | `alt`                         | `alt`                           | `val`                         | `val`                           |
