    pub(super) fn store_mutex_guard(&mut self, address: MutexAddress, guard: MutexGuard) {
        self.locked_mutexes.insert(address, guard);
    }
