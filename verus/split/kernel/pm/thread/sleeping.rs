// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # SleepingThread Implementation
//!
//! Represents a thread that is sleeping in the Nanvix kernel.
//! Manages state transitions to ReadyThread (via wakeup()) and
//! InterruptedThread (via interrupt()).
//!
//! ## Verified Properties
//!
//! - Construction (from_state) produces well-formed state with correct identity and alarm.
//! - Thread identifier (`id`) is immutable: all operations preserve it.
//! - `wakeup()` transitions to ReadyThread preserving identity, wf, mutex accounting,
//!   and drop safety.
//! - `interrupt()` transitions to InterruptedThread with the given reason,
//!   preserving identity, wf, mutex accounting, and drop safety.
//! - `alarm()` returns the alarm captured at construction.
//! - `set_thread_data_area` / `get_thread_data_area` round-trip correctly.
//! - `thread_state()` returns a reference with the same identity and full state.
//! - `id()` correctly returns the thread identifier.
//!
//! ## Out of Scope
//!
//! - **Liveness/scheduling:** The relationship between `alarm` and eventual
//!   wakeup/timeout is a scheduler-level property, not a module-level one.
//!   This module only verifies that `alarm` is faithfully stored and retrieved;
//!   the scheduler is responsible for acting on it.
//! - **Admission time:** `ReadyThread::from_state` in the real implementation
//!   captures `clock::now()` as `admission_time`. This scheduling property is
//!   intentionally omitted from the boundary model.
//!
//! ## Verification Model
//!
//! The original `SleepingThread` contains complex kernel types. For verification:
//! - `Box<ThreadState>` -> `ThreadState` directly (Box is transparent).
//! - `SystemTime` alarm -> `Option<int>` (abstract timestamp).
//! - `InterruptReason` -> `int` tag (0 = Killed, 1 = TimedOut).
//! - `Condvar` -> elided (sync boundary); `join_cond()` omitted.
//! - `VirtualAddress` -> `int` (already modeled as `Option<int>` in ThreadState).
//! - `ReadyThread` -> boundary model wrapping `ThreadState`.
//! - `InterruptedThread` -> boundary model wrapping `ThreadState` + `int` reason.
//!
//! ## Trust Boundary
//!
//! - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
//!   TODO: Add `#[verifier::external_body]` stub if Verus gains opaque token
//!   types, to at least track condvar identity preservation across the
//!   sleeping state. Currently, `Condvar` cannot be meaningfully modeled.
//! - `thread_state_mut()` is `#[verifier::external]`: returns `&mut ThreadState`
//!   which Verus cannot express. See documented trust obligations.
//! - `ReadyThread` and `InterruptedThread` are boundary models of sibling modules.
//!   When those modules are verified independently, the boundary models'
//!   postconditions must be confirmed as implied by the real implementations.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("sleeping.spec.rs");

// Include proofs.
include!("sleeping.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that is sleeping.
///
/// Verification model of `src/kernel/src/pm/thread/sleeping.rs::SleepingThread`.
/// `Box<ThreadState>` is modeled as `ThreadState` directly.
/// `Option<SystemTime>` is modeled as `Option<int>`.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics (spec access,
/// direct construction in lemmas). The original has private fields.
/// Construction should only occur via `from_state()` which establishes `wf()`.
/// All methods require `wf()` as a precondition.
pub struct SleepingThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// Optional alarm time for waking up the thread (abstract timestamp).
    pub alarm: Option<int>,
}

/// A thread that is ready to run (boundary model).
///
/// Models the original `ReadyThread` from the sibling `ready.rs` module.
/// Only `from_state` is modeled — enough to verify the wakeup() transition.
///
/// **Cross-module dependency:** When `ReadyThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `ReadyThread::from_state` spec.
/// TODO (cross-module): Validate boundary model postconditions against
/// real `ready.rs` module once it is independently verified.
///
/// **Out of scope:** The real `ReadyThread` also holds an `admission_time`
/// field set to `clock::now()` in `from_state`. This is a scheduling
/// property and is intentionally omitted from this boundary model.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread that has been interrupted (boundary model).
///
/// Models the original `InterruptedThread` from the sibling `interrupted.rs` module.
/// Only `from_state` is modeled — enough to verify the interrupt() transition.
///
/// **Cross-module dependency:** When `InterruptedThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `InterruptedThread::from_state` spec.
/// TODO (cross-module): Validate boundary model postconditions against
/// real `interrupted.rs` module once it is independently verified.
pub struct InterruptedThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The interrupt reason tag (0 = Killed, 1 = TimedOut).
    pub reason: int,
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
    /// - `result.spec_id() == state.spec_id()`
    /// - `result.spec_locked_mutex_count() == state.spec_locked_mutex_count()`
    /// - `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)`
    /// - `result.spec_drop_safe() == state.spec_drop_safe()`
    /// - `result.wf()`
    pub fn from_state(state: ThreadState) -> (result: ReadyThread)
        requires
            state.wf(),
        ensures
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
// InterruptedThread Implementation (Boundary)
//==================================================================================================

impl InterruptedThread {
    /// Creates an InterruptedThread from an existing ThreadState and reason.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state to wrap.
    /// - `reason`: The interrupt reason tag (0 = Killed, 1 = TimedOut).
    ///
    /// # Returns
    ///
    /// A well-formed InterruptedThread preserving the state's properties.
    ///
    /// # Cross-Module Verification Obligations
    ///
    /// CROSS-MODULE-CHECK: When `interrupted.rs` is verified, confirm the real
    /// `InterruptedThread::from_state` implies all of:
    /// - `result.spec_id() == state.spec_id()`
    /// - `result.spec_reason() == reason`
    /// - `result.spec_locked_mutex_count() == state.spec_locked_mutex_count()`
    /// - `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)`
    /// - `result.spec_drop_safe() == state.spec_drop_safe()`
    /// - `result.wf()`
    pub fn from_state(state: ThreadState, reason: int) -> (result: InterruptedThread)
        requires
            state.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_reason() == reason,
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        InterruptedThread { state: state, reason: reason }
    }
}

//==================================================================================================
// SleepingThread Implementation
//==================================================================================================

impl SleepingThread {
    /// Creates a sleeping thread from an existing thread state and alarm time.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state.
    /// - `alarm`: Optional alarm time for waking up the thread.
    ///
    /// # Returns
    ///
    /// A well-formed SleepingThread preserving all ThreadState properties
    /// and capturing the alarm.
    pub fn from_state(state: ThreadState, alarm: Option<int>) -> (result: SleepingThread)
        requires
            state.wf(),
            alarm.is_some() ==> alarm.unwrap() >= 0,
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_alarm() == alarm,
            result.spec_interrupt_reason() == state.spec_interrupt_reason(),
            result.spec_user_tda() == state.spec_user_tda(),
            result.spec_kernel_stack() == state.spec_kernel_stack(),
            result.spec_user_stack() == state.spec_user_stack(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        SleepingThread { state: state, alarm: alarm }
    }

    /// Wakes up the sleeping thread and transitions it to ready state.
    ///
    /// # Returns
    ///
    /// A ReadyThread preserving identity, well-formedness, mutex accounting,
    /// and drop safety.
    pub fn wakeup(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        ReadyThread::from_state(self.state)
    }

    /// Interrupts the sleeping thread with the specified reason.
    ///
    /// # Parameters
    ///
    /// - `reason`: The interrupt reason tag (0 = Killed, 1 = TimedOut).
    ///
    /// # Returns
    ///
    /// An InterruptedThread with the given reason, preserving identity,
    /// well-formedness, mutex accounting, and drop safety.
    pub fn interrupt(self, reason: int) -> (result: InterruptedThread)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_reason() == reason,
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.wf(),
    {
        InterruptedThread::from_state(self.state, reason)
    }

    /// Returns the identifier of the sleeping thread.
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

    /// Returns the alarm time of the sleeping thread.
    ///
    /// # Returns
    ///
    /// The optional alarm time for waking up the thread.
    pub fn alarm(&self) -> (result: Option<int>)
        ensures
            result == self.spec_alarm(),
    {
        self.alarm
    }

    /// Sets the base address for the user-space thread data area.
    ///
    /// # Parameters
    ///
    /// - `user_tda`: Optional thread data area pointer to set.
    pub fn set_thread_data_area(&mut self, user_tda: Option<int>)
        requires
            old(self).wf(),
        ensures
            self.spec_user_tda() == user_tda,
            self.spec_id() == old(self).spec_id(),
            self.spec_alarm() == old(self).spec_alarm(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_drop_safe() == old(self).spec_drop_safe(),
            self.wf(),
    {
        self.state.store_thread_data_area(user_tda);
    }

    /// Gets the base address for user-space thread data area.
    ///
    /// # Returns
    ///
    /// The optional base address for user-space thread data area.
    pub fn get_thread_data_area(&self) -> (result: Option<int>)
        ensures
            result == self.spec_user_tda(),
    {
        self.state.get_thread_data_area()
    }
}

} // verus!

/// Non-verus impl block for functions that cannot be expressed inside `verus!`
/// due to Verus language limitations (e.g., `&mut T` return types).
impl SleepingThread {
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
    /// Callers that mutate the `ThreadState` through this reference operate
    /// outside the verification boundary. Callers MUST preserve:
    /// - `self.wf()` — the well-formedness invariant.
    /// - `self.spec_id()` — the thread identity must not change.
    /// - `self.spec_alarm()` — the alarm value must not change.
    ///
    /// **Intended postconditions** (not machine-checked):
    /// - `ensures old(self).spec_id() == self.spec_id()` (identity preserved).
    /// - `ensures old(self).wf() ==> self.wf()` (well-formedness preserved).
    /// - `ensures old(self).spec_alarm() == self.spec_alarm()` (alarm preserved).
    ///
    /// **Partial mitigation:** Callers that only need to set the thread data area
    /// should use `set_thread_data_area()` which IS verified.
    ///
    /// TODO: Once Verus supports `&mut T` returns, replace this with a verified
    /// function carrying the above postconditions, or refactor remaining callers
    /// to use specific setter methods with per-field postconditions.
    #[verifier::external]
    pub fn thread_state_mut(&mut self) -> &mut ThreadState {
        &mut self.state
    }
}
