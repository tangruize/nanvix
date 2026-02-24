pub fn system_time_new(seconds: u64, nanoseconds: u32) -> (result: Option<SystemTimeModel>)
    ensures
        nanoseconds < 1_000_000_000u32 ==> result.is_some(),
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().seconds == seconds,
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().nanoseconds == nanoseconds,
        nanoseconds >= 1_000_000_000u32 ==> result.is_none(),
{
    if nanoseconds >= 1_000_000_000u32 {
        None
    } else {
        Some(SystemTimeModel { seconds, nanoseconds })
    }
}
