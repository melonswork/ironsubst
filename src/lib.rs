pub mod ast;
pub mod eval;
pub mod parser;

use std::collections::HashMap;

pub fn process(
    input: &str,
    env: &HashMap<String, String>,
    restrictions: eval::Restrictions,
    no_digit: bool,
    fail_fast: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let nodes = parser::parse(input, no_digit)?;
    let env_wrapper = eval::Env::new(env);

    match eval::eval_nodes(&nodes, &env_wrapper, restrictions, fail_fast) {
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
