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

/// Ghost state representing the caller's execution context.
///
/// # Description
///
/// Models the safety preconditions of the original `unsafe` functions
/// (`down()` and `up()`). The original kernel semaphore requires specific
/// caller obligations (interrupts disabled, not the kernel process, etc.).
/// This ghost struct encodes those conditions so they can be checked at
/// the spec level. Callers must provide a `Ghost<CallerContext>` satisfying
/// the appropriate predicate.
pub struct CallerContext {
    /// Whether interrupts are currently disabled.
    pub interrupts_disabled: bool,
    /// Whether the caller is the kernel process.
    pub is_kernel_process: bool,
    /// Whether the caller holds resources (relevant for `down()`).
    pub holds_resources: bool,
    /// Whether the caller holds a process manager reference (relevant for `up()`).
    pub holds_pm_ref: bool,
}

//==================================================================================================
// CallerContext Spec Functions
//==================================================================================================

impl CallerContext {
    /// Spec function: well-formedness predicate (invariant).
    ///
    /// # Description
    ///
    /// `CallerContext` is pure ghost state encoding safety preconditions for
    /// `down()` and `up()`. All fields are independent boolean flags with no
    /// internal consistency constraints, so the invariant is trivially true.
    /// Provided per the specifying-and-proving-types methodology (Step 2).
    pub closed spec fn inv(&self) -> bool {
        true
    }

    /// Spec function: caller context satisfies `down()` safety conditions.
    ///
    /// # Description
    ///
    /// The original `down()` is `unsafe` and requires:
    /// - The caller is running with interrupts disabled.
    /// - The calling process is not the kernel process.
    /// - The function is invoked without holding any resources.
    pub open spec fn safe_for_down(&self) -> bool {
        &&& self.interrupts_disabled
        &&& !self.is_kernel_process
        &&& !self.holds_resources
    }

    /// Spec function: caller context satisfies `up()` safety conditions.
    ///
    /// # Description
    ///
    /// The original `up()` is `unsafe` and requires:
    /// - The caller is running with interrupts disabled.
    /// - The calling process does not hold a reference to the process manager.
    pub open spec fn safe_for_up(&self) -> bool {
        &&& self.interrupts_disabled
        &&& !self.holds_pm_ref
    }
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Outcome of a `down_or_block()` call.
///
/// # Description
///
/// Models the two possible outcomes of the original `down()`:
/// - `Acquired`: The semaphore was available, value was decremented (instant success).
/// - `WouldBlock`: The semaphore was exhausted, the thread would sleep on the condvar.
///   In the original, this enters the `Condvar::wait()` loop. In the verified model,
///   the ghost waiter view is updated via `spec_down_blocking()`.
#[verifier::ext_equal]
pub enum DownOutcome {
    /// Semaphore was available; value decremented by 1.
    Acquired,
    /// Semaphore was exhausted; thread would block on condvar.
    WouldBlock,
}

impl Semaphore {
    /// Spec function: well-formedness predicate (invariant).
    ///
    /// # Description
    ///
    /// Enforces consistency between concrete fields and abstract state:
    /// - The concrete `value` matches the view's value.
    /// - Waiters are ghost state (always 0 in exec-constructed views).
    /// - If there are waiters, the value must be zero (threads only wait when
    ///   the semaphore count is exhausted). This constraint is enforced at the
    ///   spec level for ghost state transitions.
    ///
    /// This is `pub closed` per the specifying-and-proving-types methodology
    /// (Step 2): users must maintain the invariant but cannot see implementation
    /// internals. Use `reveal(Semaphore::wf)` in proofs that need the body.
    pub closed spec fn wf(&self) -> bool {
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
    /// Models the case when `up()` increments the value from 0 to 1 and
    /// `notify_first()` wakes a sleeping thread, which then successfully
    /// decrements the value. The net effect: value back to 0, waiters
    /// decremented by 1. In the protocol, this is always called with
    /// `value == 1` (just after `up()` on an exhausted semaphore with waiters).
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
            view.value == 1,
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
        recommends
            Semaphore::spec_wf(view),
            n <= view.waiters,
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

    /// Spec function: maps `try_down()` result to original error semantics.
    ///
    /// # Description
    ///
    /// Formalizes the correspondence between the verified model's `bool`
    /// return and the original `Result<(), Error>`:
    /// - `true`  corresponds to `Ok(())`: value was positive, decremented by 1.
    /// - `false` corresponds to `Err(ErrorCode::TryAgain)`: value was zero,
    ///   state unchanged.
    ///
    /// # Parameters
    ///
    /// - `result`: The `bool` returned by the verified `try_down()`.
    /// - `before`: The semaphore view before the call.
    /// - `after`: The semaphore view after the call.
    ///
    /// # Returns
    ///
    /// `true` if the result-to-view mapping is consistent.
    pub open spec fn spec_try_down_result_maps_ok(result: bool, before: SemaphoreView, after: SemaphoreView) -> bool {
        &&& (result ==> after.value == (before.value - 1) as nat && after.waiters == before.waiters)
        &&& (!result ==> after == before && before.value == 0)
    }

    /// Spec function: ghost view after `down_or_block()` for the WouldBlock case.
    ///
    /// # Description
    ///
    /// When `down_or_block()` returns `WouldBlock`, the exec state is unchanged
    /// but the ghost waiter count should be incremented (a thread enters the
    /// condvar queue). This function computes the updated ghost view.
    ///
    /// # Parameters
    ///
    /// - `before`: The ghost semaphore view before the call.
    /// - `outcome`: The outcome of `down_or_block()`.
    ///
    /// # Returns
    ///
    /// The updated ghost semaphore view.
    pub open spec fn spec_down_or_block_ghost_view(before: SemaphoreView, outcome: DownOutcome) -> SemaphoreView {
        match outcome {
            DownOutcome::Acquired => SemaphoreView { value: (before.value - 1) as nat, waiters: before.waiters },
            DownOutcome::WouldBlock => Self::spec_down_blocking(before),
        }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Semaphore {
    type V = SemaphoreView;

    /// NOTE: This must remain `open spec fn` because the Verus `View` trait
    /// requires the view function to be `open`. The specifying-and-proving-types
    /// methodology (Step 1) recommends `pub closed spec fn`, but the trait
    /// definition cannot be satisfied with a `closed` implementation.
    open spec fn view(&self) -> SemaphoreView {
        SemaphoreView { value: self.value as nat, waiters: 0 }
    }
}

} // verus!
