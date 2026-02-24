pub struct SleepError {
    /// The kind of sleep error.
    pub kind: SleepErrorKind,
    /// The error code (meaningful only for Generic kind).
    pub error_code: i64,
}
