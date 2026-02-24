    pub fn exit(self, status: u64) -> (result: ExitResult)
        requires
            self.inv(),
        ensures
            match result {
                ExitResult::Runnable(rp) => {
                    rp@.pid == self@.pid
                    && rp.inv()
                    // Branch: there were interrupted or sleeping threads.
                    && (self@.interrupted_thread_ids.len() > 0
                        || self@.sleeping_thread_ids.len() > 0)
                    // Zombie threads in result: original zombie + running + ready (matches original ordering).
                    && rp@.zombie_thread_ids.len() ==
                        1 + self@.ready_thread_ids.len() + self@.zombie_thread_ids.len()
                    && rp@.zombie_thread_ids =~=
                        self@.zombie_thread_ids.push(self@.running_thread_id).add(
                            self@.ready_thread_ids)
                    // No sleeping threads remain (all were converted to interrupted).
                    && rp@.sleeping_thread_ids.len() == 0
                    // Exactly one interrupted thread was resumed as ready.
                    && rp@.ready_thread_ids.len() == 1
                    // The ready thread is the first element of the combined interrupted list.
                    && rp@.ready_thread_ids[0] ==
                        self@.interrupted_thread_ids.add(self@.sleeping_thread_ids)[0]
                    // The remaining interrupted threads are the tail.
                    && rp@.interrupted_thread_ids =~=
                        self@.interrupted_thread_ids.add(self@.sleeping_thread_ids).subrange(
                            1, (self@.interrupted_thread_ids.len() + self@.sleeping_thread_ids.len()) as int)
                },
                ExitResult::Zombie(zp) => {
                    zp@.pid == self@.pid
                    && zp.inv()
                    && zp@.status == status as int
                    // Branch: no interrupted or sleeping threads.
                    && self@.interrupted_thread_ids.len() == 0
                    && self@.sleeping_thread_ids.len() == 0
                    // Zombie list: original zombie + running + all ready (matches original ordering).
                    && zp@.zombie_thread_ids.len() ==
                        1 + self@.ready_thread_ids.len() + self@.zombie_thread_ids.len()
                    && zp@.zombie_thread_ids =~=
                        self@.zombie_thread_ids.push(self@.running_thread_id).add(
                            self@.ready_thread_ids)
                },
            },
    {
        proof {
            reveal(RunningProcess::inv);
            reveal(RunnableProcess::inv);
            reveal(ZombieProcess::inv);
            reveal(InterruptedProcess::inv);
        }
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, mut interrupted_thread_ids,
            sleeping_thread_ids, mut zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        // Running thread becomes zombie. Original: push_back onto existing zombies.
        zombie_thread_ids.push(running_thread_id);
        // All ready threads become zombies. Original: append ready zombies after.
        vec_push_all(&mut zombie_thread_ids, &ready_thread_ids);

        proof {
            assert(zombie_thread_ids@.len() >= 1);
        }

        // Sleeping threads become interrupted.
        vec_push_all(&mut interrupted_thread_ids, &sleeping_thread_ids);

        if interrupted_count > 0 || sleeping_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            // In the original, self.sleeping_threads was already taken (line 208),
            // so InterruptedProcess::from_sleeping gets None for sleeping. We model
            // this faithfully with empty sleeping_thread_ids.
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids: Vec::new(),
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            ExitResult::Runnable(rp)
        } else {
            proof {
                assert(interrupted_thread_ids@.len() == 0);
            }
            ExitResult::Zombie(ZombieProcess {
                pid,
                zombie_thread_ids,
                status,
            })
        }
    }
