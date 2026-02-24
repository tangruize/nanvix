pub fn is_sleepable(number: u32) -> (result: bool)
    ensures
        result == spec_is_sleepable(number),
{
    let category: DispatchCategory = classify_kcall_number(number);
    matches!(category, DispatchCategory::LocalSleepable)
}
