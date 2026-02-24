pub fn clock_now() -> (result: SystemTimeModel)
    ensures
        result.spec_wf(),
        result.nanoseconds < 1_000_000_000u32,
        spec_system_time_wf(result.spec_view()),
{
    unimplemented!()
}
