    pub fn reference_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
