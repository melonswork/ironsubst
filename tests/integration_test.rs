use ironsubst::{eval::Restrictions, process};
use std::collections::HashMap;

fn get_fake_env() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("BAR".to_string(), "bar".to_string());
    env.insert("FOO".to_string(), "foo".to_string());
    env.insert("EMPTY".to_string(), "".to_string());
    env.insert("ALSO_EMPTY".to_string(), "".to_string());
    env.insert("A".to_string(), "AAA".to_string());
    env
}

struct ParseTest {
    name: &'static str,
    input: &'static str,
    expected: &'static str,
    has_err_relaxed: bool,
    has_err_require_values: bool,
    has_err_no_empty: bool,
    has_err_strict: bool,
}

fn get_tests() -> Vec<ParseTest> {
    vec![
        ParseTest {
            name: "empty",
            input: "",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "env only",
            input: "$BAR",
            expected: "bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "with text",
            input: "$BAR baz",
            expected: "bar baz",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "concatenated",
            input: "$BAR$FOO",
            expected: "barfoo",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "2 env var",
            input: "$BAR - $FOO",
            expected: "bar - foo",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "underscore var (unset in fake env)",
            input: "$_ bar",
            expected: " bar",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "braced underscore var (unset in fake env)",
            input: "${_} bar",
            expected: " bar",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "value of $var",
            input: "${BAR}baz",
            expected: "barbaz",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set -",
            input: "${NOTSET-$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set =",
            input: "${NOTSET=$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var set but empty -",
            input: "${EMPTY-$BAR}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var set but empty =",
            input: "${EMPTY=$BAR}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set or empty :-",
            input: "${EMPTY:-$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set or empty :=",
            input: "${EMPTY:=$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var set evaluate expression as $other +",
            input: "${EMPTY+hello}",
            expected: "hello",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            // POSIX-correct: ${VAR:+alt} fires only when VAR is set AND non-empty.
            // EMPTY is set but empty → :+ does NOT fire → result is "".
            name: "if $var set evaluate expression as $other :+",
            input: "${EMPTY:+hello}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var not set, use empty string +",
            input: "${NOTSET+hello}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var not set, use empty string :+",
            input: "${NOTSET:+hello}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "multi line string",
            input: "hello $BAR\nhello ${EMPTY:=$FOO}",
            expected: "hello bar\nhello foo",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "issue #1",
            input: "${hello:=wo_rld} ${foo:=bar_baz}",
            expected: "wo_rld bar_baz",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "issue #2",
            input: "name: ${NAME:=foo_qux}, key: ${EMPTY:=baz_bar}",
            expected: "name: foo_qux, key: baz_bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-8",
            input: "prop=${HOME_URL-http://localhost:8080}",
            expected: "prop=http://localhost:8080",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-1",
            input: "${NOTSET--1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-2",
            input: "${NOTSET:--1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-3",
            input: "${NOTSET=-1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-4",
            input: "${NOTSET:==1}",
            expected: "=1",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-43-1",
            input: "${A}",
            expected: "AAA",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "closing brace expected",
            input: "hello ${",
            expected: "",
            has_err_relaxed: true,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set",
            input: "${NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var set to empty",
            input: "${EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "gh-issue-9",
            input: "$NOTSET",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "gh-issue-9-empty",
            input: "$EMPTY",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set -",
            input: "${NOTSET-$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set :-",
            input: "${NOTSET:-$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set =",
            input: "${NOTSET=$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set :=",
            input: "${NOTSET:=$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $OTHER not set +",
            input: "${NOTSET+$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var and $OTHER not set :+",
            input: "${NOTSET:+$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set -",
            input: "${EMPTY-$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set :-",
            input: "${EMPTY:-$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
            // EMPTY is empty + colon → fallback fires; $NOTSET bare+unset → error
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set =",
            input: "${EMPTY=$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set :=",
            input: "${EMPTY:=$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
            // EMPTY is empty + colon → fallback fires; $NOTSET bare+unset → error
        },
        ParseTest {
            name: "$var empty and $OTHER not set +",
            input: "${EMPTY+$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            // POSIX-correct: EMPTY is set but empty → :+ does NOT fire →
            // $NOTSET is never evaluated → no error regardless of restrictions.
            name: "$var empty and $OTHER not set :+",
            input: "${EMPTY:+$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty -",
            input: "${NOTSET-$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty :-",
            input: "${NOTSET:-$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty =",
            input: "${NOTSET=$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty :=",
            input: "${NOTSET:=$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $OTHER empty +",
            input: "${NOTSET+$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set and $OTHER empty :+",
            input: "${NOTSET:+$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var and $DEFAULT empty -",
            input: "${EMPTY-$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty :-",
            input: "${EMPTY:-$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty =",
            input: "${EMPTY=$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty :=",
            input: "${EMPTY:=$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $OTHER empty +",
            input: "${EMPTY+$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            // POSIX-correct: EMPTY is set but empty → :+ does NOT fire →
            // $ALSO_EMPTY is never evaluated → no error regardless of restrictions.
            name: "$var and $OTHER empty :+",
            input: "${EMPTY:+$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$var",
            input: "FOO $$BAR BAZ",
            expected: "FOO $BAR BAZ",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $${subst}",
            input: "FOO $${BAR} BAZ",
            expected: "FOO ${BAR} BAZ",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$$var",
            input: "$$$BAR",
            expected: "$bar",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$${subst}",
            input: "$$${BAZ:-baz}",
            expected: "$baz",
            has_err_relaxed: false,
            has_err_require_values: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
    ]
}

fn do_test(restrictions: Restrictions, get_err: fn(&ParseTest) -> bool) {
    let env = get_fake_env();
    for test in get_tests() {
        let has_err = get_err(&test);
        let result = process(test.input, &env, restrictions, false, false, None);

        assert_eq!(
            result.is_err(),
            has_err,
            "{}=({}): expected err {}, got {:?}",
            test.name,
            test.input,
            has_err,
            result
        );

        if !has_err {
            let actual = result.unwrap();
            assert_eq!(
                actual, test.expected,
                "{}=({}): expected {}, got {:?}",
                test.name, test.input, test.expected, actual
            );
        }
    }
}

#[test]
fn test_parse_relaxed() {
    do_test(
        Restrictions {
            require_values: false,
            require_nonempty_values: false,
        },
        |t| t.has_err_relaxed,
    );
}

#[test]
fn test_parse_require_values() {
    do_test(
        Restrictions {
            require_values: true,
            require_nonempty_values: false,
        },
        |t| t.has_err_require_values,
    );
}

#[test]
fn test_parse_no_empty() {
    do_test(
        Restrictions {
            require_values: false,
            require_nonempty_values: true,
        },
        |t| t.has_err_no_empty,
    );
}

#[test]
fn test_parse_strict() {
    do_test(
        Restrictions {
            require_values: true,
            require_nonempty_values: true,
        },
        |t| t.has_err_strict,
    );
}

#[test]
fn test_require_values_dedicated() {
    let env = get_fake_env();
    let restrictions = Restrictions {
        require_values: true,
        require_nonempty_values: false,
    };

    // Bare unset variable → error
    let r = process("$NOTSET", &env, restrictions, false, false, None);
    assert!(r.is_err(),);

    // Bare set variable → ok
    let r = process("$BAR", &env, restrictions, false, false, None);
    assert_eq!(r.unwrap(), "bar");

    // Bare empty variable → ok (it IS set, just empty)
    let r = process("$EMPTY", &env, restrictions, false, false, None);
    assert_eq!(r.unwrap(), "");

    // Unset variable with default operator → ok (fallback provides a value)
    let r = process(
        "${NOTSET:-fallback}",
        &env,
        restrictions,
        false,
        false,
        None,
    );
    assert_eq!(r.unwrap(), "fallback");

    // Unset variable with = operator → ok (fallback provides a value)
    let r = process(
        "${NOTSET:=fallback}",
        &env,
        restrictions,
        false,
        false,
        None,
    );
    assert_eq!(r.unwrap(), "fallback");

    // Unset variable with + operator → ok (+ on unset → empty string, no error)
    let r = process("${NOTSET+alt}", &env, restrictions, false, false, None);
    assert_eq!(r.unwrap(), "");

    // Braced unset variable, no operator → error
    let r = process("${NOTSET}", &env, restrictions, false, false, None);
    assert!(r.is_err(),);

    // Multiple errors collected (fail_fast = false)
    let r = process(
        "$NOTSET $ALSO_NOTSET",
        &env,
        restrictions,
        false,
        false,
        None,
    );
    assert!(r.is_err());
    let msg = r.unwrap_err().to_string();
    assert!(msg.contains("$NOTSET"), "error should name $NOTSET");
    assert!(
        msg.contains("$ALSO_NOTSET"),
        "error should name $ALSO_NOTSET"
    );

    // fail_fast = true stops at first error
    let r = process(
        "$NOTSET $ALSO_NOTSET",
        &env,
        restrictions,
        false,
        true,
        None,
    );
    assert!(r.is_err());
    let msg = r.unwrap_err().to_string();
    assert!(
        msg.contains("$NOTSET"),
        "fail_fast should report first error"
    );
    // The second variable may or may not appear depending on evaluation order;
    // the important thing is that we got exactly one error line.
    assert_eq!(
        msg.lines().count(),
        1,
        "fail_fast should produce a single error"
    );
}

#[test]
fn test_require_nonempty_values_fails_on_unset() {
    // Regression: --require-nonempty-values was silently allowing unset variables
    // through (exiting 0) because the check was guarded by `is_set && is_empty`
    // rather than also catching the `!is_set` case.
    let env = get_fake_env();
    let restrictions = Restrictions {
        require_values: false,
        require_nonempty_values: true,
    };

    // Bare unset variable must fail — unset expands to "", violating the constraint.
    let r = process("$NOTSET", &env, restrictions, false, false, None);
    assert!(
        r.is_err(),
        "unset bare variable should error with --require-nonempty-values"
    );
    let msg = r.unwrap_err().to_string();
    assert!(msg.contains("not set"), "error should say 'not set'");

    // Braced unset variable must also fail.
    let r = process("${NOTSET}", &env, restrictions, false, false, None);
    assert!(
        r.is_err(),
        "unset braced variable should error with --require-nonempty-values"
    );

    // Unset with a fallback fires the fallback — no error.
    let r = process(
        "${NOTSET:-fallback}",
        &env,
        restrictions,
        false,
        false,
        None,
    );
    assert_eq!(r.unwrap(), "fallback", "unset + fallback should succeed");

    // Empty variable (is_set, is_empty) must still fail.
    let r = process("$EMPTY", &env, restrictions, false, false, None);
    assert!(
        r.is_err(),
        "empty variable should error with --require-nonempty-values"
    );

    // Set, non-empty variable must succeed.
    let r = process("$BAR", &env, restrictions, false, false, None);
    assert_eq!(r.unwrap(), "bar");

    // Both flags active + unset → single Unset error (not duplicated).
    let both = Restrictions {
        require_values: true,
        require_nonempty_values: true,
    };
    let r = process("$NOTSET", &env, both, false, false, None);
    assert!(r.is_err());
    assert_eq!(
        r.unwrap_err().to_string().lines().count(),
        1,
        "both flags active should produce exactly one error for an unset variable"
    );
}

#[test]
fn test_prefix_filter() {
    let mut env = HashMap::new();
    env.insert("APP_HOST".to_string(), "localhost".to_string());
    env.insert("APP_PORT".to_string(), "8080".to_string());
    env.insert("OTHER".to_string(), "secret".to_string());
    env.insert("EMPTY".to_string(), String::new());

    let relaxed = Restrictions::default();

    // Matching variable is substituted
    let r = process("${APP_HOST}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "localhost");

    // Non-matching variable is left verbatim (braced form preserved)
    let r = process("${OTHER}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER}");

    // Non-matching unbraced variable is left verbatim
    let r = process("$OTHER", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "$OTHER");

    // Mix: matching substituted, non-matching verbatim
    let r = process(
        "${APP_HOST}:${APP_PORT} $OTHER",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(r, "localhost:8080 $OTHER");

    // Operator on matching variable works normally
    let r = process(
        "${APP_MISSING:-default}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(r, "default");

    // Operator on non-matching variable: whole expression left verbatim
    let r = process(
        "${OTHER:-fallback}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(r, "${OTHER:-fallback}");

    // Non-matching variable does NOT trigger restriction errors even if unset
    let strict = Restrictions {
        require_values: true,
        require_nonempty_values: true,
    };
    // UNSET_OTHER is not in env and has no APP_ prefix → verbatim, no error
    let r = process("$UNSET_OTHER", &env, strict, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "$UNSET_OTHER");

    // APP_ prefixed unset variable DOES trigger restriction errors
    let r = process("$APP_MISSING", &env, strict, false, false, Some("APP_"));
    assert!(
        r.is_err(),
        "unset APP_ var should error under strict restrictions"
    );

    // No prefix = normal behaviour (existing tests cover this, just a sanity check)
    let r = process("${OTHER}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "secret");

    // Operator fallback text reconstruction: -= :- = :=
    let r = process("${OTHER-fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER-fb}");
    let r = process("${OTHER:-fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER:-fb}");
    let r = process("${OTHER=fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER=fb}");
    let r = process("${OTHER:=fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER:=fb}");
    let r = process("${OTHER+fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER+fb}");
    let r = process("${OTHER:+fb}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER:+fb}");

    // Nested: non-matching var in fallback of matching var
    // ${APP_MISSING:-$OTHER} — APP_MISSING is unset so fallback fires,
    // but $OTHER doesn't have APP_ prefix so it should be left verbatim inside
    let r = process(
        "${APP_MISSING:-$OTHER}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(r, "$OTHER");

    // Regression: $$ in the fallback of a non-matching variable must not lose the extra $
    let r = process("${OTHER:-$$}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(r, "${OTHER:-$$}");

    // Regression: bare `$` followed by a non-identifier (e.g. `$-`) in the fallback
    // of a non-matching variable was being corrupted to `$$-` by the re-escaping pass
    // in nodes_to_text.  The fix stores `$$` (encoded) in text nodes for real escapes
    // and leaves bare `$` as-is, so reconstruction is lossless.
    let r = process("${OTHER:-$-}", &env, relaxed, false, false, Some("APP_")).unwrap();
    assert_eq!(
        r, "${OTHER:-$-}",
        "bare $- in fallback must not be corrupted to $$-"
    );

    // Empty prefix = substitute everything (same as None)
    let r = process("${OTHER}", &env, relaxed, false, false, Some("")).unwrap();
    assert_eq!(r, "secret");

    // Regression: strip-operator pattern variables that don't match the prefix
    // were silently used as literal glob patterns (e.g. "$OTHER" as a pattern),
    // which usually produces the original value unchanged but is semantically wrong.
    env.insert("APP_FILE".to_string(), "helloworld".to_string());
    env.insert("OTHER_PAT".to_string(), "hello".to_string());
    // OTHER_PAT does not have the APP_ prefix → entire expression must be verbatim
    let r = process(
        "${APP_FILE#$OTHER_PAT}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(
        r, "${APP_FILE#$OTHER_PAT}",
        "non-matching pattern variable must leave the whole PrefixStrip expression verbatim"
    );

    let r = process(
        "${APP_FILE%$OTHER_PAT}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(
        r, "${APP_FILE%$OTHER_PAT}",
        "non-matching pattern variable must leave the whole SuffixStrip expression verbatim"
    );

    // Matching pattern variable should still be evaluated normally
    env.insert("APP_PAT".to_string(), "hello".to_string());
    let r = process(
        "${APP_FILE#$APP_PAT}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(r, "world", "matching pattern variable should be evaluated");

    // Regression: substring offset/length variables that don't match the prefix
    // were being coerced to 0 (via parse failure → unwrap_or(0)) rather than
    // leaving the whole expression verbatim.
    env.insert("APP_WORD".to_string(), "helloworld".to_string());
    env.insert("OFFSET".to_string(), "5".to_string());
    // OFFSET does not have the APP_ prefix → entire ${APP_WORD:$OFFSET} must be verbatim
    let r = process(
        "${APP_WORD:$OFFSET}",
        &env,
        relaxed,
        false,
        false,
        Some("APP_"),
    )
    .unwrap();
    assert_eq!(
        r, "${APP_WORD:$OFFSET}",
        "non-matching index variable must leave the whole substring expression verbatim"
    );
}
#[test]
fn test_error_operator() {
    let mut env = std::collections::HashMap::new();
    env.insert("SET".to_string(), "yes".to_string());
    env.insert("EMPTY".to_string(), "".to_string());

    // ? on unset
    let err = process(
        "${UNSET?custom error msg}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "UNSET: custom error msg");

    // :? on unset
    let err = process(
        "${UNSET:?custom error msg}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "UNSET: custom error msg");

    // ? on empty
    let res = process(
        "${EMPTY?custom error msg}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(res, ""); // succeeds because it's set

    // :? on empty
    let err = process(
        "${EMPTY:?custom error msg}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "EMPTY: custom error msg");

    // ? on set
    let res = process(
        "${SET?custom error msg}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(res, "yes");

    // default messages
    let err = process(
        "${UNSET?}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "UNSET: parameter not set");

    let err = process(
        "${EMPTY:?}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "EMPTY: parameter null or not set");
}

#[test]
fn test_fail_fast_stops_inside_fallback() {
    // Regression for: fail_fast was ignored after a fallback expression itself errored.
    // ${NOTSET_A:-$NOTSET_B} $NOTSET_C — with require_values + fail_fast should
    // produce exactly one error (from the fallback), not two.
    let env = get_fake_env();
    let restrictions = Restrictions {
        require_values: true,
        require_nonempty_values: false,
    };
    let r = process(
        "${NOTSET_A:-$NOTSET_B} $NOTSET_C",
        &env,
        restrictions,
        false,
        true,
        None,
    );
    assert!(r.is_err());
    assert_eq!(
        r.unwrap_err().to_string().lines().count(),
        1,
        "fail_fast must stop after the first error inside a fallback"
    );
}

#[test]
fn test_underscore_var_substituted_when_set() {
    // Regression for: $_ and ${_} were left verbatim even when _ was in the env.
    let mut env = HashMap::new();
    env.insert("_".to_string(), "underscore_val".to_string());
    let r = process(
        "$_ and ${_}",
        &env,
        Restrictions::default(),
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(r, "underscore_val and underscore_val");
}

#[test]
fn test_deeply_nested_expression_errors_not_crashes() {
    // Regression: deeply nested ${A:-${A:-...}} used to stack-overflow the process.
    let env = get_fake_env();
    let nested: String = "${A:-".repeat(200) + "x" + &"}".repeat(200);
    let r = process(&nested, &env, Restrictions::default(), false, false, None);
    assert!(
        r.is_err(),
        "should return an error for excessive nesting depth"
    );
    assert!(
        r.unwrap_err().to_string().contains("nested too deeply"),
        "error message should mention nesting depth"
    );
}

#[test]
fn test_length_operator() {
    let mut env = HashMap::new();
    env.insert("WORD".to_string(), "hello".to_string());
    env.insert("EMPTY".to_string(), "".to_string());
    env.insert("UNICODE".to_string(), "héllo".to_string()); // 5 chars, 6 bytes
    let relaxed = Restrictions::default();

    let r = process("${#WORD}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "5");

    let r = process("${#EMPTY}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "0");

    // Unset → length 0
    let r = process("${#NOTSET}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "0");

    // Unicode: char count, not byte count
    let r = process("${#UNICODE}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "5");

    // Works inside larger template
    let r = process("len=${#WORD}!", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "len=5!");
}

#[test]
fn test_length_operator_malformed_and_no_digit() {
    // Regression: ${#VAR:-3} was silently treated as ${#VAR} (discarding :-3).
    // And ${#1} with --no-digit was outputting 0 instead of being left verbatim.
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "abc".to_string());
    let relaxed = Restrictions::default();

    // ${#FOO:-3} has trailing content after the name — emit verbatim.
    let r = process("${#FOO:-3}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(
        r, "${#FOO:-3}",
        "malformed length should be preserved verbatim"
    );

    // ${#1} with --no-digit: digit name rejected → emit verbatim.
    let r = process("${#1}", &env, relaxed, true, false, None).unwrap();
    assert_eq!(r, "${#1}", "--no-digit should leave ${{#1}} verbatim");

    // Well-formed ${#FOO} with --no-digit still works (name starts with letter).
    let r = process("${#FOO}", &env, relaxed, true, false, None).unwrap();
    assert_eq!(
        r, "3",
        "well-formed ${{#VAR}} with --no-digit should still work"
    );
}

#[test]
fn test_prefix_suffix_strip() {
    let mut env = HashMap::new();
    env.insert("PATH_VAR".to_string(), "/usr/local/bin/node".to_string());
    env.insert("FILENAME".to_string(), "file.tar.gz".to_string());
    env.insert("PLAIN".to_string(), "helloworld".to_string());
    let relaxed = Restrictions::default();

    // ${VAR#pat} — shortest prefix strip
    // "*/": shortest prefix ending in '/' is the first '/'
    let r = process("${PATH_VAR#*/}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "usr/local/bin/node");

    // ${VAR##pat} — longest prefix strip (path basename)
    let r = process("${PATH_VAR##*/}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "node");

    // ${VAR%pat} — shortest suffix strip
    let r = process("${FILENAME%.*}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "file.tar");

    // ${VAR%%pat} — longest suffix strip (strip all extensions)
    let r = process("${FILENAME%%.*}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "file");

    // Literal prefix/suffix
    let r = process("${PLAIN#hello}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "world");
    let r = process("${PLAIN%world}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "hello");

    // No match — return original value
    let r = process("${PLAIN#xyz}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "helloworld");

    // Unset variable — treat as empty string, return ""
    let r = process("${NOTSET#prefix}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "");

    // Regression: patterns containing variable references must be evaluated.
    // ${VAR#$PAT} should expand $PAT before using the result as a glob pattern.
    env.insert("PAT".to_string(), "*/".to_string());
    let r = process("${PATH_VAR#$PAT}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(
        r, "usr/local/bin/node",
        "${{PATH_VAR#$PAT}} should expand PAT before matching"
    );

    env.insert("SUFFIX_PAT".to_string(), ".*".to_string());
    let r = process("${FILENAME%$SUFFIX_PAT}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(
        r, "file.tar",
        "${{FILENAME%$SUFFIX_PAT}} should expand SUFFIX_PAT before matching"
    );
}

#[test]
fn test_substring_operator() {
    let mut env = HashMap::new();
    env.insert("WORD".to_string(), "helloworld".to_string());
    env.insert("UNICODE".to_string(), "héllo".to_string());
    let relaxed = Restrictions::default();

    // Basic offset without length
    let r = process("${WORD:5}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "world");

    // Offset and length
    let r = process("${WORD:0:5}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "hello");
    let r = process("${WORD:2:3}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "llo");

    // Offset past end → empty
    let r = process("${WORD:100}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "");

    // Length past end → clamp to end
    let r = process("${WORD:7:100}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "rld");

    // Zero offset, zero length → empty
    let r = process("${WORD:0:0}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "");

    // Unset variable → empty
    let r = process("${NOTSET:0:3}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "");

    // Unicode: offset counts chars not bytes
    let r = process("${UNICODE:1:3}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "éll");

    // Regression: offset/length must be evaluated, not treated as literal source text.
    // ${WORD:$OFFSET} with OFFSET=5 should produce "world", not "${WORD:$OFFSET}".
    env.insert("OFFSET".to_string(), "5".to_string());
    env.insert("LEN".to_string(), "3".to_string());

    let r = process("${WORD:$OFFSET}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(
        r, "world",
        "${{WORD:$OFFSET}} with OFFSET=5 should equal ${{WORD:5}}"
    );

    let r = process("${WORD:2:$LEN}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "llo", "literal offset with variable length");

    let r = process("${WORD:$OFFSET:$LEN}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "wor", "both offset and length as variables");

    // Non-numeric offset evaluates to 0 (bash treats bad arithmetic as error;
    // we fall back to 0 since we do not implement full arithmetic evaluation).
    env.insert("BADOFFSET".to_string(), "abc".to_string());
    let r = process("${WORD:$BADOFFSET}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "helloworld", "non-numeric offset treated as 0");
}

#[test]
fn test_stringmanip_operators_respect_restrictions() {
    // Regression: Length, PrefixStrip, SuffixStrip, Substring operators
    // unconditionally set `substituted = true`, bypassing the restriction checks
    // in the `if !substituted` block. They must run the same unset/empty checks.
    let mut env = HashMap::new();
    env.insert("SET".to_string(), "hello".to_string());
    env.insert("EMPTY".to_string(), "".to_string());
    let rv = Restrictions {
        require_values: true,
        require_nonempty_values: false,
    };
    let rn = Restrictions {
        require_values: false,
        require_nonempty_values: true,
    };

    // Length operator
    let r = process("${#NOTSET}", &env, rv, false, false, None);
    assert!(
        r.is_err(),
        "Length on unset var must fail with --require-values"
    );
    let r = process("${#NOTSET}", &env, rn, false, false, None);
    assert!(
        r.is_err(),
        "Length on unset var must fail with --require-nonempty-values"
    );
    let r = process("${#EMPTY}", &env, rn, false, false, None);
    assert!(
        r.is_err(),
        "Length on empty var must fail with --require-nonempty-values"
    );
    let r = process("${#SET}", &env, rv, false, false, None);
    assert!(
        r.is_ok(),
        "Length on set var must succeed with --require-values"
    );

    // PrefixStrip operator
    let r = process("${NOTSET#x}", &env, rv, false, false, None);
    assert!(
        r.is_err(),
        "PrefixStrip on unset var must fail with --require-values"
    );
    let r = process("${EMPTY#x}", &env, rn, false, false, None);
    assert!(
        r.is_err(),
        "PrefixStrip on empty var must fail with --require-nonempty-values"
    );
    let r = process("${SET#x}", &env, rv, false, false, None);
    assert!(
        r.is_ok(),
        "PrefixStrip on set var must succeed with --require-values"
    );

    // SuffixStrip operator
    let r = process("${NOTSET%x}", &env, rv, false, false, None);
    assert!(
        r.is_err(),
        "SuffixStrip on unset var must fail with --require-values"
    );
    let r = process("${EMPTY%x}", &env, rn, false, false, None);
    assert!(
        r.is_err(),
        "SuffixStrip on empty var must fail with --require-nonempty-values"
    );
    let r = process("${SET%x}", &env, rv, false, false, None);
    assert!(
        r.is_ok(),
        "SuffixStrip on set var must succeed with --require-values"
    );

    // Substring operator
    let r = process("${NOTSET:0:2}", &env, rv, false, false, None);
    assert!(
        r.is_err(),
        "Substring on unset var must fail with --require-values"
    );
    let r = process("${EMPTY:0:2}", &env, rn, false, false, None);
    assert!(
        r.is_err(),
        "Substring on empty var must fail with --require-nonempty-values"
    );
    let r = process("${SET:0:2}", &env, rv, false, false, None);
    assert!(
        r.is_ok(),
        "Substring on set var must succeed with --require-values"
    );
}

#[test]
fn test_unsupported_operator_preserved_verbatim() {
    // Operators that are not (yet) implemented must be preserved verbatim rather
    // than silently stripping the operator and substituting the raw variable value.
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "hello".to_string());
    let relaxed = Restrictions::default();

    // String replacement — not implemented
    let r = process("${FOO/hello/world}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "${FOO/hello/world}");

    // Case modification — not implemented
    let r = process("${FOO^^}", &env, relaxed, false, false, None).unwrap();
    assert_eq!(r, "${FOO^^}");
}
