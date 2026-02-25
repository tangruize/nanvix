fn invalid_error_code(_value: i32) -> Error {
    Error {
        code: ErrorCode::InvalidArgument,
        reason: "invalid error code",
    }
}
