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
//! - **Condvar drop on get_cond success**: When get_cond succeeds, the condvar
//!   reference is always released (dropped at scope exit), regardless of
//!   whether notify succeeds or fails.
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
//!   verified in the PM module.
//! - **Liveness**: Whether waiting threads actually wake up is a scheduler
//!   concern, verified separately.
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
{
    unimplemented!()
}

/// Trust Boundary T2: Models `cond.notify_all()` or `cond.notify_first()`.
///
/// # Description
///
/// Notifies threads waiting on the condition variable. If `broadcast` is true,
/// models `cond.notify_all()`; otherwise models `cond.notify_first()`.
/// Returns the number of awakened threads on success, or an error.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address.
/// - `broadcast`: Whether to wake all waiting threads.
#[verifier::external_body]
pub fn notify_model(cond_addr: u32, broadcast: bool) -> (result: NotifyOutcomeModel)
    ensures
        result matches NotifyOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
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
/// # Parameters
///
/// - `cond_addr`: The condition variable address whose ref is being released.
#[verifier::external_body]
pub fn drop_cond_model(cond_addr: u32)
    ensures
        spec_cond_ref_released(cond_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::put_cond(cond_addr)`.
///
/// # Description
///
/// Returns the condition variable slot to the PM. This is called after the
/// Condvar has been dropped.
///
/// # Parameters
///
/// - `cond_addr`: The condition variable address.
#[verifier::external_body]
pub fn put_cond_model(cond_addr: u32) -> (result: PutCondOutcomeModel)
    requires
        spec_signal_cond_safety_preconditions(),
    ensures
        result matches PutCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        result matches PutCondOutcomeModel::Ok
            ==> spec_cond_slot_returned(cond_addr as nat),
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
        // On success, the condvar reference was released (dropped).
        spec_is_success(ret.0.spec_view()) ==>
            spec_cond_ref_released(cond_addr as nat),
        // On success, the condvar slot was returned.
        spec_is_success(ret.0.spec_view()) ==>
            spec_cond_slot_returned(cond_addr as nat),
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
