// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ScoreBoard Specification.
// This file contains spec functions and View types for the ScoreBoard.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract state of the scoreboard protocol.
///
/// # Description
///
/// Models the three-phase handshake between dispatcher and handler threads:
/// - `Idle`: No dispatch pending. The scoreboard is ready for a new dispatch.
/// - `Dispatched`: Arguments have been set and the handler has been signaled.
///   The dispatcher is waiting for the handler to complete.
/// - `Handled`: The handler has processed the call and set the result.
///   The dispatcher can now read the result.
#[verifier::ext_equal]
pub enum ScoreBoardPhase {
    /// No dispatch pending; scoreboard is ready.
    Idle,
    /// Arguments set, handler signaled, dispatcher waiting.
    Dispatched,
    /// Handler finished, result set, dispatcher can read.
    Handled,
}

/// Abstract view of kernel call arguments.
///
/// # Description
///
/// Represents the observable state of a `KcallArgs` struct at the spec level.
/// All fields are `nat` for spec-level reasoning (original uses `u32`/`i32`).
#[verifier::ext_equal]
pub struct KcallArgsView {
    /// Process identifier (abstract).
    pub pid: int,
    /// Thread identifier (abstract).
    pub tid: int,
    /// Kernel call number.
    pub number: nat,
    /// First argument.
    pub arg0: nat,
    /// Second argument.
    pub arg1: nat,
    /// Third argument.
    pub arg2: nat,
    /// Fourth argument.
    pub arg3: nat,
}

/// Abstract view of a kernel call result.
///
/// # Description
///
/// Models the `KcallResult` enum abstractly as an integer value.
/// Positive/zero values represent success (`KcallSuccess`), negative values
/// represent errors (`KcallError`).
#[verifier::ext_equal]
pub struct KcallResultView {
    /// Abstract result value.
    pub value: int,
}

/// Abstract view of the ScoreBoard.
///
/// # Description
///
/// Captures the full observable state of the scoreboard: the current protocol
/// phase, the kernel call arguments, the result, and whether mutual exclusion
/// is held.
#[verifier::ext_equal]
pub struct ScoreBoardView {
    /// Current protocol phase.
    pub phase: ScoreBoardPhase,
    /// Current kernel call arguments.
    pub args: KcallArgsView,
    /// Current kernel call result.
    pub result: KcallResultView,
    /// Whether the mutex is currently held (by a dispatcher).
    pub locked: bool,
    /// Count of completed dispatch-handle-handled cycles.
    pub completed_cycles: nat,
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for KcallArgs {
    type V = KcallArgsView;

    open spec fn view(&self) -> KcallArgsView {
        KcallArgsView {
            pid: self.pid as int,
            tid: self.tid as int,
            number: self.number as nat,
            arg0: self.arg0 as nat,
            arg1: self.arg1 as nat,
            arg2: self.arg2 as nat,
            arg3: self.arg3 as nat,
        }
    }
}

impl View for KcallResult {
    type V = KcallResultView;

    open spec fn view(&self) -> KcallResultView {
        KcallResultView { value: self.value as int }
    }
}

impl View for ScoreBoard {
    type V = ScoreBoardView;

    open spec fn view(&self) -> ScoreBoardView {
        ScoreBoardView {
            phase: self.phase,
            args: self.args@,
            result: self.result@,
            locked: self.locked,
            completed_cycles: self.completed_cycles as nat,
        }
    }
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl KcallArgs {
    /// Spec function: well-formedness of kernel call arguments.
    ///
    /// # Description
    ///
    /// Arguments are well-formed if all fields are within their valid ranges.
    /// The `number`, `arg0`..`arg3` fields are `u32`, so they must fit in 32 bits.
    /// The `pid` and `tid` fields are `i32`, so they must fit in signed 32 bits.
    pub open spec fn wf(&self) -> bool {
        &&& 0 <= self.pid && self.pid <= i32::MAX as i32
        &&& 0 <= self.tid && self.tid <= i32::MAX as i32
        &&& self.number <= u32::MAX
        &&& self.arg0 <= u32::MAX
        &&& self.arg1 <= u32::MAX
        &&& self.arg2 <= u32::MAX
        &&& self.arg3 <= u32::MAX
    }

    /// Spec function: returns the abstract view of the arguments.
    pub open spec fn spec_view(&self) -> KcallArgsView {
        self@
    }

    /// Spec function: default (uninitialized) arguments view.
    ///
    /// # Description
    ///
    /// The initial arguments before any dispatch has occurred.
    pub open spec fn spec_default_view() -> KcallArgsView {
        KcallArgsView {
            pid: i32::MAX as int,
            tid: i32::MAX as int,
            number: 0,
            arg0: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
        }
    }
}

impl KcallResult {
    /// Spec function: well-formedness of a kernel call result.
    pub open spec fn wf(&self) -> bool {
        i64::MIN as i64 <= self.value && self.value <= i64::MAX as i64
    }

    /// Spec function: the default (ok) result view.
    pub open spec fn spec_ok_view() -> KcallResultView {
        KcallResultView { value: 0 }
    }

    /// Spec function: whether the result represents success.
    pub open spec fn spec_is_ok(&self) -> bool {
        self.value >= 0
    }
}

impl ScoreBoard {
    /// Spec function: well-formedness invariant.
    ///
    /// # Description
    ///
    /// The scoreboard is well-formed when:
    /// 1. Arguments are well-formed.
    /// 2. Result is well-formed.
    /// 3. Phase-consistency: in Idle phase, the board is not locked.
    /// 4. The dispatched/handled semaphore values are consistent with the phase.
    pub open spec fn wf(&self) -> bool {
        &&& self.args.wf()
        &&& self.result.wf()
        &&& self.dispatched_value as nat == self.spec_dispatched_count()
        &&& self.handled_value as nat == self.spec_handled_count()
        &&& (self.phase == ScoreBoardPhase::Idle ==> !self.locked)
        &&& (self.phase == ScoreBoardPhase::Idle ==> self.dispatched_value == 0)
        &&& (self.phase == ScoreBoardPhase::Idle ==> self.handled_value == 0)
        &&& (self.phase == ScoreBoardPhase::Dispatched ==> self.locked)
        &&& (self.phase == ScoreBoardPhase::Dispatched ==> self.dispatched_value == 0)
        &&& (self.phase == ScoreBoardPhase::Dispatched ==> self.handled_value == 0)
        &&& (self.phase == ScoreBoardPhase::Handled ==> self.locked)
        &&& (self.phase == ScoreBoardPhase::Handled ==> self.dispatched_value == 0)
        &&& (self.phase == ScoreBoardPhase::Handled ==> self.handled_value == 1)
    }

    /// Spec function: expected dispatched semaphore count for the current phase.
    ///
    /// # Description
    ///
    /// In all phases, the dispatched semaphore is 0 because:
    /// - Idle: no pending dispatch.
    /// - Dispatched: the handler has consumed the signal (try_down succeeded).
    /// - Handled: the handler consumed the signal earlier.
    pub open spec fn spec_dispatched_count(&self) -> nat {
        0
    }

    /// Spec function: expected handled semaphore count for the current phase.
    ///
    /// # Description
    ///
    /// - Idle: 0 (no pending result).
    /// - Dispatched: 0 (handler hasn't finished yet).
    /// - Handled: 1 (handler has signaled completion).
    pub open spec fn spec_handled_count(&self) -> nat {
        if self.phase == ScoreBoardPhase::Handled { 1 } else { 0 }
    }

    /// Spec function: the scoreboard is in the idle phase.
    pub open spec fn spec_is_idle(&self) -> bool {
        self.phase == ScoreBoardPhase::Idle
    }

    /// Spec function: the scoreboard is in the dispatched phase.
    pub open spec fn spec_is_dispatched(&self) -> bool {
        self.phase == ScoreBoardPhase::Dispatched
    }

    /// Spec function: the scoreboard is in the handled phase.
    pub open spec fn spec_is_handled(&self) -> bool {
        self.phase == ScoreBoardPhase::Handled
    }

    /// Spec function: the initial view of a newly created scoreboard.
    pub open spec fn spec_initial_view() -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Idle,
            args: KcallArgs::spec_default_view(),
            result: KcallResult::spec_ok_view(),
            locked: false,
            completed_cycles: 0,
        }
    }

    /// Spec function: state transition for the begin-dispatch step.
    ///
    /// # Description
    ///
    /// Models the first half of `dispatch()`: acquire mutex, set args,
    /// signal handler via `dispatched.up()`. The phase transitions from
    /// `Idle` to `Dispatched`.
    ///
    /// # Parameters
    ///
    /// - `view`: Current scoreboard view.
    /// - `new_args`: The new kernel call arguments.
    ///
    /// # Returns
    ///
    /// The updated view after begin-dispatch.
    pub open spec fn spec_begin_dispatch(view: ScoreBoardView, new_args: KcallArgsView) -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Dispatched,
            args: new_args,
            result: view.result,
            locked: true,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: state transition for the handle step.
    ///
    /// # Description
    ///
    /// Models `handle()`: the handler consumes the dispatched signal
    /// (via `dispatched.try_down()`) and reads args. Phase stays `Dispatched`
    /// since the handler hasn't produced a result yet.
    pub open spec fn spec_handle(view: ScoreBoardView) -> ScoreBoardView {
        view
    }

    /// Spec function: state transition for the handled step.
    ///
    /// # Description
    ///
    /// Models `handled()`: the handler sets the result and signals the
    /// dispatcher via `handled.up()`. Phase transitions from `Dispatched`
    /// to `Handled`.
    ///
    /// # Parameters
    ///
    /// - `view`: Current scoreboard view (must be `Dispatched`).
    /// - `ret`: The result to store.
    ///
    /// # Returns
    ///
    /// The updated view after handled.
    pub open spec fn spec_handled(view: ScoreBoardView, ret: KcallResultView) -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Handled,
            args: view.args,
            result: ret,
            locked: view.locked,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: state transition for completing a dispatch cycle.
    ///
    /// # Description
    ///
    /// Models the second half of `dispatch()`: after `handled.down()` returns,
    /// the dispatcher reads the result, the mutex guard drops, and the phase
    /// returns to `Idle`. The completed_cycles counter is incremented.
    ///
    /// # Parameters
    ///
    /// - `view`: Current scoreboard view (must be `Handled`).
    ///
    /// # Returns
    ///
    /// The updated view after completing the dispatch.
    pub open spec fn spec_complete_dispatch(view: ScoreBoardView) -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Idle,
            args: view.args,
            result: view.result,
            locked: false,
            completed_cycles: view.completed_cycles + 1,
        }
    }

    /// Spec function: a full dispatch-handle-handled cycle.
    ///
    /// # Description
    ///
    /// Composes the three transitions into one specification of a complete cycle.
    pub open spec fn spec_full_cycle(view: ScoreBoardView, args: KcallArgsView, ret: KcallResultView) -> ScoreBoardView {
        let after_dispatch: ScoreBoardView = Self::spec_begin_dispatch(view, args);
        let after_handled: ScoreBoardView = Self::spec_handled(after_dispatch, ret);
        Self::spec_complete_dispatch(after_handled)
    }
}

} // verus!
