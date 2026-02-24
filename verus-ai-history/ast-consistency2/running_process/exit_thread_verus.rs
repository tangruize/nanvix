    pub fn exit_thread(self, status: u64) -> (result: ExitThreadResult)
        requires
            self.inv(),
        ensures
            match result {
                ExitThreadResult::Runnable(rp) => {
                    rp@.pid == self@.pid
                    && rp.inv()
                    && (self@.ready_thread_ids.len() > 0 || self@.interrupted_thread_ids.len() > 0)
                    // Zombie list includes the exited running thread.
                    && rp@.zombie_thread_ids =~=
                        self@.zombie_thread_ids.push(self@.running_thread_id)
                    && rp@.zombie_thread_ids.len() == 1 + self@.zombie_thread_ids.len()
                    // Ready branch: content preserved.
                    && (self@.ready_thread_ids.len() > 0 ==> {
                        rp@.ready_thread_ids =~= self@.ready_thread_ids
                        && rp@.interrupted_thread_ids =~= self@.interrupted_thread_ids
                        && rp@.sleeping_thread_ids =~= self@.sleeping_thread_ids
                    })
                    // Interrupted branch: details from strengthened interrupted_resume().
                    && (self@.ready_thread_ids.len() == 0 && self@.interrupted_thread_ids.len() > 0 ==> {
                        rp@.sleeping_thread_ids =~= self@.sleeping_thread_ids
                        && rp@.ready_thread_ids.len() == 1
                        && rp@.ready_thread_ids[0] == self@.interrupted_thread_ids[0]
                        && rp@.interrupted_thread_ids =~=
                            self@.interrupted_thread_ids.subrange(
                                1, self@.interrupted_thread_ids.len() as int)
                    })
                },
                ExitThreadResult::Sleeping(sp) => {
                    sp@.pid == self@.pid
                    && sp.inv()
                    && self@.ready_thread_ids.len() == 0
                    && self@.interrupted_thread_ids.len() == 0
                    && self@.sleeping_thread_ids.len() > 0
                    // Sleeping threads preserved.
                    && sp@.sleeping_thread_ids =~= self@.sleeping_thread_ids
                    // Zombie list includes the exited running thread.
                    && sp@.zombie_thread_ids =~=
                        self@.zombie_thread_ids.push(self@.running_thread_id)
                },
                ExitThreadResult::Zombie(zp) => {
                    zp@.pid == self@.pid
                    && zp.inv()
                    && zp@.status == status as int
                    && self@.ready_thread_ids.len() == 0
                    && self@.interrupted_thread_ids.len() == 0
                    && self@.sleeping_thread_ids.len() == 0
                    // Zombie list includes the running thread + original zombie.
                    && zp@.zombie_thread_ids =~=
                        self@.zombie_thread_ids.push(self@.running_thread_id)
                    && zp@.zombie_thread_ids.len() == 1 + self@.zombie_thread_ids.len()
                },
            },
    {
        proof {
            reveal(RunningProcess::inv);
            reveal(RunnableProcess::inv);
            reveal(SleepingProcess::inv);
            reveal(ZombieProcess::inv);
            reveal(InterruptedProcess::inv);
        }
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, mut zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        // Running thread becomes zombie.
        zombie_thread_ids.push(running_thread_id);

        proof {
            assert(zombie_thread_ids@.len() >= 1);
        }

        if ready_count > 0 {
            return ExitThreadResult::Runnable(RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        if interrupted_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            // Historical: original passed self.zombie.take() (=None) here. Now fixed in source.
            // We correctly pass zombie_thread_ids (includes exited thread).
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return ExitThreadResult::Runnable(rp);
        }

        if sleeping_count > 0 {
            return ExitThreadResult::Sleeping(SleepingProcess {
                pid,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        ExitThreadResult::Zombie(ZombieProcess {
            pid,
            zombie_thread_ids,
            status,
        })
    }
