pub fn classify_and_check_invalid(number: u32) -> (result: bool)
    ensures
        result == spec_returns_invalid_syscall(number),
{
    if number == 1 || number == 2 {
        true
    } else if number == 0 || number == 4 || number == 6 || number == 7
        || number == 8 || number == 10 || number == 11 || number == 12
        || number == 13 || number == 14 || number == 15 || number == 16
        || number == 17 || number == 18 || number == 19 || number == 21
        || number == 28 || number == 30 || number == 31 {
        false
    } else {
        true
    }
}
