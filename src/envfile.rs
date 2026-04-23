//! Parser for `.env`-style files.
//!
//! Supported syntax:
//! - `KEY=VALUE`
//! - `export KEY=VALUE`  (the `export` keyword is ignored)
//! - Values may be double- or single-quoted: `KEY="hello world"`, `KEY='it'\''s fine'`
//! - Inline `#` comments after an unquoted value are stripped
//! - `#` at the start of a line (after optional whitespace) is a full-line comment
//! - Blank lines are skipped
//! - Values are taken literally — no `$VAR` interpolation inside env files
//!
//! Later files (and later lines within a file) override earlier ones, so
//! passing `--env-file base.env --env-file override.env` works as expected.

use std::collections::HashMap;
use std::path::Path;

/// Error type for env file parsing.
#[derive(Debug, thiserror::Error)]
pub enum EnvFileError {
    #[error("cannot read env file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}:{line}: {msg}")]
    Parse {
        path: String,
        line: usize,
        msg: String,
    },
}

/// Parse a single `.env` file and merge its entries into `env`.
/// Entries from this file overwrite any existing keys.
pub fn load_env_file(path: &Path, env: &mut HashMap<String, String>) -> Result<(), EnvFileError> {
    let path_str = path.display().to_string();
    let content = std::fs::read_to_string(path).map_err(|e| EnvFileError::Io {
        path: path_str.clone(),
        source: e,
    })?;

    for (lineno, raw) in content.lines().enumerate() {
        let line_num = lineno + 1;
        let line = raw.trim();

        // Skip blank lines and full-line comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Strip optional `export ` prefix
        let line = line.strip_prefix("export ").unwrap_or(line).trim_start();

        // Split on the first `=`
        let eq = line.find('=').ok_or_else(|| EnvFileError::Parse {
            path: path_str.clone(),
            line: line_num,
            msg: format!("expected KEY=VALUE, got: {line}"),
        })?;

        let key = line[..eq].trim_end();
        let raw_value = &line[eq + 1..];

        if key.is_empty() {
            return Err(EnvFileError::Parse {
                path: path_str.clone(),
                line: line_num,
                msg: "empty key".to_string(),
            });
        }

        // Validate key: must be [A-Za-z_][A-Za-z0-9_]*
        if !is_valid_key(key) {
            return Err(EnvFileError::Parse {
                path: path_str.clone(),
                line: line_num,
                msg: format!("invalid variable name: {key}"),
            });
        }

        let value = parse_value(raw_value).map_err(|msg| EnvFileError::Parse {
            path: path_str.clone(),
            line: line_num,
            msg,
        })?;

        env.insert(key.to_string(), value);
    }

    Ok(())
}

/// Load multiple env files in order, each overriding the previous.
pub fn load_env_files(
    paths: &[impl AsRef<Path>],
    env: &mut HashMap<String, String>,
) -> Result<(), EnvFileError> {
    for path in paths {
        load_env_file(path.as_ref(), env)?;
    }
    Ok(())
}

/// Parse the value portion of a `KEY=VALUE` line:
/// - Double-quoted `"..."`: strip quotes, interpret `\"` and `\\`
/// - Single-quoted `'...'`: strip quotes, no escape processing
/// - Unquoted: strip trailing inline comment (`# ...`) and trim trailing whitespace
fn parse_value(raw: &str) -> Result<String, String> {
    if let Some(inner) = raw.strip_prefix('"') {
        // Double-quoted: scan for closing unescaped `"`
        let mut value = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        loop {
            match chars.next() {
                None => return Err("unterminated double-quoted value".to_string()),
                Some('"') => break,
                Some('\\') => match chars.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some(c) => {
                        // Pass through unknown escapes literally (e.g. `\$`)
                        value.push('\\');
                        value.push(c);
                    }
                    None => return Err("trailing backslash in double-quoted value".to_string()),
                },
                Some(c) => value.push(c),
            }
        }
        // Any trailing content after the closing quote is ignored (allows comments like `"val" # note`)
        Ok(value)
    } else if let Some(inner) = raw.strip_prefix('\'') {
        // Single-quoted: no escapes, scan for closing `'`
        let close = inner
            .find('\'')
            .ok_or_else(|| "unterminated single-quoted value".to_string())?;
        Ok(inner[..close].to_string())
    } else {
        // Unquoted: strip inline comment and trailing whitespace
        let value = strip_inline_comment(raw).trim_end();
        Ok(value.to_string())
    }
}

/// Strip a trailing `# comment` from an unquoted value.
/// Only strips if `#` is preceded by at least one whitespace character.
fn strip_inline_comment(s: &str) -> &str {
    // Walk the string looking for ` #` or `\t#`
    let bytes = s.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == b'#' && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            return &s[..i - 1];
        }
    }
    s
}

fn is_valid_key(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse(content: &str) -> Result<HashMap<String, String>, EnvFileError> {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let mut env = HashMap::new();
        load_env_file(f.path(), &mut env)?;
        Ok(env)
    }

    #[test]
    fn test_basic() {
        let env = parse("FOO=bar\nBAZ=qux\n").unwrap();
        assert_eq!(env["FOO"], "bar");
        assert_eq!(env["BAZ"], "qux");
    }

    #[test]
    fn test_comments_and_blanks() {
        let env = parse("# comment\n\nFOO=bar\n  # indented comment\n").unwrap();
        assert_eq!(env["FOO"], "bar");
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn test_export_prefix() {
        let env = parse("export FOO=bar\nexport BAZ=qux\n").unwrap();
        assert_eq!(env["FOO"], "bar");
        assert_eq!(env["BAZ"], "qux");
    }

    #[test]
    fn test_double_quoted() {
        let env = parse(r#"FOO="hello world""#).unwrap();
        assert_eq!(env["FOO"], "hello world");
    }

    #[test]
    fn test_double_quoted_escapes() {
        let env = parse(r#"FOO="say \"hi\"\\n""#).unwrap();
        assert_eq!(env["FOO"], r#"say "hi"\n"#);
    }

    #[test]
    fn test_single_quoted() {
        let env = parse("FOO='hello world'\n").unwrap();
        assert_eq!(env["FOO"], "hello world");
    }

    #[test]
    fn test_single_quoted_no_escapes() {
        // Backslash is literal inside single quotes
        let env = parse(r"FOO='no\escape'").unwrap();
        assert_eq!(env["FOO"], r"no\escape");
    }

    #[test]
    fn test_inline_comment() {
        let env = parse("FOO=bar # this is a comment\n").unwrap();
        assert_eq!(env["FOO"], "bar");
    }

    #[test]
    fn test_no_inline_comment_without_space() {
        // `#` without preceding space is part of the value
        let env = parse("FOO=bar#notacomment\n").unwrap();
        assert_eq!(env["FOO"], "bar#notacomment");
    }

    #[test]
    fn test_empty_value() {
        let env = parse("FOO=\n").unwrap();
        assert_eq!(env["FOO"], "");
    }

    #[test]
    fn test_empty_quoted_value() {
        let env = parse(r#"FOO="""#).unwrap();
        assert_eq!(env["FOO"], "");
    }

    #[test]
    fn test_override_order() {
        let env = parse("FOO=first\nFOO=second\n").unwrap();
        assert_eq!(env["FOO"], "second");
    }

    #[test]
    fn test_multiple_files_override() {
        let mut f1 = NamedTempFile::new().unwrap();
        let mut f2 = NamedTempFile::new().unwrap();
        f1.write_all(b"FOO=base\nBAR=keep\n").unwrap();
        f2.write_all(b"FOO=override\n").unwrap();
        let mut env = HashMap::new();
        load_env_files(&[f1.path(), f2.path()], &mut env).unwrap();
        assert_eq!(env["FOO"], "override");
        assert_eq!(env["BAR"], "keep");
    }

    #[test]
    fn test_invalid_key() {
        assert!(parse("123INVALID=val\n").is_err());
    }

    #[test]
    fn test_no_equals() {
        assert!(parse("NOEQUALS\n").is_err());
    }

    #[test]
    fn test_unterminated_double_quote() {
        assert!(parse(r#"FOO="unterminated"#).is_err());
    }

    #[test]
    fn test_unterminated_single_quote() {
        assert!(parse("FOO='unterminated\n").is_err());
    }
}
