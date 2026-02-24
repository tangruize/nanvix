pub fn now_fallback_model(timer: &TimerTicks) -> (result: (u64, u32))
    requires
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result.0 as nat == timer@.ticks / 1nat,
        result.1 as nat == timer@.nanoseconds(1nat),
        TimerTicks::spec_nanoseconds_valid(result.1 as nat),
        result.1 < 1_000_000_000u32,
        TimerTicks::spec_system_time_new_succeeds(result.1 as nat),
{
    // #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1u32;
    timer.now(timer_freq)
}
