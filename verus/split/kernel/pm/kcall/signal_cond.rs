// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Signal Condition Kernel Call Verification Model
//!
//! Formal verification of the signal_cond kernel call (`pm::kcall::signal_cond`).
//!
//! ## Overview
//!
//! The `signal_cond(pid, tid, cond_addr, broadcast)` function signals a
//! condition variable, waking either one or all waiting threads. It:
//! 1. Converts `cond_addr` to a `ConditionAddress` (type wrapper, not modeled).
//! 2. Calls `ProcessManager::get_cond(cond_addr)` to obtain a Condvar reference.
//! 3. If `broadcast`, calls `cond.notify_all()`; else calls `cond.notify_first()`.
//!    Both return `Result<u32, Error>` with the awakened thread count on success.
//! 4. The `Condvar` is dropped at end of block (reference count decreases).
//! 5. Calls `ProcessManager::put_cond(cond_addr)` to release the condition
//!    variable slot.
//! 6. Returns `Ok(awakened)` on success.
//!
//! All steps use `?` for short-circuit error propagation.
//!
//! ## Note on Parameters
//!
//! The original function takes `pid: ProcessIdentifier`, `tid: ThreadIdentifier`,
//! `cond_addr: usize`, and `broadcast: bool`. The `pid` and `tid` are used only
//! for trace logging. The `broadcast` flag selects between `notify_all` and
//! `notify_first`, but both produce the same `NotifyOutcomeView` type.
//! Ghost `pid` and `tid` parameters are included for documentation.
//!
//! ## Verified Properties
//!
//! - **Error propagation**: Each step's error propagates unchanged
//!   (`lemma_get_cond_error_propagates`, `lemma_notify_error_propagates`,
//!   `lemma_put_cond_error_propagates`, `lemma_error_code_preserved_*`).
//! - **Success requires all steps OK**: Success iff get_cond, notify, and
//!   put_cond all succeed (`lemma_success_requires_all_steps_ok`).
//! - **Result exhaustiveness**: Every input produces exactly one result category
//!   (`lemma_result_exhaustive`).
//! - **Short-circuit ordering**: Earlier errors mask later step outcomes
//!   (`lemma_short_circuit_get_cond`, `lemma_short_circuit_notify`).
//! - **Awakened count preservation**: On success, the returned count matches
//!   the notify outcome (`lemma_success_awakened_count`).
//! - **Broadcast semantics**: On success, the awakened count respects the
//!   `broadcast` flag: `notify_first` (`!broadcast`) awakens at most 1 thread
//!   (`lemma_notify_first_awakens_at_most_one`); `notify_all` (`broadcast`)
//!   uses best-effort wakeup bounded by the number of waiters
//!   (`lemma_notify_all_bounded_by_waiters`). This is propagated from the
//!   T2 trust boundary to the pipeline result
//!   (`lemma_broadcast_semantics_preserved`).
//! - **Condvar drop on get_cond success**: When get_cond succeeds, the condvar
//!   reference is always released (dropped at scope exit), regardless of
//!   whether notify succeeds or fails. This is exposed as a postcondition
//!   on `signal_cond_model` and proven by `lemma_cond_ref_released_on_get_cond_success`.
//! - **Notify error skips put_cond**: When notify fails, `put_cond` is not
//!   called and the condvar slot is not returned to the PM. This faithfully
//!   mirrors the original code's short-circuit behavior
//!   (`lemma_notify_error_skips_put_cond`). See "Known Limitations" below.
//! - **Pipeline mapping independence**: The pipeline mapping is independent
//!   of pid, tid, and broadcast (`lemma_result_mapping_independent_of_context`).
//! - **Architecture guard**: x86-32 assumption verified
//!   (`lemma_architecture_guard`).
//! - **Safety precondition enforcement**: The `unsafe` safety contract is
//!   enforced as a `requires` clause (`lemma_safety_preconditions_well_formed`).
//! - **Exec model correctness**: `signal_cond_model` matches
//!   `spec_signal_cond_result` for all inputs and PM outcomes.
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - **Condvar notify correctness**: The internal state machine of
//!   Condvar::notify_all() / Condvar::notify_first() is verified in the
//!   condvar module.
//! - **ProcessManager correctness**: get_cond / put_cond internals are
//!   verified in the PM module. Resource-release predicates
//!   (`spec_cond_ref_released`, `spec_put_cond_completed`) are intentionally
//!   uninterpreted at this trust boundary. Their concrete semantics (e.g.,
//!   refcount decrement, slot ownership transfer) are the responsibility of
//!   the PM and condvar modules respectively. At this kcall level, these
//!   predicates serve as abstract postcondition tokens: the external_body
//!   functions establish them, and the pipeline's ensures clauses propagate
//!   them to callers. No claim is made here about their concrete
//!   interpretation — that is a concern of the modules behind trust
//!   boundaries T1–T4. Introducing PM/condvar state invariants here would
//!   break the modular trust boundary separation that allows each module
//!   to be verified independently.
//! - **Liveness**: Whether waiting threads actually wake up is a scheduler
//!   concern, not provable at the kcall pipeline level. This verification
//!   assumes (but does not prove) that the condvar and scheduler modules
//!   correctly implement wakeup semantics. The assumption is: if
//!   `notify_model` returns `Ok { awakened: n }`, then exactly `n` threads
//!   have been moved to a runnable state. Liveness guarantees (that these
//!   threads eventually execute) depend on the scheduler's fairness
//!   properties, which are outside this module's scope.
//!
//! ## Known Limitations
//!
//! - **Condvar slot not returned on notify failure**: When notify_all or
//!   notify_first fails, the original code short-circuits via `?` and
//!   `ProcessManager::put_cond()` is never called. The condvar reference
//!   IS released (via Condvar::drop at scope exit), but the PM condvar
//!   slot is not explicitly returned. Whether this constitutes a resource
//!   leak depends on the PM's cleanup semantics (e.g., process exit cleanup
//!   or Condvar::drop internally calling put_cond). This behavior is
//!   intentionally mirrored in the verification model. Proven explicitly
//!   by `lemma_notify_error_skips_put_cond`. Resolving this would require
//!   either changing the original code to call put_cond on notify error,
//!   or proving a PM-level invariant that condvar slots are eventually
//!   reclaimed through other mechanisms.
//! - **Resource-release predicates are abstract tokens**: The predicates
//!   `spec_cond_ref_released` and `spec_put_cond_completed` are
//!   uninterpreted at this kcall level. They serve as composable
//!   postcondition tokens that callers can use to chain resource-release
//!   reasoning, but this module does not prove their concrete effects
//!   (e.g., refcount values, slot availability). Concrete proofs require
//!   the condvar and PM modules behind trust boundaries T3/T4.
//!
//! ## Verification Model
//!
//! The original function uses:
//! - `ConditionAddress::from(usize)` → modeled as opaque type construction.
//! - `ProcessManager::get_cond()` → modeled via `get_cond_model()` external_body.
//! - `Condvar::notify_all()` / `Condvar::notify_first()` → modeled via
//!   `notify_model()` external_body.
//! - `Condvar::drop()` → modeled via `drop_cond_model()` external_body.
//! - `ProcessManager::put_cond()` → modeled via `put_cond_model()` external_body.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessManager::get_cond()`**. Returns a Condvar reference.
//! - **T2: `Condvar::notify_all()` / `Condvar::notify_first()`**. Wakes threads.
//! - **T3: `Condvar::drop()`**. Decrements reference count.
//! - **T4: `ProcessManager::put_cond()`**. Releases condition variable slot.
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                  | Notes          |
//! |-------------------------------------------|---------------------------------|----------------|
//! | `ConditionAddress::from(usize)`           | (not modeled)                   | Type wrapper.  |
//! | `ProcessManager::get_cond(cond_addr)`     | `get_cond_model(cond_addr)`     | external_body. |
//! | `cond.notify_all()` / `cond.notify_first()` | `notify_model(cond_addr, broadcast)` | external_body. |
//! | `Condvar::drop()`                         | `drop_cond_model(cond_addr)`    | external_body. |
//! | `ProcessManager::put_cond(cond_addr)`     | `put_cond_model(cond_addr)`     | external_body. |
//! | `pub unsafe fn signal_cond(...)`          | `signal_cond_model(...)`        | Fully verified.|

use vstd::prelude::*;

// Include specifications.
include!("signal_cond.spec.rs");

// Include proofs.
include!("signal_cond.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the GetCond step outcome.
///
/// # Description
///
/// Represents the result of `ProcessManager::get_cond(cond_addr)`.
/// On success, a Condvar reference is obtained. On failure, an Error is returned.
pub enum GetCondOutcomeModel {
    /// get_cond succeeded, returning a Condvar reference.
    Ok,
    /// get_cond failed with an error code.
    Error { error_code: i32 },
}

impl GetCondOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> GetCondOutcomeView {
        match self {
            GetCondOutcomeModel::Ok => GetCondOutcomeView::GcOk,
            GetCondOutcomeModel::Error { error_code } => {
                GetCondOutcomeView::GcError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the Notify step outcome.
///
/// # Description
///
/// Represents the result of `cond.notify_all()` or `cond.notify_first()`.
/// On success, the number of awakened threads is returned.
/// On failure, an Error is returned.
pub enum NotifyOutcomeModel {
    /// notify succeeded, awakening `awakened` threads.
    Ok { awakened: u32 },
    /// notify failed with an error code.
    Error { error_code: i32 },
}

impl NotifyOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> NotifyOutcomeView {
        match self {
            NotifyOutcomeModel::Ok { awakened } => {
                NotifyOutcomeView::NOk { awakened: *awakened as nat }
            },
            NotifyOutcomeModel::Error { error_code } => {
                NotifyOutcomeView::NError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the PutCond step outcome.
///
/// # Description
///
/// Represents the result of `ProcessManager::put_cond(cond_addr)`.
pub enum PutCondOutcomeModel {
    /// put_cond succeeded.
    Ok,
    /// put_cond failed with an error code.
    Error { error_code: i32 },
}

impl PutCondOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> PutCondOutcomeView {
        match self {
            PutCondOutcomeModel::Ok => PutCondOutcomeView::PcOk,
            PutCondOutcomeModel::Error { error_code } => {
                PutCondOutcomeView::PcError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the signal_cond overall result.
///
/// # Description
///
/// Represents the final result of the signal_cond kcall, mirroring
/// `Result<u32, Error>` from the original.
pub enum SignalCondResultModel {
    /// All steps succeeded; `awakened` threads were woken.
    Success { awakened: u32 },
    /// ProcessManager::get_cond failed.
    GetCondError { error_code: i32 },
    /// notify_all or notify_first failed.
    NotifyError { error_code: i32 },
    /// ProcessManager::put_cond failed.
    PutCondError { error_code: i32 },
}

impl SignalCondResultModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> SignalCondResultView {
        match self {
            SignalCondResultModel::Success { awakened } => {
                SignalCondResultView::Success { awakened: *awakened as nat }
            },
            SignalCondResultModel::GetCondError { error_code } => {
                SignalCondResultView::GetCondError { error_code: *error_code as int }
            },
            SignalCondResultModel::NotifyError { error_code } => {
                SignalCondResultView::NotifyError { error_code: *error_code as int }
            },
            SignalCondResultModel::PutCondError { error_code } => {
                SignalCondResultView::PutCondError { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `ProcessManager::get_cond(cond_addr)`.
///
/// # Description
///
/// Retrieves a Condvar reference for the given condition variable address.
/// On success, a Condvar is returned (modeled as the get_cond outcome).
/// On failure, an Error is returned.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address.
#[verifier::external_body]
pub fn get_cond_model(cond_addr: u32) -> (result: GetCondOutcomeModel)
    requires
        spec_signal_cond_safety_preconditions(),
    ensures
        result matches GetCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        // On success, a valid Condvar reference has been acquired.
        result matches GetCondOutcomeModel::Ok
            ==> spec_condvar_acquired(cond_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T2: Models `cond.notify_all()` or `cond.notify_first()`.
///
/// # Description
///
/// Notifies threads waiting on the condition variable. If `broadcast` is true,
/// models `cond.notify_all()` (best-effort wakeup of all waiters); otherwise
/// models `cond.notify_first()` (awakens at most one waiter).
/// Returns the number of awakened threads on success, or an error.
///
/// The broadcast-dependent postcondition captures the semantic difference:
/// `notify_first` awakens at most 1 thread, while `notify_all` uses
/// best-effort wakeup bounded by the number of waiters.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address.
/// - `broadcast`: Whether to wake all waiting threads.
#[verifier::external_body]
pub fn notify_model(cond_addr: u32, broadcast: bool) -> (result: NotifyOutcomeModel)
    requires
        // A valid Condvar reference must have been acquired via get_cond.
        spec_condvar_acquired(cond_addr as nat),
    ensures
        result matches NotifyOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        // Broadcast semantics: on success, the awakened count respects the
        // broadcast flag. notify_first awakens at most 1; notify_all is
        // bounded by the number of waiters (best-effort).
        result matches NotifyOutcomeModel::Ok { awakened }
            ==> spec_broadcast_semantics(broadcast, cond_addr as nat, awakened as nat),
{
    unimplemented!()
}

/// Trust Boundary T3: Models `Condvar::drop()`.
///
/// # Description
///
/// The Condvar goes out of scope, causing its reference count to decrease.
/// This always succeeds (Rust drop cannot fail).
///
/// **Safety note on CondvarInner::drop panic**: The real `CondvarInner::drop`
/// implementation panics if threads are still sleeping on the condition
/// variable. However, the `Condvar` type is an `Arc<CondvarInner>` clone
/// obtained from `get_cond`. Dropping this clone decrements the Arc
/// reference count but does NOT invoke `CondvarInner::drop` unless this
/// is the last Arc clone — which it cannot be, since the PM retains its
/// own reference (returned via `put_cond`). Therefore, the inner drop
/// panic path is unreachable at this call site.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address whose ref is being released.
#[verifier::external_body]
pub fn drop_cond_model(cond_addr: u32)
    requires
        // A valid Condvar reference must have been acquired via get_cond.
        spec_condvar_acquired(cond_addr as nat),
    ensures
        spec_cond_ref_released(cond_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::put_cond(cond_addr)`.
///
/// # Description
///
/// Signals completion of the caller's use of the condition variable. The
/// real `put_cond` only removes the condvar entry from the PM's map if
/// `reference_count() <= 1` (i.e., this is the last user); otherwise it
/// returns Ok(()) without removal. The postcondition `spec_put_cond_completed`
/// models a successful call, not necessarily full reclamation.
///
/// The precondition `spec_cond_ref_released` enforces the structural ordering
/// from the original code: the condvar Arc clone must be dropped (via block
/// scoping) before `put_cond` is called.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address.
#[verifier::external_body]
pub fn put_cond_model(cond_addr: u32) -> (result: PutCondOutcomeModel)
    requires
        spec_signal_cond_safety_preconditions(),
        spec_cond_ref_released(cond_addr as nat),
    ensures
        result matches PutCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        result matches PutCondOutcomeModel::Ok
            ==> spec_put_cond_completed(cond_addr as nat),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `signal_cond` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn signal_cond(pid, tid,
/// cond_addr, broadcast)` control flow. It:
/// 1. Calls get_cond (external) → returns Condvar on success.
/// 2. Calls notify (external) → returns awakened count on success.
/// 3. Drops the Condvar (external) → reference count decreases.
/// 4. Calls put_cond (external) → releases condition variable slot.
/// 5. Returns Ok(awakened) on success.
///
/// All steps short-circuit on error via the `?` operator pattern.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address (from `ConditionAddress::from(usize)`).
/// - `broadcast`: Whether to wake all waiting threads.
/// - `pid`: Ghost process identifier (for documentation only).
/// - `tid`: Ghost thread identifier (for documentation only).
///
/// # Returns
///
/// A tuple of (result, ghost state) where the ghost state captures
/// all step outcomes for exec-spec linkage.
pub fn signal_cond_model(
    cond_addr: u32,
    broadcast: bool,
    pid: Ghost<u32>,
    tid: Ghost<u32>,
) -> (ret: (SignalCondResultModel, Ghost<SignalCondGhostState>))
    requires
        spec_signal_cond_safety_preconditions(),
        // ABI constraint: cond_addr originates from 32-bit usize on x86-32.
        // This is trivially satisfied for u32 (documentation-only assertion
        // making the architecture assumption explicit; see lemma_architecture_guard).
        cond_addr as nat <= USIZE_MAX_X86_32(),
    ensures
        ({
            let gs: SignalCondGhostState = ret.1@;
            // The result matches the spec for the captured step outcomes.
            ret.0.spec_view() == spec_signal_cond_result(gs.gc, gs.notify, gs.pc)
        }),
        // Success only when all steps succeed.
        spec_is_success(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            gs.gc == GetCondOutcomeView::GcOk
            && gs.pc == PutCondOutcomeView::PcOk
            && gs.notify matches NotifyOutcomeView::NOk { .. }
        }),
        // Error only when at least one step fails.
        spec_is_error(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            !(gs.gc matches GetCondOutcomeView::GcOk)
            || !(gs.notify matches NotifyOutcomeView::NOk { .. })
            || !(gs.pc matches PutCondOutcomeView::PcOk)
        }),
        // Condvar ref released whenever get_cond succeeded (not just on overall
        // success). The Condvar is dropped at scope exit regardless of whether
        // notify succeeds or fails — callers can rely on resource cleanup even
        // on NotifyError or PutCondError paths. This subsumes the success case.
        ({
            let gs: SignalCondGhostState = ret.1@;
            gs.gc == GetCondOutcomeView::GcOk
        }) ==> spec_cond_ref_released(cond_addr as nat),
        // On success, put_cond completed successfully.
        spec_is_success(ret.0.spec_view()) ==>
            spec_put_cond_completed(cond_addr as nat),
        // On success, broadcast semantics are satisfied: the awakened count
        // respects the broadcast flag (at most 1 for signal, all waiters for broadcast).
        // Uses ghost state to access the notify outcome's awakened count.
        spec_is_success(ret.0.spec_view()) ==> ({
            let gs: SignalCondGhostState = ret.1@;
            gs.notify matches NotifyOutcomeView::NOk { awakened }
                && spec_broadcast_semantics(broadcast, cond_addr as nat, awakened)
        }),
{
    // Step 1: Get condvar reference (external).
    let gc_result: GetCondOutcomeModel = get_cond_model(cond_addr);
    let ghost gc_view: GetCondOutcomeView = gc_result.spec_view();

    match gc_result {
        GetCondOutcomeModel::Error { error_code } => {
            // get_cond failed; short-circuit return.
            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                gc: gc_view,
                notify: NotifyOutcomeView::NOk { awakened: 0 },  // don't-care
                pc: PutCondOutcomeView::PcOk,  // don't-care
            };
            (
                SignalCondResultModel::GetCondError { error_code },
                Ghost(gs),
            )
        },
        GetCondOutcomeModel::Ok => {
            // Step 2: Notify threads (external).
            let notify_result: NotifyOutcomeModel = notify_model(cond_addr, broadcast);
            let ghost notify_view: NotifyOutcomeView = notify_result.spec_view();

            // Step 3: Drop condvar (always happens when get_cond succeeded).
            drop_cond_model(cond_addr);

            match notify_result {
                NotifyOutcomeModel::Error { error_code } => {
                    // notify failed; short-circuit return (put_cond not called).
                    let ghost gs: SignalCondGhostState = SignalCondGhostState {
                        gc: gc_view,
                        notify: notify_view,
                        pc: PutCondOutcomeView::PcOk,  // don't-care
                    };
                    (
                        SignalCondResultModel::NotifyError { error_code },
                        Ghost(gs),
                    )
                },
                NotifyOutcomeModel::Ok { awakened } => {
                    // Step 4: Put condvar (external).
                    let pc_result: PutCondOutcomeModel = put_cond_model(cond_addr);
                    let ghost pc_view: PutCondOutcomeView = pc_result.spec_view();

                    match pc_result {
                        PutCondOutcomeModel::Error { error_code } => {
                            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                                gc: gc_view,
                                notify: notify_view,
                                pc: pc_view,
                            };
                            (
                                SignalCondResultModel::PutCondError { error_code },
                                Ghost(gs),
                            )
                        },
                        PutCondOutcomeModel::Ok => {
                            let ghost gs: SignalCondGhostState = SignalCondGhostState {
                                gc: gc_view,
                                notify: notify_view,
                                pc: pc_view,
                            };
                            (
                                SignalCondResultModel::Success { awakened },
                                Ghost(gs),
                            )
                        },
                    }
                },
            }
        },
    }
}

} // verus!
