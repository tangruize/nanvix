    pub fn new(value: usize) -> Self {
        Self {
            value: AtomicUsize::new(value),
            sleeping: Condvar::new(),
        }
    }
