// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Fence Proofs.
// This file contains proof lemmas for the Fence type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================
//
// The following lemmas are definition-unfolding properties that serve as
// executable documentation and regression tests for spec changes. They are
// automatically discharged by Verus.

impl Fence {
    /// Lemma: A newly created fence has zero count and the given total.
    pub proof fn lemma_new_is_unsatisfied(total: nat)
        ensures
            Fence::spec_new_view(total) == (FenceView { count: 0, total: total }),
            Fence::spec_new_view(total).count == 0,
            Fence::spec_new_view(total).total == total,
    {
    }

    /// Lemma: A fence is either satisfied or waiting (totality).
    pub proof fn lemma_state_is_total(&self)
        requires
            self.wf(),
        ensures
            self.spec_is_satisfied() || self.spec_is_waiting(),
            !(self.spec_is_satisfied() && self.spec_is_waiting()),
    {
    }

    /// Lemma: spec_is_satisfied and spec_is_waiting are complementary for well-formed fences.
    pub proof fn lemma_satisfied_waiting_complementary(&self)
        requires
            self.wf(),
        ensures
            self.spec_is_satisfied() == !self.spec_is_waiting(),
    {
    }

    /// Lemma: View reflects the count and total fields.
    pub proof fn lemma_view_reflects_state(&self)
        ensures
            self@.count == self.spec_count(),
            self@.total == self.spec_total(),
    {
    }

    /// Lemma: Two fences with equal views have equal observable state.
    pub proof fn lemma_view_equality(a: &Fence, b: &Fence)
        requires
            a@ == b@,
        ensures
            a.spec_count() == b.spec_count(),
            a.spec_total() == b.spec_total(),
            a.spec_is_satisfied() == b.spec_is_satisfied(),
            a.spec_is_waiting() == b.spec_is_waiting(),
    {
    }

    /// Lemma: Well-formedness is preserved: new fences are well-formed.
    pub proof fn lemma_new_is_wf(total: nat)
        ensures ({
            let view: FenceView = Fence::spec_new_view(total);
            view.count == 0 && view.total == total && view.count <= view.total
        }),
    {
    }

    /// Lemma: A newly created fence with total > 0 is in the waiting state.
    pub proof fn lemma_new_nonzero_is_waiting(total: nat)
        requires
            total > 0,
        ensures
            Fence::spec_new_view(total).count < Fence::spec_new_view(total).total,
    {
    }

    /// Lemma: A newly created fence with total == 0 is immediately satisfied.
    pub proof fn lemma_new_zero_is_satisfied(total: nat)
        requires
            total == 0,
        ensures
            Fence::spec_new_view(total).count >= Fence::spec_new_view(total).total,
    {
    }
}

//==================================================================================================
// Proof Lemmas — Protocol Properties
//==================================================================================================
//
// The following lemmas prove protocol properties that reason across multiple
// state transitions or relate different API operations.

impl Fence {
    /// Lemma: Signaling a waiting fence decrements remaining by exactly one.
    pub proof fn lemma_signal_decrements_remaining(pre_count: nat, total: nat)
        requires
            pre_count < total,
        ensures
            (total - (pre_count + 1)) == (total - pre_count) - 1,
    {
    }

    /// Lemma: After exactly `total` signals, the fence is satisfied.
    ///
    /// # Description
    ///
    /// Proves satisfaction: when the signal count equals the total, the fence
    /// satisfaction condition (`count >= total`) holds. This captures the fact
    /// that starting from `count == 0` and applying `total` signal operations
    /// (each incrementing count by 1) yields `count == total`, which implies
    /// satisfaction. Note: this is a static arithmetic entailment, not a
    /// temporal liveness proof—it does not model the sequence of signal
    /// operations inductively.
    pub proof fn lemma_total_signals_satisfies(count: nat, total: nat)
        requires
            count == total,
        ensures
            count >= total,
    {
    }

    /// Lemma: Signal-then-check protocol: after signaling a fence with remaining == 1,
    /// the fence is satisfied.
    pub proof fn lemma_last_signal_satisfies(pre_count: nat, total: nat)
        requires
            pre_count + 1 == total,
        ensures
            (pre_count + 1) >= total,
    {
    }

    /// Lemma: Signaling preserves well-formedness when the fence is waiting.
    pub proof fn lemma_signal_preserves_wf(pre_count: nat, total: nat)
        requires
            pre_count < total,
        ensures
            pre_count + 1 <= total,
    {
    }

    /// Lemma: A satisfied fence stays satisfied (monotonicity).
    ///
    /// # Description
    ///
    /// Once count >= total, no further signals can un-satisfy the fence.
    /// This is trivially true since signal only increments count.
    pub proof fn lemma_satisfaction_is_monotone(count: nat, total: nat)
        requires
            count >= total,
        ensures
            count >= total,
            count + 1 >= total,
    {
    }

    /// Lemma: Wait on a satisfied fence returns immediately.
    ///
    /// # Description
    ///
    /// Proves that if a fence is already satisfied, the wait loop condition
    /// `count < total` is false, so wait is a no-op.
    pub proof fn lemma_wait_on_satisfied_is_noop(count: nat, total: nat)
        requires
            count >= total,
        ensures
            !(count < total),
    {
    }

    /// Lemma: An unlocked fence has the same view as a new fence with its total.
    pub proof fn lemma_unsignaled_eq_new_view(s: &Fence)
        requires
            s.spec_count() == 0,
            s.wf(),
        ensures
            s@ == Fence::spec_new_view(s.spec_total()),
    {
    }

    /// Lemma: Remaining signals is zero iff the fence is satisfied.
    pub proof fn lemma_remaining_zero_iff_satisfied(s: &Fence)
        requires
            s.wf(),
        ensures
            (s.spec_remaining() == 0) == s.spec_is_satisfied(),
    {
    }
}

//==================================================================================================
// Proof Lemmas — Concurrency Properties (Spec-Level)
//==================================================================================================
//
// The following lemmas state properties about concurrent signal ordering at
// the spec level. They hold over nat arithmetic and serve as documentation
// that the fence protocol is order-independent.

impl Fence {
    /// Lemma: Commutativity of concurrent signals.
    ///
    /// # Description
    ///
    /// The order of signal calls does not affect the final fence state.
    /// This property is essential for the concurrent use case where multiple
    /// threads call `signal()` in arbitrary order. Although the sequential
    /// `&mut self` model cannot express concurrent execution, this lemma
    /// proves the underlying arithmetic is commutative.
    pub proof fn lemma_signal_commutativity(count: nat, total: nat, signals_a: nat, signals_b: nat)
        requires
            count + signals_a + signals_b <= total,
        ensures
            count + signals_a + signals_b == count + signals_b + signals_a,
            (count + signals_a + signals_b >= total) == (count + signals_b + signals_a >= total),
    {
    }
}

} // verus!
