// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadManager Specification.
// This file contains spec functions and View types for the ThreadManager type.
//
// ## Verification Model
//
// ThreadManager tracks the next thread identifier to assign. For verification:
// - `next_id` is a ThreadIdentifier (verified dependency) that starts at 1
//   (after kernel thread gets ID 0) and increments by 1 per create_thread call.
// - ReadyThread is a boundary model wrapping ThreadState (verified dependency).
//
// The spec focuses on:
// - ID assignment correctness (each thread gets the current next_id).
// - ID monotonicity (next_id strictly increases by 1).
// - Well-formedness preservation (next_id >= 1 after construction).
// - Kernel thread initialization (ID 0).

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a ThreadManager.
#[verifier::ext_equal]
pub struct ThreadManagerView {
    /// The next thread identifier value to be assigned.
    pub next_id: int,
}

/// Abstract view of a ReadyThread (boundary type).
#[verifier::ext_equal]
pub struct ReadyThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
}

//==================================================================================================
// Spec Functions: ThreadManager
//==================================================================================================

impl ThreadManager {
    /// Spec function: returns the next thread identifier value to be assigned.
    pub open spec fn spec_next_id(&self) -> int {
        self.next_id.spec_value()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ThreadManager is well-formed when the next_id is at least 1,
    /// because ID 0 is reserved for the kernel thread and is assigned
    /// during construction.
    pub open spec fn wf(&self) -> bool {
        self.next_id.spec_value() >= 1
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

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }

    /// Spec function: checks if the state is drop-safe.
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: checks if the state has an interrupt reason.
    pub open spec fn spec_is_interrupted(&self) -> bool {
        self.state.spec_is_interrupted()
    }

    /// Spec function: returns the state's locked mutex count.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for ThreadManager {
    type V = ThreadManagerView;

    open spec fn view(&self) -> ThreadManagerView {
        ThreadManagerView {
            next_id: self.next_id.spec_value(),
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
