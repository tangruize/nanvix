// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Semaphore Implementation
//!
//! A counting semaphore controlling access to a finite number of shared resources.
//!
//! ## Verified Properties
//!
//! - A new semaphore has the specified initial value and no waiters.
//! - `try_down` succeeds iff the value was positive, decrementing by 1.
//! - `try_down` fails iff the value was zero, leaving state unchanged.
//! - `down` decrements the value by 1 (sequential model: value > 0 precondition).
//! - `up` increments the value by 1 (with overflow guard), callable with waiters.
//! - `spec_is_available` and `spec_is_exhausted` are complementary predicates.
//! - Well-formedness (`wf()`) is preserved by all operations.
//! - Down-then-up and up-then-down round-trips restore the original value.
//! - Binary semaphore (value=1) provides mutual exclusion.
//! - Resource conservation: initial value = current value + acquired count.
//! - Value monotonicity: `up` strictly increases, `down` strictly decreases.
//! - Blocking state transitions: `down_blocking` and `wake` spec functions model
//!   the sleep/wake protocol with explicit waiter tracking.
//! - Chained function postconditions: down-then-up and try_down sequences proved
//!   via function ensures clauses, not manual view construction.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicUsize` for lock-free
//! interior mutability and `Condvar` for thread sleeping. For verification, we model
//! the count as a plain `usize` field and use `&mut self` for state transitions.
//! The waiter count is tracked as ghost state in the `SemaphoreView` (not as an exec
//! field), with explicit spec-level state transition functions (`spec_down_blocking`,
//! `spec_wake`) that model the sleep/wake protocol. This is a sequential model that
//! verifies the semaphore protocol (state machine correctness) without reasoning
//! about atomicity, memory ordering, or concurrent access.
//!
//! **This verified code is a specification model, not a runtime replacement.** The
//! kernel uses the original `src/kernel/src/pm/sync/semaphore.rs` (with `AtomicUsize`
//! and `Condvar`) at runtime. The verified model proves the state machine protocol
//! is correct: every reachable state satisfies `wf()`, resource counting is sound,
//! and up/down transitions preserve invariants.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the semaphore
//! protocol. The following are explicitly **out of scope**:
//! - **Concurrency and atomicity**: The sequential `&mut self` model does not capture
//!   concurrent thread access or atomic memory ordering (`fetch_update`, `fetch_add`).
//! - **Liveness and progress**: The blocking behavior of `down()` (looping with
//!   `Condvar::wait()`) and its termination under fairness assumptions are not modeled.
//!   The `down()` precondition (`spec_is_available()`) models the instant-success case.
//! - **Condvar interaction**: The sleeping/waking protocol via `Condvar` is modeled
//!   at the spec level via `spec_down_blocking` and `spec_wake` state transitions.
//!   The condvar exec implementation is separately verified.
//! - **Error propagation**: The original `down()` returns `Result<(), SleepError>`,
//!   `try_down()` returns `Result<(), Error>` with `ErrorCode::TryAgain`, and `up()`
//!   returns `Result<(), Error>`. The sequential model simplifies: `down()` returns
//!   `()` (precondition guarantees success), `try_down()` returns `bool` (where
//!   `false` corresponds to `Err(ErrorCode::TryAgain)`), and `up()` returns `()`
//!   (condvar notification errors are external).
//!
//! ## API Mapping
//!
//! | Original API              | Verified Model         | Notes                            |
//! |---------------------------|------------------------|----------------------------------|
//! | `Semaphore::new(value)`   | `new(value)`           | Direct mapping.                  |
//! | `Semaphore::down(&self)`  | `down(&mut self)`      | `&mut self`; precondition: > 0.  |
//! | `Semaphore::try_down(&self)` | `try_down(&mut self)` | `&mut self`; both paths modeled. |
//! | `Semaphore::up(&self)`    | `up(&mut self)`        | `&mut self`; overflow guarded.   |
//! | *(condvar sleep path)*    | `spec_down_blocking()` | Spec-only state transition.      |
//! | *(condvar wake path)*     | `spec_wake()`          | Spec-only state transition.      |
//!
//! ## API Divergence
//!
//! The original `down()` uses `&self` with `AtomicUsize::fetch_update()` in a loop,
//! sleeping on `Condvar` when the count is zero. The verified `down()` takes `&mut self`
//! with a precondition that the value is positive, modeling the instant-success case.
//! The loop + sleep pattern is modeled at the spec level via `spec_down_blocking` and
//! `spec_wake` state transitions with proof lemmas verifying protocol correctness.
//!
//! The original `up()` uses `&self` with `AtomicUsize::fetch_add()` and calls
//! `Condvar::notify_first()`. The verified `up()` increments the plain `usize` value.
//! The condvar notification effect is modeled by the `spec_wake` transition.
//!
//! The original `try_down()` returns `Result<(), Error>` with `ErrorCode::TryAgain`.
//! The verified `try_down()` returns `bool` where `false` ≡ `Err(ErrorCode::TryAgain)`.
//! No other error codes are possible from the atomic `fetch_update` path.
//!
//! ## Trust Boundaries
//!
//! - `down()`: Fully verified (instant-success path). Blocking path modeled at spec level.
//! - `try_down()`: Fully verified. Both success and failure paths modeled.
//! - `up()`: Fully verified. No waiter restriction; overflow guard via precondition.
//! - `spec_down_blocking()` / `spec_wake()`: Spec-level state transitions modeling
//!   the condvar sleep/wake protocol. Proved to preserve `spec_wf()`.
//!
//! ## Trust Assumptions
//!
//! - **T1: No arithmetic overflow.** The `up()` precondition requires
//!   `self.value < usize::MAX`. The original `fetch_add(1, SeqCst)` can silently
//!   overflow in release mode. The verified model makes this an explicit precondition.
//! - **T2: Condvar correctness.** The sleeping/waking protocol via `Condvar` is
//!   modeled at the spec level. The condvar exec implementation is separately
//!   verified. The interface assumption is: `Condvar::wait()` blocks until
//!   `Condvar::notify_first()` is called, and `notify_first()` wakes exactly one
//!   blocked thread. See `spec_condvar_wake_after_notify()`.
//! - **T3: Sequential ordering.** The model assumes sequential execution. The
//!   original relies on `SeqCst` ordering for `fetch_update` and `fetch_add`.
//! - **T4: Caller safety conditions.** The original `down()` and `up()` are
//!   `unsafe` with caller obligations: interrupts must be disabled, the caller
//!   must not be the kernel process (for `down()`), and no resources must be
//!   held (for `down()`) or no process manager reference held (for `up()`).
//!   These hardware/OS-level preconditions are not expressible in the sequential
//!   Verus model and are assumed to be enforced by the calling context.
//!
//! ## Refinement Argument
//!
//! The sequential model (`&mut self`, plain `usize`) is connected to the concurrent
//! implementation (`&self`, `AtomicUsize`) by the following informal argument:
//!
//! 1. Each `fetch_update(SeqCst, SeqCst, ...)` in `down()`/`try_down()` is a
//!    linearization point: it atomically reads the count, tests if > 0, and
//!    conditionally decrements. This is equivalent to the sequential model's
//!    `if self.value > 0 { self.value -= 1; }`.
//! 2. Each `fetch_add(1, SeqCst)` in `up()` is a linearization point: it atomically
//!    increments the count, equivalent to `self.value += 1`.
//! 3. Under `SeqCst` ordering, all operations are totally ordered, so any concurrent
//!    execution is equivalent to some sequential interleaving.
//! 4. The sequential model proves every such interleaving preserves `wf()`.

use vstd::prelude::*;

// Include specifications.
include!("semaphore.spec.rs");

// Include proofs.
include!("semaphore.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A counting semaphore controlling access to shared resources.
///
/// # Description
///
/// Wraps a resource count. In the original implementation, this is an
/// `AtomicUsize` with a `Condvar` for blocking; here it is a plain `usize`.
/// The waiter count is tracked purely as ghost state in `SemaphoreView`,
/// not as an exec field, since no exec function modifies it (the condvar
/// sleep/wake protocol is modeled at the spec level only).
///
/// # Representation
///
/// The `value` field is `pub` as required by Verus for `pub open spec fn` access.
/// Per Nanvix coding standards, struct fields should be private with getter/setter
/// access; this is an exception due to Verus tooling constraints.
pub struct Semaphore {
    /// Current count of available resources.
    pub value: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Semaphore {
    /// Creates a new semaphore with the given initial value.
    ///
    /// # Parameters
    ///
    /// - `value`: Initial count of available resources.
    ///
    /// # Returns
    ///
    /// A new `Semaphore` with the specified initial value and no waiters.
    pub fn new(value: usize) -> (result: Self)
        ensures
            result.value == value,
            result@ == Semaphore::spec_new_view(value as nat),
            result@.value == value as nat,
            result@.waiters == 0,
            result.wf(),
            result.spec_drop_safe(),
    {
        Semaphore { value: value }
    }

    /// Acquires the semaphore, decrementing the count by 1.
    ///
    /// # Description
    ///
    /// In the original, this is a blocking operation that loops calling
    /// `fetch_update()` and sleeping on a `Condvar` when the count is zero.
    /// In the sequential model, the `spec_is_available()` precondition guarantees
    /// success on the first attempt, so the blocking loop is not modeled.
    ///
    /// # Safety (Original)
    ///
    /// The original `down()` is `unsafe` and requires: interrupts disabled,
    /// caller is not the kernel process, and no resources are held. These
    /// conditions are not modeled (see trust assumption T4).
    ///
    /// # Returns
    ///
    /// The semaphore with value decremented by 1.
    pub fn down(&mut self)
        requires
            old(self).wf(),
            old(self).spec_is_available(),
        ensures
            self.value == old(self).value - 1,
            self@.value == old(self)@.value - 1,
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        self.value = self.value - 1;
    }

    /// Attempts to acquire the semaphore without blocking.
    ///
    /// # Description
    ///
    /// Models a single atomic `fetch_update()` operation. Returns `true` if the
    /// semaphore was successfully acquired (value was positive and decremented),
    /// `false` if the semaphore was exhausted (value was zero, state unchanged).
    ///
    /// # Returns
    ///
    /// `true` if the semaphore was acquired, `false` otherwise.
    pub fn try_down(&mut self) -> (result: bool)
        requires
            old(self).wf(),
        ensures
            result == old(self).spec_is_available(),
            result ==> self.value == old(self).value - 1,
            result ==> self@.value == old(self)@.value - 1,
            !result ==> self@ == old(self)@,
            !result ==> self.spec_is_exhausted(),
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        if self.value > 0 {
            self.value = self.value - 1;
            true
        } else {
            false
        }
    }

    /// Releases the semaphore, incrementing the count by 1.
    ///
    /// # Description
    ///
    /// Models the `fetch_add(1, SeqCst)` operation from the original. The
    /// condvar notification (`notify_first()`) effect is modeled by the spec-level
    /// `spec_wake()` transition. This function can be called regardless of
    /// whether threads are waiting (the original `up()` always increments first,
    /// then notifies).
    ///
    /// # Safety (Original)
    ///
    /// The original `up()` is `unsafe` and requires: interrupts disabled and
    /// the caller does not hold a reference to the process manager. These
    /// conditions are not modeled (see trust assumption T4).
    ///
    /// # Precondition
    ///
    /// The value must be less than `usize::MAX` to prevent overflow. The original
    /// `fetch_add(1, SeqCst)` can silently wrap in release builds; the verified
    /// model makes this an explicit precondition. Callers should establish this
    /// bound from the resource pool size or system invariant.
    pub fn up(&mut self)
        requires
            old(self).wf(),
            old(self).value < usize::MAX,
        ensures
            self.value == old(self).value + 1,
            self@.value == old(self)@.value + 1,
            self@.waiters == old(self)@.waiters,
            self.spec_is_available(),
            self.wf(),
    {
        self.value = self.value + 1;
    }

    /// Returns the current value of the semaphore.
    ///
    /// # Description
    ///
    /// Verification-only helper (not in original API). Provides spec-connected
    /// access to the current resource count.
    ///
    /// # Returns
    ///
    /// The current count of available resources.
    pub fn get_value(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result == self.value,
            result as nat == self@.value,
    {
        self.value
    }

    /// Returns whether the semaphore has available resources.
    ///
    /// # Description
    ///
    /// Verification-only helper (not in original API). Provides spec-connected
    /// availability check.
    ///
    /// # Returns
    ///
    /// `true` if the semaphore value is greater than zero, `false` otherwise.
    pub fn is_available(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_available(),
            result == (self.value > 0),
    {
        self.value > 0
    }
}

} // verus!
