use crate::ast::{Node, Operator};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("closing brace expected")]
    UnclosedBrace,
}

pub fn parse(input: &str, no_digit: bool) -> Result<Vec<Node>, ParseError> {
    let mut parser = Parser {
        input,
        pos: 0,
        no_digit,
    };
    parser.parse_nodes(false)
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

    fn parse_nodes(&mut self, in_braces: bool) -> Result<Vec<Node>, ParseError> {
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
                                    _ => {}
                                }
                            }

                            if !valid_op && colon {
                                // backtrack the colon since it wasn't part of a valid operator
                                self.pos = saved_pos;
                            }
                        }
                    }

                    // Anything inside braces after operator is fallback (or text if no operator)
                    let fallback = self.parse_nodes(true)?;

                    if self.next() != Some('}') {
                        return Err(ParseError::UnclosedBrace);
                    }

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

        if name == "_" {
            return String::new();
        }

        name
    }
}

fn is_alpha_numeric(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}
