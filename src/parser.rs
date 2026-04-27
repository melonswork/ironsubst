use crate::ast::{Node, Operator};

const MAX_DEPTH: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("closing brace expected")]
    UnclosedBrace,
    #[error("expression nested too deeply (max {} levels)", MAX_DEPTH)]
    TooDeep,
}

pub fn parse(input: &str, no_digit: bool) -> Result<Vec<Node>, ParseError> {
    let mut parser = Parser {
        input,
        pos: 0,
        no_digit,
    };
    parser.parse_nodes(false, 0)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    no_digit: bool,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    fn parse_nodes(&mut self, in_braces: bool, depth: usize) -> Result<Vec<Node>, ParseError> {
        if depth > MAX_DEPTH {
            return Err(ParseError::TooDeep);
        }
        let mut nodes = Vec::new();
        let mut text_buf = String::new();

        while let Some(c) = self.peek() {
            if in_braces && c == '}' {
                break;
            }

            if c == '$' {
                let start_pos = self.pos;
                self.next(); // consume '$'

                // Collapsing into `if self.peek() == Some('$')` would silently
                // consume the `$` even when `peek()` returns `None`, and loses
                // the explicit `next_c` binding used by both conditions.
                #[allow(clippy::collapsible_if)]
                if let Some(next_c) = self.peek() {
                    if next_c == '$' {
                        self.next();
                        text_buf.push('$');
                        continue;
                    }
                }

                let mut is_braced = false;
                if self.peek() == Some('{') {
                    self.next();
                    is_braced = true;
                }

                // ${#VAR}: '#' before the name means "length of VAR".
                // Only treat it as the length operator when '#' is followed by a
                // valid identifier start; otherwise fall through to the empty-name
                // path which emits `${` verbatim (handles `${#}`, `${# }`, etc.).
                if is_braced && self.peek() == Some('#') {
                    let after_hash = self.pos + '#'.len_utf8();
                    let next_is_ident = self.input[after_hash..]
                        .chars()
                        .next()
                        .is_some_and(is_alpha_numeric);
                    if next_is_ident {
                        self.next(); // consume '#'
                        let name = self.read_var_name();
                        if !text_buf.is_empty() {
                            nodes.push(Node::Text(std::mem::take(&mut text_buf)));
                        }
                        // If the name is empty (--no-digit rejected a digit start) or
                        // there is trailing content before '}' (e.g. ${#FOO:-3}),
                        // the expression is malformed — emit it verbatim.
                        if name.is_empty() || self.peek() != Some('}') {
                            let _ = self.parse_nodes(true, depth + 1)?;
                            if self.next() != Some('}') {
                                return Err(ParseError::UnclosedBrace);
                            }
                            nodes.push(Node::Text(self.input[start_pos..self.pos].to_string()));
                        } else {
                            if self.next() != Some('}') {
                                return Err(ParseError::UnclosedBrace);
                            }
                            nodes.push(Node::Variable {
                                name,
                                braced: true,
                                operator: Some(Operator::Length),
                                fallback: None,
                            });
                        }
                        continue;
                    }
                }

                let name = self.read_var_name();

                if name.is_empty() {
                    if is_braced && self.peek().is_none() {
                        return Err(ParseError::UnclosedBrace);
                    }
                    let slice = &self.input[start_pos..self.pos];
                    text_buf.push_str(slice);
                    continue;
                }

                if !text_buf.is_empty() {
                    nodes.push(Node::Text(std::mem::take(&mut text_buf)));
                }

                if is_braced {
                    let mut operator = None;
                    let mut has_unsupported_op = false;

                    // Keep the two-level `if` for readability: the outer check
                    // guards against end-of-input, the inner check tests the
                    // specific character.  Collapsing them obscures the logic.
                    #[allow(clippy::collapsible_if)]
                    if let Some(op_char) = self.peek() {
                        if op_char != '}' {
                            let mut colon = false;
                            let saved_pos = self.pos;
                            if op_char == ':' {
                                self.next();
                                colon = true;
                            }

                            let mut valid_op = false;
                            if let Some(op_type) = self.peek() {
                                match op_type {
                                    '-' => {
                                        self.next();
                                        operator = Some(Operator::Default(colon));
                                        valid_op = true;
                                    }
                                    '=' => {
                                        self.next();
                                        operator = Some(Operator::Assign(colon));
                                        valid_op = true;
                                    }
                                    '+' => {
                                        self.next();
                                        operator = Some(Operator::Substitute(colon));
                                        valid_op = true;
                                    }
                                    '?' => {
                                        self.next();
                                        operator = Some(Operator::Error(colon));
                                        valid_op = true;
                                    }
                                    '#' if !colon => {
                                        self.next(); // first '#'
                                        let greedy = self.peek() == Some('#');
                                        if greedy {
                                            self.next(); // second '#'
                                        }
                                        operator = Some(Operator::PrefixStrip(greedy));
                                        valid_op = true;
                                    }
                                    '%' if !colon => {
                                        self.next(); // first '%'
                                        let greedy = self.peek() == Some('%');
                                        if greedy {
                                            self.next(); // second '%'
                                        }
                                        operator = Some(Operator::SuffixStrip(greedy));
                                        valid_op = true;
                                    }
                                    // ${VAR:N} / ${VAR:N:M} — substring. Only fires
                                    // when the colon was already consumed and the
                                    // next char is a digit. `:-` / `:=` / `:+` / `:?`
                                    // are handled by the arms above and are unambiguous.
                                    _ if colon && op_type.is_ascii_digit() => {
                                        let offset = self.read_integer();
                                        let length = if self.peek() == Some(':') {
                                            self.next();
                                            Some(self.read_integer())
                                        } else {
                                            None
                                        };
                                        operator = Some(Operator::Substring { offset, length });
                                        valid_op = true;
                                    }
                                    _ => {}
                                }
                            }

                            if !valid_op {
                                if colon {
                                    // backtrack the colon since it wasn't part of a valid operator
                                    self.pos = saved_pos;
                                }
                                has_unsupported_op = true;
                            }
                        }
                    }

                    // Consume the brace content to advance past the closing brace.
                    let fallback = self.parse_nodes(true, depth + 1)?;

                    if self.next() != Some('}') {
                        return Err(ParseError::UnclosedBrace);
                    }

                    if has_unsupported_op {
                        // Unrecognised operator (e.g. `${FOO#prefix}`, `${FOO%suffix}`,
                        // `${FOO:0:5}`) — emit verbatim so the expression is preserved.
                        nodes.push(Node::Text(self.input[start_pos..self.pos].to_string()));
                    } else {
                        nodes.push(Node::Variable {
                            name,
                            braced: true,
                            operator,
                            fallback: if fallback.is_empty() {
                                None
                            } else {
                                Some(fallback)
                            },
                        });
                    }
                } else {
                    nodes.push(Node::Variable {
                        name,
                        braced: false,
                        operator: None,
                        fallback: None,
                    });
                }
            } else {
                text_buf.push(self.next().unwrap());
            }
        }

        if !text_buf.is_empty() {
            nodes.push(Node::Text(text_buf));
        }

        Ok(nodes)
    }

    fn read_integer(&mut self) -> usize {
        let mut n: usize = 0;
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.next();
            n = n
                .saturating_mul(10)
                .saturating_add(c as usize - '0' as usize);
        }
        n
    }

    fn read_var_name(&mut self) -> String {
        let mut name = String::new();

        if let Some(c) = self.peek() {
            if self.no_digit && c.is_ascii_digit() {
                return name;
            }
            if !is_alpha_numeric(c) {
                return name;
            }
            self.next();
            name.push(c);
        }

        while let Some(c) = self.peek() {
            if !is_alpha_numeric(c) {
                break;
            }
            self.next();
            name.push(c);
        }

        name
    }
}

fn is_alpha_numeric(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}
