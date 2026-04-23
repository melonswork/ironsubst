use crate::ast::{Node, Operator};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    // Note: the variable name stored here may be unbraced (e.g. "$VAR") or braced
    // (e.g. "${VAR}") — we preserve whichever form appeared in the template so the
    // error message matches what the user actually wrote.
    #[error("variable {0} not set")]
    Unset(String),
    #[error("variable {0} set but empty")]
    Empty(String),
    #[error("{0}: {1}")]
    Custom(String, String),
}

/// Controls which missing/empty variables are treated as errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct Restrictions {
    /// Fail if a variable is not explicitly set in the environment.
    /// When a fallback/default operator fires (e.g. `${X:-default}`) the
    /// restriction check is skipped because a value *is* being provided.
    pub require_explicit_values: bool,

    /// Fail if a variable is not set AND there is no fallback operator in the
    /// template expression (i.e. the variable appears as a bare `$VAR` or
    /// `${VAR}` with no `-`/`:-`/`=`/`:=` operator).
    pub require_any_values: bool,

    /// Fail if a variable is set to an empty string (or not set at all and no
    /// fallback fires).
    pub require_nonempty_values: bool,
}

/// Evaluate a parsed AST against the given environment snapshot.
///
/// Returns the substituted string on success, or a list of [`EvalError`]s on
/// failure. Errors accumulate unless `fail_fast` is set.
///
/// When `prefix` is `Some("FOO_")`, only variables whose names start with
/// `"FOO_"` are substituted; all others are left verbatim in the output
/// (e.g. `$BAR` stays as `$BAR`, `${BAR}` stays as `${BAR}`).
pub fn eval_nodes(
    nodes: &[Node],
    env: &HashMap<String, String>,
    restrictions: Restrictions,
    fail_fast: bool,
    prefix: Option<&str>,
) -> Result<String, Vec<EvalError>> {
    let mut result = String::new();
    let mut errors = Vec::new();

    for node in nodes {
        match node {
            Node::Text(t) => result.push_str(t),
            Node::Variable {
                name,
                braced,
                operator,
                fallback,
            } => {
                // If a prefix filter is active and this variable's name does not
                // match, emit the original source text verbatim and skip all
                // substitution and restriction checks.
                if let Some(pfx) = prefix {
                    if !name.starts_with(pfx) {
                        result.push_str(&original_text(name, *braced, operator, fallback));
                        continue;
                    }
                }

                let value = env.get(name.as_str());
                let is_set = value.is_some();
                let is_empty = value.map(|s: &String| s.is_empty()).unwrap_or(true);

                // Format the variable reference for error messages in the same
                // form the user wrote it (braced or unbraced).
                let display_name = if *braced {
                    format!("${{{name}}}")
                } else {
                    format!("${name}")
                };

                // Handle substitution operators
                let mut substituted = false;
                if let Some(op) = operator {
                    match op {
                        // Default (`-`, `:-`) and Assign (`=`, `:=`) behave identically
                        // for template substitution purposes: output the fallback value
                        // when the variable is unset (or empty when the colon form is used).
                        // Note: unlike real bash, `=` does NOT write back to the environment.
                        Operator::Default(colon) | Operator::Assign(colon) => {
                            if !is_set || (*colon && is_empty) {
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(
                                        fallback_nodes,
                                        env,
                                        restrictions,
                                        fail_fast,
                                        prefix,
                                    ) {
                                        Ok(s) => result.push_str(&s),
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }
                                substituted = true;
                            }
                        }
                        // Substitute (`+`, `:+`): output the alternate value when the
                        // variable IS set (and non-empty for the colon form).
                        // POSIX-correct behaviour:
                        //   `${VAR+alt}`  → alt if VAR is set (even if empty)
                        //   `${VAR:+alt}` → alt if VAR is set AND non-empty
                        Operator::Substitute(colon) => {
                            let fires = if *colon { is_set && !is_empty } else { is_set };
                            if fires {
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(
                                        fallback_nodes,
                                        env,
                                        restrictions,
                                        fail_fast,
                                        prefix,
                                    ) {
                                        Ok(s) => result.push_str(&s),
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }
                            }
                            substituted = true;
                        }
                        // Error (`?`, `:?`): exit with an error message when the
                        // variable is unset (or empty for the colon form).
                        Operator::Error(colon) => {
                            if !is_set || (*colon && is_empty) {
                                let mut err_msg = String::new();
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(
                                        fallback_nodes,
                                        env,
                                        restrictions,
                                        fail_fast,
                                        prefix,
                                    ) {
                                        Ok(s) => err_msg = s,
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }

                                if err_msg.is_empty() {
                                    err_msg = if *colon {
                                        "parameter null or not set".to_string()
                                    } else {
                                        "parameter not set".to_string()
                                    };
                                }

                                // Bash omits braces from the parameter name in the error output
                                let unbraced_name = display_name
                                    .trim_start_matches("${")
                                    .trim_start_matches('$')
                                    .trim_end_matches('}');
                                errors.push(EvalError::Custom(unbraced_name.to_string(), err_msg));

                                if fail_fast {
                                    return Err(errors);
                                }
                            } else if let Some(v) = value {
                                result.push_str(v);
                            }

                            // `?` either errors or explicitly substitutes the original value.
                            // Set `substituted = true` so the default value logic is bypassed,
                            // preventing duplicate error messages from strictness flags.
                            substituted = true;
                        }
                    }
                }

                if !substituted {
                    // No operator fired — we are outputting the raw variable value.

                    if restrictions.require_explicit_values && !is_set {
                        errors.push(EvalError::Unset(display_name.clone()));
                        if fail_fast {
                            return Err(errors);
                        }
                    } else if restrictions.require_any_values && !is_set && operator.is_none() {
                        // `require_any_values` only fires for bare variables with no
                        // fallback operator — if an operator was present but didn't fire
                        // (e.g. `${SET-alt}`) a value is still being produced.
                        errors.push(EvalError::Unset(display_name.clone()));
                        if fail_fast {
                            return Err(errors);
                        }
                    }

                    if restrictions.require_nonempty_values && is_set && is_empty {
                        errors.push(EvalError::Empty(display_name));
                        if fail_fast {
                            return Err(errors);
                        }
                    }

                    if let Some(v) = value {
                        result.push_str(v);
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors)
    }
}

/// Reconstruct the original source text of a variable node so it can be
/// emitted verbatim when the prefix filter rejects it.
///
/// This is a best-effort reconstruction: the AST does not store raw source
/// bytes, so we rebuild the canonical form.  For the common cases (`$VAR`,
/// `${VAR}`, `${VAR:-default}`) the output is identical to the input.
fn original_text(
    name: &str,
    braced: bool,
    operator: &Option<Operator>,
    fallback: &Option<Vec<Node>>,
) -> String {
    if !braced {
        // Unbraced variables never have operators or fallbacks.
        return format!("${name}");
    }

    let op_str = match operator {
        None => String::new(),
        Some(Operator::Default(colon)) => format!("{}-", if *colon { ":" } else { "" }),
        Some(Operator::Assign(colon)) => format!("{}=", if *colon { ":" } else { "" }),
        Some(Operator::Substitute(colon)) => format!("{}+", if *colon { ":" } else { "" }),
        Some(Operator::Error(colon)) => format!("{}?", if *colon { ":" } else { "" }),
    };

    let fallback_str = match fallback {
        None => String::new(),
        Some(nodes) => nodes_to_text(nodes),
    };

    format!("${{{name}{op_str}{fallback_str}}}")
}

/// Recursively convert AST nodes back to their source text representation.
fn nodes_to_text(nodes: &[Node]) -> String {
    use crate::ast::Node;
    nodes
        .iter()
        .map(|n| match n {
            Node::Text(t) => t.clone(),
            Node::Variable {
                name,
                braced,
                operator,
                fallback,
            } => original_text(name, *braced, operator, fallback),
        })
        .collect()
}
