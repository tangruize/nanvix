// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Mutex Specification.
// This file contains spec functions for the Mutex type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a Mutex.
///
/// # Description
///
/// Represents the observable state of a mutex: whether it is locked,
/// which instance it belongs to (via the ghost `id` field), and whether
/// a lock token is currently outstanding (via `token_issued`).
#[verifier::ext_equal]
pub struct MutexView {
    /// Whether the mutex is currently held.
    pub locked: bool,
    /// Ghost identity for distinguishing mutex instances.
    pub id: nat,
    /// Whether a `MutexToken` for this mutex is currently outstanding.
    pub token_issued: bool,
}

/// Tracked ghost token representing the obligation to release a held mutex.
///
/// # Description
///
/// When `lock()` or a successful `try_lock()` acquires the mutex, a `MutexToken`
/// is produced. The caller must pass this token to `unlock()` to discharge the
/// lock-release obligation. This models the `MutexGuard`/`Drop` pattern from
/// the original implementation at the proof level.
///
/// The token carries a ghost snapshot of the mutex's view at acquisition time,
/// binding the token to the specific mutex instance (via the `id` field in the view)
/// and state. A token produced by mutex instance A cannot be used to unlock instance B,
/// because `unlock()` requires `token.view == old(self)@` which includes identity.
///
/// The `view` field is `pub ghost` because Verus requires `pub open spec fn`
/// bodies to reference only public fields (the "opaqueness" rule). A private field
/// with a `pub closed spec fn` getter would make the getter opaque, preventing
/// proof reasoning about token contents. See Trust Assumption T3 in the module
/// header for the implications.
pub tracked struct MutexToken {
    /// Ghost snapshot of the mutex's view when the lock was acquired.
    pub ghost view: MutexView,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl Mutex {
    /// Spec function: returns whether the mutex is locked.
    pub open spec fn spec_is_locked(&self) -> bool {
        self.locked
    }

    /// Spec function: returns whether the mutex is unlocked.
    pub open spec fn spec_is_unlocked(&self) -> bool {
        !self.locked
    }

    /// Spec function: well-formedness predicate (invariant).
    ///
    /// # Description
    ///
    /// Enforces the token-ownership biconditional: the mutex is held if and only if
    /// a token is outstanding. This captures the full reachable-state invariant:
    /// - `new()` produces `(locked=false, token_issued=false)`.
    /// - `lock()`/`try_lock()` transitions to `(locked=true, token_issued=true)`.
    /// - `unlock()` transitions to `(locked=false, token_issued=false)`.
    ///
    /// This is `pub closed` per the spec methodology (Step 2): public so
    /// callers can require/ensure it, but closed so implementation
    /// invariant details are not leaked to users.
    ///
    /// **Note:** This is a local (per-mutex) invariant. It does not capture the
    /// global uniqueness property that at most one token exists per mutex `id`
    /// across the entire system. A global resource algebra would be needed for
    /// that, which is beyond Verus's current tracked-token model.
    pub closed spec fn wf(&self) -> bool {
        self.locked == self.token_issued()
    }

    /// Spec function: returns whether a token is currently outstanding.
    pub open spec fn token_issued(&self) -> bool {
        self@.token_issued
    }

    /// Spec function: the view of a newly created mutex with the given identity.
    pub open spec fn spec_new_view(id: nat) -> MutexView {
        MutexView { locked: false, id: id, token_issued: false }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

/// NOTE: `view()` must be `open spec fn` because the Verus `View` trait requires it.
/// The trait signature mandates `open`, so this cannot be `closed`. Users observe
/// only the abstract `MutexView` (which uses `nat` instead of `usize`), not the
/// concrete `Mutex` fields directly.
impl View for Mutex {
    type V = MutexView;

    open spec fn view(&self) -> MutexView {
        MutexView { locked: self.locked, id: self.id as nat, token_issued: self.token_issued }
    }
}

} // verus!
