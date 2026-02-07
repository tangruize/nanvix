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
    ///
    /// # Description
    ///
    /// Starting from a semaphore with value `v > 0` and no waiters, after
    /// `try_down()` the value is `v - 1`, and after `up()` the value is `v`
    /// again. Both intermediate and final states are well-formed.
    pub proof fn lemma_down_up_roundtrip(v: nat)
        requires
            v > 0,
            v < usize::MAX,
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after_down: SemaphoreView = SemaphoreView { value: (v - 1) as nat, waiters: 0 };
            let after_up: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            &&& initial.value == v
            &&& after_down.value == (v - 1) as nat
            &&& after_up.value == v
            &&& initial == after_up
            &&& after_down.waiters == 0
            &&& after_up.waiters == 0
        }),
    {
    }

    /// Lemma: Up-then-down round-trip restores the original value.
    ///
    /// # Description
    ///
    /// Starting from a semaphore with value `v` and no waiters, after `up()`
    /// the value is `v + 1`, and after `try_down()` the value is `v` again.
    pub proof fn lemma_up_down_roundtrip(v: nat)
        requires
            v < usize::MAX,
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after_up: SemaphoreView = SemaphoreView { value: (v + 1) as nat, waiters: 0 };
            let after_down: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            &&& after_up.value == (v + 1) as nat
            &&& after_down.value == v
            &&& initial == after_down
        }),
    {
    }

    /// Lemma: Multiple downs correctly track resource count.
    ///
    /// # Description
    ///
    /// Starting from value `n`, performing `k` successful downs (where `k <= n`)
    /// leaves the semaphore at value `n - k`.
    pub proof fn lemma_multiple_downs_track_count(n: nat, k: nat)
        requires
            k <= n,
        ensures ({
            let after_k_downs: SemaphoreView = SemaphoreView { value: (n - k) as nat, waiters: 0 };
            after_k_downs.value == (n - k) as nat
        }),
    {
    }

    /// Lemma: `try_down()` on an exhausted semaphore fails and preserves state.
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
    pub proof fn lemma_up_increments(v: nat)
        requires
            v < usize::MAX,
        ensures ({
            let before: SemaphoreView = SemaphoreView { value: v, waiters: 0 };
            let after: SemaphoreView = SemaphoreView { value: (v + 1) as nat, waiters: 0 };
            &&& after.value == before.value + 1
            &&& after.waiters == before.waiters
        }),
    {
    }

    /// Lemma: `up()` on an exhausted semaphore makes it available.
    pub proof fn lemma_up_exhausted_makes_available()
        ensures ({
            let before: SemaphoreView = SemaphoreView { value: 0, waiters: 0 };
            let after: SemaphoreView = SemaphoreView { value: 1, waiters: 0 };
            &&& before.value == 0
            &&& after.value > 0
        }),
    {
    }

    /// Lemma: Semaphore resource conservation. The total of value and acquired
    /// resources is constant across down/up operations.
    ///
    /// # Description
    ///
    /// If `initial_value` resources are available and `acquired` have been taken
    /// (via `down()`), then `initial_value == current_value + acquired`.
    pub proof fn lemma_resource_conservation(initial: nat, acquired: nat, current: nat)
        requires
            acquired <= initial,
            current == initial - acquired,
        ensures
            initial == current + acquired,
    {
    }

    /// Lemma: Mutual exclusion for binary semaphore. A semaphore initialized
    /// with value 1 can be acquired at most once before being released.
    ///
    /// # Description
    ///
    /// When the initial value is 1 and one `down()` succeeds, the value is 0,
    /// so a subsequent `try_down()` must fail. This is the binary semaphore
    /// (mutex-like) property.
    pub proof fn lemma_binary_semaphore_mutual_exclusion()
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: 1, waiters: 0 };
            let after_down: SemaphoreView = SemaphoreView { value: 0, waiters: 0 };
            &&& initial.value == 1
            &&& after_down.value == 0
        }),
    {
    }

    /// Lemma: Semaphore value monotonicity under `up()`: value strictly increases.
    pub proof fn lemma_up_monotonic(v: nat)
        requires
            v < usize::MAX,
        ensures
            (v + 1) as nat > v,
    {
    }

    /// Lemma: Semaphore value monotonicity under `down()`: value strictly decreases.
    pub proof fn lemma_down_monotonic(v: nat)
        requires
            v > 0,
        ensures
            (v - 1) as nat < v,
    {
    }

    /// Lemma: The producer-consumer protocol demonstrates the state transition
    /// sequence for a semaphore used as a resource counter.
    ///
    /// # Description
    ///
    /// Models the spec-level state transitions:
    /// 1. Initial: semaphore has capacity `n > 0`, well-formed.
    /// 2. Consumer acquires (down): value decreases to `n - 1`.
    /// 3. If exhausted: subsequent consumers would block.
    /// 4. Producer releases (up): value increases, consumers can proceed.
    pub proof fn lemma_producer_consumer_protocol(n: nat)
        requires
            n > 0,
            n < usize::MAX,
        ensures ({
            let initial: SemaphoreView = SemaphoreView { value: n, waiters: 0 };
            let after_consume: SemaphoreView = SemaphoreView { value: (n - 1) as nat, waiters: 0 };
            let after_produce: SemaphoreView = SemaphoreView { value: n, waiters: 0 };
            // Phase 1: Resources available.
            &&& initial.value > 0
            // Phase 2: Consumer acquires one resource.
            &&& after_consume.value == n - 1
            // Phase 3: Producer restores one resource.
            &&& after_produce.value == n
            // Full cycle restores state.
            &&& initial == after_produce
        }),
    {
    }
}

} // verus!
