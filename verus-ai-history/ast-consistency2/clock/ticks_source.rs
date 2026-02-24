pub fn ticks() -> u64 {
    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    ((major_ticks as u64) << 32) + (minor_ticks as u64)
}
