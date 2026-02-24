    pub fn sleep(self) -> (result: SleepResult)
        requires
            self.inv(),
        ensures
            match result {
                SleepResult::Runnable(rp) => {
                    rp@.pid == self@.pid
                    && rp.inv()
                    // Branch: there were ready or interrupted threads.
                    && (self@.ready_thread_ids.len() > 0 || self@.interrupted_thread_ids.len() > 0)
                    // Sleeping threads in result include the running thread.
                    && rp@.sleeping_thread_ids.len() == self@.sleeping_thread_ids.len() + 1
                    // Zombie threads preserved.
                    && rp@.zombie_thread_ids =~= self@.zombie_thread_ids
                    // Ready branch: exact content specified.
                    && (self@.ready_thread_ids.len() > 0 ==> {
                        rp@.ready_thread_ids =~= self@.ready_thread_ids
                        && rp@.sleeping_thread_ids =~=
                            self@.sleeping_thread_ids.push(self@.running_thread_id)
                        && rp@.interrupted_thread_ids =~= self@.interrupted_thread_ids
                    })
                    // Interrupted branch: details from strengthened interrupted_resume().
                    && (self@.ready_thread_ids.len() == 0 && self@.interrupted_thread_ids.len() > 0 ==> {
                        rp@.sleeping_thread_ids =~=
                            self@.sleeping_thread_ids.push(self@.running_thread_id)
                        && rp@.ready_thread_ids.len() == 1
                        && rp@.ready_thread_ids[0] == self@.interrupted_thread_ids[0]
                        && rp@.interrupted_thread_ids =~=
                            self@.interrupted_thread_ids.subrange(
                                1, self@.interrupted_thread_ids.len() as int)
                    })
                },
                SleepResult::Sleeping(sp) => {
                    sp@.pid == self@.pid
                    && sp.inv()
                    // Branch: no ready and no interrupted threads.
                    && self@.ready_thread_ids.len() == 0
                    && self@.interrupted_thread_ids.len() == 0
                    // Sleeping list content: old sleeping + running thread.
                    && sp@.sleeping_thread_ids =~=
                        self@.sleeping_thread_ids.push(self@.running_thread_id)
                    && sp@.sleeping_thread_ids.len() ==
                        self@.sleeping_thread_ids.len() + 1
                    // Zombie threads preserved.
                    && sp@.zombie_thread_ids =~= self@.zombie_thread_ids
                },
            },
    {
        proof {
            reveal(RunningProcess::inv);
            reveal(RunnableProcess::inv);
            reveal(SleepingProcess::inv);
            reveal(InterruptedProcess::inv);
        }
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, interrupted_thread_ids,
            mut sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        sleeping_thread_ids.push(running_thread_id);

        proof {
            assert(sleeping_thread_ids@.len() >= 1);
        }

        // Check if there are ready threads.
        if ready_count > 0 {
            return SleepResult::Runnable(RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        // Check if there are interrupted threads.
        if interrupted_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return SleepResult::Runnable(rp);
        }

        // No ready or interrupted threads — become sleeping.
        SleepResult::Sleeping(SleepingProcess {
            pid,
            sleeping_thread_ids,
            zombie_thread_ids,
        })
    }
