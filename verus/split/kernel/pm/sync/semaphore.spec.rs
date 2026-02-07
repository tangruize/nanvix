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
/// and the number of threads waiting in the sleeping queue.
#[verifier::ext_equal]
pub struct SemaphoreView {
    /// Current count of available resources.
    pub value: nat,
    /// Number of threads currently waiting on the semaphore.
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
    /// - The concrete `waiters` matches the view's waiters.
    /// - If there are waiters, the value must be zero (threads only wait when
    ///   the semaphore count is exhausted).
    pub open spec fn wf(&self) -> bool {
        &&& self.value as nat == self@.value
        &&& self.waiters as nat == self@.waiters
        &&& (self@.waiters > 0 ==> self@.value == 0)
    }

    /// Spec function: returns the current count of available resources.
    pub open spec fn spec_value(&self) -> nat {
        self@.value
    }

    /// Spec function: returns the number of waiting threads.
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
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Semaphore {
    type V = SemaphoreView;

    open spec fn view(&self) -> SemaphoreView {
        SemaphoreView { value: self.value as nat, waiters: self.waiters as nat }
    }
}

} // verus!
