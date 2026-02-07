// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ZombieThread Specification.
// This file contains spec functions and View types for ZombieThread.
//
// ## Verification Model
//
// ZombieThread wraps a ThreadState (verified dependency) and an exit status.
// For verification:
// - `Box<ThreadState>` is modeled as `ThreadState` directly (Box is transparent).
// - `ExitStatus` is modeled as `int` (abstract status tag).
// - `KernelStack` / `UserStack` are modeled as `Option<int>` (abstract resource
//   tokens with identity preservation via Option::take semantics in ThreadState).
//
// ## Trust Boundary
//
// - `thread_state_mut()` is `#[verifier::external]`: returns `&mut ThreadState`
//   which Verus cannot express. See trust boundary docs on that function.
// - `harvest()` delegates to ThreadState's `take_kernel_stack()` and
//   `take_user_stack()`, which are verified in the state module.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ZombieThread.
#[verifier::ext_equal]
pub struct ZombieThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// The exit status tag.
    pub status: int,
}

//==================================================================================================
// Spec Functions
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

    /// Spec function: returns the abstract kernel stack token.
    pub open spec fn spec_kernel_stack(&self) -> Option<int> {
        self.state.spec_kernel_stack()
    }

    /// Spec function: returns the abstract user stack token.
    pub open spec fn spec_user_stack(&self) -> Option<int> {
        self.state.spec_user_stack()
    }

    /// Spec function: returns the user thread data area.
    pub open spec fn spec_user_tda(&self) -> Option<int> {
        self.state.spec_user_tda()
    }

    /// Spec function: returns whether the thread has been interrupted.
    pub open spec fn spec_is_interrupted(&self) -> bool {
        self.state.spec_is_interrupted()
    }

    /// Spec function: well-formedness predicate.
    /// A ZombieThread is well-formed when the underlying state is well-formed.
    ///
    /// Note: `status` is modeled as unbounded `int` — an intentional
    /// abstraction of the original `ExitStatus` type. Bounding `status`
    /// (e.g., to `i32` range) is not required for the properties verified
    /// here and is left as an explicit trust boundary simplification.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

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
