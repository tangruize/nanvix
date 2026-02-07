// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunningThread Specification.
// This file contains spec functions and View types for RunningThread
// and boundary types SleepingThread, ReadyThread, and ZombieThread.
//
// ## Verification Model
//
// RunningThread wraps a ThreadState (verified dependency). For verification:
// - `Box<ThreadState>` is modeled as `ThreadState` directly (Box is transparent).
// - `*mut ContextInformation` return values are omitted (HAL boundary).
// - `SystemTime` alarm is modeled as `Option<int>` (abstract timestamp).
// - `ExitStatus` is modeled as `int` (abstract status tag).
// - `MutexAddress` / `MutexGuard` are modeled via Ghost<int> accounting
//   (protocol-only model, see ThreadState docs).
// - `Condvar` is omitted (sync boundary type); `join_cond()` is omitted.
// - `SleepingThread`, `ReadyThread`, `ZombieThread` are boundary models
//   of sibling module types.
//
// ## Trust Assumptions
//
// - `join_cond()` is omitted: returns opaque `Condvar` (sync boundary).
// - `thread_state_mut()` is `#[verifier::external]`: returns `&mut T`
//   which Verus cannot express. See trust boundary docs on that function.
// - Boundary models omit fields and methods not needed for verifying
//   RunningThread state transitions.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a RunningThread.
#[verifier::ext_equal]
pub struct RunningThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
}

/// Abstract view of a SleepingThread (boundary type).
#[verifier::ext_equal]
pub struct SleepingThreadView {
    /// The abstract thread state.
    pub state: ThreadStateView,
    /// The alarm time (abstract timestamp, None = no alarm).
    pub alarm: Option<int>,
}

/// Abstract view of a ReadyThread (boundary type).
#[verifier::ext_equal]
pub struct ReadyThreadView {
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
// Spec Functions: RunningThread
//==================================================================================================

impl RunningThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
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

    /// Spec function: well-formedness predicate.
    /// A RunningThread is well-formed when the underlying state is well-formed.
    pub open spec fn wf(&self) -> bool {
        self.state.wf()
    }
}

//==================================================================================================
// Spec Functions: SleepingThread (Boundary)
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
// Spec Functions: ReadyThread (Boundary)
//==================================================================================================

impl ReadyThread {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.state.spec_id()
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
// View Implementations
//==================================================================================================

impl View for RunningThread {
    type V = RunningThreadView;

    open spec fn view(&self) -> RunningThreadView {
        RunningThreadView {
            state: self.state@,
        }
    }
}

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
