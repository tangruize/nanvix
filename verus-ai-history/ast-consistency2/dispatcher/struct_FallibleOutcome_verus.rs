pub struct FallibleOutcome {
    /// Whether the subsystem call succeeded.
    pub succeeded: bool,
    /// The success value (meaningful only when `succeeded`).
    pub value: i64,
    /// The error code (meaningful only when `!succeeded`).
    pub error_code: i32,
}
