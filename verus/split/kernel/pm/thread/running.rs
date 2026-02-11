// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RunningThread Implementation
//!
//! Represents a thread that is currently running in the Nanvix kernel.
//! Manages state transitions to SleepingThread (via sleep()), ReadyThread
//! (via schedule()), and ZombieThread (via exit()).
//!
//! ## Verified Properties
//!
//! - Construction (from_state) produces well-formed state with correct identity.
//! - Thread identifier (`id`) is immutable: all operations preserve it.
//! - `sleep()` correctly captures the alarm parameter and transitions to
//!   SleepingThread, preserving identity, wf, mutex accounting, and drop safety.
//! - `schedule()` transitions to ReadyThread, preserving identity, wf, mutex
//!   accounting, and drop safety.
//! - `exit()` transitions to ZombieThread with the provided exit status,
//!   preserving identity, wf, mutex accounting, and drop safety.
//! - `store_mutex_guard` / `take_mutex_guard` (via `put_mutex_guard` /
//!   `take_mutex_guard`) preserve identity, wf, and
//!   correctly update per-address mutex accounting.
//! - `thread_state()` returns a reference with the same identity and state.
//! - `id()` correctly returns the thread identifier.
//!
//! ## Verification Model
//!
//! The original `RunningThread` contains complex kernel types. For verification:
//! - `Box<ThreadState>` -> `ThreadState` directly (Box is transparent).
//! - `SystemTime` alarm -> `Option<int>` (abstract timestamp).
//! - `ExitStatus` -> `int` (abstract status tag).
//! - `*mut ContextInformation` -> omitted (HAL boundary, unsafe raw pointer).
//! - `MutexAddress` -> `Ghost<int>` (abstract address).
//! - `MutexGuard` -> elided (RAII payload opaque, protocol-only accounting).
//! - `Condvar` -> elided (sync boundary); `join_cond()` omitted.
//! - `SleepingThread` -> boundary model wrapping `ThreadState` + `Option<int>` alarm.
//! - `ReadyThread` -> boundary model wrapping `ThreadState`.
//! - `ZombieThread` -> boundary model wrapping `ThreadState` + `int` status.
//!
//! ## Trust Boundary
//!
//! - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
//! - `thread_state_mut()` is `#[verifier::external]`: returns `&mut ThreadState`
//!   which Verus cannot express. See documented trust obligations.
//! - `SleepingThread`, `ReadyThread`, and `ZombieThread` are boundary models
//!   of sibling modules. When those modules are verified independently, the
//!   boundary models' postconditions must be confirmed as implied by the real
//!   implementations.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("running.spec.rs");

// Include proofs.
include!("running.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that is currently running.
///
/// Verification model of `src/kernel/src/pm/thread/running.rs::RunningThread`.
/// `Box<ThreadState>` is modeled as `ThreadState` directly.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics (spec access,
/// direct construction in lemmas). The original has private fields.
/// Construction should only occur via `from_state()` which establishes `wf()`.
pub struct RunningThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread that is sleeping (boundary model).
///
/// Models the original `SleepingThread` from the sibling `sleeping.rs` module.
/// Only `from_state` is modeled — enough to verify the sleep() transition.
///
/// **Cross-module dependency:** When `SleepingThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `SleepingThread::from_state` spec.
pub struct SleepingThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The alarm time (abstract timestamp, None = no alarm).
    pub alarm: Option<int>,
}

/// A thread that is ready to run (boundary model).
///
/// Models the original `ReadyThread` from the sibling `ready.rs` module.
/// Only `from_state` is modeled — enough to verify the schedule() transition.
///
/// **Cross-module dependency:** When `ReadyThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `ReadyThread::from_state` spec.
///
/// **Out of scope:** The real `ReadyThread` also holds an `admission_time`
/// field set to `clock::now()` in `from_state`. This is a scheduling
/// property and is intentionally omitted from this boundary model.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread that has terminated (boundary model).
///
/// Models the original `ZombieThread` from the sibling `zombie.rs` module.
/// Only `from_state` is modeled — enough to verify the exit() transition.
///
/// **Cross-module dependency:** When `ZombieThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `ZombieThread::from_state` spec.
pub struct ZombieThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The exit status (abstract int).
    pub status: int,
}

//==================================================================================================
// SleepingThread Implementation (Boundary)
//==================================================================================================

impl SleepingThread {
    /// Creates a SleepingThread from an existing ThreadState and alarm.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state to wrap.
    /// - `alarm`: Optional alarm time for the sleeping thread.
    ///
    /// # Returns
    ///
    /// A well-formed SleepingThread preserving the state's properties.
    pub fn from_state(state: ThreadState, alarm: Option<int>) -> (result: SleepingThread)
        requires
            state.wf(),
        ensures
            result.state@ == state@,
            result.spec_id() == state.spec_id(),
            result.spec_alarm() == alarm,
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        SleepingThread { state: state, alarm: alarm }
    }
}

//==================================================================================================
// ReadyThread Implementation (Boundary)
//==================================================================================================

impl ReadyThread {
    /// Creates a ReadyThread from an existing ThreadState.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state to wrap.
    ///
    /// # Returns
    ///
    /// A well-formed ReadyThread preserving the state's properties.
    ///
    /// # Cross-Module Verification Obligations
    ///
    /// CROSS-MODULE-CHECK: When `ready.rs` is verified, confirm the real
    /// `ReadyThread::from_state` implies all of:
    /// - `result.state@ == state@`
    /// - `result.spec_id() == state.spec_id()`
    /// - `result.spec_locked_mutex_count() == state.spec_locked_mutex_count()`
    /// - `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)`
    /// - `result.spec_drop_safe() == state.spec_drop_safe()`
    /// - `result.wf()`
    pub fn from_state(state: ThreadState) -> (result: ReadyThread)
        requires
            state.wf(),
        ensures
            result.state@ == state@,
            result.spec_id() == state.spec_id(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        ReadyThread { state: state }
    }
}

//==================================================================================================
// ZombieThread Implementation (Boundary)
//==================================================================================================

impl ZombieThread {
    /// Creates a ZombieThread from an existing ThreadState and exit status.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state to wrap.
    /// - `status`: The exit status tag.
    ///
    /// # Returns
    ///
    /// A well-formed ZombieThread preserving the state's properties.
    ///
    /// # Cross-Module Verification Obligations
    ///
    /// CROSS-MODULE-CHECK: When `zombie.rs` is verified, confirm the real
    /// `ZombieThread::from_state` implies all of:
    /// - `result.state@ == state@`
    /// - `result.spec_id() == state.spec_id()`
    /// - `result.spec_status() == status`
    /// - `result.spec_locked_mutex_count() == state.spec_locked_mutex_count()`
    /// - `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)`
    /// - `result.spec_drop_safe() == state.spec_drop_safe()`
    /// - `result.wf()`
    pub fn from_state(state: ThreadState, status: int) -> (result: ZombieThread)
        requires
            state.wf(),
        ensures
            result.state@ == state@,
            result.spec_id() == state.spec_id(),
            result.spec_status() == status,
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        ZombieThread { state: state, status: status }
    }
}

//==================================================================================================
// RunningThread Implementation
//==================================================================================================

impl RunningThread {
    /// Creates a running thread from an existing thread state.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state.
    ///
    /// # Returns
    ///
    /// A well-formed RunningThread preserving all ThreadState properties.
    pub fn from_state(state: ThreadState) -> (result: RunningThread)
        requires
            state.wf(),
        ensures
            result.state@ == state@,
            result.spec_id() == state.spec_id(),
            result.spec_is_interrupted() == state.spec_is_interrupted(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        RunningThread { state: state }
    }

    /// Transitions the running thread to sleeping state.
    ///
    /// # Parameters
    ///
    /// - `alarm`: Optional alarm time for the sleeping thread.
    ///
    /// # Returns
    ///
    /// A SleepingThread with the given alarm time.
    ///
    /// # Modeling Note
    ///
    /// The real `RunningThread::sleep()` also returns a `*mut ContextInformation`
    /// raw pointer. This is omitted (HAL boundary, unsafe raw pointer).
    pub fn sleep(self, alarm: Option<int>) -> (result: SleepingThread)
        requires
            self.wf(),
        ensures
            result.state@ == self.state@,
            result.spec_id() == self.spec_id(),
            result.spec_alarm() == alarm,
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        SleepingThread::from_state(self.state, alarm)
    }

    /// Schedules the running thread by transitioning it to ready state.
    ///
    /// # Returns
    ///
    /// A ReadyThread preserving all properties.
    ///
    /// # Modeling Note
    ///
    /// The real `RunningThread::schedule()` also returns a `*mut ContextInformation`
    /// raw pointer. This is omitted (HAL boundary, unsafe raw pointer).
    pub fn schedule(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result.state@ == self.state@,
            result.spec_id() == self.spec_id(),
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        ReadyThread::from_state(self.state)
    }

    /// Returns the identifier of the running thread.
    ///
    /// # Returns
    ///
    /// The thread identifier, unchanged from construction.
    pub fn id(&self) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == self.spec_id(),
    {
        self.state.id()
    }

    /// Returns a reference to the thread state.
    ///
    /// # Returns
    ///
    /// A reference to the underlying ThreadState with the same identity
    /// and full state transparency.
    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
            result@ == self.state@,
    {
        &self.state
    }

    /// Terminates the running thread with the specified exit status.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status of the thread.
    ///
    /// # Returns
    ///
    /// A ZombieThread with the provided exit status.
    ///
    /// # Modeling Note
    ///
    /// The real `RunningThread::exit()` also returns a `*mut ContextInformation`
    /// raw pointer. This is omitted (HAL boundary, unsafe raw pointer).
    ///
    /// **Design Note:** `spec_drop_safe()` is intentionally NOT a precondition.
    /// The original code allows exit while holding mutexes (the `Drop` impl
    /// only logs an error). This model faithfully mirrors that behavior.
    /// Adding a `spec_drop_safe()` precondition would be a strengthening that
    /// could be considered in a future iteration.
    pub fn exit(self, status: int) -> (result: ZombieThread)
        requires
            self.wf(),
        ensures
            result.state@ == self.state@,
            result.spec_id() == self.spec_id(),
            result.spec_status() == status,
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        ZombieThread::from_state(self.state, status)
    }

    /// Stores a mutex guard address in the underlying thread state.
    ///
    /// Models `put_mutex_guard` from the original `RunningThread`. Named to
    /// match the original public API. Internally delegates to
    /// `ThreadState::store_mutex_guard`. The `MutexAddress` is modeled as
    /// `Ghost<int>` and the `MutexGuard` RAII payload is elided
    /// (protocol-only accounting).
    ///
    /// # Parameters
    ///
    /// - `address`: Ghost address of the mutex being locked.
    ///
    /// # Modeling Note
    ///
    /// The precondition `locked_mutex_count < usize::MAX` is a modeling
    /// artifact that does not exist in the original `BTreeMap::insert`. It
    /// is needed to prevent arithmetic overflow in the ghost counter. In
    /// practice this is unreachable (would require 2^64 held mutexes).
    ///
    /// The precondition `!spec_has_mutex(address@)` (trust assumption T1:
    /// no double-lock) is strictly stronger than the original, which
    /// silently overwrites via `BTreeMap::insert`. This correctly models
    /// non-recursive mutexes where double-locking causes deadlock. If
    /// recursive or reentrant mutex support were added, this precondition
    /// would need to be relaxed.
    pub fn put_mutex_guard(&mut self, address: u64)
        requires
            old(self).wf(),
            old(self).state.locked_mutex_count < usize::MAX,
            !old(self).spec_has_mutex(address as int),
        ensures
            self.spec_has_mutex(address as int),
            forall|a: int| a != address as int ==>
                self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count() + 1,
            !self.spec_drop_safe(),
            self.spec_id() == old(self).spec_id(),
            self.wf(),
    {
        self.state.store_mutex_guard(address);
    }

    /// Takes a mutex guard address from the underlying thread state.
    ///
    /// Models `take_mutex_guard` from the original. The `MutexAddress` is
    /// modeled as `Ghost<int>`. Returns unit (the `Option<MutexGuard>`
    /// return value is elided since the precondition guarantees the address
    /// is held).
    ///
    /// **API Strengthening Note:** The original returns `Option<MutexGuard>`,
    /// allowing callers to handle a `None` (address not found) case. This
    /// verified model requires `spec_has_mutex(address@)` as a precondition,
    /// making the `None` path unreachable by construction. This is trust
    /// assumption T2: callers release only what they hold. Runtime callers
    /// outside the verification boundary must ensure this precondition.
    ///
    /// # Parameters
    ///
    /// - `address`: Ghost address of the mutex being released.
    pub fn take_mutex_guard(&mut self, address: u64)
        requires
            old(self).wf(),
            old(self).spec_has_mutex(address as int),
        ensures
            !self.spec_has_mutex(address as int),
            forall|a: int| a != address as int ==>
                self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count() - 1,
            self.spec_locked_mutex_count() == 0 ==> self.spec_drop_safe(),
            self.spec_id() == old(self).spec_id(),
            self.wf(),
    {
        self.state.take_mutex_guard(address);
    }
}

} // verus!

/// Non-verus impl block for functions that cannot be expressed inside `verus!`
/// due to Verus language limitations (e.g., `&mut T` return types).
impl RunningThread {
    /// Returns a mutable reference to the thread state.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying ThreadState.
    ///
    /// # Trust Boundary
    ///
    /// Marked `#[verifier::external]` because Verus does not yet support
    /// `&mut T` return types — neither `external_body` nor normal `verus!`
    /// functions can express the signature.
    ///
    /// **Prefer verified forwarding methods when possible:**
    /// - `put_mutex_guard()` / `take_mutex_guard()` for mutex accounting.
    ///
    /// This escape hatch is still needed for opaque HAL operations
    /// (e.g., `fpu_state_mut()`, `context_mut()`) that cannot be modeled.
    ///
    /// Callers that mutate the `ThreadState` through this reference operate
    /// outside the verification boundary. Callers MUST preserve:
    /// - `self.wf()` — the well-formedness invariant.
    /// - `self.spec_id()` — the thread identity must not change.
    ///
    /// **Intended postconditions** (not machine-checked):
    /// - `ensures old(self).spec_id() == self.spec_id()` (identity preserved).
    /// - `ensures old(self).wf() ==> self.wf()` (well-formedness preserved).
    #[verifier::external]
    pub fn thread_state_mut(&mut self) -> &mut ThreadState {
        &mut self.state
    }
}
