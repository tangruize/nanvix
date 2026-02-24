    fn drop(&mut self) {
        self.0 .0.store(false, Ordering::Release);
    }
