    pub(super) fn take_mutex_guard(&mut self, address: MutexAddress) -> Option<MutexGuard> {
        self.locked_mutexes.remove(&address)
    }
