    pub fn ok() -> (result: DispatchResult)
        ensures
            result@.is_success,
            result@.value == 0,
            result.wf(),
            result@ == spec_ok_result(),
    {
        DispatchResult { is_success: true, value: 0 }
    }
