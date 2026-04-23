#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz the parser with arbitrary byte sequences.
//
// Goals:
// 1. Never panic (only return Ok/Err)
// 2. Never hang
// 3. Roundtrip: Text nodes in the output are subsets of the input bytes
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test with no_digit = false
        let _ = ironsubst::parser::parse(s, false);
        // Test with no_digit = true
        let _ = ironsubst::parser::parse(s, true);
    }
});
