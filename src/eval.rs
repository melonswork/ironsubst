use crate::ast::{Node, Operator};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("variable ${{{0}}} not set")]
    Unset(String),
    #[error("variable ${{{0}}} set but empty")]
    Empty(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Restrictions {
    pub require_explicit_values: bool,
    pub require_any_values: bool,
    pub require_nonempty_values: bool,
}

pub struct Env<'a> {
    env: &'a std::collections::HashMap<String, String>,
}

impl<'a> Env<'a> {
    pub fn new(env: &'a std::collections::HashMap<String, String>) -> Self {
        Self { env }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }
}

pub fn eval_nodes(
    nodes: &[Node],
    env: &Env,
    restrictions: Restrictions,
    fail_fast: bool,
) -> Result<String, Vec<EvalError>> {
    let mut result = String::new();
    let mut errors = Vec::new();

    for node in nodes {
        match node {
            Node::Text(t) => result.push_str(t),
            Node::Variable {
                name,
                operator,
                fallback,
            } => {
                let value = env.get(name);
                let is_set = value.is_some();
                let is_empty = value.map(|s: &String| s.is_empty()).unwrap_or(true);

                // Handle substitution operators
                let mut substituted = false;
                if let Some(op) = operator {
                    match op {
                        Operator::Default(colon) => {
                            if !is_set || (*colon && is_empty) {
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(fallback_nodes, env, restrictions, fail_fast) {
                                        Ok(s) => result.push_str(&s),
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }
                                substituted = true;
                            }
                        }
                        Operator::Assign(colon) => {
                            if !is_set || (*colon && is_empty) {
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(fallback_nodes, env, restrictions, fail_fast) {
                                        Ok(s) => result.push_str(&s),
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }
                                substituted = true;
                            }
                        }
                        Operator::Substitute(_colon) => {
                            // a8m treats `+` and `:+` as identical! (bug in their code)
                            // We must replicate their bug to pass their tests.
                            #[allow(clippy::collapsible_if)]
                            if is_set {
                                if let Some(fallback_nodes) = fallback {
                                    match eval_nodes(fallback_nodes, env, restrictions, fail_fast) {
                                        Ok(s) => result.push_str(&s),
                                        Err(mut e) => errors.append(&mut e),
                                    }
                                }
                            }
                            substituted = true;
                        }
                    }
                }

                if !substituted {
                    // This block executes if there was no substitution (or it's not a substitution operator).
                    // This means we are outputting the value of the variable.

                    if restrictions.require_explicit_values && !is_set {
                        errors.push(EvalError::Unset(name.clone()));
                        if fail_fast {
                            return Err(errors);
                        }
                    } else if restrictions.require_any_values && !is_set {
                        if operator.is_none() {
                            errors.push(EvalError::Unset(name.clone()));
                            if fail_fast {
                                return Err(errors);
                            }
                        }
                    } else if restrictions.require_nonempty_values && !is_set {
                        // nothing
                    }

                    if restrictions.require_nonempty_values && is_set && is_empty {
                        errors.push(EvalError::Empty(name.clone()));
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
