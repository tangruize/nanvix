// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Semaphore Specification.
// This file contains spec functions for the Semaphore type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a Semaphore.
///
/// # Description
///
/// Represents the observable state of a semaphore: the current resource count
/// and the number of threads waiting in the sleeping queue. The `waiters` field
/// is ghost state — it is tracked at the spec level only and is not stored in the
/// exec `Semaphore` struct. It models the condvar queue length for spec-level
/// reasoning about the sleep/wake protocol.
#[verifier::ext_equal]
pub struct SemaphoreView {
    /// Current count of available resources.
    pub value: nat,
    /// Number of threads currently waiting on the semaphore (ghost state).
    pub waiters: nat,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Semaphore {
    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Enforces consistency between concrete fields and abstract state:
    /// - The concrete `value` matches the view's value.
    /// - Waiters are ghost state (always 0 in exec-constructed views).
    /// - If there are waiters, the value must be zero (threads only wait when
    ///   the semaphore count is exhausted). This constraint is enforced at the
    ///   spec level for ghost state transitions.
    pub open spec fn wf(&self) -> bool {
        &&& self.value as nat == self@.value
        &&& (self@.waiters > 0 ==> self@.value == 0)
    }

    /// Spec function: well-formedness for abstract views (ghost state).
    ///
    /// # Description
    ///
    /// Enforces the waiter-value constraint on abstract views directly,
    /// used for reasoning about spec-level state transitions.
    pub open spec fn spec_wf(view: SemaphoreView) -> bool {
        view.waiters > 0 ==> view.value == 0
    }

    /// Spec function: returns the current count of available resources.
    pub open spec fn spec_value(&self) -> nat {
        self@.value
    }

    /// Spec function: returns the number of waiting threads (ghost state).
    pub open spec fn spec_waiters(&self) -> nat {
        self@.waiters
    }

    /// Spec function: returns whether the semaphore has available resources.
    pub open spec fn spec_is_available(&self) -> bool {
        self@.value > 0
    }

    /// Spec function: returns whether the semaphore is exhausted (count is zero).
    pub open spec fn spec_is_exhausted(&self) -> bool {
        self@.value == 0
    }

    /// Spec function: the view of a newly created semaphore with the given initial value.
    pub open spec fn spec_new_view(value: nat) -> SemaphoreView {
        SemaphoreView { value: value, waiters: 0 }
    }

    /// Spec function: returns whether the semaphore is safe to drop.
    ///
    /// # Description
    ///
    /// A semaphore is safe to drop when no threads are waiting on it.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self@.waiters == 0
    }

    /// Spec function: state transition for blocking down (thread sleeps).
    ///
    /// # Description
    ///
    /// Models the case when `down()` finds value == 0 and the thread sleeps
    /// on the condvar. The waiters count is incremented. This is a ghost
    /// state transition — no exec function performs it directly.
    ///
    /// # Parameters
    ///
    /// - `view`: The current semaphore view.
    ///
    /// # Returns
    ///
    /// The new semaphore view with waiters incremented.
    pub open spec fn spec_down_blocking(view: SemaphoreView) -> SemaphoreView {
        SemaphoreView { value: view.value, waiters: (view.waiters + 1) as nat }
    }

    /// Spec function: state transition for wake (thread wakes and acquires).
    ///
    /// # Description
    ///
    /// Models the case when `up()` increments the value and `notify_first()`
    /// wakes a sleeping thread, which then successfully decrements the value.
    /// The net effect: value unchanged (up then down cancel), waiters decremented.
    /// Precondition: there must be at least one waiter, and value must be > 0
    /// (the `up()` has already incremented it).
    ///
    /// # Parameters
    ///
    /// - `view`: The current semaphore view (after `up()` incremented value).
    ///
    /// # Returns
    ///
    /// The new semaphore view with value decremented and waiters decremented.
    pub open spec fn spec_wake(view: SemaphoreView) -> SemaphoreView
        recommends
            view.waiters > 0,
            view.value > 0,
    {
        SemaphoreView { value: (view.value - 1) as nat, waiters: (view.waiters - 1) as nat }
    }

    /// Spec function: condvar interface assumption.
    ///
    /// # Description
    ///
    /// Formal statement of the trust assumption on the condvar module:
    /// if a thread is waiting (waiters > 0) and `notify_first()` is called
    /// after `up()` increments the value, exactly one waiter is woken and
    /// successfully acquires the semaphore.
    ///
    /// # Limitation
    ///
    /// This spec defines the semaphore's *expectation* of condvar behavior.
    /// It is not imported by the condvar module (`kernel::pm::sync::condvar`)
    /// and changes to the condvar implementation will not trigger a verification
    /// failure here. Cross-module spec composition requires a shared interface
    /// contract that both modules import, which is not yet implemented.
    /// The condvar module's spec functions are defined in
    /// `verus/split/kernel/pm/sync/condvar.spec.rs` (see `spec_notify_all_result`
    /// and related functions).
    pub open spec fn spec_condvar_wake_after_notify(before_up: SemaphoreView, after_up: SemaphoreView) -> bool {
        &&& after_up.value == before_up.value + 1
        &&& after_up.waiters == before_up.waiters
        &&& before_up.waiters > 0 ==> {
            let after_wake: SemaphoreView = Self::spec_wake(after_up);
            &&& after_wake.value == before_up.value
            &&& after_wake.waiters == (before_up.waiters - 1) as nat
        }
    }

    /// Spec function: result of applying n up-wake cycles.
    ///
    /// # Description
    ///
    /// Recursively models n iterations of the up-then-wake cycle: each
    /// iteration increments `value` by 1 (up), then applies `spec_wake`
    /// (woken thread decrements value and waiters). Used to reason
    /// inductively about draining all waiters.
    ///
    /// # Parameters
    ///
    /// - `view`: The current semaphore view.
    /// - `n`: The number of up-wake cycles to apply.
    ///
    /// # Returns
    ///
    /// The semaphore view after n up-wake cycles.
    pub open spec fn spec_after_n_up_wake_cycles(view: SemaphoreView, n: nat) -> SemaphoreView
        decreases n,
    {
        if n == 0 {
            view
        } else {
            let after_up: SemaphoreView = SemaphoreView { value: view.value + 1, waiters: view.waiters };
            let after_wake: SemaphoreView = Self::spec_wake(after_up);
            Self::spec_after_n_up_wake_cycles(after_wake, (n - 1) as nat)
        }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Semaphore {
    type V = SemaphoreView;

    open spec fn view(&self) -> SemaphoreView {
        SemaphoreView { value: self.value as nat, waiters: 0 }
    }
}

} // verus!
