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
//! - `up` increments the value by 1 (with overflow guard).
//! - `spec_is_available` and `spec_is_exhausted` are complementary predicates.
//! - Well-formedness (`wf()`) is preserved by all operations.
//! - Down-then-up and up-then-down round-trips restore the original value.
//! - Binary semaphore (value=1) provides mutual exclusion.
//! - Resource conservation: initial value = current value + acquired count.
//! - Value monotonicity: `up` strictly increases, `down` strictly decreases.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicUsize` for lock-free
//! interior mutability and `Condvar` for thread sleeping. For verification, we model
//! the count as a plain `usize` field and use `&mut self` for state transitions.
//! The `waiters` field models the number of threads sleeping on the condvar. This is
//! a sequential model that verifies the semaphore protocol (state machine correctness)
//! without reasoning about atomicity, memory ordering, or concurrent access.
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
//! - **Condvar interaction**: The sleeping/waking protocol via `Condvar` is an
//!   external dependency. The condvar module is separately verified.
//! - **Error propagation**: `SleepError` from `Condvar::wait()` and `Error` from
//!   `Condvar::notify_first()` are external dependencies not modeled here.
//!
//! ## API Mapping
//!
//! | Original API              | Verified Model         | Notes                            |
//! |---------------------------|------------------------|----------------------------------|
//! | `Semaphore::new(value)`   | `new(value)`           | Direct mapping.                  |
//! | `Semaphore::down(&self)`  | `down(&mut self)`      | `&mut self`; precondition: > 0.  |
//! | `Semaphore::try_down(&self)` | `try_down(&mut self)` | `&mut self`; both paths modeled. |
//! | `Semaphore::up(&self)`    | `up(&mut self)`        | `&mut self`; overflow guarded.   |
//!
//! ## API Divergence
//!
//! The original `down()` uses `&self` with `AtomicUsize::fetch_update()` in a loop,
//! sleeping on `Condvar` when the count is zero. The verified `down()` takes `&mut self`
//! with a precondition that the value is positive, modeling the instant-success case.
//! The loop + sleep pattern is a liveness property that requires fairness assumptions
//! beyond the sequential model.
//!
//! The original `up()` uses `&self` with `AtomicUsize::fetch_add()` and calls
//! `Condvar::notify_first()`. The verified `up()` increments the plain `usize` value
//! and does not model condvar notification (external dependency).
//!
//! The original `try_down()` returns `Result<(), Error>` with `ErrorCode::TryAgain`.
//! The verified `try_down()` returns `(bool, ...)` where `true` means success,
//! matching the mutex `try_lock()` pattern for consistency.
//!
//! ## Trust Boundaries
//!
//! - `down()`: Fully verified. The sequential model's precondition (`spec_is_available()`)
//!   guarantees success on the first attempt.
//! - `try_down()`: Fully verified. Both success and failure paths modeled.
//! - `up()`: Fully verified. Overflow guard via precondition.
//! - `Condvar::wait()` and `Condvar::notify_first()`: External dependencies from
//!   the separately verified condvar module. Not modeled in semaphore verification.
//!
//! ## Trust Assumptions
//!
//! - **T1: No arithmetic overflow.** The `up()` precondition requires
//!   `self.value < usize::MAX`. The original `fetch_add(1, SeqCst)` can silently
//!   overflow in release mode. The verified model makes this an explicit precondition.
//! - **T2: Condvar correctness.** The sleeping/waking protocol via `Condvar` is
//!   assumed correct per the separately verified condvar module.
//! - **T3: Sequential ordering.** The model assumes sequential execution. The
//!   original relies on `SeqCst` ordering for `fetch_update` and `fetch_add`.
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
/// `AtomicUsize` with a `Condvar` for blocking; here it is a plain `usize`
/// with a `waiters` ghost counter for verification purposes.
///
/// # Representation
///
/// The fields are `pub` as required by Verus for `pub open spec fn` access.
/// Per Nanvix coding standards, struct fields should be private with getter/setter
/// access; this is an exception due to Verus tooling constraints.
pub struct Semaphore {
    /// Current count of available resources.
    pub value: usize,
    /// Number of threads waiting on the semaphore (models Condvar queue length).
    pub waiters: usize,
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
            result.waiters == 0,
            result@ == Semaphore::spec_new_view(value as nat),
            result@.value == value as nat,
            result@.waiters == 0,
            result.wf(),
            result.spec_drop_safe(),
    {
        Semaphore { value: value, waiters: 0 }
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
    /// condvar notification (`notify_first()`) is an external dependency not
    /// modeled here. The sequential model requires no waiters because the
    /// original atomically increments and then notifies; the notification
    /// (which would decrement waiters) is not modeled.
    ///
    /// # Precondition
    ///
    /// The value must be less than `usize::MAX` to prevent overflow.
    /// No threads must be waiting (condvar notification not modeled).
    pub fn up(&mut self)
        requires
            old(self).wf(),
            old(self).value < usize::MAX,
            old(self)@.waiters == 0,
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
