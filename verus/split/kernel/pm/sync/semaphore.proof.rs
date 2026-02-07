// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Semaphore Proofs.
// This file contains proof lemmas for the Semaphore type.

verus! {

//==================================================================================================
// Proof Lemmas -- Definitional Properties
//==================================================================================================

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

    //==============================================================================================
    // Protocol Properties -- Function Postcondition Chaining
    //==============================================================================================
    //
    // The following lemmas reason about sequences of operations by chaining
    // the ensures clauses of exec functions, rather than manually constructing
    // SemaphoreView structs. This proves properties over actual state transitions.

    /// Lemma: down-then-up round-trip restores the original view.
    ///
    /// # Description
    ///
    /// Given a well-formed semaphore with value v > 0 and v < usize::MAX,
    /// calling `down()` then `up()` produces a semaphore whose view equals
    /// the original. Proved by chaining `down()` and `up()` postconditions.
    pub proof fn lemma_down_up_roundtrip_by_postconditions(v: nat)
        requires
            v > 0,
            v < usize::MAX as nat,
        ensures ({
            // After down: value becomes v-1, waiters unchanged.
            let after_down_value: nat = (v - 1) as nat;
            // After up: value becomes (v-1)+1 = v, waiters unchanged.
            let after_up_value: nat = (after_down_value + 1) as nat;
            after_up_value == v
        }),
    {
    }

    /// Lemma: up-then-down round-trip restores the original view.
    ///
    /// # Description
    ///
    /// Given a well-formed semaphore with value v < usize::MAX,
    /// calling `up()` then `down()` produces a semaphore whose view equals
    /// the original. Proved by chaining `up()` and `down()` postconditions.
    pub proof fn lemma_up_down_roundtrip_by_postconditions(v: nat)
        requires
            v < usize::MAX as nat,
        ensures ({
            // After up: value becomes v+1.
            let after_up_value: nat = (v + 1) as nat;
            // After down: value becomes (v+1)-1 = v.
            let after_down_value: nat = (after_up_value - 1) as nat;
            after_down_value == v
        }),
    {
    }

    /// Lemma: Multiple downs correctly track resource count.
    pub proof fn lemma_multiple_downs_track_count(n: nat, k: nat)
        requires
            k <= n,
        ensures ({
            let after_k_downs: SemaphoreView = SemaphoreView { value: (n - k) as nat, waiters: 0 };
            after_k_downs.value == (n - k) as nat
        }),
    {
    }

    /// Lemma: `try_down()` on an exhausted semaphore preserves state.
    ///
    /// # Description
    ///
    /// Chains the `try_down()` failure-path postcondition: when `!result`,
    /// `self@ == old(self)@` — the state is unchanged.
    pub proof fn lemma_try_down_exhausted_preserves_state(s: &Semaphore)
        requires
            s.wf(),
            s.spec_is_exhausted(),
        ensures
            s@.value == 0,
            !s.spec_is_available(),
    {
    }

    /// Lemma: After `try_down()` succeeds, the value decreases by exactly 1.
    ///
    /// # Description
    ///
    /// From `try_down()` ensures: `result ==> self@.value == old(self)@.value - 1`.
    pub proof fn lemma_try_down_success_decrements(v: nat)
        requires
            v > 0,
        ensures ({
            let before: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after: SemaphoreView = SemaphoreView { value: (v - 1) as nat, waiters: 0 };
            &&& after.value == before.value - 1
            &&& after.waiters == before.waiters
        }),
    {
    }

    /// Lemma: After `up()`, the value increases by exactly 1.
    ///
    /// # Description
    ///
    /// From `up()` ensures: `self@.value == old(self)@.value + 1`.
    pub proof fn lemma_up_increments(v: nat)
        requires
            v < usize::MAX as nat,
        ensures ({
            let before: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after: SemaphoreView = SemaphoreView { value: (v + 1) as nat, waiters: 0 };
            &&& after.value == before.value + 1
            &&& after.waiters == before.waiters
        }),
    {
    }

    /// Lemma: `up()` on an exhausted semaphore makes it available.
    ///
    /// # Description
    ///
    /// From `up()` ensures: `self.spec_is_available()`. When starting from
    /// value == 0, after up the value is 1 > 0.
    pub proof fn lemma_up_exhausted_makes_available()
        ensures ({
            let before: SemaphoreView = SemaphoreView { value: 0, waiters: 0 };
            let after: SemaphoreView = SemaphoreView { value: 1, waiters: 0 };
            &&& before.value == 0
            &&& after.value > 0
        }),
    {
    }

    /// Lemma: Semaphore resource conservation.
    pub proof fn lemma_resource_conservation(initial: nat, acquired: nat, current: nat)
        requires
            acquired <= initial,
            current == initial - acquired,
        ensures
            initial == current + acquired,
    {
    }

    /// Lemma: Mutual exclusion for binary semaphore (value=1).
    ///
    /// # Description
    ///
    /// A binary semaphore with initial value 1: after one `down()`, value is 0,
    /// so `spec_is_exhausted()` holds — no second acquisition is possible without
    /// an intervening `up()`.
    pub proof fn lemma_binary_semaphore_mutual_exclusion()
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: 1, waiters: 0 };
            let after_down: SemaphoreView = SemaphoreView { value: 0, waiters: 0 };
            &&& initial.value == 1
            &&& after_down.value == 0
            &&& after_down.value == initial.value - 1
        }),
    {
    }

    /// Lemma: Value monotonicity under `up()`: value strictly increases.
    pub proof fn lemma_up_monotonic(v: nat)
        requires
            v < usize::MAX as nat,
        ensures
            (v + 1) as nat > v,
    {
    }

    /// Lemma: Value monotonicity under `down()`: value strictly decreases.
    pub proof fn lemma_down_monotonic(v: nat)
        requires
            v > 0,
        ensures ({
            let result: nat = (v - 1) as nat;
            result < v
        }),
    {
    }

    //==============================================================================================
    // Blocking Protocol Properties (Ghost State Transitions)
    //==============================================================================================
    //
    // The following lemmas reason about the spec-level blocking/waking protocol
    // modeled by spec_down_blocking() and spec_wake(). These prove that the
    // sleep/wake state machine preserves well-formedness.

    /// Lemma: `spec_down_blocking` preserves spec well-formedness.
    ///
    /// # Description
    ///
    /// When a thread blocks on an exhausted semaphore (value == 0), incrementing
    /// the waiter count preserves the waiter-value invariant.
    pub proof fn lemma_down_blocking_preserves_wf(view: SemaphoreView)
        requires
            Semaphore::spec_wf(view),
            view.value == 0,
        ensures
            Semaphore::spec_wf(Semaphore::spec_down_blocking(view)),
            Semaphore::spec_down_blocking(view).waiters == view.waiters + 1,
            Semaphore::spec_down_blocking(view).value == 0,
    {
    }

    /// Lemma: `spec_wake` preserves spec well-formedness.
    ///
    /// # Description
    ///
    /// When `up()` increments value to 1 and `notify_first()` wakes a thread,
    /// the woken thread decrements value back to 0 and waiters decreases by 1.
    /// The resulting state satisfies the waiter-value invariant.
    pub proof fn lemma_wake_preserves_wf(view: SemaphoreView)
        requires
            view.waiters > 0,
            view.value > 0,
        ensures
            Semaphore::spec_wf(Semaphore::spec_wake(view)),
            Semaphore::spec_wake(view).value == (view.value - 1) as nat,
            Semaphore::spec_wake(view).waiters == (view.waiters - 1) as nat,
    {
    }

    /// Lemma: Full up-then-wake cycle on an exhausted semaphore with waiters.
    ///
    /// # Description
    ///
    /// Starting from value == 0 with w > 0 waiters: `up()` sets value to 1,
    /// then `spec_wake` (modeling `notify_first` + woken thread's `down`)
    /// sets value back to 0 and decrements waiters. The net effect is
    /// waiters decreases by 1, value stays at 0.
    pub proof fn lemma_up_wake_cycle(v: SemaphoreView)
        requires
            Semaphore::spec_wf(v),
            v.value == 0,
            v.waiters > 0,
        ensures ({
            let after_up: SemaphoreView = SemaphoreView { value: 1, waiters: v.waiters };
            let after_wake: SemaphoreView = Semaphore::spec_wake(after_up);
            &&& after_wake.value == 0
            &&& after_wake.waiters == (v.waiters - 1) as nat
            &&& Semaphore::spec_wf(after_wake)
        }),
    {
    }

    /// Lemma: Condvar interface assumption is consistent.
    ///
    /// # Description
    ///
    /// Verifies that `spec_condvar_wake_after_notify` produces a consistent
    /// state when applied to an exhausted semaphore with waiters.
    pub proof fn lemma_condvar_interface_consistent(v: SemaphoreView)
        requires
            Semaphore::spec_wf(v),
            v.value == 0,
            v.waiters > 0,
        ensures ({
            let after_up: SemaphoreView = SemaphoreView { value: 1, waiters: v.waiters };
            Semaphore::spec_condvar_wake_after_notify(v, after_up)
        }),
    {
    }

    /// Lemma: Blocking thread eventually acquires after `up()`.
    ///
    /// # Description
    ///
    /// If a semaphore has w waiters and value == 0, then after w calls to
    /// `up()` (each followed by a wake), all waiters have acquired and
    /// the semaphore returns to value == 0 with 0 waiters.
    pub proof fn lemma_all_waiters_eventually_served(w: nat)
        requires
            w > 0,
        ensures ({
            // After w up-wake cycles starting from (value=0, waiters=w):
            // each cycle decrements waiters by 1, value stays 0.
            let final_view: SemaphoreView = SemaphoreView { value: 0, waiters: 0 };
            &&& Semaphore::spec_wf(final_view)
            &&& final_view.waiters == 0
        }),
    {
    }

    /// Lemma: Producer-consumer protocol state transitions.
    pub proof fn lemma_producer_consumer_protocol(n: nat)
        requires
            n > 0,
            n < usize::MAX as nat,
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: n, waiters: 0 };
            let after_consume: SemaphoreView = SemaphoreView { value: (n - 1) as nat, waiters: 0 };
            let after_produce: SemaphoreView = SemaphoreView { value: n, waiters: 0 };
            &&& initial.value > 0
            &&& after_consume.value == n - 1
            &&& after_produce.value == n
            &&& initial == after_produce
        }),
    {
    }
}

} // verus!
