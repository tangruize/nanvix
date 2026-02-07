// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ReadyThread Specification.
// This file contains spec functions and View types for ReadyThread
// and boundary types RunningThread and ZombieThread.
//
// ## Verification Model
//
// ReadyThread wraps a ThreadState (verified dependency) and an
// admission_time (abstract timestamp). For verification:
// - `admission_time` is modeled as `int` (abstract time, scheduling property).
// - `RunningThread` is a boundary model wrapping ThreadState.
// - `ZombieThread` is a boundary model wrapping ThreadState + status int.
// - `RunResult` bundles the output of `run()`.
//
// Spec functions are transparent pass-throughs to the underlying ThreadState
// specs, plus ReadyThread-specific properties (admission_time).
//
// ## Trust Assumptions
//
// - `clock_now()` is `external_body`: minimal postcondition `result >= 0`
//   reflecting that `SystemTime` is non-negative. Scheduling correctness
//   is outside scope.
// - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
// - `thread_state_mut()` is `#[verifier::external]`: returns `&mut T`
//   which Verus cannot express. See trust boundary docs on that function.
// - `RunningThread` and `ZombieThread` boundary models omit fields and
//   methods not needed for verifying ReadyThread state transitions.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a ReadyThread.
#[verifier::ext_equal]
pub struct ReadyThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// Admission time (abstract timestamp).
    pub admission_time: int,
}

/// Abstract view of a RunningThread (boundary type).
#[verifier::ext_equal]
pub struct RunningThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
}

/// Abstract view of a ZombieThread (boundary type).
#[verifier::ext_equal]
pub struct ZombieThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// The exit status tag.
    pub status: int,
}

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Abstract exit status for interrupted threads.
/// Models `ErrorCode::Interrupted.into()` from the original code.
/// Value 4 corresponds to EINTR (src/libs/sysapi/src/errno.rs:21).
pub open spec fn EXIT_STATUS_INTERRUPTED() -> int { 4 }

//==================================================================================================
// Spec Functions: ReadyThread
//==================================================================================================

impl ReadyThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the admission time.
    pub open spec fn spec_admission_time(&self) -> int {
        self.admission_time
    }

    /// Spec function: returns the state's interrupt reason.
    pub open spec fn spec_interrupt_reason(&self) -> Option<int> {
        self.state.spec_interrupt_reason()
    }

    /// Spec function: returns the state's user thread data area.
    pub open spec fn spec_user_tda(&self) -> Option<int> {
        self.state.spec_user_tda()
    }

    /// Spec function: returns the state's kernel stack token.
    pub open spec fn spec_kernel_stack(&self) -> Option<int> {
        self.state.spec_kernel_stack()
    }

    /// Spec function: returns the state's user stack token.
    pub open spec fn spec_user_stack(&self) -> Option<int> {
        self.state.spec_user_stack()
    }

    /// Spec function: returns the state's locked mutex count.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }

    /// Spec function: returns whether a specific mutex address is held.
    pub open spec fn spec_has_mutex(&self, address: int) -> bool {
        self.state.spec_has_mutex(address)
    }

    /// Spec function: checks if the state is drop-safe.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: checks if the state has an interrupt reason.
    pub open spec fn spec_is_interrupted(&self) -> bool {
        self.state.spec_is_interrupted()
    }

    /// Spec function: well-formedness predicate.
    /// A ReadyThread is well-formed when the underlying state is well-formed.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }
}

//==================================================================================================
// Spec Functions: RunningThread (Boundary)
//==================================================================================================

impl RunningThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: checks if the state has an interrupt reason.
    pub open spec fn spec_is_interrupted(&self) -> bool {
        self.state.spec_is_interrupted()
    }

    /// Spec function: returns the state's locked mutex count.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }

    /// Spec function: returns whether a specific mutex address is held.
    pub open spec fn spec_has_mutex(&self, address: int) -> bool {
        self.state.spec_has_mutex(address)
    }

    /// Spec function: checks if the state is drop-safe.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }
}

//==================================================================================================
// Spec Functions: ZombieThread (Boundary)
//==================================================================================================

impl ZombieThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the exit status tag.
    pub open spec fn spec_status(&self) -> int {
        self.status
    }

    /// Spec function: checks if the state is drop-safe.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: returns the state's locked mutex count.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for ReadyThread {
    type V = ReadyThreadView;

    open spec fn view(&self) -> ReadyThreadView {
        ReadyThreadView {
            state: self.state@,
            admission_time: self.admission_time,
        }
    }
}

impl View for RunningThread {
    type V = RunningThreadView;

    open spec fn view(&self) -> RunningThreadView {
        RunningThreadView {
            state: self.state@,
        }
    }
}

impl View for ZombieThread {
    type V = ZombieThreadView;

    open spec fn view(&self) -> ZombieThreadView {
        ZombieThreadView {
            state: self.state@,
            status: self.status,
        }
    }
}

} // verus!
