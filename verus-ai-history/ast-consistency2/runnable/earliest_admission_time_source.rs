    pub fn earliest_admission_time(&self) -> SystemTime {
        self.ready_threads
            .iter()
            .map(|thread| thread.admission_time())
            .min()
            .unwrap_or(clock::now())
    }
