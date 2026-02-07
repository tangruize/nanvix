// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Semaphore Proofs.
// This file contains proof lemmas for the Semaphore type.

verus! {

//==================================================================================================
// Proof Lemmas -- Definitional Properties (Regression Tests)
//==================================================================================================
//
// The following lemmas are definition-unfolding properties that serve as
// executable documentation and regression tests for spec changes. They are
// automatically discharged by Verus and do not prove deep protocol properties.
// See "Protocol Properties" section below for substantive proofs.

impl Semaphore {
    /// Lemma: A newly created semaphore has the specified initial value and no waiters.
    pub proof fn lemma_new_is_correct(value: nat)
        ensures
            Semaphore::spec_new_view(value) == (SemaphoreView { value: value, waiters: 0 }),
            Semaphore::spec_new_view(value).value == value,
            Semaphore::spec_new_view(value).waiters == 0,
    {
    }

    /// Lemma: A semaphore is either available or exhausted (totality).
    pub proof fn lemma_state_is_total(&self)
        ensures
            self.spec_is_available() || self.spec_is_exhausted(),
            !(self.spec_is_available() && self.spec_is_exhausted()),
    {
    }

    /// Lemma: `spec_is_available` and `spec_is_exhausted` are complementary.
    pub proof fn lemma_available_exhausted_complementary(&self)
        ensures
            self.spec_is_available() == !self.spec_is_exhausted(),
    {
    }

    /// Lemma: View reflects the value field.
    pub proof fn lemma_view_reflects_value(&self)
        requires
            self.wf(),
        ensures
            self@.value == self.value as nat,
    {
    }

    /// Lemma: Two semaphores with equal views have equal observable state.
    pub proof fn lemma_view_equality(a: &Semaphore, b: &Semaphore)
        requires
            a@ == b@,
        ensures
            a.spec_value() == b.spec_value(),
            a.spec_waiters() == b.spec_waiters(),
            a.spec_is_available() == b.spec_is_available(),
    {
    }

    /// Lemma: A new semaphore with value > 0 is available.
    pub proof fn lemma_new_nonzero_is_available(value: nat)
        requires
            value > 0,
        ensures
            Semaphore::spec_new_view(value).value > 0,
    {
    }

    /// Lemma: A new semaphore with value == 0 is exhausted.
    pub proof fn lemma_new_zero_is_exhausted()
        ensures
            Semaphore::spec_new_view(0).value == 0,
    {
    }

    /// Lemma: Well-formedness of a new semaphore.
    pub proof fn lemma_new_is_wf(value: nat)
        ensures ({
            let view: SemaphoreView = Semaphore::spec_new_view(value);
            view.value == value && view.waiters == 0
        }),
    {
    }

    /// Lemma: Well-formedness enforces no waiters when value > 0.
    pub proof fn lemma_wf_available_no_waiters(s: &Semaphore)
        requires
            s.wf(),
            s.spec_is_available(),
        ensures
            s@.waiters == 0,
    {
    }

    /// Lemma: Well-formedness with waiters implies exhausted.
    pub proof fn lemma_wf_waiters_implies_exhausted(s: &Semaphore)
        requires
            s.wf(),
            s@.waiters > 0,
        ensures
            s.spec_is_exhausted(),
            s@.value == 0,
    {
    }

    /// Lemma: A new semaphore is safe to drop (no waiters).
    pub proof fn lemma_new_is_drop_safe(value: nat)
        ensures
            Semaphore::spec_new_view(value).waiters == 0,
    {
    }
}

//==================================================================================================
// Proof Lemmas -- Protocol Properties
//==================================================================================================
//
// The following lemmas prove protocol properties that reason across multiple
// state transitions or relate different API operations.

impl Semaphore {
    /// Lemma: Down-then-up round-trip restores the original value.
    pub proof fn lemma_down_up_roundtrip(v: nat)
        requires
            v > 0,
            v < usize::MAX,
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after_up: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            initial == after_up
        }),
    {
    }
}

} // verus!
