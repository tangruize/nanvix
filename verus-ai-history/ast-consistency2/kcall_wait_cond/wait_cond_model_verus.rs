pub fn wait_cond_model(
    cond_addr: u32,
    mutex_addr: u32,
    timeout_s: u32,
    timeout_ns: u32,
    pid: Ghost<u32>,
    tid: Ghost<u32>,
) -> (ret: (WaitCondResultModel, Ghost<WaitCondGhostState>))
    requires
        USIZE_BITS() == 32,
        spec_wait_cond_safety_preconditions(pid@ as nat, tid@ as nat),
        spec_is_currently_running(pid@ as nat, tid@ as nat),
    ensures
        // Exec-spec equivalence: the result matches the spec function.
        ret.0.spec_view() == spec_wait_cond_result(
            timeout_s as nat, timeout_ns as nat,
            ret.1@.tmg, ret.1@.gc, ret.1@.cw,
            ret.1@.pc, ret.1@.gm, ret.1@.lo, ret.1@.pg,
        ),
        // Mutex protocol on stored-result return: whenever the continuation
        // pipeline completes successfully (returning the stored result), the
        // mutex was released before the wait and reacquired afterward, and the
        // condvar reference was released.
        spec_is_stored_result_return(ret.0.spec_view()) ==> (
            spec_mutex_released(mutex_addr as nat)
            && spec_cond_ref_released(cond_addr as nat)
            && spec_mutex_reacquired(mutex_addr as nat)
        ),
        // Resource release: mutex was released whenever take_mutex_guard succeeded,
        // independent of later continuation errors.
        ret.1@.tmg matches TakeMutexGuardOutcomeView::TmgOk
            ==> spec_mutex_released(mutex_addr as nat),
        // Resource release: condvar reference was released whenever put_cond
        // succeeded AND get_cond previously acquired a reference (GcOk).
        // When get_cond fails, no reference was acquired, so "released" is
        // meaningless even if put_cond somehow returned Ok (which is itself
        // unreachable — see overapproximation note in trust boundaries).
        (ret.1@.gc matches GetCondOutcomeView::GcOk
            && ret.1@.pc matches PutCondOutcomeView::PcOk)
            ==> spec_cond_ref_released(cond_addr as nat),
        // Resource release: mutex was reacquired whenever lock succeeded,
        // independent of later put_guard errors.
        ret.1@.lo matches LockOutcomeView::LoOk
            ==> spec_mutex_reacquired(mutex_addr as nat),
{
    // Step 1: Parse timeout.
    let parse_result: (bool, bool) = parse_timeout_model(timeout_s, timeout_ns);
    let timeout_ok: bool = parse_result.0;
    let has_alarm: bool = parse_result.1;

    if !timeout_ok {
        let error_code: i32 = ErrorCode::InvalidArgument as i32;
        proof {
            assert(error_code as int == ERROR_CODE_INVALID_ARGUMENT());
        }
        let result: WaitCondResultModel =
            WaitCondResultModel::InvalidTimeoutError { error_code };
        let ghost gs: WaitCondGhostState = WaitCondGhostState {
            // Don't-care values: spec short-circuits on invalid timeout,
            // so these are never inspected. Use Error variants so resource-release
            // postconditions (keyed on Ok) are not falsely triggered.
            tmg: TakeMutexGuardOutcomeView::TmgError { error_code: 0 },
            gc: GetCondOutcomeView::GcError { error_code: 0 },
            cw: CondWaitOutcomeView::CwOk,
            pc: PutCondOutcomeView::PcError { error_code: 0 },
            gm: GetMutexOutcomeView::GmError { error_code: 0 },
            lo: LockOutcomeView::LoGenericError { error_code: 0 },
            pg: PutGuardOutcomeView::PgError { error_code: 0 },
        };
        return (result, Ghost(gs));
    }

    // Step 2: take_mutex_guard → release mutex.
    let tmg_result: TakeMutexGuardOutcomeModel =
        take_mutex_guard_model(mutex_addr, pid, tid);
    let ghost tmg_view: TakeMutexGuardOutcomeView = tmg_result.spec_view();
    match tmg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::TakeMutexGuardError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                // Don't-care values: spec short-circuits on TmgError,
                // so subsequent step outcomes are never inspected. Use Error
                // variants so resource-release postconditions are not triggered.
                gc: GetCondOutcomeView::GcError { error_code: 0 },
                cw: CondWaitOutcomeView::CwOk,
                pc: PutCondOutcomeView::PcError { error_code: 0 },
                gm: GetMutexOutcomeView::GmError { error_code: 0 },
                lo: LockOutcomeView::LoGenericError { error_code: 0 },
                pg: PutGuardOutcomeView::PgError { error_code: 0 },
            };
            return (result, Ghost(gs));
        },
        TakeMutexGuardOutcomeModel::Ok => {},
    }

    // Steps 3+4: get_cond + optional cond.wait → stored result.
    // In the original code (lines 109-122), get_cond failure stores the error
    // and cond.wait is NOT called. The continuation pipeline runs regardless.
    let gcw_ret: (
        WaitCondResultModel,
        Ghost<GetCondOutcomeView>,
        Ghost<CondWaitOutcomeView>,
    ) = get_cond_and_wait_model(
        cond_addr,
        has_alarm,
        Ghost(timeout_s),
        Ghost(timeout_ns),
    );
    let stored: WaitCondResultModel = gcw_ret.0;
    let ghost gc_view: GetCondOutcomeView = gcw_ret.1@;
    let ghost cw_view: CondWaitOutcomeView = gcw_ret.2@;

    // Step 5: put_cond (runs unconditionally after get_cond+cond.wait block).
    let pc_result: PutCondOutcomeModel = put_cond_model(cond_addr);
    let ghost pc_view: PutCondOutcomeView = pc_result.spec_view();
    match pc_result {
        PutCondOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::PutCondError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                // Don't-care values: spec short-circuits on PcError. Use Error
                // variants for lo to avoid triggering spec_mutex_reacquired.
                gm: GetMutexOutcomeView::GmError { error_code: 0 },
                lo: LockOutcomeView::LoGenericError { error_code: 0 },
                pg: PutGuardOutcomeView::PgError { error_code: 0 },
            };
            return (result, Ghost(gs));
        },
        PutCondOutcomeModel::Ok => {},
    }

    // Step 6: get_mutex.
    let gm_result: GetMutexOutcomeModel = get_mutex_model(mutex_addr);
    let ghost gm_view: GetMutexOutcomeView = gm_result.spec_view();
    match gm_result {
        GetMutexOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::GetMutexError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                // Don't-care values: spec short-circuits on GmError. Use Error
                // variants for lo to avoid triggering spec_mutex_reacquired.
                lo: LockOutcomeView::LoGenericError { error_code: 0 },
                pg: PutGuardOutcomeView::PgError { error_code: 0 },
            };
            return (result, Ghost(gs));
        },
        GetMutexOutcomeModel::Ok => {},
    }

    // Step 7: mutex.lock(None) — reacquire with infinite wait.
    let lock_result: LockOutcomeModel = mutex_lock_model(mutex_addr);
    let ghost lo_view: LockOutcomeView = lock_result.spec_view();
    match lock_result {
        LockOutcomeModel::TimedOut => {
            // Unreachable: mutex_lock_model postcondition guarantees
            // !matches!(result, LockOutcomeModel::TimedOut).
            // Branch retained for exhaustive matching; postcondition vacuously true.
            let result: WaitCondResultModel = WaitCondResultModel::LockTimedOut;
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::Killed => {
            let result: WaitCondResultModel = WaitCondResultModel::LockKilled;
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::GenericError { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::LockGenericError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::Ok => {},
    }

    // Step 8: put_mutex_guard.
    let pg_result: PutGuardOutcomeModel = put_mutex_guard_model(mutex_addr);
    let ghost pg_view: PutGuardOutcomeView = pg_result.spec_view();
    match pg_result {
        PutGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::PutGuardError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: pg_view,
            };
            return (result, Ghost(gs));
        },
        PutGuardOutcomeModel::Ok => {},
    }

    // Step 9: All continuation steps succeeded. Return stored result.
    let ghost gs: WaitCondGhostState = WaitCondGhostState {
        tmg: tmg_view,
        gc: gc_view,
        cw: cw_view,
        pc: pc_view,
        gm: gm_view,
        lo: lo_view,
        pg: pg_view,
    };
    (stored, Ghost(gs))
}
