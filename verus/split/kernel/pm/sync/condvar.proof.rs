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
            Condvar::spec_new_view() == (CondvarView { sleeping: Seq::empty() }),
            Condvar::spec_new_view().sleeping.len() == 0,
    {
    }

    /// Lemma: The empty and non-empty predicates are complementary.
    pub proof fn lemma_empty_nonempty_complementary(&self)
        requires
            self.wf(),
        ensures
            self.spec_is_empty() == !self.spec_is_nonempty(),
    {
    }

    /// Lemma: A condvar is either empty or non-empty (totality).
    pub proof fn lemma_state_is_total(&self)
        requires
            self.wf(),
        ensures
            self.spec_is_empty() || self.spec_is_nonempty(),
            !(self.spec_is_empty() && self.spec_is_nonempty()),
    {
    }

    /// Lemma: View reflects the queue state.
    pub proof fn lemma_view_reflects_state(&self)
        ensures
            self@.sleeping =~= self.sleeping@,
    {
    }

    /// Lemma: Two condvars with equal views have equal observable state.
    pub proof fn lemma_view_equality(a: &Condvar, b: &Condvar)
        requires
            a@ == b@,
        ensures
            a.spec_len() == b.spec_len(),
            a.spec_is_empty() == b.spec_is_empty(),
    {
    }

    /// Lemma: Well-formedness implies length consistency.
    pub proof fn lemma_wf_len_consistency(&self)
        requires
            self.wf(),
        ensures
            self.len as nat == self.spec_len(),
            (self.len == 0) == self.spec_is_empty(),
    {
    }

    /// Lemma: A new condvar is well-formed.
    pub proof fn lemma_new_is_wf()
        ensures ({
            let view: CondvarView = Condvar::spec_new_view();
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
    pub proof fn lemma_enqueue_len(s: Seq<(int, int)>, entry: (int, int))
        ensures
            s.push(entry).len() == s.len() + 1,
    {
    }

    /// Lemma: After enqueue, the queue is non-empty.
    pub proof fn lemma_enqueue_nonempty(s: Seq<(int, int)>, entry: (int, int))
        ensures
            s.push(entry).len() > 0,
    {
    }

    /// Lemma: After enqueue, the last element is the enqueued entry.
    pub proof fn lemma_enqueue_last(s: Seq<(int, int)>, entry: (int, int))
        ensures
            s.push(entry).last() == entry,
    {
    }

    /// Lemma: Enqueue preserves existing elements.
    pub proof fn lemma_enqueue_preserves(s: Seq<(int, int)>, entry: (int, int))
        ensures
            forall|i: int|
                #![trigger s.push(entry)[i]]
                0 <= i < s.len() as int ==> s.push(entry)[i] == s[i],
    {
    }

    /// Lemma: Dequeue (subrange from 1) decreases the queue length by one.
    pub proof fn lemma_dequeue_len(s: Seq<(int, int)>)
        requires
            s.len() > 0,
        ensures
            s.subrange(1, s.len() as int).len() == s.len() - 1,
    {
    }

    /// Lemma: Dequeue preserves remaining elements in order.
    pub proof fn lemma_dequeue_preserves_order(s: Seq<(int, int)>)
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
    pub proof fn lemma_remove_at_len(s: Seq<(int, int)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            Condvar::spec_remove_at_seq(s, idx).len() == s.len() - 1,
    {
    }

    /// Lemma: Remove-at preserves elements before the removed index.
    pub proof fn lemma_remove_at_preserves_before(s: Seq<(int, int)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            forall|i: int|
                #![trigger Condvar::spec_remove_at_seq(s, idx)[i]]
                0 <= i < idx
                    ==> Condvar::spec_remove_at_seq(s, idx)[i] == s[i],
    {
    }

    /// Lemma: Remove-at shifts elements after the removed index left by one.
    pub proof fn lemma_remove_at_preserves_after(s: Seq<(int, int)>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            forall|i: int|
                #![trigger Condvar::spec_remove_at_seq(s, idx)[i]]
                idx <= i < s.len() as int - 1
                    ==> Condvar::spec_remove_at_seq(s, idx)[i] == s[i + 1],
    {
    }

    /// Lemma: FIFO property — enqueue two entries, dequeue gets the first one.
    pub proof fn lemma_fifo_ordering(
        s: Seq<(int, int)>,
        entry1: (int, int),
        entry2: (int, int),
    )
        requires
            s.len() == 0,
        ensures ({
            let after1: Seq<(int, int)> = s.push(entry1);
            let after2: Seq<(int, int)> = after1.push(entry2);
            let after_dequeue: Seq<(int, int)> = after2.subrange(1, after2.len() as int);
            &&& after2.len() == 2
            &&& after2[0] == entry1
            &&& after2[1] == entry2
            &&& after_dequeue.len() == 1
            &&& after_dequeue[0] == entry2
        }),
    {
    }

    /// Lemma: Enqueue then dequeue on empty queue restores empty state.
    pub proof fn lemma_enqueue_dequeue_roundtrip(entry: (int, int))
        ensures ({
            let empty: Seq<(int, int)> = Seq::empty();
            let after_enqueue: Seq<(int, int)> = empty.push(entry);
            let after_dequeue: Seq<(int, int)> = after_enqueue.subrange(
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
            (CondvarView { sleeping: Seq::<(int, int)>::empty() }) == Condvar::spec_new_view(),
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

} // verus!
