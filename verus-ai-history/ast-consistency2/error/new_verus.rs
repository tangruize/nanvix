    pub fn new(code: ErrorCode, reason: &'static str) -> (result: Self)
        ensures
            result.code == code,
            result.reason == reason,
    {
        Self { code, reason }
    }
