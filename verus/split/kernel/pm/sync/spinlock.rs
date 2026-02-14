// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Spinlock Implementation
//!
//! A simple spinlock providing mutual exclusion.
//!
//! ## Verified Properties
//!
//! - A new spinlock is always unlocked with no token outstanding.
//! - `try_lock` succeeds iff the lock was previously unlocked, and locks it.
//! - `try_lock` fails iff the lock was previously locked, leaving state unchanged.
//! - `unlock` transitions the lock from locked to unlocked and consumes the token.
//! - Lock-then-unlock round-trip restores the original unlocked state.
//! - `is_locked` and `is_unlocked` (on `SpinlockView`) are complementary predicates.
//! - Lock instance identity (`id`) is preserved across all state transitions.
//! - Tokens are bound to the producing lock instance via view identity.
//! - Invariant (`inv()`) enforces: unlocked implies no token outstanding.
//! - Token issuance is tracked: only one token per lock at a time.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicBool` for lock-free
//! interior mutability and `core::sync::atomic::Ordering` for memory ordering.
//! For verification, we model the lock state as a plain `bool` field and use
//! `&mut self` for state transitions. This is a sequential model that verifies
//! the lock protocol (state machine correctness) without reasoning about atomicity
//! or memory ordering.
//!
//! **This verified code is a specification model, not a runtime replacement.** The
//! kernel uses the original `src/kernel/src/pm/sync/spinlock.rs` (with `AtomicBool`
//! and spin-wait loop) at runtime. The verified model proves the state machine
//! protocol is correct: every reachable state satisfies `inv()`, tokens are
//! instance-bound, and lock/unlock transitions are sound. The `lock()` body
//! delegates to `try_lock()` (a single CAS model) because, under the sequential
//! preconditions, success is guaranteed — this proves the postconditions from
//! verified code rather than trusting an `external_body`.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the spinlock
//! protocol. The following are explicitly **out of scope**:
//! - **Concurrency and atomicity**: The sequential `&mut self` model does not capture
//!   concurrent thread access or atomic memory ordering.
//! - **Liveness and progress**: The spinning behavior of `lock()` and its termination
//!   under fairness assumptions are not modeled. The `lock()` precondition
//!   (`is_unlocked`) effectively models the instant-success case.
//! - **Drop-based RAII**: Automatic lock release via `SpinlockGuard`/`Drop` is modeled
//!   via explicit `LockToken` consumption in `unlock()`.
//!
//! ## API Divergence
//!
//! The original implementation uses `lock(&self) -> SpinlockGuard` with interior
//! mutability via `AtomicBool`. The verified version uses `lock(&mut self)` because
//! Verus requires exclusive references for state mutation. This means the verification
//! covers the *state machine protocol* (lock/unlock transitions) but not the
//! *concurrent access pattern* that motivates the spinlock's existence. In the
//! concurrent original, `&self` access is safe due to `AtomicBool` interior mutability.
//!
//! ## Trust Boundaries
//!
//! - `lock()`: Fully verified. The sequential model's preconditions (`inv()`,
//!   `is_unlocked()`, `!token_issued`) guarantee that the lock is acquirable,
//!   so `lock()` delegates to `try_lock()` without needing `external_body`. The
//!   original implementation's spin-wait loop (atomic CAS + pause) is a HAL-level
//!   concern not modeled in the sequential verification.
//! - `SpinlockGuard` and `Drop`: Modeled via tracked `LockToken` ghost state. The
//!   original uses RAII via `SpinlockGuard<'a>` to auto-release on drop. Since Verus
//!   cannot reason about `Drop` directly, we model the obligation using a tracked ghost
//!   token: `lock()` and `try_lock()` produce a `LockToken` that must be consumed by
//!   `unlock()`. The token's view includes the lock's ghost `id`, so tokens from one
//!   lock instance cannot unlock a different instance. This makes the lock-release
//!   obligation explicit and instance-bound at the type level.
//! - `arch::cpu::pause()`: CPU hint with no semantic effect on lock state.
//!
//! ## Trust Assumptions
//!
//! - **T1: ID Uniqueness.** Callers must provide a unique `id` per spinlock
//!   instance. If two spinlocks share the same `id`, token isolation between them is
//!   not guaranteed. This parallels the original's reliance on reference identity
//!   (`&'a Spinlock`) which Verus cannot reason about. A global ID allocator
//!   could enforce this mechanically but is outside the current scope.

use vstd::prelude::*;

// Include specifications.
include!("spinlock.spec.rs");

// Include proofs.
include!("spinlock.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A spinlock providing mutual exclusion.
///
/// # Description
///
/// Wraps a boolean locked state. In the original implementation, this is an
/// `AtomicBool`; here it is a plain `bool` for verification purposes.
///
/// # Representation
///
/// The fields are `pub` because the Verus `View` trait requires `open spec fn view()`,
/// which accesses struct fields directly. Verus mandates that field expressions in
/// `pub open spec fn` are well-formed everywhere, which requires `pub` visibility.
/// Verified code should use the `View` trait (`self@`) and `SpinlockView` predicates
/// rather than direct field access.
pub struct Spinlock {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
    /// Identity for distinguishing lock instances.
    /// Callers must provide a unique `id` per instance at construction time.
    pub id: usize,
    /// Tracking of whether a `LockToken` is currently outstanding.
    /// Set to `true` on lock acquisition, `false` on unlock.
    pub token_issued: bool,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Spinlock {
    /// Creates a new unlocked spinlock.
    ///
    /// # Parameters
    ///
    /// - `id`: Identity for this lock instance. Callers should ensure
    ///   unique IDs across all spinlock instances to preserve token isolation.
    ///
    /// # Returns
    ///
    /// A new `Spinlock` in the unlocked state with the given identity.
    pub fn new(id: usize) -> (result: Self)
        ensures
            !result@.locked,
            result@.is_unlocked(),
            result@ == SpinlockView::spec_new(id as nat),
            result@.id == id as nat,
            result.inv(),
    {
        proof { reveal(Spinlock::inv); }
        Spinlock { locked: false, id: id, token_issued: false }
    }

    /// Attempts to acquire the lock without spinning.
    ///
    /// # Description
    ///
    /// Models a single atomic `compare_exchange(false, true)` operation.
    /// Returns `true` if the lock was successfully acquired (was unlocked),
    /// `false` if the lock was already held (was locked).
    ///
    /// On success, produces a tracked `LockToken` that the caller must pass to
    /// `unlock()` to discharge the lock-release obligation.
    ///
    /// NOTE: Verification helper — not present in original source. Decomposes the
    /// single CAS operation from `lock()`'s loop body for verifiable reasoning.
    /// Both success and failure paths are non-vacuously verified: calling on an
    /// unlocked spinlock succeeds; calling on a locked spinlock (with token
    /// outstanding) fails and preserves state.
    ///
    /// # Returns
    ///
    /// `true` if the lock was acquired (with `LockToken` in the `Tracked<Option>`),
    /// `false` otherwise (with `None`).
    pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<LockToken>>))
        requires
            old(self).inv(),
        ensures
            result.0 == !old(self)@.locked,
            // Unconditional: lock is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self@.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.inv(),
    {
        proof { reveal(Spinlock::inv); }
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
            let tracked token: LockToken = LockToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }

    /// Acquires the spinlock, spinning until successful.
    ///
    /// # Description
    ///
    /// Repeatedly attempts `compare_exchange(false, true)` until the lock is
    /// acquired. Between attempts, calls `arch::cpu::pause()` as a CPU hint.
    ///
    /// The `requires` clause enforces sequential-model safety: calling `lock()` on an
    /// already-locked spinlock would be an infinite loop (deadlock) in the sequential model.
    /// Under `lock()`'s preconditions (`is_unlocked()` + `inv()`), `try_lock()` is
    /// guaranteed to succeed, so delegation is sound without `external_body`.
    ///
    /// Returns a tracked `LockToken` that the caller must pass to `unlock()` to
    /// discharge the lock-release obligation. This models the `SpinlockGuard` RAII
    /// pattern from the original implementation.
    pub fn lock(&mut self) -> (token: Tracked<LockToken>)
        requires
            old(self)@.is_unlocked(),
            old(self).inv(),
            !old(self)@.token_issued,
        ensures
            self@.locked,
            self@.is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.inv(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: LockToken = opt_token.tracked_unwrap();
        Tracked(token)
    }

    /// Releases the spinlock.
    ///
    /// # Description
    ///
    /// Sets the lock state to unlocked. Models the atomic
    /// `store(false, Ordering::Release)` from the original implementation.
    ///
    /// Consumes the `LockToken` produced by `lock()` or `try_lock()`, discharging
    /// the lock-release obligation. This models the `Drop` implementation of
    /// `SpinlockGuard` from the original.
    ///
    /// # Precondition
    ///
    /// The lock must be held (locked) and the token must match the current lock state.
    pub fn unlock(&mut self, Tracked(token): Tracked<LockToken>)
        requires
            old(self)@.locked,
            old(self).inv(),
            old(self)@.token_issued,
            token.view == old(self)@,
        ensures
            old(self)@.is_locked(),
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == SpinlockView::spec_new(old(self)@.id),
            self.inv(),
    {
        proof { reveal(Spinlock::inv); }
        self.locked = false;
        self.token_issued = false;
    }

    /// Checks if the spinlock is currently locked.
    ///
    /// NOTE: Verification helper — not present in original source. Provides a pure
    /// observer method for spec-level reasoning about lock state.
    ///
    /// # Returns
    ///
    /// `true` if the spinlock is locked, `false` otherwise.
    pub fn is_locked(&self) -> (result: bool)
        ensures
            result == self@.locked,
            result == self@.is_locked(),
    {
        self.locked
    }
}

} // verus!
