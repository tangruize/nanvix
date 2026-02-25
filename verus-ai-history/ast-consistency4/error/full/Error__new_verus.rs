    pub fn new(code: ErrorCode, reason: &'static str) -> (result: Error)
        ensures
            result.code == code,
            result.reason == reason,
    {
        Error { code, reason }
    }
