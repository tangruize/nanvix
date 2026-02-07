// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedThread Specification.
// Defines View types, spec functions, and invariants for InterruptedThread
// and the boundary-type ReadyThread.
//
// ## Verification Model
//
// The original `InterruptedThread` wraps a `Box<ThreadState>` and an
// `InterruptReason` enum (Killed, TimedOut). For verification:
// - `InterruptReason` is abstracted to an `int` tag value (0 = Killed,
//   1 = TimedOut). Well-formedness requires the tag is one of the two
//   valid variants.
// - `InterruptedThread` holds a `ThreadState` (verified dependency) and
//   a reason `int`. Well-formedness requires the state is wf and the
//   reason is a valid variant.
// - `ReadyThread` is a minimal boundary model of the original `ReadyThread`
//   from the sibling `ready.rs` module. It wraps a `ThreadState` and is
//   only used to verify the `resume` state transition.
//
// ## Trust Assumptions
//
// - `join_cond()` is omitted from the spec: it returns an opaque `Condvar`
//   from the sync subsystem that cannot be meaningfully modeled in a pure
//   spec. Synchronization correctness of thread-join operations involving
//   interrupted threads is outside the verification boundary.
// - `ThreadState::set_interrupt_reason` accepts any `int` without constraining
//   it to a valid `InterruptReason` variant. The constraint is enforced at
//   the `InterruptedThread` level via `wf()` (which requires `spec_valid_reason`).
//   Only `InterruptedThread::resume` calls `set_interrupt_reason`, and it
//   does so with `self.reason` which is guaranteed valid by `self.wf()`.
// - `thread_state_mut` returns `&mut ThreadState` outside the `verus!` block.
//   Callers must preserve `wf()` and `spec_id()`. See the trust boundary
//   documentation on that function.
//
// ## Verified Properties
//
// - spec_id: Thread identity is derived from the underlying ThreadState.
// - spec_reason: The interrupt reason tag is accessible.
// - wf: The compound is well-formed iff state is wf and reason is valid.
// - State accessors (interrupt reason, mutex count, drop safety) are
//   transparent pass-throughs to the underlying ThreadState specs.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

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
// Spec Constants
//==================================================================================================

/// Abstract value of the Killed interrupt reason variant.
pub open spec fn INTERRUPT_REASON_KILLED() -> int { 0 }

/// Abstract value of the TimedOut interrupt reason variant.
pub open spec fn INTERRUPT_REASON_TIMED_OUT() -> int { 1 }

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
        self.reason
    }

    /// Spec function: checks if a reason tag is valid (one of the two variants).
    pub open spec fn spec_valid_reason(reason: int) -> bool {
        reason == INTERRUPT_REASON_KILLED() || reason == INTERRUPT_REASON_TIMED_OUT()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// An InterruptedThread is well-formed when:
    /// - The underlying ThreadState is well-formed.
    /// - The interrupt reason is a valid variant.
    pub open spec fn wf(&self) -> bool {
        self.state.wf() && Self::spec_valid_reason(self.reason)
    }

    /// Spec function: checks if the reason is Killed.
    pub open spec fn spec_is_killed(&self) -> bool {
        self.reason == INTERRUPT_REASON_KILLED()
    }

    /// Spec function: checks if the reason is TimedOut.
    pub open spec fn spec_is_timed_out(&self) -> bool {
        self.reason == INTERRUPT_REASON_TIMED_OUT()
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

impl View for InterruptedThread {
    type V = InterruptedThreadView;

    open spec fn view(&self) -> InterruptedThreadView {
        InterruptedThreadView {
            state: self.state@,
            reason: self.reason,
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
