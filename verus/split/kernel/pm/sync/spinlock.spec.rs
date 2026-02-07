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

/// Tracked ghost token representing the obligation to release a held spinlock.
///
/// # Description
///
/// When `lock()` or a successful `try_lock()` acquires the spinlock, a `LockToken`
/// is produced. The caller must pass this token to `unlock()` to discharge the
/// lock-release obligation. This models the `SpinlockGuard`/`Drop` pattern from
/// the original implementation at the proof level.
///
/// The token carries a ghost snapshot of the spinlock's view at acquisition time,
/// binding the token to the specific lock instance and state.
///
/// # Soundness
///
/// Each `LockToken` must correspond to exactly one lock acquisition.
/// Callers must not duplicate or forge tokens.
pub tracked struct LockToken {
    /// Ghost snapshot of the spinlock's view when the lock was acquired.
    pub ghost view: SpinlockView,
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
