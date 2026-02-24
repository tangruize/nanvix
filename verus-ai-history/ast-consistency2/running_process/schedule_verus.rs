    pub fn schedule(self) -> (result: ScheduleResult)
        requires
            self.inv(),
        ensures
            // PID preserved.
            result.process@.pid == self@.pid,
            // The formerly running thread is now in the ready list.
            result.process@.ready_thread_ids.len() == self@.ready_thread_ids.len() + 1,
            // Content: ready list is old ready + running thread appended.
            result.process@.ready_thread_ids =~=
                self@.ready_thread_ids.push(self@.running_thread_id),
            // Other lists preserved.
            result.process@.interrupted_thread_ids =~= self@.interrupted_thread_ids,
            result.process@.sleeping_thread_ids =~= self@.sleeping_thread_ids,
            result.process@.zombie_thread_ids =~= self@.zombie_thread_ids,
            // Result is well-formed (non-empty ready list).
            result.process.inv(),
    {
        proof {
            reveal(RunningProcess::inv);
            reveal(RunnableProcess::inv);
        }
        let RunningProcess {
            pid, running_thread_id, mut ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        ready_thread_ids.push(running_thread_id);

        proof {
            assert(ready_thread_ids@.len() >= 1);
        }

        ScheduleResult {
            process: RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            },
        }
    }
