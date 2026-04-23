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
    has_err_no_unset: bool,
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
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "env only",
            input: "$BAR",
            expected: "bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "with text",
            input: "$BAR baz",
            expected: "bar baz",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "concatenated",
            input: "$BAR$FOO",
            expected: "barfoo",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "2 env var",
            input: "$BAR - $FOO",
            expected: "bar - foo",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "invalid var",
            input: "$_ bar",
            expected: "$_ bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "invalid subst var",
            input: "${_} bar",
            expected: "${_} bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "value of $var",
            input: "${BAR}baz",
            expected: "barbaz",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set -",
            input: "${NOTSET-$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set =",
            input: "${NOTSET=$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var set but empty -",
            input: "${EMPTY-$BAR}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var set but empty =",
            input: "${EMPTY=$BAR}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set or empty :-",
            input: "${EMPTY:-$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set or empty :=",
            input: "${EMPTY:=$BAR}",
            expected: "bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var set evaluate expression as $other +",
            input: "${EMPTY+hello}",
            expected: "hello",
            has_err_relaxed: false,
            has_err_no_unset: false,
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
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var not set, use empty string +",
            input: "${NOTSET+hello}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "if $var not set, use empty string :+",
            input: "${NOTSET:+hello}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "multi line string",
            input: "hello $BAR\nhello ${EMPTY:=$FOO}",
            expected: "hello bar\nhello foo",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "issue #1",
            input: "${hello:=wo_rld} ${foo:=bar_baz}",
            expected: "wo_rld bar_baz",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "issue #2",
            input: "name: ${NAME:=foo_qux}, key: ${EMPTY:=baz_bar}",
            expected: "name: foo_qux, key: baz_bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-8",
            input: "prop=${HOME_URL-http://localhost:8080}",
            expected: "prop=http://localhost:8080",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-1",
            input: "${NOTSET--1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-2",
            input: "${NOTSET:--1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-3",
            input: "${NOTSET=-1}",
            expected: "-1",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-41-4",
            input: "${NOTSET:==1}",
            expected: "=1",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "gh-issue-43-1",
            input: "${A}",
            expected: "AAA",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "closing brace expected",
            input: "hello ${",
            expected: "",
            has_err_relaxed: true,
            has_err_no_unset: true,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set",
            input: "${NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var set to empty",
            input: "${EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "gh-issue-9",
            input: "$NOTSET",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "gh-issue-9-empty",
            input: "$EMPTY",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set -",
            input: "${NOTSET-$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set :-",
            input: "${NOTSET:-$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set =",
            input: "${NOTSET=$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT not set :=",
            input: "${NOTSET:=$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $OTHER not set +",
            input: "${NOTSET+$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var and $OTHER not set :+",
            input: "${NOTSET:+$ALSO_NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set -",
            input: "${EMPTY-$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set :-",
            input: "${EMPTY:-$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set =",
            input: "${EMPTY=$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $DEFAULT not set :=",
            input: "${EMPTY:=$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var empty and $OTHER not set +",
            input: "${EMPTY+$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: true,
            has_err_no_empty: false,
            has_err_strict: true,
        },
        ParseTest {
            // POSIX-correct: EMPTY is set but empty → :+ does NOT fire →
            // $NOTSET is never evaluated → no error regardless of restrictions.
            name: "$var empty and $OTHER not set :+",
            input: "${EMPTY:+$NOTSET}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty -",
            input: "${NOTSET-$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty :-",
            input: "${NOTSET:-$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty =",
            input: "${NOTSET=$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $DEFAULT empty :=",
            input: "${NOTSET:=$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var not set and $OTHER empty +",
            input: "${NOTSET+$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var not set and $OTHER empty :+",
            input: "${NOTSET:+$EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "$var and $DEFAULT empty -",
            input: "${EMPTY-$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty :-",
            input: "${EMPTY:-$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty =",
            input: "${EMPTY=$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $DEFAULT empty :=",
            input: "${EMPTY:=$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: true,
            has_err_strict: true,
        },
        ParseTest {
            name: "$var and $OTHER empty +",
            input: "${EMPTY+$ALSO_EMPTY}",
            expected: "",
            has_err_relaxed: false,
            has_err_no_unset: false,
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
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$var",
            input: "FOO $$BAR BAZ",
            expected: "FOO $BAR BAZ",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $${subst}",
            input: "FOO $${BAR} BAZ",
            expected: "FOO ${BAR} BAZ",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$$var",
            input: "$$$BAR",
            expected: "$bar",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
        ParseTest {
            name: "escape $$${subst}",
            input: "$$${BAZ:-baz}",
            expected: "$baz",
            has_err_relaxed: false,
            has_err_no_unset: false,
            has_err_no_empty: false,
            has_err_strict: false,
        },
    ]
}

fn do_test(restrictions: Restrictions, get_err: fn(&ParseTest) -> bool) {
    let env = get_fake_env();
    for test in get_tests() {
        let has_err = get_err(&test);
        let result = process(test.input, &env, restrictions, false, false);

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
            require_explicit_values: false,
            require_any_values: false,
            require_nonempty_values: false,
        },
        |t| t.has_err_relaxed,
    );
}

#[test]
fn test_parse_no_unset() {
    do_test(
        Restrictions {
            require_explicit_values: true,
            require_any_values: false,
            require_nonempty_values: false,
        },
        |t| t.has_err_no_unset,
    );
}

#[test]
fn test_parse_no_empty() {
    do_test(
        Restrictions {
            require_explicit_values: false,
            require_any_values: false,
            require_nonempty_values: true,
        },
        |t| t.has_err_no_empty,
    );
}

#[test]
fn test_parse_strict() {
    do_test(
        Restrictions {
            require_explicit_values: true,
            require_any_values: false,
            require_nonempty_values: true,
        },
        |t| t.has_err_strict,
    );
}
