pub fn make_invalid_syscall_error() -> (result: HandlerKcallResult)
    ensures
        result.is_error,
        result.error_code == SPEC_ERROR_INVALID_SYSCALL() as i32,
{
    HandlerKcallResult {
        is_error: true,
        error_code: 88i32,
    }
}
