pub fn standalone_now(timer: &TimerTicks, timer_freq: u32) -> (result: (u64, u32))
    requires
        timer_freq > 0,
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result.0 as nat == timer@.ticks / timer_freq as nat,
        result.1 as nat == timer@.nanoseconds(timer_freq as nat),
        TimerTicks::spec_nanoseconds_valid(result.1 as nat),
        result.1 < 1_000_000_000u32,
        TimerTicks::spec_system_time_new_succeeds(result.1 as nat),
{
    timer.now(timer_freq)
}
