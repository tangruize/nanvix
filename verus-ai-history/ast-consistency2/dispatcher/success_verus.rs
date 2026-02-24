    pub fn success(value: i64) -> (result: DispatchResult)
        ensures
            result@.is_success,
            result@.value == value as int,
            result.wf(),
            result@ == spec_success_result(value as int),
    {
        DispatchResult { is_success: true, value }
    }
