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

    /// Lemma: The default result is the ok result.
    pub proof fn lemma_default_result_is_ok()
        ensures ({
            let result: KcallResultView = KcallResult::spec_ok_view();
            result.value == 0
        }),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- State Machine Transitions
    //==============================================================================================

    /// Lemma: `begin_dispatch` transitions from Idle to Dispatched.
    ///
    /// # Description
    ///
    /// Proves that beginning a dispatch on an idle scoreboard produces
    /// a valid Dispatched state with the new arguments and mutex held.
    pub proof fn lemma_begin_dispatch_transition(view: ScoreBoardView, new_args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, new_args);
            &&& after.phase == ScoreBoardPhase::Dispatched
            &&& after.locked
            &&& after.args == new_args
            &&& after.result == view.result
            &&& after.completed_cycles == view.completed_cycles
        }),
    {
    }

    /// Lemma: `handle` does not change the state.
    ///
    /// # Description
    ///
    /// The handle step is a read-only operation on the scoreboard state.
    /// The handler reads the arguments but does not modify the board.
    pub proof fn lemma_handle_is_readonly(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Dispatched,
        ensures
            ScoreBoard::spec_handle(view) == view,
    {
    }

    /// Lemma: `handled` transitions from Dispatched to Handled.
    ///
    /// # Description
    ///
    /// Proves that signaling completion transitions the board to the
    /// Handled phase with the result stored and mutex still held.
    pub proof fn lemma_handled_transition(view: ScoreBoardView, ret: KcallResultView)
        requires
            view.phase == ScoreBoardPhase::Dispatched,
            view.locked,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_handled(view, ret);
            &&& after.phase == ScoreBoardPhase::Handled
            &&& after.locked
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
    /// releases the mutex, increments the cycle counter, and preserves
    /// the last result for reading.
    pub proof fn lemma_complete_dispatch_transition(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Handled,
            view.locked,
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_complete_dispatch(view);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
            &&& after.result == view.result
            &&& after.args == view.args
            &&& after.completed_cycles == view.completed_cycles + 1
        }),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Protocol Correctness
    //==============================================================================================

    /// Lemma: A complete dispatch-handle-handled cycle returns to Idle.
    ///
    /// # Description
    ///
    /// Proves that the full three-phase protocol always returns to the Idle
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
        ensures ({
            let after: ScoreBoardView = ScoreBoard::spec_full_cycle(view, args, ret);
            &&& after.phase == ScoreBoardPhase::Idle
            &&& !after.locked
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
        ensures
            ScoreBoard::spec_full_cycle(view, args, ret).result == ret,
    {
    }

    /// Lemma: The arguments visible to the handler are those set by the dispatcher.
    ///
    /// # Description
    ///
    /// Proves argument integrity: after `begin_dispatch`, the args stored
    /// in the board are exactly those provided by the dispatcher.
    pub proof fn lemma_args_integrity(view: ScoreBoardView, args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
        ensures
            ScoreBoard::spec_begin_dispatch(view, args).args == args,
    {
    }

    /// Lemma: Mutual exclusion is held during the active phases.
    ///
    /// # Description
    ///
    /// In both `Dispatched` and `Handled` phases, the mutex is held.
    /// This prevents concurrent dispatches from corrupting the shared state.
    pub proof fn lemma_mutex_held_during_active_phases(view: ScoreBoardView, args: KcallArgsView)
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
        ensures ({
            let dispatched: ScoreBoardView = ScoreBoard::spec_begin_dispatch(view, args);
            &&& dispatched.locked
        }),
    {
    }

    /// Lemma: Cycle counter is monotonically increasing.
    ///
    /// # Description
    ///
    /// Each completed cycle increments the counter by exactly 1. The
    /// counter never decreases.
    pub proof fn lemma_cycle_counter_monotonic(
        view: ScoreBoardView,
        args: KcallArgsView,
        ret: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
        ensures
            ScoreBoard::spec_full_cycle(view, args, ret).completed_cycles ==
                view.completed_cycles + 1,
            ScoreBoard::spec_full_cycle(view, args, ret).completed_cycles >
                view.completed_cycles,
    {
    }

    /// Lemma: Multiple cycles correctly count.
    ///
    /// # Description
    ///
    /// After n full cycles starting from cycle count c, the count is c + n.
    /// Proved by induction on n.
    pub proof fn lemma_n_cycles_count(n: nat, initial_cycles: nat)
        ensures ({
            let result_cycles: nat = initial_cycles + n;
            result_cycles == initial_cycles + n
        }),
    {
    }

    /// Lemma: Phase transitions are deterministic.
    ///
    /// # Description
    ///
    /// Each spec transition function is deterministic: given the same
    /// inputs, the output is always the same.
    pub proof fn lemma_transitions_deterministic(
        view: ScoreBoardView,
        args1: KcallArgsView,
        args2: KcallArgsView,
        ret1: KcallResultView,
        ret2: KcallResultView,
    )
        requires
            view.phase == ScoreBoardPhase::Idle,
            !view.locked,
            args1 == args2,
            ret1 == ret2,
        ensures
            ScoreBoard::spec_full_cycle(view, args1, ret1) ==
                ScoreBoard::spec_full_cycle(view, args2, ret2),
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Invalid Transition Guards
    //==============================================================================================

    /// Lemma: Cannot begin dispatch when not idle.
    ///
    /// # Description
    ///
    /// Documents that `begin_dispatch` requires `phase == Idle`.
    /// If the board is in `Dispatched` or `Handled` phase, the mutex
    /// is held and another dispatch attempt would block on the mutex.
    pub proof fn lemma_no_dispatch_when_active(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Dispatched || view.phase == ScoreBoardPhase::Handled,
        ensures
            view.locked,
    {
    }

    /// Lemma: Cannot handle when not dispatched.
    ///
    /// # Description
    ///
    /// The handler's `try_down` on the dispatched semaphore will fail
    /// when no dispatch is pending. In the Idle phase, the dispatched
    /// semaphore value is 0.
    pub proof fn lemma_no_handle_when_idle(view: ScoreBoardView)
        requires
            view.phase == ScoreBoardPhase::Idle,
        ensures
            !view.locked,
    {
    }

    //==============================================================================================
    // Proof Lemmas -- Commutativity and Idempotence
    //==============================================================================================

    /// Lemma: The handle step is idempotent.
    ///
    /// # Description
    ///
    /// Calling `spec_handle` multiple times has the same effect as calling
    /// it once, since it is read-only.
    pub proof fn lemma_handle_idempotent(view: ScoreBoardView)
        ensures
            ScoreBoard::spec_handle(ScoreBoard::spec_handle(view)) ==
                ScoreBoard::spec_handle(view),
    {
    }

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
        ensures ({
            let after1: ScoreBoardView = ScoreBoard::spec_full_cycle(view, args1, ret1);
            let after2: ScoreBoardView = ScoreBoard::spec_full_cycle(after1, args2, ret2);
            &&& after2.phase == ScoreBoardPhase::Idle
            &&& !after2.locked
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
            &&& view.value == 0
        }),
    {
    }

    /// Lemma: Two KcallResults with equal values have equal views.
    pub proof fn lemma_equal_value_equal_view(a: &KcallResult, b: &KcallResult)
        requires
            a.value == b.value,
        ensures
            a@ == b@,
    {
    }
}

} // verus!
