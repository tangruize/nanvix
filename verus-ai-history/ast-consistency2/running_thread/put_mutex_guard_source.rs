    pub fn put_mutex_guard(&mut self, mutex_addr: MutexAddress, guard: MutexGuard) {
        self.state.store_mutex_guard(mutex_addr, guard);
    }
