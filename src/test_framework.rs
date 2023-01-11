use std::fmt::{Debug, Formatter};

pub struct TestError {
    pub(crate) text: String,
}

impl Debug for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test failure: {}", self.text)
    }
}

pub type TestResult = Result<(), TestError>;

pub fn assert_equal<T: PartialEq + Debug>(expected: T, obtained: T, msg: &str) -> TestResult {
    if expected != obtained {
        return Err(TestError {
            text: format!(" Equality: {msg}\n\texpected:[{expected:?}]\n\tobtained:[{obtained:?}]"),
        });
    }
    Ok(())
}
pub fn assert_not_equal<T: PartialEq + Debug>(control: T, obtained: T, msg: &str) -> TestResult {
    if control == obtained {
        return Err(TestError {
            text: format!(" Inequality: {msg}\n\tcontrol:[{control:?}]\n\tobtained:[{obtained:?}]"),
        });
    }
    Ok(())
}

pub fn assert_custom<T: Eq + Debug, F: FnOnce(&T, &T) -> bool>(
    control: T,
    obtained: T,
    msg: &str,
    check: F,
) -> TestResult {
    if !check(&control, &obtained) {
        return Err(TestError {
            text: format!(" Custom: {msg}\n\tcontrol:[{control:?}]\n\tobtained:[{obtained:?}]"),
        });
    }
    Ok(())
}
