    pub fn new() -> Self {
        Self(Arc::new(MutexInner {
            locked: AtomicBool::new(false),
            sleeping: Condvar::new(),
        }))
    }
