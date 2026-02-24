    pub fn resume(self, admission_time: u64) -> (result: RunnableProcess)
        requires
            self.wf(),
        ensures
            result@.pid == self@.pid,
            result.wf(),
            // Exactly one ready thread: the front interrupted thread.
            result@.ready_thread_ids.len() == 1,
            result@.ready_thread_ids[0] == self@.interrupted_thread_ids[0],
            // Admission time matches the oracle parameter.
            result@.ready_admission_times.len() == 1,
            result@.ready_admission_times[0] == admission_time as int,
            // Remaining interrupted threads (tail of original list).
            result@.interrupted_thread_ids =~=
                self@.interrupted_thread_ids.subrange(
                    1, self@.interrupted_thread_ids.len() as int),
            // Sleeping threads preserved.
            result@.sleeping_thread_ids =~= self@.sleeping_thread_ids,
            // Zombie threads preserved.
            result@.zombie_thread_ids =~= self@.zombie_thread_ids,
    {
        // Destructure self to work with individual fields.
        let pid: u64 = self.pid;
        let mut interrupted: Vec<u64> = self.interrupted_thread_ids;
        let sleeping: Vec<u64> = self.sleeping_thread_ids;
        let zombie: Vec<u64> = self.zombie_thread_ids;
        let ghost old_interrupted: Seq<u64> = interrupted@;

        // Pop front of interrupted list.
        let front_tid: u64 = interrupted.remove(0);

        // Build ready list (singleton).
        let mut ready: Vec<u64> = Vec::new();
        ready.push(front_tid);

        // Build admission times list (singleton).
        let mut admit_times: Vec<u64> = Vec::new();
        admit_times.push(admission_time);

        proof {
            reveal(InterruptedProcess::wf);
            reveal(RunnableProcess::wf);

            // Connect Vec::remove(0) with subrange(1, len).
            assert(old_interrupted.remove(0int) =~=
                old_interrupted.subrange(1, old_interrupted.len() as int));

            let ready_spec: Seq<u64> = ready@;
            assert(ready_spec.len() == 1);
            assert(ready_spec[0] == front_tid);

            // Prove no-duplicates on the tail of the interrupted list.
            Self::lemma_subrange_preserves_no_duplicates(old_interrupted);

            // Prove the front element is not in the tail.
            Self::lemma_front_not_in_tail(old_interrupted);

            // Prove tail of interrupted is disjoint from sleeping and zombie.
            Self::lemma_tail_disjoint_sleeping(old_interrupted, sleeping@);
            Self::lemma_tail_disjoint_zombie(old_interrupted, zombie@);

            // Prove ready (singleton) is no-duplicates trivially.
            assert(InterruptedProcess::spec_no_duplicates(ready_spec)) by {
                assert forall|i: int, j: int| 0 <= i < j < ready_spec.len()
                    implies ready_spec[i] != ready_spec[j]
                by {
                    // ready_spec.len() == 1, so no i < j pair exists.
                }
            }

            // Prove ready is disjoint from remaining interrupted.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, interrupted@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < interrupted@.len()
                    implies ready_spec[i] != interrupted@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(interrupted@[j] == old_interrupted[j + 1]);
                }
            }

            // Prove ready is disjoint from sleeping.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, sleeping@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < sleeping@.len()
                    implies ready_spec[i] != sleeping@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(0 <= 0int < old_interrupted.len());
                    assert(old_interrupted[0] == front_tid);
                }
            }

            // Prove ready is disjoint from zombie.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, zombie@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < zombie@.len()
                    implies ready_spec[i] != zombie@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(0 <= 0int < old_interrupted.len());
                    assert(old_interrupted[0] == front_tid);
                }
            }

            // Admission times: singleton matching oracle.
            let admit_spec: Seq<u64> = admit_times@;
            assert(admit_spec.len() == 1);
            assert(admit_spec[0] == admission_time);
        }

        RunnableProcess {
            pid,
            ready_thread_ids: ready,
            ready_admission_times: admit_times,
            interrupted_thread_ids: interrupted,
            sleeping_thread_ids: sleeping,
            zombie_thread_ids: zombie,
        }
    }
