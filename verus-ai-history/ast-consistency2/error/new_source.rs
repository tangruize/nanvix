    pub fn new(code: ErrorCode, reason: &'static str) -> Self {
        Self { code, reason }
    }
