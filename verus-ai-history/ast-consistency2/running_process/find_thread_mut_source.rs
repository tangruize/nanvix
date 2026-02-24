    pub fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Option<ThreadRefMut<'_>> {
        // Check if the running thread matches.
        if self.running.id() == tid {
            return Some(ThreadRefMut::Running(&mut self.running));
        }

        // Search in the list of ready threads.
        if let Some(ready_threads) = &mut self.ready {
            if let Some(thread) = ready_threads.iter_mut().find(|thread| thread.id() == tid) {
                return Some(ThreadRefMut::Ready(thread));
            }
        }

        // Search in the list of interrupted threads.
        if let Some(interrupted_threads) = &mut self.interrupted_threads {
            if let Some(thread) = interrupted_threads
                .iter_mut()
                .find(|thread| thread.id() == tid)
            {
                return Some(ThreadRefMut::Interrupted(thread));
            }
        }

        // Search in the list of sleeping threads.
        if let Some(sleeping_threads) = &mut self.sleeping_threads {
            if let Some(thread) = sleeping_threads
                .iter_mut()
                .find(|thread| thread.id() == tid)
            {
                return Some(ThreadRefMut::Sleeping(thread));
            }
        }

        // Search in the list of zombie threads.
        if let Some(zombie_threads) = &mut self.zombie {
            if let Some(thread) = zombie_threads.iter_mut().find(|thread| thread.id() == tid) {
                return Some(ThreadRefMut::Zombie(thread));
            }
        }

        None
    }
