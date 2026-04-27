#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Default(bool),     // -, :- (bool is true if it has colon)
    Assign(bool),      // =, :=
    Substitute(bool),  // +, :+
    Error(bool),       // ?, :?
    Length,            // ${#VAR}
    PrefixStrip(bool), // #, ## (bool = greedy/longest)
    SuffixStrip(bool), // %, %% (bool = greedy/longest)
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
