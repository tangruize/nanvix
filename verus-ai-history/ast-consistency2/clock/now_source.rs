pub fn now() -> SystemTime {
    #[cfg(feature = "pit")]
    let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1;

    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    let seconds: u64 = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
    let nanoseconds: u32 = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);

    match SystemTime::new(seconds, nanoseconds) {
        Some(time) => time,
        None => {
            // SAFETY: This should not happen because `ticks` should be always in a valid range of `SystemTime`.
            unreachable!(
                "now(): failed to get system time (major_ticks={major_ticks:?}, \
                 minor_ticks={minor_ticks:?}, timer_freq={timer_freq:?})"
            )
        },
    }
}
