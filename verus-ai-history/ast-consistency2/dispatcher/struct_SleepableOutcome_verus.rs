pub struct SleepableOutcome {
    /// Whether the subsystem call succeeded.
    pub succeeded: bool,
    /// The success value (meaningful only when `succeeded`).
    pub value: i64,
    /// The sleep error kind (meaningful only when `!succeeded`).
    pub sleep_error_kind: SleepErrorKind,
    /// The sleep error code (meaningful only when `!succeeded` and `Generic`).
    pub sleep_error_code: i64,
}
