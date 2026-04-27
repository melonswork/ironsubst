#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Default(bool),     // `-` / `:-`; bool = colon form (treats empty as unset)
    Assign(bool),      // `=` / `:=`; bool = colon form
    Substitute(bool),  // `+` / `:+`; bool = colon form
    Error(bool),       // `?` / `:?`; bool = colon form
    Length,            // ${#VAR}
    PrefixStrip(bool), // `#` / `##`; bool = greedy (longest match)
    SuffixStrip(bool), // `%` / `%%`; bool = greedy (longest match)
    Substring {
        offset: Vec<Node>,
        length: Option<Vec<Node>>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Text(String),
    Variable {
        name: String,
        /// `true` if the variable was written as `${VAR}`, `false` for bare `$VAR`.
        /// Used to produce error messages that match the original template syntax.
        braced: bool,
        operator: Option<Operator>,
        fallback: Option<Vec<Node>>,
    },
}
