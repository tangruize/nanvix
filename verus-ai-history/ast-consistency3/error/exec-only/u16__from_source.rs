    fn from(errno: ErrorCode) -> Self {
        errno as u16
    }
