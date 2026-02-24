pub fn lock_mutex_model(mutex_addr: u32, timeout_s: u32, timeout_ns: u32) -> (ret: (
    LockMutexResultModel,
    Ghost<GetMutexOutcomeView>,
    Ghost<LockOutcomeView>,
    Ghost<PutGuardOutcomeView>,
    Ghost<Option<TimeoutView>>,
))
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        timeout_s as nat <= USIZE_MAX_X86_32(),
        timeout_ns as nat <= USIZE_MAX_X86_32(),
    ensures
        // The result matches the spec for the captured PM outcomes.
        ret.0.spec_view() == spec_lock_mutex_result(
            timeout_s as nat,
            timeout_ns as nat,
            ret.1@,
            ret.2@,
            ret.3@,
        ),
        // Invalid timeout → always InvalidTimeoutError.
        !spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat) ==>
            spec_is_timeout_error(ret.0.spec_view()),
        // Success only when all steps succeed.
        spec_is_success(ret.0.spec_view()) ==> (
            spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat)
            && ret.1@ == GetMutexOutcomeView::GmOk
            && ret.2@ == LockOutcomeView::LoOk
            && ret.3@ == PutGuardOutcomeView::PgOk
        ),
        // TimedOut impossible with infinite timeout (from mutex_lock_model contract).
        !spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat) ==>
            !matches!(ret.0, LockMutexResultModel::LockTimedOut),
        // Timeout value correctness: the parsed timeout is threaded to the lock step.
        // When the lock step is reached (timeout parsed OK, get_mutex OK), the
        // timeout value passed to mutex_lock_model matches spec_parsed_timeout_for_lock.
        ret.4@ == spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat),
        // Guard ownership release: when the pipeline reaches the put_guard step
        // (i.e., timeout valid, get_mutex OK, lock OK), the guard ownership for
        // mutex_addr is released regardless of whether put_guard succeeds or fails.
        // This follows from put_mutex_guard_model's postcondition.
        (ret.2@ == LockOutcomeView::LoOk && ret.1@ == GetMutexOutcomeView::GmOk
            && spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat))
            ==> spec_guard_ownership_released(mutex_addr as nat),
{
    // Step 1: Parse timeout.
    let parsed: Result<bool, LockMutexResultModel> = parse_timeout(timeout_s, timeout_ns);

    match parsed {
        Err(err) => {
            // Invalid timeout → return error.
            // Ghost PM outcomes are don't-care values: the spec
            // `spec_lock_mutex_result` ignores them on the timeout-error
            // path (the first `match` arm returns `InvalidTimeoutError`
            // regardless of get_mutex/lock/put_guard outcomes). Any
            // ghost values satisfy the postcondition vacuously.
            (err, Ghost(GetMutexOutcomeView::GmOk), Ghost(LockOutcomeView::LoOk), Ghost(PutGuardOutcomeView::PgOk), Ghost(spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat)))
        },
        Ok(has_timeout) => {
            // Compute ghost timeout value for the lock step.
            let ghost timeout_for_lock: Option<TimeoutView> = spec_parsed_timeout_for_lock(timeout_s as nat, timeout_ns as nat);

            // Step 2: Get mutex (external).
            let gm_result: GetMutexOutcomeModel = get_mutex_model(mutex_addr);
            let ghost gm_view: GetMutexOutcomeView = gm_result.spec_view();

            match gm_result {
                GetMutexOutcomeModel::Error { error_code } => {
                    // get_mutex failed → wrap in SleepError::Generic.
                    // Ghost lock/put_guard outcomes are don't-cares (pipeline short-circuited).
                    (
                        LockMutexResultModel::GetMutexError { error_code },
                        Ghost(gm_view),
                        Ghost(LockOutcomeView::LoOk),
                        Ghost(PutGuardOutcomeView::PgOk),
                        Ghost(timeout_for_lock),
                    )
                },
                GetMutexOutcomeModel::Ok => {
                    // Step 3: Lock mutex (external), threading the parsed timeout value.
                    let lock_pair: (LockOutcomeModel, Ghost<Option<u32>>) = mutex_lock_model(has_timeout, Ghost(timeout_for_lock), Ghost(mutex_addr));
                    let lock_result: LockOutcomeModel = lock_pair.0;
                    let guard_token: Ghost<Option<u32>> = lock_pair.1;
                    let ghost lo_view: LockOutcomeView = lock_result.spec_view();

                    match lock_result {
                        LockOutcomeModel::TimedOut => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockTimedOut,
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::Killed => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockKilled,
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::GenericError { error_code } => {
                            // Ghost put_guard outcome is don't-care (pipeline short-circuited).
                            (
                                LockMutexResultModel::LockGenericError { error_code },
                                Ghost(gm_view),
                                Ghost(lo_view),
                                Ghost(PutGuardOutcomeView::PgOk),
                                Ghost(timeout_for_lock),
                            )
                        },
                        LockOutcomeModel::Ok => {
                            // Step 4: Put mutex guard (external), passing the guard token.
                            let pg_result: PutGuardOutcomeModel = put_mutex_guard_model(mutex_addr, guard_token);
                            let ghost pg_view: PutGuardOutcomeView = pg_result.spec_view();

                            match pg_result {
                                PutGuardOutcomeModel::Error { error_code } => {
                                    (
                                        LockMutexResultModel::PutGuardError { error_code },
                                        Ghost(gm_view),
                                        Ghost(lo_view),
                                        Ghost(pg_view),
                                        Ghost(timeout_for_lock),
                                    )
                                },
                                PutGuardOutcomeModel::Ok => {
                                    (
                                        LockMutexResultModel::Success,
                                        Ghost(gm_view),
                                        Ghost(lo_view),
                                        Ghost(pg_view),
                                        Ghost(timeout_for_lock),
                                    )
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}
