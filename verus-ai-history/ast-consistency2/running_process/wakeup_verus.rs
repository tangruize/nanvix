    pub fn wakeup(self, tid: u64, found: bool) -> (result: Result<RunningProcess, RunningProcess>)
        requires
            self.inv(),
            found == RunningProcessView::seq_contains(self@.sleeping_thread_ids, tid as int),
            self@.ready_thread_ids.len() < u64::MAX as int,
        ensures
            match result {
                Ok(r) => {
                    found
                    && r@.pid == self@.pid
                    && r@.running_thread_id == self@.running_thread_id
                    && r@.ready_thread_ids.len() == self@.ready_thread_ids.len() + 1
                    && r@.sleeping_thread_ids.len() == self@.sleeping_thread_ids.len() - 1
                    && r@.interrupted_thread_ids.len() == self@.interrupted_thread_ids.len()
                    && r@.zombie_thread_ids.len() == self@.zombie_thread_ids.len()
                    // Content: ready list gets the woken thread appended.
                    && r@.ready_thread_ids =~= self@.ready_thread_ids.push(tid as int)
                    // Sleeping list has the found thread removed.
                    && (exists|idx: int| 0 <= idx < self@.sleeping_thread_ids.len()
                        && self@.sleeping_thread_ids[idx] == tid as int
                        && r@.sleeping_thread_ids =~=
                            RunningProcessView::seq_remove_at(self@.sleeping_thread_ids, idx))
                    // Other lists preserved exactly.
                    && r@.interrupted_thread_ids =~= self@.interrupted_thread_ids
                    && r@.zombie_thread_ids =~= self@.zombie_thread_ids
                    && r.inv()
                },
                Err(r) => {
                    !found
                    && r@ == self@
                    && r.inv()
                },
            },
    {
        proof { reveal(RunningProcess::inv); }
        if !found {
            return Err(self);
        }

        let RunningProcess {
            pid, running_thread_id, mut ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        proof {
            // Derive sleeping_count > 0 from wf() and found == spec_seq_contains.
            assert(sleeping_thread_ids@.len() > 0);
            assert(sleeping_count > 0);
        }

        // Find the first occurrence of tid in sleeping_thread_ids.
        let slen: usize = sleeping_thread_ids.len();
        let mut idx: usize = 0;
        let mut found_it: bool = false;
        while idx < slen && !found_it
            invariant
                0 <= idx <= slen,
                slen == sleeping_thread_ids@.len(),
                !found_it ==> forall|j: int| 0 <= j < idx as int
                    ==> sleeping_thread_ids@[j] != tid,
                found_it ==> (
                    idx < slen
                    && sleeping_thread_ids@[idx as int] == tid
                    && forall|j: int| 0 <= j < idx as int
                        ==> sleeping_thread_ids@[j] != tid
                ),
                Self::spec_seq_contains(sleeping_thread_ids@, tid),
            decreases slen - idx, if found_it { 0int } else { 1int },
        {
            if sleeping_thread_ids[idx] == tid {
                found_it = true;
            } else {
                idx = idx + 1;
            }
        }

        proof {
            // After loop: found_it must be true.
            if !found_it {
                assert(idx >= slen);
                assert(forall|j: int| 0 <= j < slen as int
                    ==> sleeping_thread_ids@[j] != tid);
                assert(false);
            }
            assert(found_it);
            assert(idx < slen);
            assert(sleeping_thread_ids@[idx as int] == tid);
        }

        // Remove tid from sleeping list.
        let new_sleeping: Vec<u64> = vec_remove_at(&sleeping_thread_ids, idx);

        proof {
            // Prove new sleeping length.
            let s: Seq<u64> = sleeping_thread_ids@;
            let left: Seq<u64> = s.subrange(0, idx as int);
            let right: Seq<u64> = s.subrange(idx as int + 1, s.len() as int);
            assert(left.len() == idx as nat);
            assert(right.len() == (s.len() - idx as nat - 1) as nat);
            assert(left.add(right).len() == (s.len() - 1) as nat);
        }

        // Push tid onto ready.
        ready_thread_ids.push(tid);

        proof {
            assert(ready_thread_ids@.len() == ready_count as int + 1);

            // Bridge concrete Seq<u64> to abstract Seq<int> for view-level postconditions.
            // ready_thread_ids was push(tid) onto old ready list.
            let old_ready: Seq<u64> = self.ready_thread_ids@;
            lemma_seq_as_int_push(old_ready, tid);

            // sleeping list had remove_at applied.
            lemma_seq_as_int_remove_at(sleeping_thread_ids@, idx as int);
        }

        Ok(RunningProcess {
            pid,
            running_thread_id,
            ready_thread_ids,
            interrupted_thread_ids,
            sleeping_thread_ids: new_sleeping,
            zombie_thread_ids,
            ready_count: ready_count + 1,
            interrupted_count,
            sleeping_count: sleeping_count - 1,
            zombie_count,
        })
    }
