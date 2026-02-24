pub fn parse_timeout(timeout_s: u32, timeout_ns: u32) -> (result: Result<bool, LockMutexResultModel>)
    ensures
        // Infinite timeout case.
        spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            ==> result == Ok::<bool, LockMutexResultModel>(false),
        // Valid finite timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && spec_timeout_ns_valid(timeout_ns as nat))
            ==> result == Ok::<bool, LockMutexResultModel>(true),
        // Invalid timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && !spec_timeout_ns_valid(timeout_ns as nat))
            ==> result.is_err(),
        // Error case produces correct error code.
        result matches Err(err) ==> err.spec_view() == (LockMutexResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        }),
        // Result matches spec_parse_timeout.
        result.is_ok() <==> spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        // Both MAX → infinite timeout (None in original).
        Ok(false)
    } else {
        // Try to construct SystemTime.
        let st: Option<SystemTimeModel> = system_time_new(timeout_s as u64, timeout_ns);
        match st {
            Some(_) => Ok(true),
            None => {
                proof {
                    assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
                }
                Err(LockMutexResultModel::InvalidTimeoutError { error_code: 22i32 })
            },
        }
    }
}
