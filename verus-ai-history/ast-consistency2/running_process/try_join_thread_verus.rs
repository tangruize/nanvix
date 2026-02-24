    pub fn try_join_thread(&mut self, tid: u64, tag: u8) -> (result: u8)
        requires
            old(self).inv(),
            tag as int == old(self)@.try_join_tag(tid as int),
        ensures
            result == tag,
            result as int == old(self)@.try_join_tag(tid as int),
            // PID and running thread unchanged.
            self@.pid == old(self)@.pid,
            self@.running_thread_id == old(self)@.running_thread_id,
            // Ready, interrupted, sleeping lists unchanged.
            self@.ready_thread_ids =~= old(self)@.ready_thread_ids,
            self@.interrupted_thread_ids =~= old(self)@.interrupted_thread_ids,
            self@.sleeping_thread_ids =~= old(self)@.sleeping_thread_ids,
            // Zombie list: removed on success, unchanged otherwise.
            (tag == JOIN_TAG_ZOMBIE) ==> (
                (exists|idx: int|
                    0 <= idx < old(self)@.zombie_thread_ids.len()
                    && old(self)@.zombie_thread_ids[idx] == tid as int
                    && self@.zombie_thread_ids =~=
                        RunningProcessView::seq_remove_at(old(self)@.zombie_thread_ids, idx))
                && self@.zombie_thread_ids.len() == old(self)@.zombie_thread_ids.len() - 1
            ),
            (tag != JOIN_TAG_ZOMBIE) ==> (
                self@.zombie_thread_ids =~= old(self)@.zombie_thread_ids
            ),
            self.inv(),
    {
        proof { reveal(RunningProcess::inv); }
        if tag == JOIN_TAG_ZOMBIE {
            // Zombie found — remove it from the zombie list.
            proof {
                assert(old(self).spec_has_zombie_thread(tid));
                assert(old(self).zombie_thread_ids@.len() > 0);
                assert(old(self).zombie_count > 0);
            }

            // Find the first occurrence of tid using a flag (no break).
            let zlen: usize = self.zombie_thread_ids.len();
            let mut idx: usize = 0;
            let mut found_it: bool = false;
            while idx < zlen && !found_it
                invariant
                    0 <= idx <= zlen,
                    zlen == old(self).zombie_thread_ids@.len(),
                    self.zombie_thread_ids@ =~= old(self).zombie_thread_ids@,
                    !found_it ==> forall|j: int| 0 <= j < idx as int
                        ==> self.zombie_thread_ids@[j] != tid,
                    found_it ==> (
                        idx < zlen
                        && self.zombie_thread_ids@[idx as int] == tid
                        && forall|j: int| 0 <= j < idx as int
                            ==> self.zombie_thread_ids@[j] != tid
                    ),
                    Self::spec_seq_contains(old(self).zombie_thread_ids@, tid),
                    self.pid == old(self).pid,
                    self.running_thread_id == old(self).running_thread_id,
                    self.ready_thread_ids@ =~= old(self).ready_thread_ids@,
                    self.interrupted_thread_ids@ =~= old(self).interrupted_thread_ids@,
                    self.sleeping_thread_ids@ =~= old(self).sleeping_thread_ids@,
                    self.ready_count == old(self).ready_count,
                    self.interrupted_count == old(self).interrupted_count,
                    self.sleeping_count == old(self).sleeping_count,
                    self.zombie_count == old(self).zombie_count,
                decreases zlen - idx, if found_it { 0int } else { 1int },
            {
                if self.zombie_thread_ids[idx] == tid {
                    found_it = true;
                } else {
                    idx = idx + 1;
                }
            }

            proof {
                // After loop: found_it must be true (otherwise all elements != tid,
                // contradicting spec_seq_contains).
                if !found_it {
                    assert(idx >= zlen);
                    assert(forall|j: int| 0 <= j < zlen as int
                        ==> self.zombie_thread_ids@[j] != tid);
                    // This contradicts spec_seq_contains.
                    assert(false);
                }
                assert(found_it);
                assert(idx < zlen);
                assert(self.zombie_thread_ids@[idx as int] == tid);
            }

            let new_zombies: Vec<u64> = vec_remove_at(&self.zombie_thread_ids, idx);

            proof {
                // Prove the new zombie list length.
                let s: Seq<u64> = old(self).zombie_thread_ids@;
                Self::lemma_remove_at_length(s, idx as int);
            }

            self.zombie_thread_ids = new_zombies;
            self.zombie_count = self.zombie_count - 1;

            proof {
                // Witness for the existential in ensures.
                assert(0 <= idx as int && (idx as int) < old(self).zombie_thread_ids@.len());
                assert(old(self).zombie_thread_ids@[idx as int] == tid);
                assert(self.zombie_thread_ids@ =~=
                    Self::spec_remove_at(old(self).zombie_thread_ids@, idx as int));

                // Bridge concrete Seq<u64> to abstract Seq<int> for view-level postcondition.
                lemma_seq_as_int_remove_at(old(self).zombie_thread_ids@, idx as int);
            }

            JOIN_TAG_ZOMBIE
        } else {
            tag
        }
    }
