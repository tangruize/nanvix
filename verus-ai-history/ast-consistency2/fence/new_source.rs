    pub const fn new(total: usize) -> Self {
        Self {
            count: AtomicUsize::new(0),
            total,
        }
    }
