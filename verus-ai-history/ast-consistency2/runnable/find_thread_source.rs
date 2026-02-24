    pub fn find_thread(&self, tid: ThreadIdentifier) -> Option<ThreadRef<'_>> {
        // Search in the list of ready threads.
        if let Some(thread) = self.ready_threads.iter().find(|thread| thread.id() == tid) {
            return Some(ThreadRef::Ready(thread));
        }

        // Search in the list of interrupted threads.
        if let Some(interrupted_threads) = &self.interrupted_threads {
            if let Some(thread) = interrupted_threads.iter().find(|thread| thread.id() == tid) {
                return Some(ThreadRef::Interrupted(thread));
            }
        }

        // Search in the list of sleeping threads.
        if let Some(sleeping_threads) = &self.sleeping_threads {
            if let Some(thread) = sleeping_threads.iter().find(|thread| thread.id() == tid) {
                return Some(ThreadRef::Sleeping(thread));
            }
        }

        // Search in the list of zombie threads.
        if let Some(zombie_threads) = &self.zombie_threads {
            if let Some(thread) = zombie_threads.iter().find(|thread| thread.id() == tid) {
                return Some(ThreadRef::Zombie(thread));
            }
        }

        None
    }
