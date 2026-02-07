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
//! ## Trust Boundaries
//!
//! - `lock()`: Uses `external_body` because the spin-wait loop relies on atomic CAS
//!   and cannot be proven to terminate without reasoning about concurrent unlock.
//!   This is justified as a HAL-level operation (atomic CPU instructions).
//! - `SpinlockGuard` and `Drop`: Not modeled in this verification. The original uses
//!   RAII via `SpinlockGuard<'a>` to auto-release on drop. Drop-based reasoning
//!   requires lifetime-aware resource tracking beyond Verus's current scope.
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
/// the provided methods rather than direct field access.
pub struct Spinlock {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Spinlock {
    /// Creates a new unlocked spinlock.
    ///
    /// # Returns
    ///
    /// A new `Spinlock` in the unlocked state.
    pub fn new() -> (result: Self)
        ensures
            !result.locked,
            result.spec_is_unlocked(),
            result@ == Spinlock::spec_new_view(),
            result.wf(),
    {
        Spinlock { locked: false }
    }

    /// Attempts to acquire the lock without spinning.
    ///
    /// # Description
    ///
    /// Models a single atomic `compare_exchange(false, true)` operation.
    /// Returns `true` if the lock was successfully acquired (was unlocked),
    /// `false` if the lock was already held (was locked).
    ///
    /// # Returns
    ///
    /// `true` if the lock was acquired, `false` otherwise.
    pub fn try_lock(&mut self) -> (result: bool)
        ensures
            result == !old(self).locked,
            self.locked,
    {
        if !self.locked {
            self.locked = true;
            true
        } else {
            false
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
    #[verifier::external_body]
    pub fn lock(&mut self)
        ensures
            self.locked,
            self.spec_is_locked(),
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
    /// # Precondition
    ///
    /// The lock must be held (locked).
    pub fn unlock(&mut self)
        requires
            old(self).locked,
        ensures
            !self.locked,
            self.spec_is_unlocked(),
            self@ == Spinlock::spec_new_view(),
    {
        self.locked = false;
    }

    /// Checks if the spinlock is currently locked.
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
