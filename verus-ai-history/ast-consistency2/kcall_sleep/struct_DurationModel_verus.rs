pub struct DurationModel {
    /// Whole seconds.
    pub seconds: u64,
    /// Fractional nanoseconds (< 1_000_000_000).
    pub nanoseconds: u32,
}
