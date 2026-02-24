pub fn encode_result(result: &DispatchResult) -> (encoded: i64)
    ensures
        encoded as int == spec_encode_result(result@),
{
    result.value
}
