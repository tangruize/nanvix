    pub fn add_thread(self, ready_tid: u64) -> (result: RunnableProcess)
        requires
            self.wf(),
            // The added thread must not collide with existing sleeping or zombie threads.
            // In the original code, Rust ownership prevents this; we enforce it explicitly.
            !SleepingProcessView::spec_seq_contains(
                self@.sleeping_thread_ids, ready_tid as int),
            !SleepingProcessView::spec_seq_contains(
                self@.zombie_thread_ids, ready_tid as int),
        ensures
            result@.pid == self@.pid,
            result.wf(),
            // Exactly one ready thread: the added thread.
            result@.ready_thread_ids.len() == 1,
            result@.ready_thread_ids[0] == ready_tid as int,
            // No interrupted threads.
            result@.interrupted_thread_ids.len() == 0,
            // Sleeping threads preserved.
            result@.sleeping_thread_ids =~= self@.sleeping_thread_ids,
            // Zombie threads preserved.
            result@.zombie_thread_ids =~= self@.zombie_thread_ids,
    {
        proof {
            reveal(SleepingProcess::wf);
            reveal(RunnableProcess::wf);
        }
        let mut ready: Vec<u64> = Vec::new();
        ready.push(ready_tid);

        proof {
            assert(ready@.len() == 1);
            assert(ready@[0] == ready_tid);
        }

        RunnableProcess {
            pid: self.pid,
            ready_thread_ids: ready,
            interrupted_thread_ids: Vec::new(),
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }
