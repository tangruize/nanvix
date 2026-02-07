// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ReadyThread Implementation
//!
//! Represents a thread that is ready to run in the Nanvix kernel.
//! Manages state transitions to RunningThread (via run()) and
//! ZombieThread (via terminate()).
//!
//! ## Verified Properties
//!
//! - Construction (new, from_state) produces well-formed state with correct identity.
//! - Thread identifier (`id`) is immutable: all operations preserve it.
//! - `run()` correctly extracts the interrupt reason and user TDA from the
//!   state, passes the state (with interrupt reason cleared) to RunningThread.
//! - `run()` preserves well-formedness, mutex accounting, and drop safety.
//! - `terminate()` transitions to ZombieThread with Interrupted exit status.
//! - `terminate()` preserves identity and well-formedness.
//! - `admission_time()` returns the time captured at construction.
//! - `from_state()` preserves all ThreadState properties.
//!
//! ## Verification Model
//!
//! The original `ReadyThread` contains complex kernel types. For verification:
//! - `Box<ThreadState>` -> `ThreadState` directly (Box is transparent).
//! - `SystemTime` -> `int` (abstract timestamp).
//! - `clock::now()` -> `clock_now()` external_body returning `int`.
//! - `ContextInformation`, `FpuState` -> elided (HAL boundary).
//! - `Condvar` -> elided (sync boundary); `join_cond()` omitted.
//! - `RunningThread` -> boundary model wrapping `ThreadState`.
//! - `ZombieThread` -> boundary model wrapping `ThreadState` + `int` status.
//! - `ErrorCode::Interrupted.into()` -> `EXIT_STATUS_INTERRUPTED()` constant.
//! - `InterruptReason` -> `Option<int>` (already modeled in ThreadState).
//! - `VirtualAddress` -> `int` (already modeled as `Option<int>` in ThreadState).
//! - `run()` returns `RunResult` struct omitting the raw pointer
//!   `*mut ContextInformation` (unsafe HAL boundary).
//!
//! ## Trust Boundary
//!
//! - `clock_now()` is `external_body`: returns an abstract timestamp with no
//!   spec-level constraint. The actual value is a scheduling property, not
//!   a safety property.
//! - `thread_state_mut()` is `#[verifier::external]`: returns `&mut ThreadState`
//!   which Verus cannot express. See documented trust obligations.
//! - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
//! - `RunningThread` and `ZombieThread` are boundary models of sibling modules.
//!   When those modules are verified independently, the boundary models'
//!   postconditions must be confirmed as implied by the real implementations.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("ready.spec.rs");

// Include proofs.
include!("ready.proof.rs");

verus! {

//==================================================================================================
// External Dependencies
//==================================================================================================

/// Abstract model of `clock::now()`.
///
/// Returns an abstract timestamp. No spec-level constraint is provided
/// because the actual clock value is a scheduling property, not a
/// safety or identity property.
#[verifier::external_body]
fn clock_now() -> (result: int) {
    unimplemented!()
}

/// Exec-level accessor for the EXIT_STATUS_INTERRUPTED spec constant.
///
/// Bridges the spec/exec boundary for the abstract exit status value
/// used in `terminate()`. Marked `external_body` because `int` literals
/// cannot be directly constructed in Verus exec code.
#[verifier::external_body]
fn exit_status_interrupted_value() -> (result: int)
    ensures
        result == EXIT_STATUS_INTERRUPTED(),
{
    unimplemented!()
}

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that is ready to run.
///
/// Verification model of `src/kernel/src/pm/thread/ready.rs::ReadyThread`.
/// `Box<ThreadState>` is modeled as `ThreadState` directly.
/// `SystemTime` is modeled as `int`.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// Time when the thread was admitted to the ready queue.
    pub admission_time: int,
}

/// A thread that is currently running (boundary model).
///
/// Models the original `RunningThread` from the sibling `running.rs` module.
/// Only `from_state` is modeled — enough to verify the run() transition.
///
/// **Cross-module dependency:** When `RunningThread` is verified independently,
/// the postconditions of this boundary model's `from_state` must be confirmed
/// as implied by the real `RunningThread::from_state` spec.
pub struct RunningThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread that has terminated (boundary model).
///
/// Models the original `ZombieThread` from the sibling `zombie.rs` module.
/// Only `from_state` is modeled — enough to verify the terminate() transition.
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

/// Result of `ReadyThread::run()`.
///
/// Bundles the running thread, extracted interrupt reason, and user TDA.
/// The raw pointer `*mut ContextInformation` from the original return
/// type is omitted (unsafe HAL boundary, out of verification scope).
pub struct RunResult {
    /// The running thread with interrupt reason cleared.
    pub running: RunningThread,
    /// The interrupt reason extracted from the state (if any).
    pub interrupt_reason: Option<int>,
    /// The user-space thread data area address (if any).
    pub user_tda: Option<int>,
}

//==================================================================================================
// RunningThread Implementation (Boundary)
//==================================================================================================

impl RunningThread {
    /// Creates a RunningThread from an existing ThreadState.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state to wrap.
    ///
    /// # Returns
    ///
    /// A well-formed RunningThread preserving the state's properties.
    pub fn from_state(state: ThreadState) -> (result: RunningThread)
        requires
            state.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_is_interrupted() == state.spec_is_interrupted(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        RunningThread { state: state }
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
    pub fn from_state(state: ThreadState, status: int) -> (result: ZombieThread)
        requires
            state.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_status() == status,
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            result.wf(),
    {
        ZombieThread { state: state, status: status }
    }
}

//==================================================================================================
// ReadyThread Implementation
//==================================================================================================

impl ReadyThread {
    /// Creates a new ready thread.
    ///
    /// # Parameters
    ///
    /// - `id`: Thread identifier.
    /// - `kernel_stack`: Optional abstract kernel stack resource token.
    /// - `user_stack`: Optional abstract user stack resource token.
    /// - `user_tda`: Optional base address for user-space thread data area.
    ///
    /// # Omitted Parameters
    ///
    /// The original constructor also takes `context: ContextInformation` and
    /// `fpu_state: FpuState` (opaque HAL types, out of verification scope).
    ///
    /// # Returns
    ///
    /// A new, well-formed, drop-safe ReadyThread.
    pub fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        ensures
            result.spec_id() == id.spec_value(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            result.spec_drop_safe(),
            result.wf(),
    {
        ReadyThread {
            state: ThreadState::new(id, kernel_stack, user_stack, user_tda),
            admission_time: clock_now(),
        }
    }

    /// Creates a ready thread from an existing thread state.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state.
    ///
    /// # Returns
    ///
    /// A well-formed ReadyThread preserving all ThreadState properties.
    pub fn from_state(state: ThreadState) -> (result: ReadyThread)
        requires
            state.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_interrupt_reason() == state.spec_interrupt_reason(),
            result.spec_user_tda() == state.spec_user_tda(),
            result.spec_kernel_stack() == state.spec_kernel_stack(),
            result.spec_user_stack() == state.spec_user_stack(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        ReadyThread {
            state: state,
            admission_time: clock_now(),
        }
    }

    /// Returns the identifier of the ready thread.
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

    /// Returns the admission time of the ready thread.
    ///
    /// # Returns
    ///
    /// The time when the thread was admitted to the ready queue.
    pub fn admission_time(&self) -> (result: int)
        ensures
            result == self.spec_admission_time(),
    {
        self.admission_time
    }

    /// Transitions the ready thread to running state.
    ///
    /// Extracts the interrupt reason and user TDA from the state,
    /// then creates a RunningThread with the interrupt reason cleared.
    /// This is the key scheduling transition: a ready thread becomes
    /// the currently executing thread.
    ///
    /// # Returns
    ///
    /// A RunResult containing:
    /// - The running thread (state with interrupt reason cleared).
    /// - The extracted interrupt reason (if any).
    /// - The user-space thread data area address (if any).
    pub fn run(self) -> (result: RunResult)
        requires
            self.wf(),
        ensures
            result.running.spec_id() == self.spec_id(),
            !result.running.spec_is_interrupted(),
            result.interrupt_reason == self.spec_interrupt_reason(),
            result.user_tda == self.spec_user_tda(),
            result.running.wf(),
            result.running.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
            forall|a: int| #![auto] result.running.spec_has_mutex(a) == self.spec_has_mutex(a),
            result.running.spec_drop_safe() == self.spec_drop_safe(),
    {
        let mut state: ThreadState = self.state;
        let interrupt_reason: Option<int> = state.take_interrupt_reason();
        let user_tda: Option<int> = state.get_thread_data_area();
        RunResult {
            running: RunningThread::from_state(state),
            interrupt_reason: interrupt_reason,
            user_tda: user_tda,
        }
    }

    /// Terminates the ready thread and transitions it to zombie state.
    ///
    /// The exit status is set to the Interrupted constant, modeling
    /// `ErrorCode::Interrupted.into()` from the original code.
    ///
    /// # Returns
    ///
    /// A ZombieThread with Interrupted exit status.
    pub fn terminate(self) -> (result: ZombieThread)
        requires
            self.wf(),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_status() == EXIT_STATUS_INTERRUPTED(),
            result.wf(),
            result.spec_drop_safe() == self.spec_drop_safe(),
            result.spec_locked_mutex_count() == self.spec_locked_mutex_count(),
    {
        let status: int = exit_status_interrupted_value();
        ZombieThread::from_state(self.state, status)
    }
}

} // verus!

/// Non-verus impl block for functions that cannot be expressed inside `verus!`
/// due to Verus language limitations (e.g., `&mut T` return types).
impl ReadyThread {
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
    ///
    /// **Intended postconditions** (not machine-checked):
    /// - `ensures old(self).spec_id() == self.spec_id()` (identity preserved).
    /// - `ensures old(self).wf() ==> self.wf()` (well-formedness preserved).
    #[verifier::external]
    pub fn thread_state_mut(&mut self) -> &mut ThreadState {
        &mut self.state
    }
}
