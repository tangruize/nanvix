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
/// Models the four-phase handshake between dispatcher and handler threads:
/// - `Idle`: No dispatch pending. The scoreboard is ready for a new dispatch.
/// - `Signaled`: Arguments have been set and the dispatched semaphore has been
///   incremented (`up()`). The handler has not yet consumed the signal.
/// - `Dispatched`: The handler has consumed the dispatched signal (`try_down()`)
///   and is processing the call.
/// - `Handled`: The handler has processed the call, set the result, and signaled
///   the handled semaphore. The dispatcher can now read the result.
#[verifier::ext_equal]
pub enum ScoreBoardPhase {
    /// No dispatch pending; scoreboard is ready.
    Idle,
    /// Args set, dispatched semaphore signaled, handler has not consumed yet.
    Signaled,
    /// Handler consumed dispatched signal; processing the call.
    Dispatched,
    /// Handler finished, result set, dispatcher can read.
    Handled,
}

/// Abstract view of kernel call arguments.
///
/// # Description
///
/// Represents the observable state of a `KcallArgs` struct at the spec level.
/// All fields are `nat`/`int` for spec-level reasoning (original uses `u32`/`i32`).
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
/// Models the `KcallResult` enum: `Success(KcallSuccess(i64))` or
/// `Error(KcallError(i32))`. The `is_success` flag distinguishes the variant;
/// the `value` field holds the payload.
#[verifier::ext_equal]
pub struct KcallResultView {
    /// Whether this result represents a success variant.
    pub is_success: bool,
    /// Payload value: any i64 for success, must fit i32 for error.
    pub value: int,
}

/// Abstract view of the ScoreBoard.
///
/// # Description
///
/// Captures the full observable state of the scoreboard: the current protocol
/// phase, the kernel call arguments, the result, mutex and semaphore state.
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
    /// Dispatched semaphore value (0 or 1).
    pub dispatched_value: nat,
    /// Handled semaphore value (0 or 1).
    pub handled_value: nat,
    /// Count of completed dispatch-handle-handled cycles (verification-only).
    pub completed_cycles: nat,
}

/// Abstract view of the scoreboard slot (global singleton model).
///
/// # Description
///
/// Models the `Option<ScoreBoard>` global state. When `initialized` is false,
/// the slot corresponds to `None`; when true, it corresponds to `Some(ScoreBoard)`.
#[verifier::ext_equal]
pub struct ScoreBoardSlotView {
    /// Whether the scoreboard has been initialized.
    pub initialized: bool,
    /// Abstract view of the contained scoreboard.
    pub board: ScoreBoardView,
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
        KcallResultView { is_success: self.is_success, value: self.value as int }
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
            dispatched_value: self.dispatched_value as nat,
            handled_value: self.handled_value as nat,
            completed_cycles: self.completed_cycles@,
        }
    }
}

impl View for ScoreBoardSlot {
    type V = ScoreBoardSlotView;

    open spec fn view(&self) -> ScoreBoardSlotView {
        ScoreBoardSlotView {
            initialized: self.initialized,
            board: self.board@,
        }
    }
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl KcallArgs {
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
    ///
    /// # Description
    ///
    /// Models the original enum constraints:
    /// - `Success(KcallSuccess(i64))`: any i64 value is valid.
    /// - `Error(KcallError(i32))`: the value must fit in an i32.
    pub open spec fn wf(&self) -> bool {
        self.is_success || (i32::MIN as i64 <= self.value && self.value <= i32::MAX as i64)
    }

    /// Spec function: the default (ok) result view.
    pub open spec fn spec_ok_view() -> KcallResultView {
        KcallResultView { is_success: true, value: 0 }
    }

    /// Spec function: whether the result represents a success variant.
    pub open spec fn spec_is_ok(&self) -> bool {
        self.is_success
    }
}

impl ScoreBoard {
    /// Spec function: well-formedness invariant.
    ///
    /// # Description
    ///
    /// The scoreboard is well-formed when:
    /// 1. The result is well-formed (error values fit in i32).
    /// 2. The dispatched/handled semaphore values match phase expectations.
    /// 3. The mutex is held in all non-Idle phases (Signaled, Dispatched, Handled).
    /// 4. The mutex is released in the Idle phase.
    pub open spec fn wf(&self) -> bool {
        &&& self.result.wf()
        &&& self.dispatched_value as nat == self.spec_dispatched_count()
        &&& self.handled_value as nat == self.spec_handled_count()
        &&& (self.phase == ScoreBoardPhase::Idle ==> !self.locked)
        &&& (self.phase != ScoreBoardPhase::Idle ==> self.locked)
    }

    /// Spec function: expected dispatched semaphore count for the current phase.
    ///
    /// # Description
    ///
    /// - Idle: 0 (no pending dispatch).
    /// - Signaled: 1 (dispatcher called `dispatched.up()`, handler has not consumed).
    /// - Dispatched: 0 (handler consumed the signal via `try_down()`).
    /// - Handled: 0 (handler consumed the signal earlier).
    pub open spec fn spec_dispatched_count(&self) -> nat {
        if self.phase == ScoreBoardPhase::Signaled { 1 } else { 0 }
    }

    /// Spec function: expected handled semaphore count for the current phase.
    ///
    /// # Description
    ///
    /// - Idle: 0 (no pending result).
    /// - Signaled: 0 (handler has not started yet).
    /// - Dispatched: 0 (handler hasn't finished yet).
    /// - Handled: 1 (handler has signaled completion via `handled.up()`).
    pub open spec fn spec_handled_count(&self) -> nat {
        if self.phase == ScoreBoardPhase::Handled { 1 } else { 0 }
    }

    /// Spec function: the scoreboard is in the idle phase.
    pub open spec fn spec_is_idle(&self) -> bool {
        self.phase == ScoreBoardPhase::Idle
    }

    /// Spec function: the scoreboard is in the signaled phase.
    pub open spec fn spec_is_signaled(&self) -> bool {
        self.phase == ScoreBoardPhase::Signaled
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
            dispatched_value: 0,
            handled_value: 0,
            completed_cycles: 0,
        }
    }

    /// Spec function: state transition for the begin-dispatch step.
    ///
    /// # Description
    ///
    /// Models the first part of `dispatch()`: acquire mutex, set args,
    /// signal handler via `dispatched.up()`. The phase transitions from
    /// `Idle` to `Signaled`, and `dispatched_value` becomes 1.
    ///
    /// # Parameters
    ///
    /// - `view`: Current scoreboard view (must be Idle).
    /// - `new_args`: The new kernel call arguments.
    ///
    /// # Returns
    ///
    /// The updated view after begin-dispatch.
    pub open spec fn spec_begin_dispatch(view: ScoreBoardView, new_args: KcallArgsView) -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Signaled,
            args: new_args,
            result: view.result,
            locked: true,
            dispatched_value: 1,
            handled_value: 0,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: state transition for the handle step.
    ///
    /// # Description
    ///
    /// Models `handle()`: the handler consumes the dispatched signal
    /// (via `dispatched.try_down()`) and reads args. The phase transitions
    /// from `Signaled` to `Dispatched`, and `dispatched_value` becomes 0.
    pub open spec fn spec_handle(view: ScoreBoardView) -> ScoreBoardView {
        ScoreBoardView {
            phase: ScoreBoardPhase::Dispatched,
            args: view.args,
            result: view.result,
            locked: view.locked,
            dispatched_value: 0,
            handled_value: view.handled_value,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: state transition for the handled step.
    ///
    /// # Description
    ///
    /// Models `handled()`: the handler sets the result and signals the
    /// dispatcher via `handled.up()`. Phase transitions from `Dispatched`
    /// to `Handled`, and `handled_value` becomes 1.
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
            dispatched_value: 0,
            handled_value: 1,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: state transition for completing a dispatch cycle.
    ///
    /// # Description
    ///
    /// Models the second half of `dispatch()`: after `handled.down()` returns,
    /// the dispatcher reads the result, the mutex guard drops, and the phase
    /// returns to `Idle`. The `completed_cycles` counter is incremented.
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
            dispatched_value: 0,
            handled_value: 0,
            completed_cycles: view.completed_cycles + 1,
        }
    }

    /// Spec function: state transition for an abandoned dispatch.
    ///
    /// # Description
    ///
    /// Models the error path when `handled.down()` is interrupted (or when
    /// any active-phase operation fails): the mutex guard drops (releasing
    /// the lock) but the protocol is not completed. The board is left in
    /// its current phase with mutex unlocked.
    ///
    /// The resulting state violates `wf()` (phase is non-Idle but locked is
    /// false), reflecting a genuine stuck state that requires external recovery.
    pub open spec fn spec_abandon_dispatch(view: ScoreBoardView) -> ScoreBoardView {
        ScoreBoardView {
            phase: view.phase,
            args: view.args,
            result: view.result,
            locked: false,
            dispatched_value: view.dispatched_value,
            handled_value: view.handled_value,
            completed_cycles: view.completed_cycles,
        }
    }

    /// Spec function: full dispatch outcome on success (lock acquired, down not interrupted).
    ///
    /// # Description
    ///
    /// Models the successful path of the original `dispatch()`: the board goes
    /// through Idle → Signaled → Dispatched → Handled → Idle, returning the
    /// handler's result. Equivalent to `spec_full_cycle`.
    pub open spec fn spec_dispatch_success(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    ) -> ScoreBoardView {
        Self::spec_full_cycle(view, args, ret)
    }

    /// Spec function: dispatch outcome on down interruption.
    ///
    /// # Description
    ///
    /// Models the error path where the lock is acquired but `handled.down()`
    /// is interrupted. The interruption is modeled from the Signaled phase
    /// (before the handler runs), which is the most conservative: the
    /// dispatched semaphore has been signaled (value 1) but the handler has
    /// not consumed it, and the result is unchanged from the prior state.
    ///
    /// In the real implementation, `handled.down()` blocks and the interrupt
    /// can catch the protocol in any active phase (Signaled, Dispatched, or
    /// Handled). The `spec_abandon_dispatch` function models abandonment from
    /// arbitrary active phases for fine-grained reasoning.
    pub open spec fn spec_dispatch_interrupted(
        view: ScoreBoardView,
        args: KcallArgsView,
    ) -> ScoreBoardView {
        let after_signal: ScoreBoardView = Self::spec_begin_dispatch(view, args);
        Self::spec_abandon_dispatch(after_signal)
    }

    /// Spec function: a full dispatch-handle-handled cycle.
    ///
    /// # Description
    ///
    /// Composes the four transitions into one specification of a complete cycle:
    /// `begin_dispatch` → `handle` → `handled` → `complete_dispatch`.
    pub open spec fn spec_full_cycle(view: ScoreBoardView, args: KcallArgsView, ret: KcallResultView) -> ScoreBoardView {
        let after_signal: ScoreBoardView = Self::spec_begin_dispatch(view, args);
        let after_handle: ScoreBoardView = Self::spec_handle(after_signal);
        let after_handled: ScoreBoardView = Self::spec_handled(after_handle, ret);
        Self::spec_complete_dispatch(after_handled)
    }

    /// Spec function: apply n identical full cycles.
    ///
    /// # Description
    ///
    /// Recursively composes `n` full cycles using the same args and result.
    /// Used to prove inductive properties about the cycle counter.
    ///
    /// Note: this uses identical args/ret for all cycles. The cycle counter
    /// property (`completed_cycles == initial + n`) generalizes to varying
    /// inputs, since each `spec_full_cycle` increments the counter by 1
    /// regardless of the specific args/ret values.
    pub open spec fn spec_n_identical_cycles(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
        n: nat,
    ) -> ScoreBoardView
        decreases n,
    {
        if n == 0 {
            view
        } else {
            let after_one: ScoreBoardView = Self::spec_full_cycle(view, args, ret);
            Self::spec_n_identical_cycles(after_one, args, ret, (n - 1) as nat)
        }
    }
}

impl ScoreBoardSlot {
    /// Spec function: well-formedness of the scoreboard slot.
    ///
    /// # Description
    ///
    /// The slot is well-formed when: if initialized, the contained board
    /// is well-formed. An uninitialized slot is trivially well-formed.
    pub open spec fn wf(&self) -> bool {
        self.initialized ==> self.board.wf()
    }

    /// Spec function: whether the slot has been initialized.
    pub open spec fn spec_is_initialized(&self) -> bool {
        self.initialized
    }

    /// Spec function: the initial (uninitialized) slot view.
    pub open spec fn spec_initial_slot_view() -> ScoreBoardSlotView {
        ScoreBoardSlotView {
            initialized: false,
            board: ScoreBoard::spec_initial_view(),
        }
    }
}

} // verus!
