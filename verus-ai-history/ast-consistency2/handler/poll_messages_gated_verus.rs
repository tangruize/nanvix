pub fn poll_messages_gated(stdio_enabled: bool) -> (result: bool)
    ensures
        !stdio_enabled ==> !result,
{
    if stdio_enabled {
        poll_messages_raw()
    } else {
        false
    }
}
