# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.2.0](https://github.com/melonswork/ironsubst/compare/0.1.0..0.2.0) - 2026-05-02

### 🚀 Features

- *(eval)* Implement ${#VAR} string-length operator
- *(glob)* Add pure-Rust glob matcher for parameter expansion patterns
- *(eval)* Implement ${VAR#pat}, ${VAR##pat}, ${VAR%pat}, ${VAR%%pat} strip operators
- *(eval)* Implement ${VAR:offset} and ${VAR:offset:length} substring operator

### 🐛 Bug Fixes

- *(envfile)* Reject non-comment trailing content after quoted values
- *(cli)* Make --input and inline template mutually exclusive
- *(eval)* Respect --fail-fast when a fallback expression itself errors
- *(parser)* Allow $_ and ${_} to be substituted like any other variable
- *(envfile)* Implement POSIX '\'' single-quote concatenation
- *(main)* Preserve file permissions when overwriting with -o
- *(eval)* Re-escape \$ as \$\$ when reconstructing verbatim prefix output
- *(parser)* Limit expression nesting depth to prevent stack overflow
- *(parser)* Preserve unsupported operators verbatim instead of silently dropping them
- *(eval)* --require-nonempty-values now also fails on unset variables
- *(parser)* Emit ${#VAR} verbatim when malformed or --no-digit rejects name
- *(eval)* Preserve bare \$ verbatim in prefix-filter reconstruction
- *(eval)* String-manipulation operators now respect --require-values/--require-nonempty-values
- *(eval)* Evaluate variable references inside strip-operator patterns
- *(parser/eval)* Evaluate variable references in substring offset/length
- *(eval)* Preserve ${VAR:$IDX} verbatim when index variables don't match --prefix
- *(eval)* Preserve strip expressions verbatim when pattern variable doesn't match --prefix
- *(eval)* Simplify Length original_text format string
- *(eval)* Recursive unmatched-var check for prefix-filter guards
- *(glob)* Memoize glob_rec to eliminate exponential backtracking
- *(mise)* Add actionlint to tools list
- *(eval)* Suppress outer Error-operator error when fallback itself errors
- *(deps)* Update rust crate clap_mangen to 0.3 ([#7](https://github.com/melonswork/ironsubst/pull/7))
- *(main)* New output files now respect umask
- *(main)* Reject symlink paths passed to -o

### 🚜 Refactor

- *(eval)* Extract check_restrictions helper
- *(eval)* Extract eval_to_string helper

### 📚 Documentation

- Document --env-file, --ignore-env, and --prefix in README
- Document new string-manipulation operators in README
- *(readme)* Replace embedded comparison table with link to comparison.md
- *(ast)* Clarify bool semantics in Operator comments
- Add --locked to cargo install command in README
- Note GETTEXT_ENVSUBST_PATH requirement for compare task

### 🧪 Testing

- *(cli)* Add CLI integration test suite using assert_cmd
- Regression tests for fail-fast-in-fallback and $_ substitution
- *(glob)* Add negated-range and verbatim passthrough tests

### ⚙️ Miscellaneous Tasks

- Release v0.1.0
- Formatting
- Add .dockerignore to exclude build artifacts from context
## [0.1.0] - 2026-04-24

### 🚀 Features

- Complete Rust rewrite
- Shell `completions` subcommand, `--generate-man-page` hidden flag
- Add --prefix filter to restrict substitution to matching variable names
- Add --env-file flag to load environment from .env files
- Add support for the POSIX `?` and `:?` error operators

### 🐛 Bug Fixes

- Polish project metadata
- Remove --no-skip-checkout from act config, use local dir instead of fetching from GitHub
- Correctly specify SBOM filename for Snyk test

### 💼 Other

- Snyk tool and security test task
- Deny unsafe code; write output atomically via tempfile

### 🚜 Refactor

- Remove Env wrapper, merge Assign/Default arms, fix :+ POSIX semantics, improve error messages

### 📚 Documentation

- Setup vhs to auto-generate a terminal demo GIF
- Update README, add binary size callout, document := limitation
- Enrich README.md with outputs from `demo`, `bench`, `compare`
- Draft CHANGELOG.md

### ⚡ Performance

- Add bench task to mise.toml comparing ironsubst and envsubst

### 🧪 Testing

- Add parity-test task to mise.toml
- Add require_any_values dedicated tests and CI matrix with macOS + cargo audit
- Rework parity-test to show documented diffs, expand example.txt with divergences

### ⚙️ Miscellaneous Tasks

- LICENSE
- Use mise-action to run cargo tasks
- Setup hk for pre-commit hooks and checks
- Setup release-plz for automated rust releases
- Setup git-cliff for automatic changelog generation
- Add renovate.json for automated dependency updates
- Add codecov.yml configuration for coverage reporting
- Update Dockerfile to rust:latest, add MSRV 1.85, release profile LTO
- Use caret version ranges for clap/thiserror, add additional review findings to REVIEW.md
- Configure renovate.json, codecov.yml, multi-platform Docker, fuzz harnesses, mise tasks
- Simplify release-plz workflow to use mise instead of custom actions
- Add Swatinem/rust-cache to all CI jobs for faster builds
- Untrack generated man page and completions, add to .gitignore
- Add mise hooks task to install hk git hooks
- Fix mise test-gha task for podman-machine, add .actrc
- Check gh auth login before running test-gha
- Run gh auth login automatically if not authenticated in test-gha
- Remove --container-architecture from .actrc to allow native arm64 runs on M-series chips
- Remove redundant --require-any-values, rename --require-explicit-values to --require-values
- Add portable compare.py script for tool behavior comparison table
- Remove parity-test task and references (replaced by compare.py)
- Untrack comparison.md, add to .gitignore
- Switch all workflows to master, release-pr on workflow_dispatch
- Codecov action bumps, secrets, refactors & general bodacification
- Integrate git-cliff into release-plz for changelog generation
