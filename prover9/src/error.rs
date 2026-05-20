#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InteriorNul,
    Parse(String),
    TooManyVariables { var: usize, max: usize },
    InvalidRewriteRule(String),
}
