    pub fn signal(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
