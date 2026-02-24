    pub fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Option<ThreadRefMut<'_>> {
        if let Some(thread) = self
            .sleeping_threads
            .iter_mut()
            .find(|thread| thread.id() == tid)
        {
            return Some(ThreadRefMut::Sleeping(thread));
        }

        if let Some(zombie_threads) = &mut self.zombie_threads {
            if let Some(thread) = zombie_threads.iter_mut().find(|thread| thread.id() == tid) {
                return Some(ThreadRefMut::Zombie(thread));
            }
        }

        None
    }
