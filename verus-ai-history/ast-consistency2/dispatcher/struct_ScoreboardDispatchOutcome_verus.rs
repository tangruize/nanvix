pub struct ScoreboardDispatchOutcome {
    /// Whether the scoreboard dispatch succeeded.
    pub succeeded: bool,
    /// The result success/error flag (meaningful only when `succeeded`).
    pub result_is_success: bool,
    /// The result value (meaningful only when `succeeded`).
    pub result_value: i64,
    /// The sleep error kind (meaningful only when `!succeeded`).
    pub sleep_error_kind: SleepErrorKind,
    /// The sleep error code (meaningful only when `!succeeded` and `Generic`).
    pub sleep_error_code: i64,
}
