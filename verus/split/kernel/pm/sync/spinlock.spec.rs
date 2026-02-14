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
/// Represents the observable state of a spinlock: whether it is locked or unlocked,
/// which instance it belongs to (via the ghost `id` field), and whether a lock token
/// is currently outstanding (via `token_issued`).
#[verifier::ext_equal]
pub struct SpinlockView {
    /// Whether the spinlock is currently held.
    pub locked: bool,
    /// Ghost identity for distinguishing lock instances.
    pub id: nat,
    /// Whether a `LockToken` for this lock is currently outstanding.
    /// When `true`, the lock is held and a token exists. When `false`,
    /// either the lock is unlocked or the token has been consumed.
    pub token_issued: bool,
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
/// binding the token to the specific lock instance (via the `id` field in the view)
/// and state. A token produced by lock instance A cannot be used to unlock instance B,
/// because `unlock()` requires `token.view == old(self)@` which includes identity.
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

    /// Invariant predicate for internal consistency.
    ///
    /// # Description
    ///
    /// Enforces the token-ownership biconditional: the lock is held if and only if
    /// a token is outstanding. This captures the full reachable-state invariant:
    /// - `new()` produces `(locked=false, token_issued=false)`.
    /// - `lock()`/`try_lock()` transitions to `(locked=true, token_issued=true)`.
    /// - `unlock()` transitions to `(locked=false, token_issued=false)`.
    ///
    /// This prevents both "unlocked with token outstanding" (double-unlock) and
    /// "locked without token" (unreachable from API, but now excluded by inv).
    pub closed spec fn inv(&self) -> bool {
        self.locked == self.token_issued()
    }

    /// Spec function: returns whether a token is currently outstanding.
    pub open spec fn token_issued(&self) -> bool {
        self@.token_issued
    }

    /// Spec function: the view of a newly created spinlock with the given identity.
    pub open spec fn spec_new_view(id: nat) -> SpinlockView {
        SpinlockView { locked: false, id: id, token_issued: false }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

/// NOTE: `view()` must be `open spec fn` because the Verus `View` trait requires it.
/// The trait signature mandates `open`, so this cannot be `closed`. Users observe
/// only the abstract `SpinlockView` (which uses `nat` instead of `usize`), not the
/// concrete `Spinlock` fields directly.
impl View for Spinlock {
    type V = SpinlockView;

    open spec fn view(&self) -> SpinlockView {
        SpinlockView { locked: self.locked, id: self.id as nat, token_issued: self.token_issued }
    }
}

} // verus!
