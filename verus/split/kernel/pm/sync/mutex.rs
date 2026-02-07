// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Mutex Implementation
//!
//! A mutual exclusion lock providing exclusive access to a shared resource.
//!
//! ## Verified Properties
//!
//! - A new mutex is always unlocked with no token outstanding.
//! - `try_lock` succeeds iff the mutex was previously unlocked, and locks it.
//! - `try_lock` fails iff the mutex was previously locked, leaving state unchanged.
//! - `unlock` transitions the mutex from locked to unlocked and consumes the token.
//! - Lock-then-unlock round-trip restores the original unlocked state.
//! - `spec_is_locked` and `spec_is_unlocked` are complementary predicates.
//! - Mutex instance identity (`id`) is preserved across all state transitions.
//! - Tokens are bound to the producing mutex instance via view identity.
//! - Well-formedness (`wf()`) enforces: unlocked implies no token outstanding.
//! - Token issuance is tracked: only one token per mutex at a time.
//! - After unlock, the mutex is relockable (satisfies lock preconditions).
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicBool` for lock-free
//! interior mutability, `alloc::sync::Arc` for shared ownership, and
//! `Condvar` for thread sleeping. For verification, we model the lock state as
//! a plain `bool` field and use `&mut self` for state transitions. This is a
//! sequential model that verifies the lock protocol (state machine correctness)
//! without reasoning about atomicity, memory ordering, or shared ownership.
//!
//! **This verified code is a specification model, not a runtime replacement.** The
//! kernel uses the original `src/kernel/src/pm/sync/mutex.rs` (with `AtomicBool`,
//! `Arc`, and `Condvar`) at runtime. The verified model proves the state machine
//! protocol is correct: every reachable state satisfies `wf()`, tokens are
//! instance-bound, and lock/unlock transitions are sound. The `lock()` body
//! delegates to `try_lock()` (a single CAS model) because, under the sequential
//! preconditions, success is guaranteed.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the mutex
//! protocol. The following are explicitly **out of scope**:
//! - **Concurrency and atomicity**: The sequential `&mut self` model does not capture
//!   concurrent thread access or atomic memory ordering.
//! - **Liveness and progress**: The blocking behavior of `lock()` (sleeping on a
//!   `Condvar`) and its termination under fairness assumptions are not modeled.
//!   The `lock()` precondition (`spec_is_unlocked`) models the instant-success case.
//! - **Condvar interaction**: The sleeping/waking protocol via `Condvar` is an
//!   external dependency. The condvar module is separately verified.
//! - **Arc reference counting**: Shared ownership via `Arc<MutexInner>` is not modeled.
//! - **Drop-based RAII**: Automatic lock release via `MutexGuard`/`Drop` is modeled
//!   via explicit `MutexToken` consumption in `unlock()`.
//! - **Timeout handling**: The `lock()` timeout parameter interacts with
//!   `ProcessManager::sleep()`, which is an external dependency.
//!
//! ## API Mapping
//!
//! | Original API              | Verified Model         | Notes                         |
//! |---------------------------|------------------------|-------------------------------|
//! | `Mutex::new()`            | `new(id)`              | Ghost id for instance identity. |
//! | `Mutex::try_lock(&self)`  | `try_lock(&mut self)`  | `&mut self` for state mutation. |
//! | `Mutex::lock(&self, t)`   | `lock(&mut self)`      | Timeout not modeled.          |
//! | `MutexGuard::drop()`      | `unlock(&mut self)`    | Explicit token consumption.   |
//! | `Mutex::reference_count()`| (not modeled)          | Arc-specific, out of scope.   |
//! | `MutexInner::unlock_unchecked()` | `unlock(&mut self)` | Safe wrapper with token.   |
//!
//! ## API Divergence
//!
//! The original implementation uses `&self` with `AtomicBool` interior mutability
//! and `Arc` shared ownership. The verified version uses `&mut self` because Verus
//! requires exclusive references for state mutation. This means the verification
//! covers the *state machine protocol* (lock/unlock transitions) but not the
//! *concurrent access pattern* that motivates the mutex's existence.
//!
//! The original `lock()` takes an `Option<SystemTime>` timeout and returns
//! `Result<MutexGuard, SleepError>`. The verified model omits timeout handling
//! and the `SleepError` variant, focusing purely on the lock state machine.
//!
//! The original `MutexGuard` holds an `Arc<MutexInner>` and calls
//! `unlock_unchecked()` in its `Drop` implementation. The verified model replaces
//! this RAII pattern with an explicit `MutexToken` that must be consumed by `unlock()`.
//!
//! ## Trust Boundaries
//!
//! - `lock()`: Fully verified. The sequential model's preconditions (`wf()`,
//!   `spec_is_unlocked()`, `!token_issued()`) guarantee that the lock is acquirable,
//!   so `lock()` delegates to `try_lock()` without needing `external_body`.
//! - `MutexGuard` and `Drop`: Modeled via tracked `MutexToken` ghost state.
//!   See spinlock verification for detailed explanation of this pattern.
//! - `Condvar::wait()` and `Condvar::notify_first()`: External dependencies from
//!   the separately verified condvar module. Not modeled in the mutex verification.
//! - `Arc` reference counting: Not modeled. Correct lifetime management is assumed.
//!
//! ## Trust Assumptions
//!
//! - **T1: ID Uniqueness.** Callers must provide a unique ghost `id` per mutex
//!   instance. If two mutexes share the same `id`, token isolation between them is
//!   not guaranteed. This parallels the original's reliance on reference identity
//!   (`Arc<MutexInner>`) which Verus cannot reason about.
//! - **T2: Arc lifetime management.** The original `reference_count()` returns
//!   `Arc::strong_count()`. The verified model does not model reference counting.
//!   Correct lifetime management is assumed.

use vstd::prelude::*;

// Include specifications.
include!("mutex.spec.rs");

// Include proofs.
include!("mutex.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A mutex providing mutual exclusion.
///
/// # Description
///
/// Wraps a boolean locked state. In the original implementation, this is an
/// `AtomicBool` inside `MutexInner` behind `Arc`; here it is a plain `bool`
/// for verification purposes.
///
/// # Representation
///
/// The fields are `pub` as required by Verus for `pub open spec fn` access.
/// The ghost `id` field provides instance identity for token binding.
pub struct Mutex {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
    /// Ghost identity for distinguishing mutex instances.
    /// Callers must provide a unique `id` per instance at construction time.
    pub id: Ghost<nat>,
    /// Ghost tracking of whether a `MutexToken` is currently outstanding.
    pub token_issued: Ghost<bool>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Mutex {
    /// Creates a new unlocked mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: Ghost identity for this mutex instance. Callers should ensure
    ///   unique IDs across all mutex instances to preserve token isolation.
    ///
    /// # Returns
    ///
    /// A new `Mutex` in the unlocked state with the given identity.
    pub fn new(Ghost(id): Ghost<nat>) -> (result: Self)
        ensures
            !result.locked,
            result.spec_is_unlocked(),
            result@ == Mutex::spec_new_view(id),
            result@.id == id,
            result.wf(),
    {
        Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) }
    }

    /// Attempts to acquire the mutex without blocking.
    ///
    /// # Description
    ///
    /// Models a single atomic `compare_exchange(false, true)` operation.
    /// Returns `true` if the lock was successfully acquired (was unlocked),
    /// `false` if the lock was already held (was locked).
    ///
    /// On success, produces a tracked `MutexToken` that the caller must pass to
    /// `unlock()` to discharge the lock-release obligation.
    ///
    /// # Returns
    ///
    /// `true` if the lock was acquired (with `MutexToken` in the `Tracked<Option>`),
    /// `false` otherwise (with `None`).
    pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<MutexToken>>))
        requires
            old(self).wf(),
        ensures
            result.0 == !old(self).locked,
            // Unconditional: mutex is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.wf(),
    {
        if !self.locked {
            self.locked = true;
            self.token_issued = Ghost(true);
            let tracked token: MutexToken = MutexToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }

    /// Acquires the mutex, blocking until successful.
    ///
    /// # Description
    ///
    /// In the original, this loops calling `try_lock()` and sleeping on a
    /// `Condvar` on failure. In the sequential model, the preconditions
    /// guarantee `try_lock()` succeeds on the first attempt.
    ///
    /// Returns a tracked `MutexToken` that the caller must pass to `unlock()`
    /// to discharge the lock-release obligation. This models the `MutexGuard`
    /// RAII pattern from the original implementation.
    pub fn lock(&mut self) -> (token: Tracked<MutexToken>)
        requires
            old(self).spec_is_unlocked(),
            old(self).wf(),
            !old(self).token_issued(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }

    /// Releases the mutex.
    ///
    /// # Description
    ///
    /// Sets the lock state to unlocked. Models the `MutexGuard::drop()` which
    /// calls `MutexInner::unlock_unchecked()` to store `false` and notify the
    /// first sleeping thread.
    ///
    /// Consumes the `MutexToken` produced by `lock()` or `try_lock()`, discharging
    /// the lock-release obligation.
    ///
    /// # Precondition
    ///
    /// The mutex must be held (locked) and the token must match the current state.
    pub fn unlock(&mut self, Tracked(token): Tracked<MutexToken>)
        requires
            old(self).locked,
            old(self).wf(),
            old(self).token_issued(),
            token.view == old(self)@,
        ensures
            old(self).spec_is_locked(),
            !self.locked,
            self.spec_is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == Mutex::spec_new_view(old(self)@.id),
            self.wf(),
    {
        self.locked = false;
        self.token_issued = Ghost(false);
    }

    /// Checks if the mutex is currently locked.
    ///
    /// # Returns
    ///
    /// `true` if the mutex is locked, `false` otherwise.
    pub fn is_locked(&self) -> (result: bool)
        ensures
            result == self.locked,
            result == self.spec_is_locked(),
    {
        self.locked
    }
}

} // verus!
