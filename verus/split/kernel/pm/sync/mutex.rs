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
//! - `is_locked` and `is_unlocked` are complementary predicates.
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
//!   The `lock()` precondition (`is_unlocked`) models the instant-success case.
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
//! | `Mutex::new()`            | `new(id)`              | Concrete id for instance identity. |
//! | `Mutex::try_lock(&self) -> Result<MutexGuard, ()>` | `try_lock(&mut self) -> (bool, Tracked<Option<MutexToken>>)` | `&mut self` for state mutation; `Result` replaced by `(bool, Tracked)` for Verus tracked token support. |
//! | `Mutex::lock(&self, Option<SystemTime>) -> Result<MutexGuard, SleepError>` | `lock(&mut self) -> Tracked<MutexToken>` | Timeout and `SleepError` error path not modeled; infallible under sequential preconditions. |
//! | `MutexGuard::drop()`      | `unlock(&mut self)`    | Explicit token consumption.   |
//! | `Mutex::reference_count()`| `reference_count(&self)` | Returns constant 1; Arc ref-counting not modeled (see below). |
//! | `MutexInner::unlock_unchecked() -> Result<(), Error>` | `unlock_unchecked(&mut self) -> ()` | Error path from `notify_first()` not modeled (trust boundary, see below). |
//! | `fmt::Debug for MutexGuard` | (not modeled)       | Verus lacks `fmt::Debug` trait support. |
//! | (none in original)          | `is_locked(&self)`   | Verification-only helper.     |
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
//! and the `SleepError` error path entirely, making `lock()` infallible. Lock
//! failures due to timeout or process-level errors are not representable in the
//! sequential model. This focuses verification purely on the lock state machine.
//!
//! The original `try_lock()` returns `Result<MutexGuard, ()>`. The verified model
//! returns `(bool, Tracked<Option<MutexToken>>)` because Verus lacks `Result`
//! ergonomics and requires tracked ghost tokens for proof obligations. The bool
//! maps to `Ok`/`Err` and the `MutexToken` replaces `MutexGuard`.
//!
//! The original `MutexGuard` holds an `Arc<MutexInner>` and calls
//! `unlock_unchecked()` in its `Drop` implementation. The verified model replaces
//! this RAII pattern with an explicit `MutexToken` that must be consumed by `unlock()`.
//!
//! The original `unlock_unchecked()` calls `self.sleeping.notify_first()` which can
//! return an error, handled with a `warn!()` log in `Drop`. The verified `unlock()`
//! has no error path because `Condvar` notification failure is an external dependency
//! not modeled here.
//!
//! ## Trust Boundaries
//!
//! - `lock()`: Fully verified. The sequential model's preconditions (`wf()`,
//!   `spec_is_unlocked()`, `!token_issued`) guarantee that the lock is acquirable,
//!   so `lock()` delegates to `try_lock()` without needing `external_body`.
//! - `MutexGuard` and `Drop`: Modeled via tracked `MutexToken` ghost state.
//!   See spinlock verification for detailed explanation of this pattern.
//! - `unlock_unchecked()` error path: The original returns `Result<(), Error>`
//!   because `Condvar::notify_first()` can fail; the `Drop` impl logs this with
//!   `warn!()`. The verified version returns `()`, eliminating this error path.
//!   This is a trust boundary: correctness assumes condvar notification does not
//!   fail in a way that violates the mutex protocol. The condvar module is
//!   separately verified.
//! - `Condvar::wait()` and `Condvar::notify_first()`: External dependencies from
//!   the separately verified condvar module. Not modeled in the mutex verification.
//! - `Arc` reference counting: Not modeled. Correct lifetime management is assumed.
//!   The original `Mutex` derives `Clone` (cloning the `Arc`), so `reference_count()`
//!   can return >1 in multi-owner scenarios. The verified `reference_count()` is a
//!   constant stub returning 1, modeling only the single-owner case.
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
//! - **T3: Token construction uniqueness.** The `MutexToken` tracked struct has
//!   `pub ghost view` because Verus's opaqueness rule requires `pub open spec fn`
//!   bodies to reference only public fields. A private field with a `pub closed
//!   spec fn` getter would make the getter opaque, preventing proof reasoning
//!   about token contents. As a result, external code could theoretically
//!   construct a token without calling `lock()`/`try_lock()`. The mutual exclusion
//!   guarantee relies on the assumption that tokens are created solely via the
//!   module's API.
//!
//! ## Refinement Argument
//!
//! The sequential model (`&mut self`, plain `bool`) is connected to the concurrent
//! implementation (`&self`, `AtomicBool`) by the following informal argument:
//!
//! 1. Each `compare_exchange(false, true, Acquire, Relaxed)` in `try_lock()` is a
//!    linearization point: it atomically reads and conditionally writes the lock
//!    state, so the lock-state transition is equivalent to the sequential model's
//!    `if !self.locked { self.locked = true; ... }`.
//! 2. Each `store(false, Relaxed)` in `unlock_unchecked()` is a linearization point:
//!    it atomically sets the lock state to unlocked, equivalent to the sequential
//!    model's `self.locked = false`.
//! 3. If the atomic operations are linearizable (guaranteed by x86 TSO, which
//!    upgrades all stores to effective release semantics), then any concurrent
//!    execution is equivalent to some sequential interleaving of
//!    `try_lock()`/`unlock()` calls. Note: the original uses `Relaxed` ordering
//!    for the unlock store, so this argument is architecture-specific to x86 TSO
//!    and does not hold on weakly-ordered architectures (e.g., ARM, RISC-V).
//! 4. The sequential model proves every such interleaving preserves `wf()`, so the
//!    concurrent implementation also preserves `wf()` under linearizability.
//!
//! This argument is informal and not machine-checked. A formal refinement proof
//! would require a concurrent verification framework (e.g., Iris, RustBelt) that
//! is beyond Verus's current capabilities.

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
/// Per Nanvix coding standards, struct fields should be private with getter/setter
/// access; this is an exception due to Verus tooling constraints.
/// The `id` field provides instance identity for token binding.
pub struct Mutex {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
    /// Identity for distinguishing mutex instances.
    /// Callers must provide a unique `id` per instance at construction time.
    pub id: usize,
    /// Tracking of whether a `MutexToken` is currently outstanding.
    pub token_issued: bool,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Mutex {
    /// Creates a new unlocked mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: Identity for this mutex instance. Callers should ensure
    ///   unique IDs across all mutex instances to preserve token isolation.
    ///
    /// # Returns
    ///
    /// A new `Mutex` in the unlocked state with the given identity.
    pub fn new(id: usize) -> (result: Self)
        ensures
            !result@.locked,
            result@.is_unlocked(),
            result@ == MutexView::spec_new(id as nat),
            result@.id == id as nat,
            result.wf(),
    {
        proof { reveal(Mutex::wf); }
        Mutex { locked: false, id: id, token_issued: false }
    }

    /// Returns the reference count of the mutex.
    ///
    /// # Description
    ///
    /// In the original implementation, returns `Arc::strong_count()`. In the
    /// sequential verification model, `Arc` shared ownership is not modeled,
    /// so the reference count is always 1 (single owner).
    ///
    /// # Returns
    ///
    /// The reference count of the mutex (always 1 in the sequential model).
    pub fn reference_count(&self) -> (result: usize)
        ensures
            result == 1usize,
    {
        1
    }

    /// Attempts to acquire the mutex without blocking.
    ///
    /// # Description
    ///
    /// Models a single atomic `compare_exchange(false, true)` operation.
    /// Returns `true` if the lock was successfully acquired (was unlocked),
    /// `false` if the lock was already held (was locked).
    ///
    /// **Note:** The original takes `&self` with atomic interior mutability.
    /// The verified version takes `&mut self` (exclusive reference), so this
    /// verification covers state machine transitions only, not the concurrent
    /// correctness that `compare_exchange` provides.
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
            result.0 == !old(self)@.locked,
            // Unconditional: mutex is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self@.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.wf(),
    {
        proof { reveal(Mutex::wf); }
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
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
    /// In the original, this is a blocking operation that loops calling `try_lock()`
    /// and sleeping on a `Condvar` on failure, and can be called on an already-locked
    /// mutex. In the sequential model, the `spec_is_unlocked()` precondition guarantees
    /// `try_lock()` succeeds on the first attempt, so contended locking (the primary
    /// concurrent use case) is outside the verified model's coverage.
    ///
    /// Returns a tracked `MutexToken` that the caller must pass to `unlock()`
    /// to discharge the lock-release obligation. This models the `MutexGuard`
    /// RAII pattern from the original implementation.
    pub fn lock(&mut self) -> (token: Tracked<MutexToken>)
        requires
            old(self)@.is_unlocked(),
            old(self).wf(),
            !old(self)@.token_issued,
        ensures
            self@.locked,
            self@.is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }

    /// Releases the mutex without requiring a token.
    ///
    /// # Description
    ///
    /// Models `MutexInner::unlock_unchecked()` from the original implementation,
    /// which stores `false` to the `AtomicBool` and notifies the first sleeping
    /// thread via `Condvar::notify_first()`. The Condvar notification is an
    /// external dependency not modeled here.
    ///
    /// In the original, this is `unsafe` because the caller must ensure the lock
    /// is held. In Verus, safety is enforced via the `requires` clause instead.
    /// The original returns `Result<(), Error>` due to the `notify_first()` error
    /// path; here it returns `()` since condvar notification is not modeled.
    fn unlock_unchecked(&mut self)
        requires
            old(self)@.locked,
            old(self).wf(),
            old(self)@.token_issued,
        ensures
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == MutexView::spec_new(old(self)@.id),
            self.wf(),
    {
        proof { reveal(Mutex::wf); }
        self.locked = false;
        self.token_issued = false;
    }

    /// Releases the mutex.
    ///
    /// # Description
    ///
    /// Models `MutexGuard::drop()` from the original implementation. The original
    /// `Drop` impl calls `MutexInner::unlock_unchecked()` to store `false` and
    /// notify the first sleeping thread. This function consumes the `MutexToken`
    /// (modeling the `MutexGuard` RAII pattern) and delegates to `unlock_unchecked()`
    /// for the actual state transition.
    ///
    /// # Precondition
    ///
    /// The mutex must be held (locked) and the token must match the current state.
    pub fn unlock(&mut self, Tracked(token): Tracked<MutexToken>)
        requires
            old(self)@.locked,
            old(self).wf(),
            old(self)@.token_issued,
            token.view == old(self)@,
        ensures
            old(self)@.is_locked(),
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == MutexView::spec_new(old(self)@.id),
            self.wf(),
    {
        self.unlock_unchecked();
    }

    /// Checks if the mutex is currently locked.
    ///
    /// # Returns
    ///
    /// `true` if the mutex is locked, `false` otherwise.
    pub fn is_locked(&self) -> (result: bool)
        ensures
            result == self@.locked,
            result == self@.is_locked(),
    {
        self.locked
    }
}

} // verus!
