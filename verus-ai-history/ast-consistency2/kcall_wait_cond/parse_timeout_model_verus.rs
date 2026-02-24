pub fn parse_timeout_model(timeout_s: u32, timeout_ns: u32) -> (result: (bool, bool))
    ensures
        result.0 == spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
        result.0 ==> (result.1 == spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat)),
        !result.0 ==> !result.1,
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        (true, false)
    } else if timeout_ns < 1_000_000_000u32 {
        (true, true)
    } else {
        (false, false)
    }
}
