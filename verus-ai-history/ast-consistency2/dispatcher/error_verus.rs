    pub fn error(code: i32) -> (result: DispatchResult)
        ensures
            !result@.is_success,
            result@.value == code as int,
            result.wf(),
            result@ == spec_error_result(code as int),
    {
        DispatchResult { is_success: false, value: code as i64 }
    }
