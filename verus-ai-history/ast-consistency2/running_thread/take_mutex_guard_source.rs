    pub fn take_mutex_guard(&mut self, mutex_addr: MutexAddress) -> Option<MutexGuard> {
        self.state.take_mutex_guard(mutex_addr)
    }
