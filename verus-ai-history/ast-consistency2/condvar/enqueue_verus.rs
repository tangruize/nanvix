    pub fn enqueue(&mut self, pid_val: i32, tid_val: i32)
        requires
            old(self).wf(),
            old(self)@.spec_len() < usize::MAX,
            !old(self)@.spec_contains_entry(pid_val as int, tid_val as int),
            // The kernel process must not sleep (matches original panic guard).
            pid_val as int != CondvarView::spec_kernel_pid(),
        ensures
            self@.spec_len() == old(self)@.spec_len() + 1,
            self@.sleeping =~= old(self)@.sleeping.push((pid_val as int, tid_val as int)),
            !self@.spec_is_empty(),
            self.wf(),
    {
        proof {
            // Bridge from wf() to raw-Seq uniqueness on concrete fields.
            let entry: (i32, i32) = (pid_val, tid_val);
            let s: Seq<(i32, i32)> = self.sleeping@;
            // Uniqueness of s follows from wf() which includes concrete_all_unique().
            assert forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
            implies s[i] != s[j] by {
                assert(self.sleeping@[i] == s[i]);
                assert(self.sleeping@[j] == s[j]);
            }
            // No existing element equals the new entry (from !spec_contains_entry).
            assert forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int
            implies s[i] != entry by {
                if s[i] == entry {
                    assert(s[i].0 == pid_val && s[i].1 == tid_val);
                    // Bridge: concrete equality implies abstract equality.
                    assert(self@.sleeping[i] == (s[i].0 as int, s[i].1 as int));
                    assert(self@.sleeping[i] == (pid_val as int, tid_val as int));
                }
            }
            Condvar::lemma_enqueue_preserves_unique(s, entry);
            // Prove no-kernel-pid is preserved after push.
            let new_s: Seq<(i32, i32)> = s.push(entry);
            assert forall|i: int|
                #![trigger new_s[i]]
                0 <= i < new_s.len() as int
            implies new_s[i].0 as int != CondvarView::spec_kernel_pid() by {
                if i < s.len() as int {
                    assert(new_s[i] == s[i]);
                    assert(self.sleeping@[i] == s[i]);
                } else {
                    assert(new_s[i] == entry);
                }
            }
        }
        self.len = self.len + 1;
        self.sleeping.push((pid_val, tid_val));
    }
