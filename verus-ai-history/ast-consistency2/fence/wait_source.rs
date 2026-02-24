    pub fn wait(&self) {
        while self.count.load(Ordering::Acquire) < self.total {
            ::arch::cpu::pause();
        }
    }
