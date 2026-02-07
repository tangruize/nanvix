// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Spinlock Specification.
// This file contains spec functions for the Spinlock type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a Spinlock.
///
/// # Description
///
/// Represents the observable state of a spinlock: whether it is locked or unlocked.
#[verifier::ext_equal]
pub struct SpinlockView {
    /// Whether the spinlock is currently held.
    pub locked: bool,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Spinlock {
    /// Spec function: returns whether the spinlock is locked.
    pub open spec fn spec_is_locked(&self) -> bool {
        self.locked
    }

    /// Spec function: returns whether the spinlock is unlocked.
    pub open spec fn spec_is_unlocked(&self) -> bool {
        !self.locked
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Trivially true for Spinlock (no structural invariants beyond a valid bool).
    /// Included for API consistency with other verified modules that have meaningful
    /// `wf()` predicates. Would become non-trivial if the exec struct gains fields.
    pub open spec fn wf(&self) -> bool {
        true
    }

    /// Spec function: the view of a newly created spinlock.
    pub open spec fn spec_new_view() -> SpinlockView {
        SpinlockView { locked: false }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for Spinlock {
    type V = SpinlockView;

    open spec fn view(&self) -> SpinlockView {
        SpinlockView { locked: self.locked }
    }
}

} // verus!
