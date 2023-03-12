impl ShowError {
    /// Create an default error message to be shown to the user.
    ///
    ///
    pub(crate) fn new(err: String) -> Self {
        Self {
            error: err,
            context: "An error occurred".to_string(),
        }
    }

    /// Create an error message to be shown to the user. customize the context
    ///
    ///
    #[allow(dead_code)]
    pub(crate) fn new_custom_context(err: String, context: String) -> Self {
        Self {
            error: err,
            context,
        }
    }
}

/// Holds state for an error message to show to the user, and provides a feedback mechanism for the
/// user to make a decision on how to handle the error.
#[derive(Debug, Clone)]
pub struct ShowError {
    /// The error message.
    pub error: String,
    /// Simple description for the user
    pub context: String,
}
