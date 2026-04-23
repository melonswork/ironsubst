#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Default(bool),    // -, :- (bool is true if it has colon)
    Assign(bool),     // =, :=
    Substitute(bool), // +, :+
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Text(String),
    Variable {
        name: String,
        operator: Option<Operator>,
        fallback: Option<Vec<Node>>,
    },
}
