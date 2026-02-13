// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingThread Specification.
// This file contains spec functions and View types for SleepingThread
// and boundary types ReadyThread and InterruptedThread.
//
// ## Verification Model
//
// SleepingThread wraps a ThreadState (verified dependency) and an
// alarm (abstract optional timestamp). For verification:
// - `alarm` is modeled as `Option<int>` (abstract time, scheduling property).
// - `ReadyThread` is a boundary model wrapping ThreadState.
// - `InterruptedThread` is a boundary model wrapping ThreadState + reason int.
//
// Spec functions are transparent pass-throughs to the underlying ThreadState
// specs, plus SleepingThread-specific properties (alarm).
//
// ## Trust Assumptions
//
// - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
// - `thread_state_mut()` is `#[verifier::external]`: returns `&mut T`
//   which Verus cannot express. See trust boundary docs on that function.
// - `ReadyThread` and `InterruptedThread` boundary models omit fields and
//   methods not needed for verifying SleepingThread state transitions.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a SleepingThread.
#[verifier::ext_equal]
pub struct SleepingThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// Alarm time (abstract optional timestamp).
    pub alarm: Option<int>,
}

/// Abstract view of a ReadyThread (boundary type).
#[verifier::ext_equal]
pub struct ReadyThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// Admission time (abstract timestamp).
    pub admission_time: int,
}

/// Abstract view of an InterruptedThread (boundary type).
#[verifier::ext_equal]
pub struct InterruptedThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// The interrupt reason tag.
    pub reason: int,
}

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Abstract value of the Killed interrupt reason variant.
/// Corresponds to `InterruptReason::Killed` in the original source.
/// TODO (cross-module): When `InterruptReason` is independently verified,
/// confirm `InterruptReason::Killed as int == INTERRUPT_REASON_KILLED()`.
pub open spec fn INTERRUPT_REASON_KILLED() -> int { 0 }

/// Abstract value of the TimedOut interrupt reason variant.
/// Corresponds to `InterruptReason::TimedOut` in the original source.
/// TODO (cross-module): When `InterruptReason` is independently verified,
/// confirm `InterruptReason::TimedOut as int == INTERRUPT_REASON_TIMED_OUT()`.
pub open spec fn INTERRUPT_REASON_TIMED_OUT() -> int { 1 }

//==================================================================================================
// Spec Functions: SleepingThread
//==================================================================================================

impl SleepingThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the alarm time.
    pub open spec fn spec_alarm(&self) -> Option<int> {
        self.alarm
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
    /// A SleepingThread is well-formed when the underlying state is well-formed
    /// and the alarm (if present) is non-negative.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
        && (self.alarm.is_some() ==> self.alarm.unwrap() >= 0)
    }

    /// Spec function: checks if a reason tag is valid (one of the two variants).
    ///
    /// This predicate models the Rust enum `InterruptReason { Killed, TimedOut }`
    /// as an int tag domain `{0, 1}`. Soundness depends on the enum↔int
    /// correspondence documented on `INTERRUPT_REASON_KILLED()` and
    /// `INTERRUPT_REASON_TIMED_OUT()` above.
    /// TODO (cross-module): Discharge via a verified conversion lemma in the
    /// `InterruptReason` module once it is verified.
    pub open spec fn spec_valid_reason(reason: int) -> bool {
        reason == INTERRUPT_REASON_KILLED() || reason == INTERRUPT_REASON_TIMED_OUT()
    }
}

//==================================================================================================
// Spec Functions: ReadyThread (Boundary)
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
        && self.admission_time >= 0
    }
}

//==================================================================================================
// Spec Functions: InterruptedThread (Boundary)
//==================================================================================================

impl InterruptedThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the interrupt reason value.
    pub open spec fn spec_reason(&self) -> int {
        self.reason
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
        self.state.wf() && SleepingThread::spec_valid_reason(self.reason)
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for SleepingThread {
    type V = SleepingThreadView;

    open spec fn view(&self) -> SleepingThreadView {
        SleepingThreadView {
            state: self.state@,
            alarm: self.alarm,
        }
    }
}

impl View for ReadyThread {
    type V = ReadyThreadView;

    open spec fn view(&self) -> ReadyThreadView {
        ReadyThreadView {
            state: self.state@,
            admission_time: self.admission_time,
        }
    }
}

impl View for InterruptedThread {
    type V = InterruptedThreadView;

    open spec fn view(&self) -> InterruptedThreadView {
        InterruptedThreadView {
            state: self.state@,
            reason: self.reason,
        }
    }
}

} // verus!
