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
//! - **Sequential-to-concurrent refinement**: The informal linearizability argument
//!   in "Refinement Argument" below is not mechanically verified.
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
//! | `Semaphore::down(&self)`  | `down_or_block(ctx)`   | Primary model; returns outcome.  |
//! | *(instant-success only)*  | `down_available(ctx)`  | Precondition: value > 0.         |
//! | `Semaphore::try_down(&self)` | `try_down(&mut self)` | `&mut self`; both paths modeled. |
//! | `Semaphore::up(&self)`    | `up(&mut self, ctx)`    | `&mut self`; ghost safety ctx.  |
//! | *(condvar sleep path)*    | `spec_down_blocking()` | Spec-only state transition.      |
//! | *(condvar wake path)*     | `spec_wake()`          | Spec-only state transition.      |
//!
//! ## API Divergence
//!
//! The original `down()` uses `&self` with `AtomicUsize::fetch_update()` in a loop,
//! sleeping on `Condvar` when the count is zero. The verified `down_or_block()` models
//! both paths: `Acquired` for instant success, `WouldBlock` for the blocking case.
//! The convenience function `down_available()` models only the instant-success path
//! with a precondition that the value is positive. The loop + sleep pattern is modeled
//! at the spec level via `spec_down_blocking` and `spec_wake` state transitions.
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
//! - `down_available()`: Fully verified (instant-success path only).
//! - `down_or_block()`: Fully verified (both paths). Blocking path modeled at spec level.
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
//!   These conditions are encoded as ghost `CallerContext` preconditions on
//!   `down_available()`, `down_or_block()`, and `up()`. Callers must provide a
//!   `Ghost<CallerContext>` satisfying `safe_for_down()` or `safe_for_up()`.
//! - **T5: `notify_first()` success.** The original `up()` returns
//!   `Result<(), Error>` because `Condvar::notify_first()` may fail. The
//!   verified model assumes `notify_first()` always succeeds. If it fails in
//!   practice, the semaphore value has been incremented but no waiter is woken,
//!   which could cause a thread to remain sleeping indefinitely.
//! - **T6: `Condvar::wait()` success.** The original `down()` returns
//!   `Result<(), SleepError>` because `Condvar::wait()` may fail (e.g., signal
//!   interruption). The verified model's `down_or_block()` returns `WouldBlock`
//!   without modeling the `SleepError` failure case. The sleep-then-error path
//!   (thread woken with error, must retry or propagate) is not represented.
//!
//! ## Ghost State Architecture
//!
//! The `SemaphoreView.waiters` field is **pure ghost state**: the `View`
//! implementation always returns `waiters: 0` because no exec field tracks
//! waiting threads (the original uses `Condvar`'s internal queue). The waiter
//! count exists solely for spec-level protocol reasoning.
//!
//! The blocking protocol lemmas (`lemma_down_blocking_preserves_wf`,
//! `lemma_wake_preserves_wf`, `lemma_up_wake_cycle`,
//! `lemma_all_waiters_eventually_served`) operate on manually constructed
//! `SemaphoreView` values — not on any exec `Semaphore`'s `@` view. They
//! prove properties of the **abstract state machine** (the semaphore protocol)
//! independent of exec state. This is intentional: the blocking protocol is
//! a spec-level model verified for internal consistency, providing assurance
//! that the sleep/wake algorithm preserves invariants if the condvar behaves
//! as specified in T2.
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
///
/// # Struct Divergence (AST-consistency)
///
/// The original struct has fields `value: AtomicUsize` and `sleeping: Condvar`.
/// Verus cannot model `AtomicUsize` or `Condvar` directly, so:
/// - `AtomicUsize` is replaced by plain `usize` (sequential model).
/// - `Condvar` is omitted; its sleep/wake protocol is modeled at the spec
///   level via `spec_down_blocking()` and `spec_wake()`.
/// See "Verification Model" in module docs for the refinement argument.
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
    ///
    /// # Exec Equivalence (AST-consistency)
    ///
    /// Original: `Self { value: AtomicUsize::new(value), sleeping: Condvar::new() }`.
    /// Verus: `Semaphore { value: value }`. Semantically equivalent — both
    /// initialize the resource count to `value`. The `AtomicUsize::new` and
    /// `Condvar::new` calls are replaced per the struct divergence (see above).
    pub fn new(value: usize) -> (result: Self)
        ensures
            result@ == Semaphore::spec_new_view(value as nat),
            result@.value == value as nat,
            result@.waiters == 0,
            result.wf(),
            result.spec_drop_safe(),
    {
        proof { reveal(Semaphore::wf); }
        Semaphore { value: value }
    }

    /// Acquires the semaphore, decrementing the count by 1.
    ///
    /// # Description
    ///
    /// Models the original `down(&self) -> Result<(), SleepError>`. The original
    /// loops with `AtomicUsize::fetch_update` + `Condvar::wait` until the value
    /// is successfully decremented. In the sequential Verus model, the loop and
    /// condvar cannot be represented directly; this function models the
    /// instant-success path (value > 0) with a precondition. The blocking path
    /// (value == 0) is modeled by `down_or_block()` which returns `WouldBlock`,
    /// and the sleep/wake protocol is modeled at the spec level via
    /// `spec_down_blocking()` and `spec_wake()`.
    ///
    /// # Safety (Original)
    ///
    /// The original `down()` is `unsafe` and requires: interrupts disabled,
    /// caller is not the kernel process, and no resources are held. These
    /// conditions are encoded via the ghost `CallerContext` parameter (see T4).
    ///
    /// # Parameters
    ///
    /// - `ctx`: Ghost caller context proving safety conditions are satisfied.
    ///
    /// # Returns
    ///
    /// The semaphore with value decremented by 1.
    ///
    /// # Exec Equivalence (AST-consistency)
    ///
    /// Original uses `&self` + `AtomicUsize::fetch_update` in a loop, returning
    /// `Result<(), SleepError>`. Verus uses `&mut self` (sequential mutation model)
    /// with a precondition `spec_is_available()` that guarantees success, making
    /// the loop and error path unnecessary. The return type `()` corresponds to
    /// the `Ok(())` path. See "API Divergence" in module docs.
    pub fn down(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self).spec_is_available(),
            ctx@.safe_for_down(),
        ensures
            self@.value == old(self)@.value - 1,
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        self.value = self.value - 1;
    }

    /// Acquires the semaphore, decrementing the count by 1 (instant-success path).
    ///
    /// # Description
    ///
    /// Models the instant-success path of the original `down()`: the caller
    /// has established that the semaphore is available (`spec_is_available()`).
    /// For the full decision logic covering both immediate acquisition and
    /// blocking, use `down_or_block()` instead.
    ///
    /// # Safety (Original)
    ///
    /// The original `down()` is `unsafe` and requires: interrupts disabled,
    /// caller is not the kernel process, and no resources are held. These
    /// conditions are encoded via the ghost `CallerContext` parameter (see T4).
    ///
    /// # Parameters
    ///
    /// - `ctx`: Ghost caller context proving safety conditions are satisfied.
    ///
    /// # Returns
    ///
    /// The semaphore with value decremented by 1.
    ///
    /// # Extra Function Justification (AST-consistency)
    ///
    /// Not in original API. Extracted as a verification helper that isolates the
    /// instant-success path of `down()` with an explicit availability precondition.
    /// Used by proofs that need the stronger guarantee that acquisition always succeeds.
    pub fn down_available(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self).spec_is_available(),
            ctx@.safe_for_down(),
        ensures
            self@.value == old(self)@.value - 1,
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        self.value = self.value - 1;
    }

    /// Attempts to acquire the semaphore, returning the outcome.
    ///
    /// # Description
    ///
    /// Models the full decision logic of the original `down()`:
    /// - If value > 0: decrements and returns `Acquired` (instant success path).
    /// - If value == 0: returns `WouldBlock` (the original would enter the
    ///   `Condvar::wait()` loop). The exec state is unchanged; the ghost view
    ///   should be updated via `spec_down_or_block_ghost_view()` to reflect the
    ///   waiter increment.
    ///
    /// This function bridges the exec and spec layers for the blocking path:
    /// the exec code handles the decision, and the caller uses the ghost view
    /// spec function to track the waiter state change.
    ///
    /// # Safety (Original)
    ///
    /// Same as `down()`: requires interrupts disabled, non-kernel caller,
    /// no held resources (see T4).
    ///
    /// # Parameters
    ///
    /// - `ctx`: Ghost caller context proving safety conditions are satisfied.
    ///
    /// # Returns
    ///
    /// `DownOutcome::Acquired` if the semaphore was available and acquired,
    /// `DownOutcome::WouldBlock` if the semaphore was exhausted.
    ///
    /// # Extra Function Justification (AST-consistency)
    ///
    /// Not in original API. Decomposition of the original `down()` that models
    /// both the instant-success and would-block paths without a loop or condvar.
    /// Returns `DownOutcome` so callers can reason about both paths. The original
    /// `down()` loop is unrolled: one iteration's decision is captured here, and
    /// the blocking/waking protocol is modeled at the spec level.
    pub fn down_or_block(&mut self, ctx: Ghost<CallerContext>) -> (result: DownOutcome)
        requires
            old(self).wf(),
            ctx@.safe_for_down(),
        ensures
            result == DownOutcome::Acquired ==> old(self).spec_is_available(),
            result == DownOutcome::Acquired ==> self@.value == old(self)@.value - 1,
            result == DownOutcome::Acquired ==> self@.waiters == old(self)@.waiters,
            result == DownOutcome::Acquired ==> self@ == Semaphore::spec_down_or_block_ghost_view(old(self)@, result),
            result == DownOutcome::WouldBlock ==> old(self).spec_is_exhausted(),
            result == DownOutcome::WouldBlock ==> self@ == old(self)@,
            result == DownOutcome::WouldBlock ==> Semaphore::spec_down_or_block_ghost_view(old(self)@, result).waiters == old(self)@.waiters + 1,
            result == DownOutcome::WouldBlock ==> Semaphore::spec_down_or_block_ghost_view(old(self)@, result).value == 0,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        if self.value > 0 {
            self.value = self.value - 1;
            DownOutcome::Acquired
        } else {
            DownOutcome::WouldBlock
        }
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
    ///
    /// # Exec Equivalence (AST-consistency)
    ///
    /// Original: `fn try_down(&self) -> Result<(), Error>` using
    /// `AtomicUsize::fetch_update`. Returns `Ok(())` on success,
    /// `Err(ErrorCode::TryAgain)` when exhausted.
    /// Verus: `fn try_down(&mut self) -> bool`. Returns `true` ≡ `Ok(())`,
    /// `false` ≡ `Err(TryAgain)`. The decision logic is identical:
    /// `if value > 0 { value -= 1; success } else { fail }`.
    /// Signature changes: `&mut self` for sequential mutation model,
    /// `bool` return because `Result`/`Error` types are not modeled.
    /// See `spec_try_down_result_maps_ok()` for the formal mapping.
    pub fn try_down(&mut self) -> (result: bool)
        requires
            old(self).wf(),
        ensures
            result == old(self).spec_is_available(),
            result ==> self@.value == old(self)@.value - 1,
            !result ==> self@ == old(self)@,
            !result ==> self.spec_is_exhausted(),
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
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
    /// conditions are encoded via the ghost `CallerContext` parameter (see T4).
    ///
    /// # Parameters
    ///
    /// - `ctx`: Ghost caller context proving safety conditions are satisfied.
    ///
    /// # Precondition
    ///
    /// The value must be less than `usize::MAX` to prevent overflow. The original
    /// `fetch_add(1, SeqCst)` can silently wrap in release builds; the verified
    /// model makes this an explicit precondition. Callers should establish this
    /// bound from the resource pool size or system invariant.
    ///
    /// # Exec Equivalence (AST-consistency)
    ///
    /// Original: `unsafe fn up(&self) -> Result<(), Error>` with
    /// `self.value.fetch_add(1, SeqCst); self.sleeping.notify_first().map(|_| ())`
    /// Verus: `fn up(&mut self, ctx: Ghost<CallerContext>)` with
    /// `self.value = self.value + 1`.
    /// Core increment logic is identical. Differences:
    /// - `&mut self` for sequential model (vs `&self` + atomics).
    /// - `notify_first()` omitted; modeled at spec level via `spec_wake()`.
    /// - Returns `()` instead of `Result<(), Error>` (see T5: notify success assumed).
    /// - Ghost `ctx` parameter encodes original `unsafe` preconditions.
    pub fn up(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self)@.value < usize::MAX as nat,
            ctx@.safe_for_up(),
        ensures
            self@.value == old(self)@.value + 1,
            self@.waiters == old(self)@.waiters,
            self.spec_is_available(),
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
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
    ///
    /// # Extra Function Justification (AST-consistency)
    ///
    /// Not in original API. Verification helper providing exec-level access to
    /// the value with a spec-connected postcondition (`result as nat == self@.value`).
    /// Required for proofs that need to bridge exec and spec state.
    pub fn get_value(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result as nat == self@.value,
    {
        proof { reveal(Semaphore::wf); }
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
    ///
    /// # Extra Function Justification (AST-consistency)
    ///
    /// Not in original API. Verification helper providing exec-level availability
    /// check with a spec-connected postcondition (`result == self.spec_is_available()`).
    /// Enables proofs to establish availability from exec state.
    pub fn is_available(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_available(),
    {
        proof { reveal(Semaphore::wf); }
        self.value > 0
    }
}

} // verus!
