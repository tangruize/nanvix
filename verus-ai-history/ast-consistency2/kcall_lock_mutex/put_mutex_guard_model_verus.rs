pub fn put_mutex_guard_model(mutex_addr: u32, guard_token: Ghost<Option<u32>>) -> (result: PutGuardOutcomeModel)
    requires
        // The caller must hold a valid guard token for this specific mutex.
        guard_token@ == Some(mutex_addr),
    ensures
        // Error codes from the PM module are valid ErrorCode discriminants.
        result matches PutGuardOutcomeModel::Error { error_code } ==> spec_is_valid_error_code(error_code as int),
        // Guard ownership is always released, whether put_mutex_guard succeeds
        // or fails. On success the guard is stored; on failure MutexGuard::drop()
        // unlocks the mutex. The concrete interpretation of this predicate is
        // provided by the mutex/PM module.
        spec_guard_ownership_released(mutex_addr as nat),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Trust Boundary T1: Models `SystemTime::new(seconds, nanoseconds)`.
///
/// # Description
///
/// Creates a SystemTime if nanoseconds < 1_000_000_000, returns None otherwise.
/// This is a verified function (not external_body) because the logic is simple
/// and well-specified.
///
/// # Parameters
///
/// - `seconds`: Seconds component (from usize cast to u64).
/// - `nanoseconds`: Nanoseconds component (from usize cast to u32).
///
/// # Returns
///
/// Some(SystemTimeModel) if valid, None if nanoseconds >= 1_000_000_000.
pub fn system_time_new(seconds: u64, nanoseconds: u32) -> (result: Option<SystemTimeModel>)
    ensures
        nanoseconds < 1_000_000_000u32 ==> result.is_some(),
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().seconds == seconds,
        nanoseconds < 1_000_000_000u32 ==> result.unwrap().nanoseconds == nanoseconds,
        nanoseconds >= 1_000_000_000u32 ==> result.is_none(),
{
    if nanoseconds >= 1_000_000_000u32 {
        None
    } else {
        Some(SystemTimeModel { seconds, nanoseconds })
    }
}

/// Verified model of timeout parsing.
///
/// # Description
///
/// Implements the original's timeout parsing logic:
/// ```ignore
/// let timeout = if timeout_s == usize::MAX && timeout_ns == usize::MAX {
///     None
/// } else {
///     match SystemTime::new(timeout_s as u64, timeout_ns as u32) {
///         Some(t) => Some(t),
///         None => return Err(SleepError::Generic(Error::new(InvalidArgument, ...)))
///     }
/// };
/// ```
///
/// Returns Ok(has_timeout) where has_timeout indicates whether a finite timeout
/// was parsed, or Err with the error model for invalid timeout.
///
/// # Parameters
///
/// - `timeout_s`: Timeout seconds (from usize, 32-bit on x86).
/// - `timeout_ns`: Timeout nanoseconds (from usize, 32-bit on x86).
///
/// # Returns
///
/// Ok(bool): true if finite timeout, false if infinite.
/// Err(LockMutexResultModel): InvalidTimeoutError on failure.
pub fn parse_timeout(timeout_s: u32, timeout_ns: u32) -> (result: Result<bool, LockMutexResultModel>)
    ensures
        // Infinite timeout case.
        spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            ==> result == Ok::<bool, LockMutexResultModel>(false),
        // Valid finite timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && spec_timeout_ns_valid(timeout_ns as nat))
            ==> result == Ok::<bool, LockMutexResultModel>(true),
        // Invalid timeout case.
        (!spec_is_infinite_timeout(timeout_s as nat, timeout_ns as nat)
            && !spec_timeout_ns_valid(timeout_ns as nat))
            ==> result.is_err(),
        // Error case produces correct error code.
        result matches Err(err) ==> err.spec_view() == (LockMutexResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        }),
        // Result matches spec_parse_timeout.
        result.is_ok() <==> spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        // Both MAX → infinite timeout (None in original).
        Ok(false)
    } else {
        // Try to construct SystemTime.
        let st: Option<SystemTimeModel> = system_time_new(timeout_s as u64, timeout_ns);
        match st {
            Some(_) => Ok(true),
            None => {
                proof {
                    assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
                }
                Err(LockMutexResultModel::InvalidTimeoutError { error_code: 22i32 })
            },
        }
    }
}

/// Verified exec model of the `lock_mutex` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn lock_mutex(...)` control
/// flow. It:
/// 1. Parses timeout from (timeout_s, timeout_ns).
/// 2. Calls get_mutex (external).
/// 3. Calls lock (external).
/// 4. Calls put_mutex_guard (external).
///
/// The postconditions prove that the result matches `spec_lock_mutex_result`
/// for all inputs and all PM outcomes.
///
/// The original also takes `pid` and `tid` parameters, which are used only
/// in the `trace!()` diagnostic macro and are omitted from this model.
///
/// # Parameters
///
/// - `mutex_addr`: Mutex address (from `MutexAddress::from(usize)`).
/// - `timeout_s`: Timeout seconds (from usize, 32-bit on x86).
/// - `timeout_ns`: Timeout nanoseconds (from usize, 32-bit on x86).
///
/// # Returns
///
/// A tuple of (result, ghost get_mutex_outcome, ghost lock_outcome, ghost put_guard_outcome)
/// where the ghosts capture the PM outcomes for postcondition linking.
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

/// Verified wrapper matching the original `lock_mutex` function signature.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn lock_mutex(pid, tid,
/// mutex_addr, timeout_s, timeout_ns) -> Result<(), SleepError>` function
/// signature, adapted for verification model types. The `pid` and `tid`
/// parameters are included for signature fidelity even though they are used
/// only in `trace!()` logging in the original and do not affect control flow.
///
/// The implementation delegates to `lock_mutex_model`, which performs the
/// actual verified pipeline logic. The result is independent of `pid`/`tid`
/// (proven by `lemma_result_independent_of_pid_tid`).
///
/// # Parameters
///
/// - `pid`: Process identifier (used only in trace logging in original).
/// - `tid`: Thread identifier (used only in trace logging in original).
/// - `mutex_addr`: Mutex address (32-bit on x86).
/// - `timeout_s`: Timeout seconds (32-bit on x86).
/// - `timeout_ns`: Timeout nanoseconds (32-bit on x86).
///
/// # Returns
///
/// The result of the lock_mutex pipeline as a `LockMutexResultModel`.
pub fn lock_mutex(pid: u32, tid: u32, mutex_addr: u32, timeout_s: u32, timeout_ns: u32) -> (result: LockMutexResultModel)
    requires
        // ABI constraint: inputs originate from 32-bit usize on x86-32.
        timeout_s as nat <= USIZE_MAX_X86_32(),
        timeout_ns as nat <= USIZE_MAX_X86_32(),
    ensures
        // Invalid timeout always produces InvalidTimeoutError.
        !spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat) ==>
            spec_is_timeout_error(result.spec_view()),
        // TimedOut impossible with infinite timeout.
        !spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat) ==>
            !matches!(result, LockMutexResultModel::LockTimedOut),
{
    // pid and tid are intentionally unused — they affect only trace logging
    // in the original function. See lemma_result_independent_of_pid_tid.
    let ret: (
        LockMutexResultModel,
        Ghost<GetMutexOutcomeView>,
        Ghost<LockOutcomeView>,
        Ghost<PutGuardOutcomeView>,
        Ghost<Option<TimeoutView>>,
    ) = lock_mutex_model(mutex_addr, timeout_s, timeout_ns);
    ret.0
}

} // verus!
