pub struct HandlerKcallResult {
    /// Whether this result is an error.
    pub is_error: bool,
    /// The error code value (meaningful only when `is_error` is true).
    pub error_code: i32,
}
