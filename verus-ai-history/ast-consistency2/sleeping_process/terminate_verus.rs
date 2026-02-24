    pub fn terminate(self) -> (result: InterruptedProcess)
        requires
            self.wf(),
        ensures
            result@.pid == self@.pid,
            result.wf(),
            // All sleeping threads become interrupted (ID-preserving transition).
            result@.interrupted_thread_ids =~= self@.sleeping_thread_ids,
            result@.interrupted_thread_ids.len() == self@.sleeping_thread_ids.len(),
            // No sleeping threads remain (all were interrupted).
            result@.sleeping_thread_ids.len() == 0,
            // Zombie threads are preserved.
            result@.zombie_thread_ids =~= self@.zombie_thread_ids,
    {
        proof {
            reveal(SleepingProcess::wf);
            reveal(InterruptedProcess::wf);
            assert(self.sleeping_thread_ids@.len() >= 1);
        }

        InterruptedProcess {
            pid: self.pid,
            interrupted_thread_ids: self.sleeping_thread_ids,
            sleeping_thread_ids: Vec::new(),
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }
