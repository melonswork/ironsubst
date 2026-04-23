#![no_main]

use ironsubst::eval::Restrictions;
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

// Fuzz the full process() pipeline (parse + eval) with all restriction combinations.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Build a small deterministic environment for evaluation
        let mut env = HashMap::new();
        env.insert("A".to_string(), "hello".to_string());
        env.insert("EMPTY".to_string(), String::new());

        // Try all restriction combinations
        for require_explicit in [false, true] {
            for require_any in [false, true] {
                for require_nonempty in [false, true] {
                    let restrictions = Restrictions {
                        require_explicit_values: require_explicit,
                        require_any_values: require_any,
                        require_nonempty_values: require_nonempty,
                    };
                    let _ = ironsubst::process(s, &env, restrictions, false, false, None);
                    let _ = ironsubst::process(s, &env, restrictions, true, true, None);
                }
            }
        }
    }
});
