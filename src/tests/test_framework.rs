use std::fmt::{Debug, Formatter};

pub struct TestError {
    text: String,
}

impl Debug for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test failure: {}", self.text)
    }
}

pub type TestResult = Result<(), TestError>;

pub fn assert_equal<T: Eq + Debug>(expected: T, obtained: T, msg: &str) -> TestResult {
    if expected != obtained {
        return Err(TestError {
            text: format!(" Equality: {msg}\n\texpected:[{expected:?}]\n\tobtained:[{obtained:?}]"),
        });
    }
    Ok(())
}
