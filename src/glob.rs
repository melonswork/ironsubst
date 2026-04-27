/// Returns `true` if `s` matches shell glob `pattern`.
///
/// Supported metacharacters:
///   `*`       — any sequence of characters (including empty)
///   `?`       — exactly one character
///   `[abc]`   — character class (literal members)
///   `[a-z]`   — character class (range)
///   `[!abc]`  / `[^abc]` — negated character class
///
/// All other characters are matched literally. An unclosed `[` is treated as
/// a literal `[`.
pub fn glob_matches(pattern: &str, s: &str) -> bool {
    glob_rec(pattern, s)
}

/// Strip the shortest (greedy=false) or longest (greedy=true) prefix of `s`
/// that fully matches `pattern`. Returns the remaining suffix, or `s` unchanged
/// if no prefix matches.
pub fn strip_prefix<'a>(s: &'a str, pattern: &str, greedy: bool) -> &'a str {
    // Collect byte positions of each char boundary (including 0 and s.len()).
    let positions: Vec<usize> = std::iter::once(0)
        .chain(s.char_indices().map(|(i, c)| i + c.len_utf8()))
        .collect();

    if greedy {
        for &pos in positions.iter().rev() {
            if glob_matches(pattern, &s[..pos]) {
                return &s[pos..];
            }
        }
    } else {
        for &pos in positions.iter() {
            if glob_matches(pattern, &s[..pos]) {
                return &s[pos..];
            }
        }
    }

    s
}

/// Strip the shortest (greedy=false) or longest (greedy=true) suffix of `s`
/// that fully matches `pattern`. Returns the remaining prefix, or `s` unchanged
/// if no suffix matches.
pub fn strip_suffix<'a>(s: &'a str, pattern: &str, greedy: bool) -> &'a str {
    // Suffix start positions from rightmost (shortest suffix) to leftmost (longest).
    let positions: Vec<usize> = std::iter::once(s.len())
        .chain(s.char_indices().rev().map(|(i, _)| i))
        .collect();

    if greedy {
        for &pos in positions.iter().rev() {
            if glob_matches(pattern, &s[pos..]) {
                return &s[..pos];
            }
        }
    } else {
        for &pos in positions.iter() {
            if glob_matches(pattern, &s[pos..]) {
                return &s[..pos];
            }
        }
    }

    s
}

fn glob_rec(p: &str, s: &str) -> bool {
    if p.is_empty() {
        return s.is_empty();
    }

    let (p_char, p_rest) = split_first(p);

    match p_char {
        '*' => {
            // Match zero chars first (most efficient for the non-greedy case).
            if glob_rec(p_rest, s) {
                return true;
            }
            let mut pos = 0;
            while pos < s.len() {
                let c = s[pos..].chars().next().unwrap();
                pos += c.len_utf8();
                if glob_rec(p_rest, &s[pos..]) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if s.is_empty() {
                return false;
            }
            let (_, s_rest) = split_first(s);
            glob_rec(p_rest, s_rest)
        }
        '[' => {
            if s.is_empty() {
                return false;
            }
            let (sc, s_rest) = split_first(s);
            match match_class(p_rest, sc) {
                Some((ok, after_bracket)) => ok && glob_rec(after_bracket, s_rest),
                None => sc == '[' && glob_rec(p_rest, s_rest),
            }
        }
        pc => {
            if s.is_empty() {
                return false;
            }
            let (sc, s_rest) = split_first(s);
            pc == sc && glob_rec(p_rest, s_rest)
        }
    }
}

/// Split `s` at the first char boundary. Panics if `s` is empty.
fn split_first(s: &str) -> (char, &str) {
    let c = s.chars().next().unwrap();
    (c, &s[c.len_utf8()..])
}

/// Parse a bracket expression starting just after the opening `[`.
/// Returns `Some((matched, rest_of_pattern))` on success, or `None` if unclosed.
fn match_class(body: &str, target: char) -> Option<(bool, &str)> {
    let (negated, mut s) = if body.starts_with('!') || body.starts_with('^') {
        (true, &body[1..])
    } else {
        (false, body)
    };

    let mut matched = false;

    while !s.is_empty() {
        let (ch, after_ch) = split_first(s);

        if ch == ']' {
            return Some((if negated { !matched } else { matched }, after_ch));
        }

        // Check for a-z range syntax: current char, '-', end char (end != ']').
        if let Some(after_dash) = after_ch.strip_prefix('-') {
            if let Some((end_ch, after_end)) = after_dash
                .chars()
                .next()
                .filter(|&c| c != ']')
                .map(|c| (c, &after_dash[c.len_utf8()..]))
            {
                if target >= ch && target <= end_ch {
                    matched = true;
                }
                s = after_end;
                continue;
            }
        }

        if ch == target {
            matched = true;
        }
        s = after_ch;
    }

    None // unclosed bracket
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_match() {
        assert!(glob_matches("hello", "hello"));
        assert!(!glob_matches("hello", "world"));
        assert!(!glob_matches("hello", "hell"));
        assert!(!glob_matches("hell", "hello"));
    }

    #[test]
    fn star_wildcard() {
        assert!(glob_matches("*", "anything"));
        assert!(glob_matches("*", ""));
        assert!(glob_matches("*.rs", "main.rs"));
        assert!(!glob_matches("*.rs", "main.py"));
        assert!(glob_matches("foo*bar", "fooXYZbar"));
        assert!(glob_matches("*/", "/usr/local/"));
        assert!(!glob_matches("*/", "/usr/local/bin"));
    }

    #[test]
    fn question_wildcard() {
        assert!(glob_matches("?", "a"));
        assert!(!glob_matches("?", ""));
        assert!(!glob_matches("?", "ab"));
        assert!(glob_matches("f?o", "foo"));
        assert!(glob_matches("f?o", "fXo"));
    }

    #[test]
    fn char_class() {
        assert!(glob_matches("[abc]", "a"));
        assert!(glob_matches("[abc]", "b"));
        assert!(!glob_matches("[abc]", "d"));
        assert!(glob_matches("[a-z]", "m"));
        assert!(!glob_matches("[a-z]", "M"));
        assert!(glob_matches("[!abc]", "d"));
        assert!(!glob_matches("[!abc]", "a"));
        assert!(glob_matches("[^abc]", "d"));
        assert!(glob_matches("[!a-z]", "M")); // negated range: uppercase passes
        assert!(!glob_matches("[!a-z]", "m")); // negated range: lowercase fails
        assert!(glob_matches("[^a-z]", "M")); // ^ synonym for !
        assert!(glob_matches("[abc", "[abc")); // unclosed bracket treated as literal '['
    }

    #[test]
    fn strip_prefix_shortest() {
        // */  matches the shortest prefix ending in '/': just the first '/'
        assert_eq!(strip_prefix("/usr/local/bin", "*/", false), "usr/local/bin");
        assert_eq!(strip_prefix("file.tar.gz", "file", false), ".tar.gz");
        assert_eq!(strip_prefix("hello", "xyz", false), "hello"); // no match
        assert_eq!(strip_prefix("hello", "*", false), "hello"); // * matches empty → strip nothing
    }

    #[test]
    fn strip_prefix_longest() {
        assert_eq!(strip_prefix("/usr/local/bin", "*/", true), "bin");
        assert_eq!(strip_prefix("hello", "*", true), ""); // * matches all
    }

    #[test]
    fn strip_suffix_shortest() {
        assert_eq!(strip_suffix("file.tar.gz", ".*", false), "file.tar");
        assert_eq!(strip_suffix("file.tar.gz", ".gz", false), "file.tar");
        assert_eq!(strip_suffix("hello", "xyz", false), "hello");
    }

    #[test]
    fn strip_suffix_longest() {
        assert_eq!(strip_suffix("file.tar.gz", ".*", true), "file");
        assert_eq!(strip_suffix("hello", "*", true), ""); // * matches all
    }
}
