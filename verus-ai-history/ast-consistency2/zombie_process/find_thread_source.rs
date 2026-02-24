    pub fn find_thread(&self, tid: ThreadIdentifier) -> Option<ThreadRef<'_>> {
        self.zombie_threads
            .iter()
            .find(|thread| thread.id() == tid)
            .map(ThreadRef::Zombie)
    }
