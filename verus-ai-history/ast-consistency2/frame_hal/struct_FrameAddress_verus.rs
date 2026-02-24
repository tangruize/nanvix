pub struct FrameAddress {
    /// NOTE: Field is pub for cross-module struct literal construction.
    /// Per methodology Step 1, new callers should use from_frame_number() instead.
    pub raw_addr: usize,
}
