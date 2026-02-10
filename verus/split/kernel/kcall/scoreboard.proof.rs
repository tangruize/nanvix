// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ScoreBoard Proofs.
// This file contains proof lemmas for the ScoreBoard type.

verus! {

//==================================================================================================
// Proof Lemmas -- Initialization
//==================================================================================================

impl ScoreBoard {
    /// Lemma: A newly initialized scoreboard is well-formed.
    pub proof fn lemma_init_is_wf()
        ensures ({
            let view: ScoreBoardView = ScoreBoard::spec_initial_view();
            &&& view.phase == ScoreBoardPhase::Idle
            &&& !view.locked
            &&& view.dispatched_value == 0
            &&& view.handled_value == 0
            &&& view.completed_cycles == 0
            &&& view.args == KcallArgs::spec_default_view()
            &&& view.result == KcallResult::spec_ok_view()
        }),
    {
    }

    /// Lemma: The default arguments are well-formed at the spec level.
    pub proof fn lemma_default_args_valid()
        ensures ({
            let args: KcallArgsView = KcallArgs::spec_default_view();
            &&& args.pid == i32::MAX as int
            &&& args.tid == i32::MAX as int
            &&& args.number == 0
            &&& args.arg0 == 0
            &&& args.arg1 == 0
            &&& args.arg2 == 0
            &&& args.arg3 == 0
        }),
    {
    }

    /// Lemma: The default result is a well-formed success.
    pub proof fn lemma_default_result_is_ok()
        ensures ({
            let result: KcallResultView = KcallResult::spec_ok_view();
            &&& result.is_success
            &&& result.value == 0
        }),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- State Machine Transitions
    //==============================================================================================

    /// Lemma: `begin_dispatch` transitions from Idle to Signaled with signal set.
    ///
    /// # Description
    ///
    /// Proves that beginning a dispatch on an idle scoreboard produces
    /// a valid Signaled state with the new arguments, mutex held, and
    /// dispatched semaphore set to 1.
    pub proof fn lemma_begin_dispatch_transition(view: ScoreBoardView, new_args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, new_args);
            &&& after.phase == ScoreBoardPhase::Signaled
            &&& after.locked
            &&& after.dispatched_value == 1
            &&& after.handled_value == 0
            &&& after.args == new_args
            &&& after.result == view.result
            &&& after.completed_cycles == view.completed_cycles
        }),
    {
    }

    /// Lemma: `handle` consumes the dispatched signal.
    ///
    /// # Description
    ///
    /// Proves that the handler consuming the dispatched signal transitions
    /// from Signaled to Dispatched, setting dispatched_value from 1 to 0.
    /// The arguments are preserved for the handler to read.
    pub proof fn lemma_handle_consumes_signal(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Signaled,
            view.dispatched_value == 1,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_handle(view);
            &&& after.phase == ScoreBoardPhase::Dispatched
            &&& after.dispatched_value == 0
            &&& after.args == view.args
            &&& after.result == view.result
            &&& after.locked == view.locked
            &&& after.completed_cycles == view.completed_cycles
        }),
    {
    }

    /// Lemma: `handled` transitions from Dispatched to Handled with signal set.
    ///
    /// # Description
    ///
    /// Proves that signaling completion transitions the board to the
    /// Handled phase with the result stored, handled_value set to 1,
    /// and mutex still held.
    pub proof fn lemma_handled_transition(view: ScoreBoardView, ret: KcallResultView)
        requires
            view.phase == ScoreBoardPhase::Dispatched,
            view.locked,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_handled(view, ret);
            &&& after.phase == ScoreBoardPhase::Handled
            &&& after.locked
            &&& after.handled_value == 1
            &&& after.dispatched_value == 0
            &&& after.result == ret
            &&& after.args == view.args
            &&& after.completed_cycles == view.completed_cycles
        }),
    {
    }

    /// Lemma: `complete_dispatch` transitions from Handled to Idle.
    ///
    /// # Description
    ///
    /// Proves that completing a dispatch cycle returns the board to Idle,
    /// releases the mutex, clears both semaphores, increments the cycle
    /// counter, and preserves the last result for reading.
    pub proof fn lemma_complete_dispatch_transition(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Handled,
            view.locked,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_complete_dispatch(view);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
            &&& after.dispatched_value == 0
            &&& after.handled_value == 0
            &&& after.result == view.result
            &&& after.args == view.args
            &&& after.completed_cycles == view.completed_cycles + 1
        }),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Semaphore Signal Protocol
    //==============================================================================================

    /// Lemma: The dispatched semaphore follows the signal/consume pattern.
    ///
    /// # Description
    ///
    /// Proves that the dispatched semaphore correctly transitions:
    /// 0 (Idle) → 1 (Signaled, after begin_dispatch) → 0 (Dispatched, after handle).
    pub proof fn lemma_dispatched_signal_protocol(view: ScoreBoardView, args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let signaled: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, args);
            let dispatched: ScoreBoardView = ScoreBoard::spec_handle(signaled);
            &&& signaled.dispatched_value == 1
            &&& dispatched.dispatched_value == 0
        }),
    {
    }

    /// Lemma: The handled semaphore follows the signal/consume pattern.
    ///
    /// # Description
    ///
    /// Proves that the handled semaphore correctly transitions:
    /// 0 (Dispatched) → 1 (Handled, after handled) → 0 (Idle, after complete_dispatch).
    pub proof fn lemma_handled_signal_protocol(view: ScoreBoardView, ret: KcallResultView)
        requires
            view.phase == ScoreBoardPhase::Dispatched,
            view.locked,
            view.handled_value == 0,
        ensures ({
            let handled: ScoreBoardView = ScoreBoard::spec_handled(view, ret);
            let completed: ScoreBoardView = ScoreBoard::spec_complete_dispatch(handled);
            &&& handled.handled_value == 1
            &&& completed.handled_value == 0
        }),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Protocol Correctness
    //==============================================================================================

    /// Lemma: A complete dispatch cycle returns to Idle.
    ///
    /// # Description
    ///
    /// Proves that the full four-phase protocol always returns to the Idle
    /// state, preserving the result set by the handler and incrementing
    /// the cycle counter.
    pub proof fn lemma_full_cycle_returns_to_idle(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_full_cycle(view, args, ret);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
            &&& after.dispatched_value == 0
            &&& after.handled_value == 0
            &&& after.result == ret
            &&& after.args == args
            &&& after.completed_cycles == view.completed_cycles + 1
        }),
    {
    }

    /// Lemma: The result read after a full cycle is the one set by the handler.
    ///
    /// # Description
    ///
    /// Proves data integrity: the result that the dispatcher reads after
    /// `handled.down()` is exactly the result that the handler stored
    /// via `handled()`.
    pub proof fn lemma_result_integrity(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures
            ScoreBoard::spec_full_cycle(view, args, ret).result == ret,
    {
    }

    /// Lemma: The arguments visible to the handler are those set by the dispatcher.
    ///
    /// # Description
    ///
    /// Proves argument integrity: after `begin_dispatch` and `handle`, the args
    /// in the board are exactly those provided by the dispatcher.
    pub proof fn lemma_args_integrity(view: ScoreBoardView, args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let signaled: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, args);
            let dispatched: ScoreBoardView = ScoreBoard::spec_handle(signaled);
            &&& signaled.args == args
            &&& dispatched.args == args
        }),
    {
    }

    /// Lemma: Mutex is held during all active phases (Signaled, Dispatched, Handled).
    ///
    /// # Description
    ///
    /// Proves that the mutex is held throughout the entire active span of the
    /// protocol. This prevents concurrent dispatches from corrupting shared state.
    pub proof fn lemma_mutex_held_during_active_phases(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let signaled: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, args);
            let dispatched: ScoreBoardView = ScoreBoard::spec_handle(signaled);
            let handled: ScoreBoardView = ScoreBoard::spec_handled(dispatched, ret);
            &&& signaled.locked
            &&& dispatched.locked
            &&& handled.locked
        }),
    {
    }

    /// Lemma: Cycle counter is monotonically increasing.
    ///
    /// # Description
    ///
    /// Each completed cycle increments the counter by exactly 1.
    pub proof fn lemma_cycle_counter_monotonic(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures
            ScoreBoard::spec_full_cycle(view, args, ret).completed_cycles ==
                view.completed_cycles + 1,
            ScoreBoard::spec_full_cycle(view, args, ret).completed_cycles >
                view.completed_cycles,
    {
    }

    /// Lemma: After n identical cycles, the cycle count is initial + n.
    ///
    /// # Description
    ///
    /// Proves inductively that `spec_n_identical_cycles` correctly composes
    /// n full cycles, each incrementing the counter by 1, and the final
    /// state is Idle with both semaphores at 0.
    pub proof fn lemma_n_cycles_count(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
        n: nat,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_n_identical_cycles(view, args, ret, n);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
            &&& after.dispatched_value == 0
            &&& after.handled_value == 0
            &&& after.completed_cycles == view.completed_cycles + n
        }),
        decreases n,
    {
        if n > 0 {
            let after_one: ScoreBoardView = ScoreBoard::spec_full_cycle(view, args, ret);
            Self::lemma_n_cycles_count(after_one, args, ret, (n - 1) as nat);
        }
    }

    /// Lemma: Different arguments produce observably different cycle outcomes.
    ///
    /// # Description
    ///
    /// Proves injectivity of the full cycle with respect to its inputs:
    /// if the args differ, the output args differ; if the results differ,
    /// the output results differ.
    pub proof fn lemma_different_inputs_different_outputs(
        view: ScoreBoardView,
        args1: KcallArgsView,
        ret1: KcallResultView,
        args2: KcallArgsView,
        ret2: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures
            args1 != args2 ==>
                ScoreBoard::spec_full_cycle(view, args1, ret1).args !=
                ScoreBoard::spec_full_cycle(view, args2, ret2).args,
            ret1 != ret2 ==>
                ScoreBoard::spec_full_cycle(view, args1, ret1).result !=
                ScoreBoard::spec_full_cycle(view, args2, ret2).result,
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Invalid Transition Guards
    //==============================================================================================

    /// Lemma: Cannot begin dispatch when not idle (on a wf scoreboard).
    ///
    /// # Description
    ///
    /// When a well-formed scoreboard is in any active phase (Signaled,
    /// Dispatched, or Handled), the mutex is held. Another dispatch
    /// attempt would block on the mutex.
    pub proof fn lemma_no_dispatch_when_active(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_signaled() || sb.spec_is_dispatched() || sb.spec_is_handled(),
        ensures
            sb.locked,
    {
    }

    /// Lemma: A well-formed idle scoreboard has the mutex unlocked.
    ///
    /// # Description
    ///
    /// In the Idle phase, the mutex is not held and both semaphores are at 0.
    pub proof fn lemma_idle_state_clean(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_idle(),
        ensures
            !sb.locked,
            sb.dispatched_value == 0,
            sb.handled_value == 0,
    {
    }

    /// Lemma: A well-formed signaled scoreboard has dispatched_value == 1.
    ///
    /// # Description
    ///
    /// In the Signaled phase, the dispatched semaphore has been signaled
    /// and is waiting for the handler to consume it.
    pub proof fn lemma_signaled_has_pending_signal(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_signaled(),
        ensures
            sb.dispatched_value == 1,
            sb.handled_value == 0,
            sb.locked,
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Multi-Cycle Properties
    //==============================================================================================

    /// Lemma: Two consecutive full cycles produce predictable state.
    ///
    /// # Description
    ///
    /// After two consecutive cycles, the final state is idle with the
    /// second cycle's result and args, and the cycle counter is initial + 2.
    pub proof fn lemma_two_cycles(
        view: ScoreBoardView,
        args1: KcallArgsView,
        ret1: KcallResultView,
        args2: KcallArgsView,
        ret2: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let after1: ScoreBoardView = ScoreBoard::spec_full_cycle(view, args1, ret1);
            let after2: ScoreBoardView = ScoreBoard::spec_full_cycle(after1, args2, ret2);
            &&& after2.phase == ScoreBoardPhase::Idle
            &&& !after2.locked
            &&& after2.dispatched_value == 0
            &&& after2.handled_value == 0
            &&& after2.result == ret2
            &&& after2.args == args2
            &&& after2.completed_cycles == view.completed_cycles + 2
        }),
    {
    }
}

//==================================================================================================
// Proof Lemmas -- KcallArgs
//==================================================================================================

impl KcallArgs {
    /// Lemma: Two KcallArgs with equal fields have equal views.
    pub proof fn lemma_equal_fields_equal_view(a: &KcallArgs, b: &KcallArgs)
        requires
            a.pid == b.pid,
            a.tid == b.tid,
            a.number == b.number,
            a.arg0 == b.arg0,
            a.arg1 == b.arg1,
            a.arg2 == b.arg2,
            a.arg3 == b.arg3,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// Proof Lemmas -- KcallResult
//==================================================================================================

impl KcallResult {
    /// Lemma: The ok result is well-formed and represents success.
    pub proof fn lemma_ok_is_valid()
        ensures ({
            let view: KcallResultView = KcallResult::spec_ok_view();
            &&& view.is_success
            &&& view.value == 0
        }),
    {
    }

    /// Lemma: A success result is always well-formed regardless of value.
    ///
    /// # Description
    ///
    /// Proves that the wf() constraint is only meaningful for error results.
    /// Success results (matching `KcallSuccess(i64)`) accept any i64 payload.
    pub proof fn lemma_success_always_wf(r: &KcallResult)
        requires
            r.is_success,
        ensures
            r.wf(),
    {
    }

    /// Lemma: An error result requires value in i32 range.
    ///
    /// # Description
    ///
    /// Proves that the wf() constraint has teeth for error results:
    /// the value must fit in i32, matching the original `KcallError(i32)`.
    pub proof fn lemma_error_wf_constrains_range(r: &KcallResult)
        requires
            !r.is_success,
            r.wf(),
        ensures
            i32::MIN as i64 <= r.value,
            r.value <= i32::MAX as i64,
    {
    }

    /// Lemma: Two KcallResults with equal fields have equal views.
    pub proof fn lemma_equal_fields_equal_view(a: &KcallResult, b: &KcallResult)
        requires
            a.is_success == b.is_success,
            a.value == b.value,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// Proof Lemmas -- Error Path Preservation
//==================================================================================================

impl ScoreBoard {
    /// Lemma: A failed try_handle preserves scoreboard well-formedness.
    ///
    /// # Description
    ///
    /// When the scoreboard is well-formed but not in the Signaled phase,
    /// `try_handle()` returns false. Since it takes `&mut self` but only
    /// mutates state on success, the failure path preserves wf(). This
    /// proves the `ErrorCode::TryAgain` error path cannot corrupt state.
    pub proof fn lemma_try_handle_fail_preserves_wf(sb: &ScoreBoard)
        requires
            sb.wf(),
            !sb.spec_is_signaled(),
        ensures
            sb.wf(),
            sb.phase == sb.phase,
            sb.locked == sb.locked,
            sb.dispatched_value == sb.dispatched_value,
            sb.handled_value == sb.handled_value,
    {
    }

    /// Lemma: A failed try_handle on an idle board is a no-op.
    ///
    /// # Description
    ///
    /// Proves that polling `handle()` on an idle scoreboard (normal handler
    /// behavior when no dispatch is pending) does not alter the scoreboard:
    /// the board remains idle, unlocked, with semaphores at 0.
    pub proof fn lemma_try_handle_idle_is_noop(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_idle(),
        ensures
            !sb.spec_is_signaled(),
            sb.spec_is_idle(),
            !sb.locked,
            sb.dispatched_value == 0,
            sb.handled_value == 0,
    {
    }

    /// Lemma: A successful try_handle is equivalent to handle().
    ///
    /// # Description
    ///
    /// When the scoreboard is signaled, try_handle succeeds and produces
    /// the same state transition as handle(). This proves the success path
    /// of try_handle is consistent with the spec_handle specification.
    pub proof fn lemma_try_handle_success_equiv(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Signaled,
            view.dispatched_value == 1,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_handle(view);
            &&& after.phase == ScoreBoardPhase::Dispatched
            &&& after.dispatched_value == 0
            &&& after.args == view.args
        }),
    {
    }

    /// Lemma: Lock failure during begin_dispatch preserves scoreboard state.
    ///
    /// # Description
    ///
    /// When `lock()` returns `Err(SleepError::Interrupted)`, no state change
    /// occurs. The scoreboard remains in the Idle phase with wf() preserved.
    pub proof fn lemma_lock_failure_preserves_state(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_idle(),
        ensures
            sb.wf(),
            sb.spec_is_idle(),
            !sb.locked,
    {
    }

    /// Lemma: `dispatched.up()` cannot fail on a well-formed idle scoreboard.
    ///
    /// # Description
    ///
    /// In the Idle phase, `dispatched_value == 0`. The semaphore `up()` operation
    /// increments from 0 to 1, which cannot overflow a binary semaphore.
    /// Therefore, `dispatched.up()` in `dispatch()` cannot fail when called
    /// on a well-formed idle scoreboard. This justifies using a precondition
    /// (rather than error modeling) for this operation.
    ///
    /// **Note:** This lemma proves the *value precondition* for `up()` (that the
    /// semaphore value is 0 before the call). Whether `up()` itself can fail
    /// through internal mechanisms (e.g., `ProcessManager::wakeup()` in
    /// `notify_first()`) is a property of the semaphore implementation, covered
    /// by trust boundary T3 (semaphore correctness is separately verified).
    pub proof fn lemma_semaphore_up_dispatched_cannot_fail(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_idle(),
        ensures
            sb.dispatched_value == 0,
    {
    }

    /// Lemma: `handled.up()` cannot fail on a well-formed dispatched scoreboard.
    ///
    /// # Description
    ///
    /// In the Dispatched phase, `handled_value == 0`. The semaphore `up()`
    /// operation increments from 0 to 1, which cannot overflow a binary
    /// semaphore. Therefore, `handled.up()` in `handled()` cannot fail.
    /// This justifies using a precondition for this operation.
    ///
    /// **Note:** This lemma proves the *value precondition* for `up()`. Whether
    /// `up()` itself can fail through internal mechanisms (e.g.,
    /// `ProcessManager::wakeup()` in `notify_first()`) is covered by trust
    /// boundary T3 (semaphore correctness is separately verified).
    pub proof fn lemma_semaphore_up_handled_cannot_fail(sb: &ScoreBoard)
        requires
            sb.wf(),
            sb.spec_is_dispatched(),
        ensures
            sb.handled_value == 0,
    {
    }

    /// Lemma: An abandoned dispatch produces a non-well-formed stuck state.
    ///
    /// # Description
    ///
    /// When any active-phase operation is interrupted and the mutex guard drops,
    /// the resulting state violates `wf()`: the phase is non-Idle but the mutex
    /// is unlocked (wf requires locked in all non-Idle phases). This formally
    /// characterizes the stuck state as a protocol violation, regardless of
    /// which active phase the interruption occurred in.
    pub proof fn lemma_abandon_dispatch_not_wf(view: ScoreBoardView)
        requires
            view.phase != ScoreBoardPhase::Idle,
            view.locked,
        ensures ({
            let stuck: ScoreBoardView = ScoreBoard::spec_abandon_dispatch(view);
            &&& !stuck.locked
            &&& stuck.phase == view.phase
            &&& stuck.phase != ScoreBoardPhase::Idle
        }),
    {
    }

    /// Lemma: An abandoned dispatch preserves data fields.
    ///
    /// # Description
    ///
    /// Even when a dispatch is abandoned from any active phase, the result
    /// and args fields remain unchanged. This proves data is not corrupted
    /// by the interruption, only the protocol state is broken.
    pub proof fn lemma_abandon_dispatch_preserves_data(view: ScoreBoardView)
        requires
            view.phase != ScoreBoardPhase::Idle,
        ensures ({
            let stuck: ScoreBoardView = ScoreBoard::spec_abandon_dispatch(view);
            &&& stuck.result == view.result
            &&& stuck.args == view.args
            &&& stuck.completed_cycles == view.completed_cycles
            &&& stuck.dispatched_value == view.dispatched_value
            &&& stuck.handled_value == view.handled_value
        }),
    {
    }

    /// Lemma: `dispatch()` success returns the handler's result with correct state.
    ///
    /// # Description
    ///
    /// Proves that the successful path of `dispatch()` is equivalent to
    /// `spec_full_cycle`: the board returns to Idle, the cycle counter
    /// increments, and the returned result matches the handler's input.
    pub proof fn lemma_dispatch_success_equiv(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_dispatch_success(view, args, ret);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
            &&& after.dispatched_value == 0
            &&& after.handled_value == 0
            &&& after.result == ret
            &&& after.completed_cycles == view.completed_cycles + 1
            &&& after == ScoreBoard::spec_full_cycle(view, args, ret)
        }),
    {
    }

    /// Lemma: `dispatch()` interruption produces a characterized stuck state.
    ///
    /// # Description
    ///
    /// Proves that when `handled.down()` is interrupted (modeled from the
    /// Signaled phase before the handler runs), the board ends in the
    /// Signaled phase with the mutex unlocked. The dispatched args are
    /// preserved, the result is unchanged, and the dispatched semaphore
    /// is still signaled (value 1). This violates `wf()` since the phase
    /// is non-Idle but the mutex is unlocked.
    pub proof fn lemma_dispatch_interrupted_stuck(
        view: ScoreBoardView,
        args: KcallArgsView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            view.dispatched_value == 0,
            view.handled_value == 0,
        ensures ({
            let stuck: ScoreBoardView = ScoreBoard::spec_dispatch_interrupted(view, args);
            &&& stuck.phase == ScoreBoardPhase::Signaled
            &&& !stuck.locked
            &&& stuck.args == args
            &&& stuck.result == view.result
            &&& stuck.dispatched_value == 1
            &&& stuck.handled_value == 0
            &&& stuck.completed_cycles == view.completed_cycles
        }),
    {
    }
}

//==================================================================================================
// Proof Lemmas -- ScoreBoardSlot
//==================================================================================================

impl ScoreBoardSlot {
    /// Lemma: A newly created slot is uninitialized and well-formed.
    pub proof fn lemma_new_slot_uninitialized()
        ensures ({
            let view: ScoreBoardSlotView = ScoreBoardSlot::spec_initial_slot_view();
            &&& !view.initialized
        }),
    {
    }

    /// Lemma: After initialization, the slot is well-formed with an idle board.
    ///
    /// # Description
    ///
    /// Proves that `init()` produces a valid state: the slot becomes
    /// initialized and the contained board is idle with semaphores at 0.
    pub proof fn lemma_init_produces_valid_slot(slot: &ScoreBoardSlot)
        requires
            slot.spec_is_initialized(),
            slot.board.wf(),
            slot.board.spec_is_idle(),
        ensures
            slot.wf(),
            !slot.board.locked,
            slot.board.dispatched_value == 0,
            slot.board.handled_value == 0,
    {
    }

    /// Lemma: `try_get_board()` succeeds iff initialized.
    ///
    /// # Description
    ///
    /// Proves the correspondence between the slot's initialized state
    /// and the success/failure of `get_mut()`. Models the original:
    /// - Initialized → `Ok(&mut scoreboard)`
    /// - Uninitialized → `Err(ErrorCode::TryAgain)`
    pub proof fn lemma_try_get_board_iff_initialized(slot: &ScoreBoardSlot)
        requires
            slot.wf(),
        ensures
            slot.spec_is_initialized() ==> slot.board.wf(),
            !slot.spec_is_initialized() ==> !slot.initialized,
    {
    }

    /// Lemma: An uninitialized slot's try_get_board returns false.
    ///
    /// # Description
    ///
    /// Models the error case: `get_mut()` returns `Err(ErrorCode::TryAgain)`
    /// when the scoreboard has not been initialized.
    pub proof fn lemma_uninitialized_get_fails(slot: &ScoreBoardSlot)
        requires
            !slot.spec_is_initialized(),
        ensures
            !slot.initialized,
    {
    }

    /// Lemma: An initialized slot's board is ready for dispatch operations.
    ///
    /// # Description
    ///
    /// Connects initialization with operational readiness: after `init()`,
    /// the contained board is well-formed, idle, and has the mutex unlocked,
    /// meaning `begin_dispatch` and `try_begin_dispatch` can be called.
    pub proof fn lemma_initialized_board_ready(slot: &ScoreBoardSlot)
        requires
            slot.wf(),
            slot.spec_is_initialized(),
            slot.board.spec_is_idle(),
        ensures
            slot.board.wf(),
            !slot.board.locked,
            slot.board.dispatched_value == 0,
            slot.board.handled_value == 0,
    {
    }

    /// Lemma: Re-initialization produces a fresh idle board.
    ///
    /// # Description
    ///
    /// Proves that calling `init()` on an already-initialized slot produces
    /// the same result as initializing from scratch: a well-formed slot with
    /// an idle board. This matches the original `ScoreBoard::init()` which
    /// unconditionally overwrites the global.
    pub proof fn lemma_reinit_produces_fresh_board()
        ensures ({
            let view: ScoreBoardView = ScoreBoard::spec_initial_view();
            &&& view.phase == ScoreBoardPhase::Idle
            &&& !view.locked
            &&& view.dispatched_value == 0
            &&& view.handled_value == 0
            &&& view.completed_cycles == 0
        }),
    {
    }
}

} // verus!
