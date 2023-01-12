use std::fmt::{Debug, Formatter};

pub struct TestError {
    pub(crate) text: String,
}

impl Debug for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test failure: {}", self.text)
    }
}
#[deprecated]
pub type TestResult = Result<(), TestError>;
