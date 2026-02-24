    pub fn wakeup_alarm(
        self,
        has_expired: bool,
        interrupted_ids: Vec<u64>,
        remaining_ids: Vec<u64>,
    ) -> (result: Result<InterruptedProcess, SleepingProcess>)
        requires
            self.wf(),
            // Oracle: partition must be a valid decomposition.
            has_expired == (interrupted_ids@.len() > 0),
            // Length conservation.
            interrupted_ids@.len() + remaining_ids@.len()
                == self@.sleeping_thread_ids.len(),
            // Content conservation: all partition elements come from original sleeping list.
            forall|i: int| #![auto] 0 <= i < interrupted_ids@.len() ==>
                SleepingProcessView::spec_seq_contains(
                    self@.sleeping_thread_ids, interrupted_ids@[i] as int),
            forall|i: int| #![auto] 0 <= i < remaining_ids@.len() ==>
                SleepingProcessView::spec_seq_contains(
                    self@.sleeping_thread_ids, remaining_ids@[i] as int),
            // Partition integrity: no duplicates within or across partitions.
            SleepingProcessView::spec_no_duplicates(
                spec_u64_seq_as_int(interrupted_ids@)),
            SleepingProcessView::spec_no_duplicates(
                spec_u64_seq_as_int(remaining_ids@)),
            SleepingProcessView::spec_seqs_disjoint(
                spec_u64_seq_as_int(interrupted_ids@),
                spec_u64_seq_as_int(remaining_ids@)),
            // Stable partition: both partitions preserve relative order.
            SleepingProcessView::spec_is_subsequence(
                spec_u64_seq_as_int(interrupted_ids@), self@.sleeping_thread_ids),
            SleepingProcessView::spec_is_subsequence(
                spec_u64_seq_as_int(remaining_ids@), self@.sleeping_thread_ids),
            // If not expired, sleeping list is preserved.
            !has_expired ==> spec_u64_seq_as_int(remaining_ids@)
                =~= self@.sleeping_thread_ids,
        ensures
            match result {
                Ok(ip) => {
                    has_expired
                    && ip@.pid == self@.pid
                    && ip.wf()
                    && ip@.interrupted_thread_ids
                        =~= spec_u64_seq_as_int(interrupted_ids@)
                    && ip@.interrupted_thread_ids.len() >= 1
                    && ip@.sleeping_thread_ids
                        =~= spec_u64_seq_as_int(remaining_ids@)
                    && ip@.zombie_thread_ids =~= self@.zombie_thread_ids
                    // Conservation: partition sizes sum to original.
                    && ip@.interrupted_thread_ids.len() + ip@.sleeping_thread_ids.len()
                        == self@.sleeping_thread_ids.len()
                    // Stable ordering: partitions are subsequences of the original.
                    && SleepingProcessView::spec_is_subsequence(
                        ip@.interrupted_thread_ids, self@.sleeping_thread_ids)
                    && SleepingProcessView::spec_is_subsequence(
                        ip@.sleeping_thread_ids, self@.sleeping_thread_ids)
                },
                Err(sp) => {
                    !has_expired
                    && sp@.pid == self@.pid
                    && sp.wf()
                    && sp@.sleeping_thread_ids =~= self@.sleeping_thread_ids
                    && sp@.zombie_thread_ids =~= self@.zombie_thread_ids
                },
            },
    {
        proof {
            reveal(SleepingProcess::wf);
            reveal(InterruptedProcess::wf);
        }
        if has_expired {
            proof {
                assert(interrupted_ids@.len() >= 1);
            }

            Ok(InterruptedProcess {
                pid: self.pid,
                interrupted_thread_ids: interrupted_ids,
                sleeping_thread_ids: remaining_ids,
                zombie_thread_ids: self.zombie_thread_ids,
            })
        } else {
            // No alarm expired: the process remains sleeping with all state unchanged.
            Err(SleepingProcess {
                pid: self.pid,
                sleeping_thread_ids: self.sleeping_thread_ids,
                zombie_thread_ids: self.zombie_thread_ids,
            })
        }
    }
