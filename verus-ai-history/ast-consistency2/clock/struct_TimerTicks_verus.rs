pub struct TimerTicks {
    /// Low 32 bits of the tick counter.
    pub minor: u32,
    /// High 32 bits of the tick counter.
    pub major: u32,
}
