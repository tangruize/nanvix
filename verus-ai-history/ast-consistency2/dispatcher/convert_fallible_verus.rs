fn convert_fallible(outcome: FallibleOutcome) -> (result: DispatchResult)
    requires
        outcome.wf(),
    ensures
        result.wf(),
        outcome.succeeded ==> (result@.is_success && result@.value == outcome.value as int),
        !outcome.succeeded ==> (!result@.is_success && result@.value == outcome.error_code as int),
{
    if outcome.succeeded {
        DispatchResult::success(outcome.value)
    } else {
        DispatchResult::error(outcome.error_code)
    }
}
