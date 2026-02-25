    pub fn new(code: ErrorCode, reason: &'static str) -> Error
    {
        Error { code, reason }
    }
