pub struct ScoreBoardPollResult {
    /// Whether a kcall was found.
    pub has_call: bool,
    /// The kcall number (meaningful only when `has_call` is true).
    pub kcall_number: u32,
    /// Whether a scoreboard access error occurred (unreachable in practice).
    pub has_error: bool,
}
