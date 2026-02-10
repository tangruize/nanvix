// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Signal Condition Kernel Call Proofs.
// Proof lemmas for the signal_cond kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
//==================================================================================================

/// Proof: get_cond error propagates as the final result.
///
/// # Description
///
/// If `ProcessManager::get_cond` fails, the error is propagated unchanged
/// via the `?` operator and no further steps execute.
pub proof fn lemma_get_cond_error_propagates(
    error_code: int,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcError { error_code },
            notify_outcome,
            put_cond_outcome,
        ) == (SignalCondResultView::GetCondError { error_code }),
        spec_is_get_cond_error(
            spec_signal_cond_result(
                GetCondOutcomeView::GcError { error_code },
                notify_outcome,
                put_cond_outcome,
            )
        ),
{
}

/// Proof: notify error propagates as the final result.
///
/// # Description
///
/// If `notify_all` or `notify_first` fails, the error is propagated
/// unchanged via the `?` operator. `put_cond` is not called.
pub proof fn lemma_notify_error_propagates(
    error_code: int,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NError { error_code },
            put_cond_outcome,
        ) == (SignalCondResultView::NotifyError { error_code }),
        spec_is_notify_error(
            spec_signal_cond_result(
                GetCondOutcomeView::GcOk,
                NotifyOutcomeView::NError { error_code },
                put_cond_outcome,
            )
        ),
{
}

/// Proof: put_cond error propagates as the final result.
///
/// # Description
///
/// If `ProcessManager::put_cond` fails after successful get_cond and notify,
/// the error is propagated unchanged via the `?` operator.
pub proof fn lemma_put_cond_error_propagates(
    awakened: nat,
    error_code: int,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NOk { awakened },
            PutCondOutcomeView::PcError { error_code },
        ) == (SignalCondResultView::PutCondError { error_code }),
        spec_is_put_cond_error(
            spec_signal_cond_result(
                GetCondOutcomeView::GcOk,
                NotifyOutcomeView::NOk { awakened },
                PutCondOutcomeView::PcError { error_code },
            )
        ),
{
}

/// Proof: success requires all three steps to succeed.
///
/// # Description
///
/// The result is Success if and only if get_cond, notify, and put_cond
/// all succeeded.
pub proof fn lemma_success_requires_all_steps_ok(
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        spec_is_success(
            spec_signal_cond_result(get_cond_outcome, notify_outcome, put_cond_outcome)
        ) <==> (
            matches!(get_cond_outcome, GetCondOutcomeView::GcOk)
            && matches!(notify_outcome, NotifyOutcomeView::NOk { .. })
            && matches!(put_cond_outcome, PutCondOutcomeView::PcOk)
        ),
{
    match get_cond_outcome {
        GetCondOutcomeView::GcOk => {
            match notify_outcome {
                NotifyOutcomeView::NOk { .. } => {
                    match put_cond_outcome {
                        PutCondOutcomeView::PcOk => {},
                        PutCondOutcomeView::PcError { .. } => {},
                    }
                },
                NotifyOutcomeView::NError { .. } => {},
            }
        },
        GetCondOutcomeView::GcError { .. } => {},
    }
}

/// Proof: the result is always exactly one of the four categories.
///
/// # Description
///
/// The signal_cond result is exhaustive: every possible combination of
/// step outcomes produces exactly one result category.
pub proof fn lemma_result_exhaustive(
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        ({
            let result: SignalCondResultView = spec_signal_cond_result(
                get_cond_outcome, notify_outcome, put_cond_outcome,
            );
            // Exhaustive: every result is one of the four categories.
            spec_is_success(result)
            || spec_is_get_cond_error(result)
            || spec_is_notify_error(result)
            || spec_is_put_cond_error(result)
        }),
        ({
            let result: SignalCondResultView = spec_signal_cond_result(
                get_cond_outcome, notify_outcome, put_cond_outcome,
            );
            // Mutual exclusion: success and error are disjoint.
            !(spec_is_success(result) && spec_is_error(result))
        }),
{
    match get_cond_outcome {
        GetCondOutcomeView::GcOk => {
            match notify_outcome {
                NotifyOutcomeView::NOk { .. } => {
                    match put_cond_outcome {
                        PutCondOutcomeView::PcOk => {},
                        PutCondOutcomeView::PcError { .. } => {},
                    }
                },
                NotifyOutcomeView::NError { .. } => {},
            }
        },
        GetCondOutcomeView::GcError { .. } => {},
    }
}

/// Proof: success and error are complementary and exhaustive.
///
/// # Description
///
/// For any combination of step outcomes, the result is either success or
/// error, and these two categories are mutually exclusive and jointly exhaustive.
pub proof fn lemma_success_error_complementary(
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        ({
            let result: SignalCondResultView = spec_signal_cond_result(
                get_cond_outcome, notify_outcome, put_cond_outcome,
            );
            spec_is_success(result) <==> !spec_is_error(result)
        }),
{
    match get_cond_outcome {
        GetCondOutcomeView::GcOk => {
            match notify_outcome {
                NotifyOutcomeView::NOk { .. } => {
                    match put_cond_outcome {
                        PutCondOutcomeView::PcOk => {},
                        PutCondOutcomeView::PcError { .. } => {},
                    }
                },
                NotifyOutcomeView::NError { .. } => {},
            }
        },
        GetCondOutcomeView::GcError { .. } => {},
    }
}

/// Proof: error codes are preserved through the pipeline.
///
/// # Description
///
/// When any step fails with error code `ec`, the final result contains
/// exactly the same error code — no wrapping or transformation.
pub proof fn lemma_error_code_preserved_get_cond(error_code: int)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcError { error_code },
            // Don't-care values for unreached steps.
            NotifyOutcomeView::NOk { awakened: 0 },
            PutCondOutcomeView::PcOk,
        ) == (SignalCondResultView::GetCondError { error_code }),
{
}

/// Proof: error code preserved for notify errors.
pub proof fn lemma_error_code_preserved_notify(error_code: int)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NError { error_code },
            PutCondOutcomeView::PcOk,
        ) == (SignalCondResultView::NotifyError { error_code }),
{
}

/// Proof: error code preserved for put_cond errors.
pub proof fn lemma_error_code_preserved_put_cond(error_code: int, awakened: nat)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NOk { awakened },
            PutCondOutcomeView::PcError { error_code },
        ) == (SignalCondResultView::PutCondError { error_code }),
{
}

/// Proof: the pipeline mapping is independent of pid, tid, and broadcast.
///
/// # Description
///
/// Although `pid`, `tid`, and `broadcast` are passed to the original function,
/// the pipeline's mapping from step outcomes to the final result is the same
/// regardless of these parameters. `broadcast` selects which notify call is
/// made but both produce the same `NotifyOutcomeView`.
pub proof fn lemma_result_mapping_independent_of_context(
    pid1: nat,
    pid2: nat,
    tid1: nat,
    tid2: nat,
    broadcast1: bool,
    broadcast2: bool,
    get_cond_outcome: GetCondOutcomeView,
    notify_outcome: NotifyOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
)
    ensures
        spec_signal_cond_result_with_context(
            pid1, tid1, broadcast1, get_cond_outcome, notify_outcome, put_cond_outcome,
        ) == spec_signal_cond_result_with_context(
            pid2, tid2, broadcast2, get_cond_outcome, notify_outcome, put_cond_outcome,
        ),
{
}

/// Proof: architecture guard — USIZE_BITS is 32 and USIZE_MAX matches u32::MAX.
pub proof fn lemma_architecture_guard()
    ensures
        USIZE_BITS() == 32,
        USIZE_MAX_X86_32() == u32::MAX as nat,
        USIZE_MAX_X86_32() == 4294967295nat,
        USIZE_MAX_X86_32() + 1 == 4294967296nat,
{
}

/// Proof: the safety preconditions predicate is well-formed.
pub proof fn lemma_safety_preconditions_well_formed()
    ensures
        spec_signal_cond_safety_preconditions() ==> spec_caller_no_pm_reference(),
{
}

/// Proof: on success, the awakened count equals the notify outcome count.
///
/// # Description
///
/// When the pipeline succeeds, the returned awakened count matches the
/// value from the notify step.
pub proof fn lemma_success_awakened_count(
    awakened: nat,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NOk { awakened },
            PutCondOutcomeView::PcOk,
        ) == (SignalCondResultView::Success { awakened }),
{
}

/// Proof: short-circuit ordering — get_cond errors prevent all later steps.
///
/// # Description
///
/// If get_cond fails, neither notify nor put_cond results affect the outcome.
/// This proves the error-priority ordering of the pipeline.
pub proof fn lemma_short_circuit_get_cond(
    error_code: int,
    notify1: NotifyOutcomeView,
    notify2: NotifyOutcomeView,
    pc1: PutCondOutcomeView,
    pc2: PutCondOutcomeView,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcError { error_code }, notify1, pc1,
        ) == spec_signal_cond_result(
            GetCondOutcomeView::GcError { error_code }, notify2, pc2,
        ),
{
}

/// Proof: short-circuit ordering — notify errors prevent put_cond from affecting result.
pub proof fn lemma_short_circuit_notify(
    error_code: int,
    pc1: PutCondOutcomeView,
    pc2: PutCondOutcomeView,
)
    ensures
        spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NError { error_code },
            pc1,
        ) == spec_signal_cond_result(
            GetCondOutcomeView::GcOk,
            NotifyOutcomeView::NError { error_code },
            pc2,
        ),
{
}

} // verus!
