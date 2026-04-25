use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn cmd() -> Command {
    Command::cargo_bin("ironsubst").unwrap()
}

#[test]
fn basic_inline_substitution() {
    cmd()
        .env("GREETING", "hello")
        .args(["--", "$GREETING world"])
        .assert()
        .success()
        .stdout("hello world");
}

#[test]
fn env_file_basic() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "MYVAR=from_file").unwrap();
    cmd()
        .env_remove("MYVAR")
        .arg("--env-file")
        .arg(f.path())
        .args(["--", "$MYVAR"])
        .assert()
        .success()
        .stdout("from_file");
}

#[test]
fn env_file_overrides_shell_env() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "MYVAR=from_file").unwrap();
    cmd()
        .env("MYVAR", "from_shell")
        .arg("--env-file")
        .arg(f.path())
        .args(["--", "$MYVAR"])
        .assert()
        .success()
        .stdout("from_file");
}

#[test]
fn ignore_env_excludes_shell() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "FILEVAR=present").unwrap();
    cmd()
        .env("SHELLVAR", "should_not_appear")
        .arg("--env-file")
        .arg(f.path())
        .arg("--ignore-env")
        .args(["--", "$SHELLVAR $FILEVAR"])
        .assert()
        .success()
        .stdout(" present");
}

#[test]
fn prefix_filter_leaves_non_matching_verbatim() {
    cmd()
        .env("APP_HOST", "localhost")
        .env("OTHER", "ignored")
        .arg("--prefix")
        .arg("APP_")
        .args(["--", "$APP_HOST $OTHER"])
        .assert()
        .success()
        .stdout("localhost $OTHER");
}

#[test]
fn input_and_positional_conflict() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "hello").unwrap();
    cmd()
        .arg("--input")
        .arg(f.path())
        .args(["--", "inline"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn env_file_trailing_garbage_after_double_quote_is_error() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, r#"FOO="bar"oops"#).unwrap();
    cmd()
        .arg("--env-file")
        .arg(f.path())
        .args(["--", "$FOO"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected content after closing quote",
        ));
}

#[test]
fn env_file_trailing_garbage_after_single_quote_is_error() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "FOO='bar'oops").unwrap();
    cmd()
        .arg("--env-file")
        .arg(f.path())
        .args(["--", "$FOO"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected content after closing quote",
        ));
}

#[test]
fn env_file_trailing_comment_after_quoted_value_is_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, r#"FOO="bar" # a comment"#).unwrap();
    cmd()
        .arg("--env-file")
        .arg(f.path())
        .args(["--", "$FOO"])
        .assert()
        .success()
        .stdout("bar");
}

#[test]
fn require_values_errors_on_unset() {
    cmd()
        .env_remove("DEFINITELY_UNSET_VAR_XYZ")
        .arg("--require-values")
        .args(["--", "$DEFINITELY_UNSET_VAR_XYZ"])
        .assert()
        .failure();
}

#[test]
fn missing_env_file_exits_nonzero() {
    cmd()
        .arg("--env-file")
        .arg("/nonexistent/path/that/does/not/exist.env")
        .args(["--", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot read env file"));
}
