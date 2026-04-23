#![deny(unsafe_code)]

pub mod ast;
pub mod eval;
pub mod parser;

use std::collections::HashMap;

/// Process a template string, substituting environment variables.
///
/// # Arguments
/// * `input`        — The template string (may contain `$VAR`, `${VAR}`, `${VAR:-default}`, etc.)
/// * `env`          — A snapshot of the environment (typically from `std::env::vars().collect()`)
/// * `restrictions` — Controls which missing/empty variables are treated as errors
/// * `no_digit`     — When `true`, skip variables whose name starts with a digit (e.g. `$1`)
/// * `fail_fast`    — When `true`, return on the first error instead of collecting all errors
///
/// # Errors
/// Returns an error string listing all variable errors if any restrictions are violated,
/// or if the template contains a parse error (e.g. unclosed `${`).
///
/// # Examples
/// ```
/// use ironsubst::{eval::Restrictions, process};
/// use std::collections::HashMap;
///
/// let mut env = HashMap::new();
/// env.insert("NAME".to_string(), "world".to_string());
///
/// let result = process("Hello ${NAME}!", &env, Restrictions::default(), false, false).unwrap();
/// assert_eq!(result, "Hello world!");
/// ```
///
/// ```
/// use ironsubst::{eval::Restrictions, process};
/// use std::collections::HashMap;
///
/// let env = HashMap::new(); // NAME not set
/// let result = process("Hello ${NAME:-stranger}!", &env, Restrictions::default(), false, false).unwrap();
/// assert_eq!(result, "Hello stranger!");
/// ```
pub fn process(
    input: &str,
    env: &HashMap<String, String>,
    restrictions: eval::Restrictions,
    no_digit: bool,
    fail_fast: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let nodes = parser::parse(input, no_digit)?;

    match eval::eval_nodes(&nodes, env, restrictions, fail_fast) {
        Ok(result) => Ok(result),
        Err(errors) => {
            let msg = errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            Err(msg.into())
        }
    }
}
