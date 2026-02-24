pub struct FrameNumber {
    /// NOTE: Field is pub for cross-module struct literal construction.
    /// Per methodology Step 1, new callers should use from_raw_value() instead.
    pub value: usize,
}
