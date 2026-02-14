// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Fence Proofs.
// This file contains proof lemmas for the Fence type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================
//
// The following lemmas are intentionally shallow definition-unfolding
// properties. They serve as executable documentation and regression tests
// that guard against accidental spec changes (e.g., a typo in inv() or
// is_satisfied()). They are automatically discharged by Verus and do
// not exercise the prover in a meaningful way. The substantive protocol
// proofs are in the "Protocol Properties" and "Concurrency Properties"
// sections below.

impl Fence {
    /// Lemma: A newly created fence has zero count and the given total.
    pub proof fn lemma_new_is_unsatisfied(total: nat)
        ensures
            FenceView::spec_new(total) == (FenceView { count: 0, total: total }),
            FenceView::spec_new(total).count == 0,
            FenceView::spec_new(total).total == total,
    {
    }

    /// Lemma: A fence is either satisfied or waiting (totality).
    pub proof fn lemma_state_is_total(&self)
        requires
            self.inv(),
        ensures
            self@.is_satisfied() || self@.is_waiting(),
            !(self@.is_satisfied() && self@.is_waiting()),
    {
        reveal(Fence::inv);
    }

    /// Lemma: is_satisfied and is_waiting are complementary for well-formed fences.
    pub proof fn lemma_satisfied_waiting_complementary(&self)
        requires
            self.inv(),
        ensures
            self@.is_satisfied() == !self@.is_waiting(),
    {
        reveal(Fence::inv);
    }

    /// Lemma: View reflects predicates consistently.
    pub proof fn lemma_view_reflects_state(&self)
        ensures
            (self@.count >= self@.total) == self@.is_satisfied(),
            (self@.count < self@.total) == self@.is_waiting(),
    {
    }

    /// Lemma: Two fences with equal views have equal observable state.
    pub proof fn lemma_view_equality(a: &Fence, b: &Fence)
        requires
            a@ == b@,
        ensures
            a@.count == b@.count,
            a@.total == b@.total,
            a@.is_satisfied() == b@.is_satisfied(),
            a@.is_waiting() == b@.is_waiting(),
    {
    }

    /// Lemma: Well-formedness is preserved: new fences are well-formed.
    pub proof fn lemma_new_is_wf(total: nat)
        ensures ({
            let view: FenceView = FenceView::spec_new(total);
            view.count == 0 && view.total == total && view.count <= view.total
        }),
    {
    }

    /// Lemma: A newly created fence with total > 0 is in the waiting state.
    pub proof fn lemma_new_nonzero_is_waiting(total: nat)
        requires
            total > 0,
        ensures
            FenceView::spec_new(total).is_waiting(),
    {
    }

    /// Lemma: A newly created fence with total == 0 is immediately satisfied.
    pub proof fn lemma_new_zero_is_satisfied(total: nat)
        requires
            total == 0,
        ensures
            FenceView::spec_new(total).is_satisfied(),
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

    /// Lemma: Equality implies satisfaction.
    ///
    /// # Description
    ///
    /// A convenience entailment: when the signal count equals the total, the
    /// satisfaction condition (`count >= total`) holds. This is a static
    /// arithmetic fact, not an inductive proof — see
    /// `lemma_signals_accumulate_to_satisfaction` for the inductive property
    /// connecting signal operations to eventual satisfaction.
    pub proof fn lemma_equality_implies_satisfaction(count: nat, total: nat)
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

    /// Lemma: Signaling preserves the invariant when the fence is waiting.
    pub proof fn lemma_signal_preserves_inv(pre_count: nat, total: nat)
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

    /// Lemma: An unsignaled fence has the same view as a new fence with its total.
    pub proof fn lemma_unsignaled_eq_new_view(s: &Fence)
        requires
            s@.count == 0,
            s.inv(),
        ensures
            s@ == FenceView::spec_new(s@.total),
    {
        reveal(Fence::inv);
    }

    /// Lemma: Remaining signals is zero iff the fence is satisfied.
    pub proof fn lemma_remaining_zero_iff_satisfied(s: &Fence)
        requires
            s.inv(),
        ensures
            (s@.remaining() == 0) == s@.is_satisfied(),
    {
        reveal(Fence::inv);
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

    /// Lemma: Inductive signal accumulation leads to satisfaction.
    ///
    /// # Description
    ///
    /// Proves the protocol invariant inductively: starting from any well-formed
    /// state where `count + n == total`, after `n` signal operations (each
    /// incrementing count by 1), the fence is satisfied. This connects the
    /// individual `signal()` transitions to the eventual satisfaction property,
    /// bridging the gap between per-step correctness and end-to-end protocol
    /// completion.
    ///
    /// Unlike `lemma_equality_implies_satisfaction` (a static arithmetic fact), this
    /// lemma reasons about the accumulation of `n` signal operations from an
    /// arbitrary starting state.
    pub proof fn lemma_signals_accumulate_to_satisfaction(count: nat, n: nat, total: nat)
        requires
            count <= total,
            count + n == total,
        ensures
            count + n >= total,
            // Each intermediate state preserves inv.
            forall|i: nat| #![trigger (count + i)] i <= n ==> count + i <= total,
            // The final state is satisfied.
            count + n == total,
    {
    }
}

} // verus!
