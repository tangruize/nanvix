    pub fn generic(error_code: i32) -> (result: SleepError)
        ensures
            result.spec_kind() =~= SleepErrorKind::Generic,
            result.spec_error_code() == error_code as int,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::Generic, error_code: error_code as i64 }
    }
