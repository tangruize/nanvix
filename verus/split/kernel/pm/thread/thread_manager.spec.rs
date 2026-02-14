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
    pub closed spec fn spec_next_id(&self) -> int {
        self.next_id.spec_value()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ThreadManager is well-formed when the next_id is at least 1,
    /// because ID 0 is reserved for the kernel thread and is assigned
    /// during construction.
    ///
    /// Note: `next_id <= i32::MAX` is NOT part of `wf()` by design.
    /// Overflow is an operational constraint checked separately in
    /// `create_thread`'s precondition. A manager with `next_id == i32::MAX`
    /// is structurally valid but cannot create more threads.
    pub closed spec fn wf(&self) -> bool {
        self.next_id.spec_value() >= 1
    }
}

//==================================================================================================
// Spec Functions: ReadyThread (Boundary)
//==================================================================================================

impl ReadyThread {
    /// Spec function: returns the thread identifier value.
    pub closed spec fn spec_id(&self) -> int {
        self.state.spec_id()
    }

    /// Spec function: returns the abstract kernel stack token.
    pub closed spec fn spec_kernel_stack(&self) -> Option<int> {
        self.state.spec_kernel_stack()
    }

    /// Spec function: returns the abstract user stack token.
    pub closed spec fn spec_user_stack(&self) -> Option<int> {
        self.state.spec_user_stack()
    }

    /// Spec function: returns the user thread data area.
    pub closed spec fn spec_user_tda(&self) -> Option<int> {
        self.state.spec_user_tda()
    }

    /// Spec function: well-formedness predicate.
    pub closed spec fn wf(&self) -> bool {
        self.state.wf()
    }

    /// Spec function: checks if the state is drop-safe.
    pub closed spec fn spec_drop_safe(&self) -> bool {
        self.state.spec_drop_safe()
    }

    /// Spec function: checks if the state has an interrupt reason.
    pub closed spec fn spec_is_interrupted(&self) -> bool {
        self.state.spec_is_interrupted()
    }

    /// Spec function: returns the state's locked mutex count.
    pub closed spec fn spec_locked_mutex_count(&self) -> nat {
        self.state.spec_locked_mutex_count()
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for ThreadManager {
    type V = ThreadManagerView;

    // NOTE: view() must remain `open` because the Verus `View` trait requires
    // implementations to use `open spec fn`. This is a justified exception to
    // the guideline that view() should be `pub closed spec fn`.
    open spec fn view(&self) -> ThreadManagerView {
        ThreadManagerView {
            next_id: self.next_id.spec_value(),
        }
    }
}

impl View for ReadyThread {
    type V = ReadyThreadView;

    // NOTE: view() must remain `open` because the Verus `View` trait requires
    // implementations to use `open spec fn`. This is a justified exception to
    // the guideline that view() should be `pub closed spec fn`.
    open spec fn view(&self) -> ReadyThreadView {
        ReadyThreadView {
            state: self.state@,
        }
    }
}

//==================================================================================================
// Spec Functions: ThreadRefModel
//==================================================================================================

impl ThreadRefModel {
    /// Spec function: returns the thread identifier value for any variant.
    pub open spec fn spec_id(&self) -> int {
        match *self {
            ThreadRefModel::Ready(ref s) => s.spec_id(),
            ThreadRefModel::Running(ref s) => s.spec_id(),
            ThreadRefModel::Sleeping(ref s) => s.spec_id(),
            ThreadRefModel::Interrupted(ref s) => s.spec_id(),
            ThreadRefModel::Zombie(ref s) => s.spec_id(),
        }
    }

    /// Spec function: returns the thread state for any variant.
    pub open spec fn spec_state(&self) -> ThreadState {
        match *self {
            ThreadRefModel::Ready(ref s) => *s,
            ThreadRefModel::Running(ref s) => *s,
            ThreadRefModel::Sleeping(ref s) => *s,
            ThreadRefModel::Interrupted(ref s) => *s,
            ThreadRefModel::Zombie(ref s) => *s,
        }
    }
}

//==================================================================================================
// Spec Functions: ThreadRefMutModel
//==================================================================================================

impl ThreadRefMutModel {
    /// Spec function: returns the thread identifier value for any variant.
    pub open spec fn spec_id(&self) -> int {
        match *self {
            ThreadRefMutModel::Ready(ref s) => s.spec_id(),
            ThreadRefMutModel::Running(ref s) => s.spec_id(),
            ThreadRefMutModel::Sleeping(ref s) => s.spec_id(),
            ThreadRefMutModel::Interrupted(ref s) => s.spec_id(),
            ThreadRefMutModel::Zombie(ref s) => s.spec_id(),
        }
    }

    /// Spec function: returns the thread state for any variant.
    pub open spec fn spec_state(&self) -> ThreadState {
        match *self {
            ThreadRefMutModel::Ready(ref s) => *s,
            ThreadRefMutModel::Running(ref s) => *s,
            ThreadRefMutModel::Sleeping(ref s) => *s,
            ThreadRefMutModel::Interrupted(ref s) => *s,
            ThreadRefMutModel::Zombie(ref s) => *s,
        }
    }
}

} // verus!