    pub fn remove_at(&mut self, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        proof {
            // Uniqueness preservation (uses _cv variant to avoid trigger mismatch).
            self.lemma_remove_at_preserves_unique_cv(idx as int);
            // No-kernel-pid preservation: map result elements back to originals.
            let s: Seq<(i32, i32)> = self.sleeping@;
            let idx_int: int = idx as int;
            let sub1: Seq<(i32, i32)> = s.subrange(0, idx_int);
            let sub2: Seq<(i32, i32)> = s.subrange(idx_int + 1, s.len() as int);
            let result: Seq<(i32, i32)> = sub1 + sub2;
            assert(result =~= concrete_remove_at_seq(s, idx_int));
            assert forall|i: int|
                #![trigger result[i]]
                0 <= i < result.len() as int
            implies result[i].0 as int != CondvarView::spec_kernel_pid() by {
                if i < sub1.len() as int {
                    assert(result[i] == sub1[i]);
                    assert(sub1[i] == s[i]);
                } else {
                    let j: int = i - sub1.len() as int;
                    assert(result[i] == sub2[j]);
                    assert(sub2[j] == s[idx_int + 1 + j]);
                }
            }
        }
        self.len = self.len - 1;
        let _removed: (i32, i32) = self.sleeping.remove(idx);
        true
    }
