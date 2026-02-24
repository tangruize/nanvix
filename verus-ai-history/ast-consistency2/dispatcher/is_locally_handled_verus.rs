pub fn is_locally_handled(number: u32) -> (result: bool)
    ensures
        result == spec_is_locally_handled(number),
{
    let category: DispatchCategory = classify_kcall_number(number);
    !matches!(category, DispatchCategory::Remote)
}
