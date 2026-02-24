    pub fn find_thread(&self, tid: ThreadIdentifier) -> Option<ThreadRef<'_>> {
        if let Some(thread) = self
            .sleeping_threads
            .iter()
            .find(|thread| thread.id() == tid)
        {
            return Some(ThreadRef::Sleeping(thread));
        }

        if let Some(zombie_threads) = &self.zombie_threads {
            if let Some(thread) = zombie_threads.iter().find(|thread| thread.id() == tid) {
                return Some(ThreadRef::Zombie(thread));
            }
        }

        None
    }
