    const fn new() -> Self {
        Self {
            minor: AtomicU32::new(0),
            major: AtomicU32::new(0),
        }
    }
