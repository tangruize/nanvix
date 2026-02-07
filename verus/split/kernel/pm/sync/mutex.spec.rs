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
/// because `unlock()` requires `token.spec_view() == old(self)@` which includes identity.
///
/// The `view` field is private to prevent external code from constructing forged
/// tokens. External callers read the token's view via `spec_view()`.
pub tracked struct MutexToken {
    /// Ghost snapshot of the mutex's view when the lock was acquired.
    ghost view: MutexView,
}

//==================================================================================================
// MutexToken Spec Functions
//==================================================================================================

impl MutexToken {
    /// Spec function: returns the ghost snapshot of the mutex's view at lock time.
    pub open spec fn spec_view(&self) -> MutexView {
        self.view
    }
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

    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// Enforces the token-ownership biconditional: the mutex is held if and only if
    /// a token is outstanding. This captures the full reachable-state invariant:
    /// - `new()` produces `(locked=false, token_issued=false)`.
    /// - `lock()`/`try_lock()` transitions to `(locked=true, token_issued=true)`.
    /// - `unlock()` transitions to `(locked=false, token_issued=false)`.
    ///
    /// **Note:** This is a local (per-mutex) invariant. It does not capture the
    /// global uniqueness property that at most one token exists per mutex `id`
    /// across the entire system. A global resource algebra would be needed for
    /// that, which is beyond Verus's current tracked-token model.
    pub open spec fn wf(&self) -> bool {
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

impl View for Mutex {
    type V = MutexView;

    open spec fn view(&self) -> MutexView {
        MutexView { locked: self.locked, id: self.id@, token_issued: self.token_issued@ }
    }
}

} // verus!
