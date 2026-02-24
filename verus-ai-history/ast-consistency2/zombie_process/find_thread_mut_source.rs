    pub fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Option<ThreadRefMut<'_>> {
        self.zombie_threads
            .iter_mut()
            .find(|thread| thread.id() == tid)
            .map(ThreadRefMut::Zombie)
    }
