pub fn get_cond_and_wait_model(
    cond_addr: u32,
    has_alarm: bool,
    timeout_s: Ghost<u32>,
    timeout_ns: Ghost<u32>,
) -> (ret: (
    WaitCondResultModel,
    Ghost<GetCondOutcomeView>,
    Ghost<CondWaitOutcomeView>,
))
    requires
        has_alarm ==> spec_is_finite_timeout(timeout_s@ as nat, timeout_ns@ as nat),
    ensures
        ret.0.spec_view() == spec_stored_result(ret.1@, ret.2@),
        ret.1@ matches GetCondOutcomeView::GcError { error_code }
            ==> spec_is_valid_error_code(error_code),
        ret.2@ matches CondWaitOutcomeView::CwTimedOut ==> has_alarm,
{
    let gc_result: GetCondOutcomeModel = get_cond_model(cond_addr);
    let ghost gc_view: GetCondOutcomeView = gc_result.spec_view();

    match gc_result {
        GetCondOutcomeModel::Error { error_code } => {
            // get_cond failed: store error, cond.wait is never called.
            // Don't-care value: spec_stored_result ignores cw when gc fails,
            // proven by lemma_get_cond_fail_ignores_cond_wait.
            let ghost cw_view: CondWaitOutcomeView = CondWaitOutcomeView::CwOk;
            (
                WaitCondResultModel::GetCondError { error_code },
                Ghost(gc_view),
                Ghost(cw_view),
            )
        },
        GetCondOutcomeModel::Ok => {
            // get_cond succeeded: call cond.wait and store its result.
            let cw_result: CondWaitOutcomeModel =
                cond_wait_model(has_alarm, timeout_s, timeout_ns);
            let ghost cw_view: CondWaitOutcomeView = cw_result.spec_view();
            let stored: WaitCondResultModel = match cw_result {
                CondWaitOutcomeModel::Ok => WaitCondResultModel::Success,
                CondWaitOutcomeModel::TimedOut => WaitCondResultModel::CondWaitTimedOut,
                CondWaitOutcomeModel::Killed => WaitCondResultModel::CondWaitKilled,
                CondWaitOutcomeModel::GenericError { error_code } => {
                    WaitCondResultModel::CondWaitGenericError { error_code }
                },
            };
            (stored, Ghost(gc_view), Ghost(cw_view))
        },
    }
}

/// Verified exec model of the `wait_cond` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn wait_cond(...)` control flow:
/// 1. Parse timeout.
/// 2. take_mutex_guard → release mutex (short-circuits on error).
/// 3. get_cond + cond.wait → compute "stored result". If get_cond fails, the
///    error is stored and cond.wait is NOT called. NO short-circuit.
/// 4. put_cond → continuation pipeline (overrides stored result on error).
/// 5. get_mutex → continuation pipeline.
/// 6. mutex.lock(None) → continuation pipeline.
/// 7. put_mutex_guard → continuation pipeline.
/// 8. Return stored result from step 3.
///
/// **Critical semantic detail**: Steps 4-7 execute regardless of step 3's outcome.
/// Errors in steps 4-7 override the stored result via the `?` operator. The stored
/// result is returned only when all continuation steps succeed.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address.
/// - `mutex_addr`: Mutex address.
/// - `timeout_s`: Timeout seconds (u32 on x86-32).
/// - `timeout_ns`: Timeout nanoseconds (u32 on x86-32).
/// - `pid`: Ghost process identifier (matches original's pid parameter).
/// - `tid`: Ghost thread identifier (matches original's tid parameter).
///
/// # Returns
///
/// A tuple of the exec result model and a ghost `WaitCondGhostState` capturing
/// all step outcomes for exec-spec equivalence.
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

/// Wrapper matching the original `pub unsafe fn wait_cond(...)` signature.
///
/// # Description
///
/// Thin wrapper that delegates to `wait_cond_model` and extracts the exec result,
/// following the established pattern (see `lock_mutex`, `unlock_mutex`).
/// The `pid` and `tid` parameters are forwarded as ghost values because they affect
/// only trace logging and PM-internal ownership validation in the original, not the
/// pipeline control flow.
///
/// # Parameters
///
/// - `pid`: Process identifier (forwarded as ghost to wait_cond_model).
/// - `tid`: Thread identifier (forwarded as ghost to wait_cond_model).
/// - `cond_addr`: Condition variable address.
/// - `mutex_addr`: Mutex address.
/// - `timeout_s`: Timeout seconds (u32 on x86-32).
/// - `timeout_ns`: Timeout nanoseconds (u32 on x86-32).
pub fn wait_cond(
    pid: u32,
    tid: u32,
    cond_addr: u32,
    mutex_addr: u32,
    timeout_s: u32,
    timeout_ns: u32,
) -> (result: WaitCondResultModel)
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        USIZE_BITS() == 32,
        // Safety: caller must satisfy the wait_cond safety preconditions.
        spec_wait_cond_safety_preconditions(pid as nat, tid as nat),
        // The supplied pid/tid must be the currently-running thread.
        spec_is_currently_running(pid as nat, tid as nat),
    ensures
        // Mutex protocol: on stored-result return, the mutex was released
        // before the wait and reacquired afterward.
        spec_is_stored_result_return(result.spec_view()) ==> (
            spec_mutex_released(mutex_addr as nat)
            && spec_cond_ref_released(cond_addr as nat)
            && spec_mutex_reacquired(mutex_addr as nat)
        ),
{
    // pid and tid are forwarded as ghost — they affect only trace logging
    // and PM-internal ownership validation, not the pipeline control flow.
    let ret: (WaitCondResultModel, Ghost<WaitCondGhostState>) =
        wait_cond_model(cond_addr, mutex_addr, timeout_s, timeout_ns, Ghost(pid), Ghost(tid));
    ret.0
}

} // verus!
