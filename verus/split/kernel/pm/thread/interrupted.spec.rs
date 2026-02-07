// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedThread Specification.
// Defines View types, spec functions, and invariants for InterruptedThread,
// InterruptReason, and the boundary-type ReadyThread.
//
// ## Verification Model
//
// The original `InterruptedThread` wraps a `Box<ThreadState>` and an
// `InterruptReason` enum (Killed, TimedOut). For verification:
// - `InterruptReason` is modeled as a struct with an `int` tag field.
//   Well-formedness requires the tag is one of the two valid variants.
// - `InterruptedThread` holds a `ThreadState` (verified dependency) and
//   an `InterruptReason`. Well-formedness requires both constituents are wf.
// - `ReadyThread` is a minimal boundary model of the original `ReadyThread`
//   from the sibling `ready.rs` module. It wraps a `ThreadState` and is
//   only used to verify the `resume` state transition.
//
// ## Verified Properties
//
// - spec_id: Thread identity is derived from the underlying ThreadState.
// - spec_reason: The interrupt reason tag is accessible.
// - wf: The compound is well-formed iff both state and reason are wf.
// - State accessors (interrupt reason, mutex count, drop safety) are
//   transparent pass-throughs to the underlying ThreadState specs.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of an InterruptReason.
#[verifier::ext_equal]
pub struct InterruptReasonView {
    /// The abstract reason tag (0 = Killed, 1 = TimedOut).
    pub value: int,
}

/// Abstract view of an InterruptedThread.
#[verifier::ext_equal]
pub struct InterruptedThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// The interrupt reason tag.
    pub reason: int,
}

/// Abstract view of a ReadyThread (boundary type).
#[verifier::ext_equal]
pub struct ReadyThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
}

//==================================================================================================
// Spec Functions: InterruptReason
//==================================================================================================

impl InterruptReason {
    /// Abstract value of the Killed variant.
    pub open spec fn KILLED_VALUE() -> int { 0 }

    /// Abstract value of the TimedOut variant.
    pub open spec fn TIMED_OUT_VALUE() -> int { 1 }

    /// Spec function: returns the abstract reason value.
    pub open spec fn spec_value(&self) -> int {
        self.value
    }

    /// Spec function: well-formedness (value is a valid variant).
    pub open spec fn wf(&self) -> bool {
        self.value == Self::KILLED_VALUE() || self.value == Self::TIMED_OUT_VALUE()
    }

    /// Spec function: checks if this is the Killed reason.
    pub open spec fn spec_is_killed(&self) -> bool {
        self.value == Self::KILLED_VALUE()
    }

    /// Spec function: checks if this is the TimedOut reason.
    pub open spec fn spec_is_timed_out(&self) -> bool {
        self.value == Self::TIMED_OUT_VALUE()
    }
}

//==================================================================================================
// Spec Functions: InterruptedThread
//==================================================================================================

impl InterruptedThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the interrupt reason value.
    pub open spec fn spec_reason(&self) -> int {
        self.reason.spec_value()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// An InterruptedThread is well-formed when:
    /// - The underlying ThreadState is well-formed.
    /// - The interrupt reason is a valid variant.
    pub open spec fn wf(&self) -> bool {
        self.state.wf() && self.reason.wf()
    }

    /// Spec function: returns the underlying state's interrupt reason.
    pub open spec fn spec_state_interrupt_reason(&self) -> Option<int> {
        self.state.spec_interrupt_reason()
    }

    /// Spec function: returns the underlying state's locked mutex count.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }

    /// Spec function: checks if the underlying state is drop-safe.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: checks if the underlying state has a specific mutex.
    pub open spec fn spec_has_mutex(&self, address: int) -> bool {
        self.state.spec_has_mutex(address)
    }

    /// Spec function: returns the underlying state's kernel stack.
    pub open spec fn spec_kernel_stack(&self) -> Option<int> {
        self.state.spec_kernel_stack()
    }

    /// Spec function: returns the underlying state's user stack.
    pub open spec fn spec_user_stack(&self) -> Option<int> {
        self.state.spec_user_stack()
    }
}

//==================================================================================================
// Spec Functions: ReadyThread
//==================================================================================================

impl ReadyThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }

    /// Spec function: returns the state's interrupt reason.
    pub open spec fn spec_interrupt_reason(&self) -> Option<int> {
        self.state.spec_interrupt_reason()
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for InterruptReason {
    type V = InterruptReasonView;

    open spec fn view(&self) -> InterruptReasonView {
        InterruptReasonView { value: self.value }
    }
}

impl View for InterruptedThread {
    type V = InterruptedThreadView;

    open spec fn view(&self) -> InterruptedThreadView {
        InterruptedThreadView {
            state: self.state@,
            reason: self.reason.spec_value(),
        }
    }
}

impl View for ReadyThread {
    type V = ReadyThreadView;

    open spec fn view(&self) -> ReadyThreadView {
        ReadyThreadView {
            state: self.state@,
        }
    }
}

} // verus!
