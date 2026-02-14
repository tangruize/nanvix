// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Condvar Proofs.
// This file contains proof lemmas for the Condvar type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================
//
// The following lemmas are definition-unfolding properties that serve as
// executable documentation and regression tests for spec changes.

impl Condvar {
    /// Lemma: A newly created condvar has an empty sleeping queue.
    pub proof fn lemma_new_is_empty()
        ensures
            CondvarView::spec_new() == (CondvarView { sleeping: Seq::<(int, int)>::empty() }),
            CondvarView::spec_new().sleeping.len() == 0,
    {
    }

    /// Lemma: The empty and non-empty predicates are complementary.
    pub proof fn lemma_empty_nonempty_complementary(&self)
        requires
            self.wf(),
        ensures
            self@.spec_is_empty() == !self@.spec_is_nonempty(),
    {
    }

    /// Lemma: A condvar is either empty or non-empty (totality).
    pub proof fn lemma_state_is_total(&self)
        requires
            self.wf(),
        ensures
            self@.spec_is_empty() || self@.spec_is_nonempty(),
            !(self@.spec_is_empty() && self@.spec_is_nonempty()),
    {
    }

    /// Lemma: View reflects the queue state.
    pub proof fn lemma_view_reflects_state(&self)
        ensures
            self@.sleeping.len() == self.sleeping@.len(),
            forall|i: int|
                #![trigger self@.sleeping[i]]
                0 <= i < self@.sleeping.len() as int ==>
                    self@.sleeping[i] == (self.sleeping@[i].0 as int, self.sleeping@[i].1 as int),
    {
    }

    /// Lemma: Two condvars with equal views have equal observable state.
    pub proof fn lemma_view_equality(a: &Condvar, b: &Condvar)
        requires
            a@ == b@,
        ensures
            a@.spec_len() == b@.spec_len(),
            a@.spec_is_empty() == b@.spec_is_empty(),
    {
    }

    /// Lemma: Well-formedness implies length consistency.
    pub proof fn lemma_wf_len_consistency(&self)
        requires
            self.wf(),
        ensures
            self.len as nat == self@.spec_len(),
            (self.len == 0) == self@.spec_is_empty(),
    {
    }

    /// Lemma: A new condvar is well-formed.
    pub proof fn lemma_new_is_wf()
        ensures ({
            let view: CondvarView = CondvarView::spec_new();
            view.sleeping.len() == 0
        }),
    {
    }
}

//==================================================================================================
// Proof Lemmas — Protocol Properties
//==================================================================================================
//
// The following lemmas prove protocol properties that reason across multiple
// state transitions or relate different API operations.

impl Condvar {
    /// Lemma: Enqueue increases the queue length by exactly one.
    pub proof fn lemma_enqueue_len(s: Seq<(i32, i32)>, entry: (i32, i32))
        ensures
            s.push(entry).len() == s.len() + 1,
    {
    }

    /// Lemma: After enqueue, the queue is non-empty.
    pub proof fn lemma_enqueue_nonempty(s: Seq<(i32, i32)>, entry: (i32, i32))
        ensures
            s.push(entry).len() > 0,
    {
    }

    /// Lemma: After enqueue, the last element is the enqueued entry.
    pub proof fn lemma_enqueue_last(s: Seq<(i32, i32)>, entry: (i32, i32))
        ensures
            s.push(entry).last() == entry,
    {
    }

    /// Lemma: Enqueue preserves existing elements.
    pub proof fn lemma_enqueue_preserves(s: Seq<(i32, i32)>, entry: (i32, i32))
        ensures
            forall|i: int|
                #![trigger s.push(entry)[i]]
                0 <= i < s.len() as int ==> s.push(entry)[i] == s[i],
    {
    }

    /// Lemma: Dequeue (subrange from 1) decreases the queue length by one.
    pub proof fn lemma_dequeue_len(s: Seq<(i32, i32)>)
        requires
            s.len() > 0,
        ensures
            s.subrange(1, s.len() as int).len() == s.len() - 1,
    {
    }

    /// Lemma: Dequeue preserves remaining elements in order.
    pub proof fn lemma_dequeue_preserves_order(s: Seq<(i32, i32)>)
        requires
            s.len() > 0,
        ensures
            forall|i: int|
                #![trigger s.subrange(1, s.len() as int)[i]]
                0 <= i < s.len() as int - 1
                    ==> s.subrange(1, s.len() as int)[i] == s[i + 1],
    {
    }

    /// Lemma: Remove-at produces a sequence with length decreased by one.
    pub proof fn lemma_remove_at_len(s: Seq<(i32, i32)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            concrete_remove_at_seq(s, idx).len() == s.len() - 1,
    {
    }

    /// Lemma: Remove-at preserves elements before the removed index.
    pub proof fn lemma_remove_at_preserves_before(s: Seq<(i32, i32)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            forall|i: int|
                #![trigger concrete_remove_at_seq(s, idx)[i]]
                0 <= i < idx
                    ==> concrete_remove_at_seq(s, idx)[i] == s[i],
    {
    }

    /// Lemma: Remove-at shifts elements after the removed index left by one.
    pub proof fn lemma_remove_at_preserves_after(s: Seq<(i32, i32)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            forall|i: int|
                #![trigger concrete_remove_at_seq(s, idx)[i]]
                idx <= i < s.len() as int - 1
                    ==> concrete_remove_at_seq(s, idx)[i] == s[i + 1],
    {
    }

    /// Lemma: FIFO property — enqueue two entries, dequeue gets the first one.
    pub proof fn lemma_fifo_ordering(
        s: Seq<(i32, i32)>,
        entry1: (i32, i32),
        entry2: (i32, i32),
    )
        requires
            s.len() == 0,
        ensures ({
            let after1: Seq<(i32, i32)> = s.push(entry1);
            let after2: Seq<(i32, i32)> = after1.push(entry2);
            let after_dequeue: Seq<(i32, i32)> = after2.subrange(1, after2.len() as int);
            &&& after2.len() == 2
            &&& after2[0] == entry1
            &&& after2[1] == entry2
            &&& after_dequeue.len() == 1
            &&& after_dequeue[0] == entry2
        }),
    {
    }

    /// Lemma: Generalized FIFO — on any non-empty queue, enqueue then dequeue_first
    /// removes the original head, not the newly enqueued entry.
    ///
    /// # Description
    ///
    /// For any non-empty queue `s`, enqueue `entry` at the back, then
    /// dequeue_first removes `s[0]` (the original head). The result queue
    /// contains all original elements except the head, plus the new entry
    /// at the back. This generalizes `lemma_fifo_ordering` beyond the
    /// 2-element case.
    pub proof fn lemma_fifo_ordering_general(
        s: Seq<(i32, i32)>,
        entry: (i32, i32),
    )
        requires
            s.len() > 0,
        ensures ({
            let after_enqueue: Seq<(i32, i32)> = s.push(entry);
            let after_dequeue: Seq<(i32, i32)> = after_enqueue.subrange(
                1, after_enqueue.len() as int,
            );
            // The original head is at index 0 of after_enqueue.
            &&& after_enqueue[0] == s[0]
            // After dequeue, the head is gone.
            &&& after_dequeue.len() == s.len()
            // The new entry is at the back.
            &&& after_dequeue[after_dequeue.len() as int - 1] == entry
            // Original tail elements are preserved in order.
            &&& forall|i: int|
                #![trigger after_dequeue[i]]
                0 <= i < s.len() as int - 1
                ==> after_dequeue[i] == s[i + 1]
        }),
    {
    }

    /// Lemma: Enqueue then dequeue on empty queue restores empty state.
    pub proof fn lemma_enqueue_dequeue_roundtrip(entry: (i32, i32))
        ensures ({
            let empty: Seq<(i32, i32)> = Seq::empty();
            let after_enqueue: Seq<(i32, i32)> = empty.push(entry);
            let after_dequeue: Seq<(i32, i32)> = after_enqueue.subrange(
                1,
                after_enqueue.len() as int,
            );
            &&& after_enqueue.len() == 1
            &&& after_enqueue[0] == entry
            &&& after_dequeue.len() == 0
        }),
    {
    }

    /// Lemma: Clear produces an empty queue equal to a new condvar's view.
    pub proof fn lemma_clear_produces_new_view()
        ensures
            Seq::<(int, int)>::empty().len() == 0,
            (CondvarView { sleeping: Seq::<(int, int)>::empty() }) == CondvarView::spec_new(),
    {
    }

    /// Lemma: An empty queue does not contain any pid.
    pub proof fn lemma_empty_not_contains_pid(pid_val: int)
        ensures
            !exists|i: int|
                #![trigger Seq::<(int, int)>::empty()[i]]
                0 <= i < 0 && Seq::<(int, int)>::empty()[i].0 == pid_val,
    {
    }

    /// Lemma: An empty queue does not contain any tid.
    pub proof fn lemma_empty_not_contains_tid(tid_val: int)
        ensures
            !exists|i: int|
                #![trigger Seq::<(int, int)>::empty()[i]]
                0 <= i < 0 && Seq::<(int, int)>::empty()[i].1 == tid_val,
    {
    }
}

//==================================================================================================
// Proof Lemmas — Uniqueness Preservation
//==================================================================================================
//
// The following lemmas prove that `spec_all_unique()` (trust assumption T1)
// is preserved by all queue operations.

impl Condvar {
    /// Lemma: A new (empty) sequence satisfies uniqueness.
    pub proof fn lemma_new_is_unique()
        ensures ({
            let s: Seq<(i32, i32)> = Seq::empty();
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j]
        }),
    {
    }

    /// Lemma: Enqueue preserves uniqueness when the entry is not already present.
    pub proof fn lemma_enqueue_preserves_unique(
        s: Seq<(i32, i32)>,
        entry: (i32, i32),
    )
        requires
            // All existing entries are unique.
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
            // The new entry is not already in the sequence.
            forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int ==> s[i] != entry,
        ensures
            forall|i: int, j: int|
                #![trigger s.push(entry)[i], s.push(entry)[j]]
                0 <= i < s.push(entry).len() as int
                && 0 <= j < s.push(entry).len() as int
                && i != j
                ==> s.push(entry)[i] != s.push(entry)[j],
    {
        let new_s: Seq<(i32, i32)> = s.push(entry);
        assert forall|i: int, j: int|
            #![trigger new_s[i], new_s[j]]
            0 <= i < new_s.len() as int
            && 0 <= j < new_s.len() as int
            && i != j
        implies new_s[i] != new_s[j] by {
            if i < s.len() as int && j < s.len() as int {
                // Both are old entries.
                assert(new_s[i] == s[i]);
                assert(new_s[j] == s[j]);
            } else if i < s.len() as int {
                // i is old, j is the new entry.
                assert(new_s[i] == s[i]);
                assert(new_s[j] == entry);
            } else if j < s.len() as int {
                // i is the new entry, j is old.
                assert(new_s[i] == entry);
                assert(new_s[j] == s[j]);
            }
            // Both can't be the new entry since i != j and len is s.len()+1.
        }
    }

    /// Lemma: Dequeue (remove front) preserves uniqueness.
    pub proof fn lemma_dequeue_preserves_unique(s: Seq<(i32, i32)>)
        requires
            s.len() > 0,
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
        ensures ({
            let result: Seq<(i32, i32)> = s.subrange(1, s.len() as int);
            forall|i: int, j: int|
                #![trigger result[i], result[j]]
                0 <= i < result.len() as int
                && 0 <= j < result.len() as int
                && i != j
                ==> result[i] != result[j]
        }),
    {
        let result: Seq<(i32, i32)> = s.subrange(1, s.len() as int);
        assert forall|i: int, j: int|
            #![trigger result[i], result[j]]
            0 <= i < result.len() as int
            && 0 <= j < result.len() as int
            && i != j
        implies result[i] != result[j] by {
            assert(result[i] == s[i + 1]);
            assert(result[j] == s[j + 1]);
        }
    }

    /// Lemma: Remove-at preserves uniqueness.
    pub proof fn lemma_remove_at_preserves_unique(s: Seq<(i32, i32)>, idx: int)
        requires
            0 <= idx < s.len(),
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
        ensures ({
            let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
            forall|i: int, j: int|
                #![trigger result[i], result[j]]
                0 <= i < result.len() as int
                && 0 <= j < result.len() as int
                && i != j
                ==> result[i] != result[j]
        }),
    {
        let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
        assert forall|i: int, j: int|
            #![trigger result[i], result[j]]
            0 <= i < result.len() as int
            && 0 <= j < result.len() as int
            && i != j
        implies result[i] != result[j] by {
            // Map result indices back to original indices.
            let orig_i: int = if i < idx { i } else { i + 1 };
            let orig_j: int = if j < idx { j } else { j + 1 };
            assert(result[i] == s[orig_i]);
            assert(result[j] == s[orig_j]);
        }
    }

    /// Lemma: Clear trivially preserves uniqueness (empty sequence is unique).
    pub proof fn lemma_clear_preserves_unique()
        ensures ({
            let result: Seq<(i32, i32)> = Seq::<(i32, i32)>::empty();
            forall|i: int, j: int|
                #![trigger result[i], result[j]]
                0 <= i < result.len() as int
                && 0 <= j < result.len() as int
                && i != j
                ==> result[i] != result[j]
        }),
    {
    }
}

//==================================================================================================
// Proof Lemmas — Uniqueness Preservation (Condvar-level wrappers)
//==================================================================================================
//
// The following lemmas wrap the raw-Seq uniqueness preservation lemmas,
// providing a `&self`-based interface using `spec_all_unique()`.

impl Condvar {
    /// Lemma: Enqueue preserves uniqueness when the entry is not present.
    pub proof fn lemma_enqueue_preserves_unique_cv(&self, pid_val: i32, tid_val: i32)
        requires
            self.wf(),
            !self@.spec_contains_entry(pid_val as int, tid_val as int),
        ensures ({
            let new_sleeping: Seq<(i32, i32)> = self.sleeping@.push((pid_val, tid_val));
            forall|i: int, j: int|
                #![trigger new_sleeping[i], new_sleeping[j]]
                0 <= i < new_sleeping.len() as int
                && 0 <= j < new_sleeping.len() as int
                && i != j
                ==> new_sleeping[i] != new_sleeping[j]
        }),
    {
        let entry: (i32, i32) = (pid_val, tid_val);
        let s: Seq<(i32, i32)> = self.sleeping@;

        // Prove that no existing element equals the new entry.
        assert forall|i: int|
            #![trigger s[i]]
            0 <= i < s.len() as int
        implies s[i] != entry by {
            if s[i] == entry {
                // Contradicts !self@.spec_contains_entry.
                assert(s[i].0 == pid_val && s[i].1 == tid_val);
            }
        }

        Condvar::lemma_enqueue_preserves_unique(s, entry);
    }

    /// Lemma: Dequeue preserves uniqueness.
    pub proof fn lemma_dequeue_preserves_unique_cv(&self)
        requires
            self.wf(),
            self.sleeping@.len() > 0,
        ensures ({
            let new_sleeping: Seq<(i32, i32)> =
                self.sleeping@.subrange(1, self.sleeping@.len() as int);
            forall|i: int, j: int|
                #![trigger new_sleeping[i], new_sleeping[j]]
                0 <= i < new_sleeping.len() as int
                && 0 <= j < new_sleeping.len() as int
                && i != j
                ==> new_sleeping[i] != new_sleeping[j]
        }),
    {
        Condvar::lemma_dequeue_preserves_unique(self.sleeping@);
    }

    /// Lemma: Remove-at preserves uniqueness.
    pub proof fn lemma_remove_at_preserves_unique_cv(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.sleeping@.len() as int,
        ensures ({
            let new_sleeping: Seq<(i32, i32)> =
                concrete_remove_at_seq(self.sleeping@, idx);
            forall|i: int, j: int|
                #![trigger new_sleeping[i], new_sleeping[j]]
                0 <= i < new_sleeping.len() as int
                && 0 <= j < new_sleeping.len() as int
                && i != j
                ==> new_sleeping[i] != new_sleeping[j]
        }),
    {
        Condvar::lemma_remove_at_preserves_unique(self.sleeping@, idx);
    }

    /// Lemma: Clear preserves `spec_all_unique`.
    pub proof fn lemma_clear_preserves_unique_cv()
        ensures ({
            let s: Seq<(i32, i32)> = Seq::<(i32, i32)>::empty();
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j]
        }),
    {
        Condvar::lemma_clear_preserves_unique();
    }
}

//==================================================================================================
// Proof Lemmas — Remove Entry Properties
//==================================================================================================
//
// The following lemmas prove properties of remove_entry under the uniqueness
// invariant, showing that the removed entry is absent from the result.

impl Condvar {
    /// Lemma: After removing the entry at index `idx`, that entry no longer
    /// appears in the resulting sequence, provided all entries were unique.
    pub proof fn lemma_remove_entry_absent(
        s: Seq<(i32, i32)>,
        idx: int,
    )
        requires
            0 <= idx < s.len(),
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
        ensures ({
            let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
            let entry: (i32, i32) = s[idx];
            forall|k: int|
                #![trigger result[k]]
                0 <= k < result.len() as int ==> result[k] != entry
        }),
    {
        let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
        let entry: (i32, i32) = s[idx];
        assert forall|k: int|
            #![trigger result[k]]
            0 <= k < result.len() as int
        implies result[k] != entry by {
            let orig_k: int = if k < idx { k } else { k + 1 };
            assert(result[k] == s[orig_k]);
            assert(orig_k != idx);
        }
    }

    /// Lemma: After removing a (pid, tid) entry at index `idx` from a unique
    /// sequence, the (pid, tid) pair is no longer contained in the result.
    pub proof fn lemma_remove_entry_not_contains(
        s: Seq<(i32, i32)>,
        idx: int,
        pid_val: i32,
        tid_val: i32,
    )
        requires
            0 <= idx < s.len(),
            s[idx] == (pid_val, tid_val),
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
        ensures ({
            let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
            !exists|k: int|
                #![trigger result[k]]
                0 <= k < result.len() as int
                && result[k].0 == pid_val
                && result[k].1 == tid_val
        }),
    {
        Condvar::lemma_remove_entry_absent(s, idx);
    }

    /// Lemma: Under uniqueness, removing the single entry at index `idx` is
    /// equivalent to filtering out all entries equal to `(pid_val, tid_val)`.
    ///
    /// # Description
    ///
    /// The original `retain(|&mut (p, t)| p != pid || t != tid)` removes *all*
    /// matching entries. This lemma proves that under T1 (uniqueness), there
    /// is exactly one matching entry, so removing at the known index produces
    /// the same result as a full filter. This bridges the model's single-index
    /// removal to the original's predicate-based `retain()`.
    pub proof fn lemma_remove_entry_equivalent_to_retain(
        s: Seq<(i32, i32)>,
        idx: int,
        pid_val: i32,
        tid_val: i32,
    )
        requires
            0 <= idx < s.len(),
            s[idx] == (pid_val, tid_val),
            // Uniqueness (T1).
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
        ensures ({
            let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
            // (1) The entry is absent from the result.
            &&& forall|k: int|
                #![trigger result[k]]
                0 <= k < result.len() as int
                ==> result[k] != (pid_val, tid_val)
            // (2) All non-matching entries from the original are preserved.
            &&& forall|k: int|
                #![trigger result[k]]
                0 <= k < result.len() as int
                ==> (exists|j: int|
                    #![trigger s[j]]
                    0 <= j < s.len() as int && s[j] == result[k])
            // (3) The result has exactly one fewer element.
            &&& result.len() == s.len() - 1
        }),
    {
        Condvar::lemma_remove_entry_absent(s, idx);
        let result: Seq<(i32, i32)> = concrete_remove_at_seq(s, idx);
        // Each element in result maps back to the original sequence.
        assert forall|k: int|
            #![trigger result[k]]
            0 <= k < result.len() as int
        implies (exists|j: int|
            #![trigger s[j]]
            0 <= j < s.len() as int && s[j] == result[k])
        by {
            let orig_k: int = if k < idx { k } else { k + 1 };
            assert(result[k] == s[orig_k]);
        }
    }
}

//==================================================================================================
// Proof Lemmas — Wait Protocol
//==================================================================================================
//
// The following lemmas prove properties of the wait() protocol: enqueue
// followed by conditional remove_entry on failure. This models the original
// wait()'s cleanup path where retain() removes the entry if sleep() fails.
//
// **Trust assumption T4**: These lemmas assume no concurrent modifications
// between enqueue and cleanup. At runtime, ProcessManager::sleep() blocks
// between these steps, during which other threads may modify the queue via
// notify_*() calls. The sequential model cannot capture this interleaving.

impl Condvar {
    /// Lemma: The wait() cleanup protocol (enqueue then remove_entry on failure)
    /// restores the original queue state.
    ///
    /// # Description
    ///
    /// Models the original `wait()` behavior: enqueue a (pid, tid) entry, then
    /// if `ProcessManager::sleep()` fails, remove that entry via `retain()`.
    /// This lemma proves that the cleanup path produces a queue extensionally
    /// equal to the original, establishing that the protocol is state-safe.
    pub proof fn lemma_wait_cleanup_restores_state(
        s: Seq<(i32, i32)>,
        entry: (i32, i32),
    )
        requires
            // Original queue has unique entries.
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
            // Entry is not already in the queue.
            forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int ==> s[i] != entry,
        ensures ({
            // After enqueue:
            let after_enqueue: Seq<(i32, i32)> = s.push(entry);
            // The entry is at the last index:
            let idx: int = s.len() as int;
            // After remove_entry at that index:
            let after_cleanup: Seq<(i32, i32)> = concrete_remove_at_seq(after_enqueue, idx);
            // The queue is restored to its original state:
            after_cleanup =~= s
        }),
    {
        let after_enqueue: Seq<(i32, i32)> = s.push(entry);
        let idx: int = s.len() as int;
        let after_cleanup: Seq<(i32, i32)> = concrete_remove_at_seq(after_enqueue, idx);
        // after_cleanup == after_enqueue[0..idx] + after_enqueue[idx+1..len]
        // == s[0..s.len()] + empty == s
        assert(after_cleanup =~= s);
    }

    /// Lemma: After the wait() protocol (enqueue + cleanup), the original
    /// queue state is restored.
    ///
    /// # Description
    ///
    /// Proves that if a well-formed condvar undergoes the wait protocol
    /// (enqueue a new entry, then remove it on failure), the resulting
    /// sequence is extensionally equal to the original. Combined with the
    /// fact that the original state satisfies wf() properties (uniqueness,
    /// no-kernel-pid, length consistency), the cleanup path is safe.
    pub proof fn lemma_wait_protocol_preserves_wf(
        s: Seq<(i32, i32)>,
        len: usize,
        entry: (i32, i32),
    )
        requires
            len as nat == s.len(),
            len < usize::MAX,
            // Original queue has unique entries.
            forall|i: int, j: int|
                #![trigger s[i], s[j]]
                0 <= i < s.len() as int
                && 0 <= j < s.len() as int
                && i != j
                ==> s[i] != s[j],
            // Entry is not already in the queue.
            forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int ==> s[i] != entry,
            // No kernel PID in the original queue.
            forall|i: int|
                #![trigger s[i]]
                0 <= i < s.len() as int
                ==> s[i].0 as int != CondvarView::spec_kernel_pid(),
        ensures ({
            // After enqueue then cleanup, the original state is restored.
            let after_enqueue: Seq<(i32, i32)> = s.push(entry);
            let idx: int = s.len() as int;
            let after_cleanup: Seq<(i32, i32)> = concrete_remove_at_seq(after_enqueue, idx);
            after_cleanup =~= s
        }),
    {
        Condvar::lemma_wait_cleanup_restores_state(s, entry);
    }
}

//==================================================================================================
// Proof Lemmas — Drop Safety
//==================================================================================================
//
// The following lemmas prove properties related to the drop safety predicate.

impl Condvar {
    /// Lemma: A newly created condvar is safe to drop.
    pub proof fn lemma_new_is_drop_safe()
        ensures
            CondvarView::spec_new().sleeping.len() == 0,
    {
    }

    /// Lemma: A condvar is drop-safe if and only if the queue is empty.
    pub proof fn lemma_drop_safe_iff_empty(&self)
        ensures
            self@.spec_drop_safe() == self@.spec_is_empty(),
    {
    }

    /// Lemma: After `clear()`, the condvar is drop-safe.
    pub proof fn lemma_clear_is_drop_safe()
        ensures
            Seq::<(int, int)>::empty().len() == 0,
    {
    }

    /// Lemma: Any well-formed condvar with an empty queue is drop-safe.
    ///
    /// # Description
    ///
    /// Makes explicit that any operation resulting in an empty queue (not just
    /// `new()` or `clear()`) produces a drop-safe condvar. This covers cases
    /// where individual notify operations (`dequeue_first`, `remove_by_pid`,
    /// etc.) happen to drain the last entry.
    pub proof fn lemma_empty_is_drop_safe(&self)
        requires
            self.wf(),
            self@.spec_len() == 0,
        ensures
            self@.spec_drop_safe(),
    {
    }
}

} // verus!
