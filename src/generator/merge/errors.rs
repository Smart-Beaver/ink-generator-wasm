use std::error::Error;

#[derive(Debug)]
pub struct ComparisonError(String);
impl ComparisonError {
    pub fn new(message: &str) -> ComparisonError {
        ComparisonError(message.to_string())
    }
}
impl std::fmt::Display for ComparisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Comparison failed: {}", self.0)
    }
}
impl Error for ComparisonError {}
