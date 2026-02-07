// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # InterruptedThread Implementation
//!
//! Represents a thread that has been interrupted and manages the transition
//! back to ready state via resume.
//!
//! ## Verified Properties
//!
//! - Construction preserves thread identity and captures interrupt reason.
//! - Construction produces well-formed state when inputs are well-formed.
//! - Thread identifier (`id`) is immutable: all operations preserve it.
//! - `resume()` correctly stamps the interrupt reason onto the ThreadState
//!   before transitioning to ReadyThread — the key state-transition safety
//!   property.
//! - `resume()` preserves well-formedness, mutex accounting, drop safety,
//!   and stack ownership across the state transition.
//! - InterruptReason variants (Killed, TimedOut) are well-formed, distinct,
//!   and exhaustive.
//! - ReadyThread boundary model preserves identity, well-formedness, and
//!   interrupt reason through from_state.
//!
//! ## Verification Model
//!
//! The original `InterruptedThread` wraps a `Box<ThreadState>` and an
//! `InterruptReason` enum. For verification:
//! - `Box<ThreadState>` is modeled as `ThreadState` directly (Box is a
//!   transparent allocation wrapper with no logical effect).
//! - `InterruptReason` is modeled as a struct with an `int` tag field
//!   (0 = Killed, 1 = TimedOut) with well-formedness validation.
//! - `ReadyThread` is a minimal boundary model of the sibling module type,
//!   wrapping a `ThreadState`. Only `from_state` is modeled.
//! - `Condvar` is opaque (sync boundary type); `join_cond()` is omitted.
//!
//! ## Trust Boundary
//!
//! - `join_cond()` is omitted: it returns an opaque `Condvar` from the sync
//!   subsystem that cannot be meaningfully modeled in a pure spec.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("interrupted.spec.rs");

// Include proofs.
include!("interrupted.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Models the InterruptReason enum from the original code.
///
/// The original enum has two variants: Killed and TimedOut.
/// This verification model uses an int tag with well-formedness
/// constraining the value to one of the two valid variants.
pub struct InterruptReason {
    /// The abstract reason tag (0 = Killed, 1 = TimedOut).
    pub value: int,
}

/// A thread that has been interrupted.
///
/// This is the verification model of `src/kernel/src/pm/thread/interrupted.rs::InterruptedThread`.
/// `Box<ThreadState>` is modeled as `ThreadState` directly.
pub struct InterruptedThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The reason for the interruption.
    pub reason: InterruptReason,
}

/// A ready thread (boundary model for state transition verification).
///
/// Models the original `ReadyThread` from the sibling `ready.rs` module.
/// Only the `from_state` constructor is modeled — enough to verify the
/// resume transition protocol.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

//==================================================================================================
// InterruptReason Implementation
//==================================================================================================

impl InterruptReason {
    /// Creates a Killed interrupt reason.
    ///
    /// # Returns
    ///
    /// A well-formed InterruptReason with the Killed variant tag.
    pub fn killed() -> (result: InterruptReason)
        ensures
            result.spec_value() == InterruptReason::KILLED_VALUE(),
            result.spec_is_killed(),
            !result.spec_is_timed_out(),
            result.wf(),
    {
        InterruptReason { value: 0int }
    }

    /// Creates a TimedOut interrupt reason.
    ///
    /// # Returns
    ///
    /// A well-formed InterruptReason with the TimedOut variant tag.
    pub fn timed_out() -> (result: InterruptReason)
        ensures
            result.spec_value() == InterruptReason::TIMED_OUT_VALUE(),
            result.spec_is_timed_out(),
            !result.spec_is_killed(),
            result.wf(),
    {
        InterruptReason { value: 1int }
    }
}

//==================================================================================================
// ReadyThread Implementation
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
    /// A well-formed ReadyThread preserving the state's identity and
    /// interrupt reason.
    pub fn from_state(state: ThreadState) -> (result: ReadyThread)
        requires
            state.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_interrupt_reason() == state.spec_interrupt_reason(),
            result.wf(),
    {
        ReadyThread { state: state }
    }
}

//==================================================================================================
// InterruptedThread Implementation
//==================================================================================================

impl InterruptedThread {
    /// Creates a new InterruptedThread from a ThreadState and InterruptReason.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state.
    /// - `reason`: The reason for the interruption.
    ///
    /// # Returns
    ///
    /// A well-formed InterruptedThread preserving the state's identity
    /// and capturing the interrupt reason.
    pub fn from_state(state: ThreadState, reason: InterruptReason) -> (result: InterruptedThread)
        requires
            state.wf(),
            reason.wf(),
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_reason() == reason.spec_value(),
            result.wf(),
    {
        InterruptedThread { state: state, reason: reason }
    }

    /// Returns the identifier of the interrupted thread.
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
    /// A reference to the underlying ThreadState with the same identity.
    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
    {
        &self.state
    }

    /// Returns a mutable reference to the thread state.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying ThreadState. The initial
    /// state of the reference has the same identity as the thread.
    pub fn thread_state_mut(&mut self) -> (result: &mut ThreadState)
        ensures
            result.spec_id() == old(self).spec_id(),
    {
        &mut self.state
    }

    /// Resumes the interrupted thread by setting the interrupt reason
    /// on the ThreadState and transitioning to a ReadyThread.
    ///
    /// This is the key state-transition function. It ensures:
    /// - The interrupt reason is correctly propagated to the ThreadState.
    /// - Thread identity is preserved across the transition.
    /// - Well-formedness is maintained.
    ///
    /// # Returns
    ///
    /// A ReadyThread with the interrupt reason stamped on the state.
    pub fn resume(self) -> (result: ReadyThread)
        requires
            self.wf(),
        ensures
            result.spec_id() == self.spec_id(),
            result.spec_interrupt_reason() == Some(self.spec_reason()),
            result.wf(),
    {
        let reason_value: int = self.reason.value;
        let mut state: ThreadState = self.state;
        state.set_interrupt_reason(reason_value);
        ReadyThread::from_state(state)
    }
}

} // verus!
