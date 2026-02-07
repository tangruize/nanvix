// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Spinlock Implementation
//!
//! A simple spinlock providing mutual exclusion.
//!
//! ## Verified Properties
//!
//! - A new spinlock is always unlocked.
//! - `try_lock` succeeds iff the lock was previously unlocked, and locks it.
//! - `try_lock` fails iff the lock was previously locked, leaving state unchanged.
//! - `unlock` transitions the lock from locked to unlocked.
//! - Lock-then-unlock round-trip restores the original unlocked state.
//! - `spec_is_locked` and `spec_is_unlocked` are complementary predicates.
//! - Lock instance identity (`id`) is preserved across all state transitions.
//! - Tokens are bound to the producing lock instance via view identity.
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
//! - `lock()`: Uses `external_body` because the spin-wait loop relies on atomic CAS
//!   and cannot be proven to terminate without reasoning about concurrent unlock.
//!   This is justified as a HAL-level operation (atomic CPU instructions).
//!   In the sequential model, `lock()` requires the lock to be unlocked to prevent
//!   modeling infinite loops (deadlocks).
//! - `SpinlockGuard` and `Drop`: Modeled via tracked `LockToken` ghost state. The
//!   original uses RAII via `SpinlockGuard<'a>` to auto-release on drop. Since Verus
//!   cannot reason about `Drop` directly, we model the obligation using a tracked ghost
//!   token: `lock()` and `try_lock()` produce a `LockToken` that must be consumed by
//!   `unlock()`. The token's view includes the lock's ghost `id`, so tokens from one
//!   lock instance cannot unlock a different instance. This makes the lock-release
//!   obligation explicit and instance-bound at the type level.
//! - `arch::cpu::pause()`: CPU hint with no semantic effect on lock state.

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
/// The `locked` field is `pub` for Verus spec reasoning. The original type
/// uses `AtomicBool` with interior mutability. Verified code should use
/// the provided methods rather than direct field access. The ghost `id`
/// field provides instance identity for token binding (erased at runtime).
pub struct Spinlock {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
    /// Ghost identity for distinguishing lock instances.
    /// Callers must provide a unique `id` per instance at construction time.
    pub ghost id: nat,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Spinlock {
    /// Creates a new unlocked spinlock.
    ///
    /// # Parameters
    ///
    /// - `id`: Ghost identity for this lock instance. Callers should ensure
    ///   unique IDs across all spinlock instances to preserve token isolation.
    ///
    /// # Returns
    ///
    /// A new `Spinlock` in the unlocked state with the given identity.
    pub fn new(Ghost(id): Ghost<nat>) -> (result: Self)
        ensures
            !result.locked,
            result.spec_is_unlocked(),
            result@ == Spinlock::spec_new_view(id),
            result@.id == id,
            result.wf(),
    {
        Spinlock { locked: false, id: id }
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
    ///
    /// # Returns
    ///
    /// `true` if the lock was acquired (with `LockToken` in the `Tracked<Option>`),
    /// `false` otherwise (with `None`).
    pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<LockToken>>))
        ensures
            result.0 == !old(self).locked,
            self.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
    {
        if !self.locked {
            self.locked = true;
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
    /// Uses `external_body` because:
    /// - The original implementation uses `AtomicBool::compare_exchange` in a loop.
    /// - Termination depends on another execution context releasing the lock.
    /// - Verus cannot reason about atomic memory operations or spin-wait termination.
    ///
    /// The `requires` clause enforces sequential-model safety: calling `lock()` on an
    /// already-locked spinlock would be an infinite loop (deadlock) in the sequential model.
    ///
    /// Returns a tracked `LockToken` that the caller must pass to `unlock()` to
    /// discharge the lock-release obligation. This models the `SpinlockGuard` RAII
    /// pattern from the original implementation.
    #[verifier::external_body]
    pub fn lock(&mut self) -> (token: Tracked<LockToken>)
        requires
            old(self).spec_is_unlocked(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            token@.view == self@,
    {
        unimplemented!()
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
            old(self).locked,
            token.view == old(self)@,
        ensures
            old(self).spec_is_locked(),
            !self.locked,
            self.spec_is_unlocked(),
            self@.id == old(self)@.id,
            self@ == Spinlock::spec_new_view(old(self)@.id),
    {
        self.locked = false;
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
            result == self.locked,
            result == self.spec_is_locked(),
    {
        self.locked
    }
}

} // verus!
