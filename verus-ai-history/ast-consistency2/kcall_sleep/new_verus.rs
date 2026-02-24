    pub fn new(seconds: u64, nanoseconds: u32) -> (result: Self)
        requires
            nanoseconds < 1_000_000_000u32,
        ensures
            result.seconds == seconds,
            result.nanoseconds == nanoseconds,
            result.spec_wf(),
    {
        SystemTimeModel { seconds, nanoseconds }
    }
