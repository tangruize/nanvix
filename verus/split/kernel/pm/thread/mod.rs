// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ThreadManager Implementation
//!
//! Manages thread creation and identifier assignment in the Nanvix kernel.
//! Responsible for creating the kernel thread (ID 0) during initialization
//! and assigning monotonically increasing IDs to subsequently created threads.
//!
//! ## Verified Properties
//!
//! - Construction (new) produces a well-formed manager with next_id=1 and
//!   a kernel thread with ID 0.
//! - create_thread assigns the current next_id to the new thread and
//!   increments next_id by exactly 1.
//! - Thread IDs are monotonically increasing (each new ID > previous).
//! - Global ID uniqueness: any two IDs assigned at different manager states
//!   are distinct, and the kernel thread ID (0) is unique from all of them.
//! - Kernel thread ID (0) is distinct from all created thread IDs (>= 1).
//! - Well-formedness (wf): next_id >= 1, preserved by all operations.
//! - Newly created threads are well-formed, drop-safe, not interrupted,
//!   and hold no locked mutexes.
//! - init() is equivalent to new().
//! - ThreadRef dispatch: `thread_state()` returns the correct state with
//!   identity preservation regardless of which thread variant is active.
//! - ThreadRefMut dispatch: read aspect of `thread_state_mut()` preserves
//!   identity; mutation is a documented trust boundary.
//!
//! ## Verification Model
//!
//! The original ThreadManager uses complex kernel types:
//! - `KernelStack`, `UserStack` -> `Option<int>` (abstract resource tokens).
//! - `VirtualAddress` -> `Option<int>` (abstract address).
//! - `ContextInformation`, `FpuState` -> elided (HAL boundary types).
//! - `ReadyThread` -> boundary model wrapping ThreadState.
//!
//! The `ThreadRef` and `ThreadRefMut` enums from the original module are
//! modeled as `ThreadRefModel` and `ThreadRefMutModel` respectively.
//! The original enums use lifetime-parameterized reference fields
//! (`&'a ReadyThread`, `&'a mut RunningThread`, etc.) which Verus cannot
//! express. The models use value-based `ThreadState` fields instead and
//! verify the dispatch semantics: `thread_state()` correctly returns the
//! active variant's underlying state with identity preservation.
//! Mutation through `thread_state_mut()` is a trust boundary (Verus cannot
//! express `&mut T` return types); callers must preserve `wf()` and
//! `spec_id()` as documented on each thread type's `thread_state_mut()`.
//!
//! ## Trust Boundary
//!
//! - ReadyThread::new is a boundary model: postconditions match the real
//!   ReadyThread::new's verified spec from the ready module.
//!   CROSS-MODULE-CHECK: When ready.rs is verified, confirm the real
//!   ReadyThread::new implies all postconditions listed here.
//! - FpuState::new() and ContextInformation::default() are elided (HAL boundary).
//! - Heap allocation via `Box::new` is assumed to succeed. The original code
//!   panics on OOM (no-std default allocator behavior); the verification model
//!   elides allocation by using `ThreadState` directly instead of `Box<ThreadState>`.
//! - Overflow: create_thread requires next_id.value < i32::MAX. This is a
//!   **strengthening** over the original code, which does not check for
//!   overflow. The original `<i32>::from(self.next_id) + 1` wraps silently
//!   in release mode, producing a negative thread ID — a latent bug.
//!   The precondition makes the implicit no-overflow assumption explicit
//!   and machine-checked. This divergence is intentional: the spec is
//!   stronger than the original, not semantically equivalent.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("mod.spec.rs");

// Include proofs.
include!("mod.proof.rs");

pub mod interrupted;
pub mod ready;
pub mod running;
pub mod sleeping;
pub mod state;
pub mod zombie;

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that is ready to run (boundary model).
///
/// Models the original `ReadyThread` from the sibling `ready.rs` module.
/// Only `new()` is modeled — enough to verify ThreadManager operations.
///
/// **Intentional omission:** The real `ReadyThread` (and the standalone
/// `ready.rs` verification model) also holds an `admission_time` field
/// set to `clock_now()` in the constructor. This scheduling property is
/// not relevant to ThreadManager's core logic (ID assignment and thread
/// creation). Callers that need scheduling properties should use the
/// `ready.rs` verification module's `ReadyThread` type.
///
/// **Cross-module dependency:** When `ReadyThread` is verified independently,
/// the postconditions of this boundary model's `new` must be confirmed
/// as implied by the real `ReadyThread::new` spec.
///
/// **Field visibility:** The `state` field is `pub` because the Verus `View`
/// trait requires `open spec fn view()`, which in turn requires accessed
/// fields to be visible outside the module. This is a Verus framework
/// constraint, not a design choice.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread manager responsible for creating and managing threads.
///
/// Verification model of `src/kernel/src/pm/thread/mod.rs::ThreadManager`.
/// Tracks the next thread identifier to assign.
///
/// **Field visibility:** The `next_id` field is `pub` because the Verus `View`
/// trait requires `open spec fn view()`, which in turn requires accessed
/// fields to be visible outside the module. This is a Verus framework
/// constraint, not a design choice.
pub struct ThreadManager {
    /// Next thread identifier to be assigned.
    pub next_id: ThreadIdentifier,
}

//==================================================================================================
// ThreadRef / ThreadRefMut Dispatch Models
//==================================================================================================

/// Verification model of the original `ThreadRef<'a>` enum.
///
/// The original enum holds `&'a ReadyThread`, `&'a RunningThread`, etc.
/// Verus cannot express lifetime-parameterized enums or reference fields
/// in enum variants. Instead, we model the dispatch by storing a
/// `ThreadState` value — the object that `thread_state()` returns
/// across all variants. This models the key semantic property: regardless
/// of which thread variant is active, `thread_state()` returns its
/// underlying state with identity and well-formedness preserved.
///
/// Each variant's `thread_state()` is individually verified in its own
/// module (ready.rs, running.rs, sleeping.rs, interrupted.rs, zombie.rs)
/// with ensures `result.spec_id() == self.spec_id()` and
/// `result@ == self.state@`. This model verifies the dispatch layer
/// on top of those per-variant guarantees.
///
/// **Aliasing/exclusivity:** The original `ThreadRef` is an immutable
/// borrow that prevents modification while the borrow is live. This
/// aliasing property is enforced by Rust's borrow checker and is
/// outside the Verus verification model.
pub enum ThreadRefModel {
    /// Ready variant (models `ThreadRef::Ready(&ReadyThread)`).
    Ready(ThreadState),
    /// Running variant (models `ThreadRef::Running(&RunningThread)`).
    Running(ThreadState),
    /// Sleeping variant (models `ThreadRef::Sleeping(&SleepingThread)`).
    Sleeping(ThreadState),
    /// Interrupted variant (models `ThreadRef::Interrupted(&InterruptedThread)`).
    Interrupted(ThreadState),
    /// Zombie variant (models `ThreadRef::Zombie(&ZombieThread)`).
    Zombie(ThreadState),
}

impl ThreadRefModel {
    /// Returns the thread state from whichever variant is active.
    ///
    /// Models `ThreadRef::thread_state(&self) -> &ThreadState`.
    /// Verifies that the dispatch correctly returns the state regardless
    /// of which variant is active, preserving the thread identity.
    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
            result@ == self.spec_state()@,
    {
        match self {
            ThreadRefModel::Ready(state) => state,
            ThreadRefModel::Running(state) => state,
            ThreadRefModel::Sleeping(state) => state,
            ThreadRefModel::Interrupted(state) => state,
            ThreadRefModel::Zombie(state) => state,
        }
    }
}

/// Verification model of the original `ThreadRefMut<'a>` enum.
///
/// The original enum holds `&'a mut ReadyThread`, etc. Verus cannot
/// express `&mut T` return types or lifetime-parameterized enums.
/// We model `thread_state_mut()` as a read-only dispatch returning
/// `&ThreadState`, verifying identity preservation. The mutability
/// aspect is a trust boundary: callers that mutate through the
/// real `thread_state_mut()` must preserve `wf()` and `spec_id()`,
/// as documented on each thread type's `thread_state_mut()` function.
///
/// Each variant's `thread_state_mut()` is marked `#[verifier::external]`
/// in its own module due to the `&mut T` return type limitation.
/// This model verifies the dispatch semantics; mutation safety relies
/// on the per-module trust boundary documentation.
///
/// **Aliasing/exclusivity:** The original `ThreadRefMut` holds an
/// exclusive (`&mut`) borrow, guaranteeing no aliased access while the
/// borrow is live. This property is enforced by Rust's borrow checker
/// and is outside the Verus verification model. The value-based model
/// cannot express exclusive access.
///
/// **Caller proof obligations for mutation:** Any caller that mutates
/// thread state through the real `thread_state_mut()` MUST ensure:
/// 1. `self.wf()` is preserved after mutation.
/// 2. `self.spec_id()` is unchanged after mutation.
/// Failure to maintain these invariants invalidates all proven
/// properties. See trust boundary documentation on each thread type's
/// `thread_state_mut()` function (e.g., ready.rs:528, running.rs:492).
pub enum ThreadRefMutModel {
    /// Ready variant (models `ThreadRefMut::Ready(&mut ReadyThread)`).
    Ready(ThreadState),
    /// Running variant (models `ThreadRefMut::Running(&mut RunningThread)`).
    Running(ThreadState),
    /// Sleeping variant (models `ThreadRefMut::Sleeping(&mut SleepingThread)`).
    Sleeping(ThreadState),
    /// Interrupted variant (models `ThreadRefMut::Interrupted(&mut InterruptedThread)`).
    Interrupted(ThreadState),
    /// Zombie variant (models `ThreadRefMut::Zombie(&mut ZombieThread)`).
    Zombie(ThreadState),
}

impl ThreadRefMutModel {
    /// Returns the thread state from whichever variant is active.
    ///
    /// Models the read aspect of `ThreadRefMut::thread_state_mut()`.
    /// The original returns `&mut ThreadState`; Verus cannot express
    /// mutable return references. This models the dispatch identity
    /// preservation: the returned state belongs to the active variant.
    ///
    /// **Trust boundary:** Mutation through the real `thread_state_mut()`
    /// is unverified. Callers MUST preserve `wf()` and `spec_id()`.
    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
            result@ == self.spec_state()@,
    {
        match self {
            ThreadRefMutModel::Ready(state) => state,
            ThreadRefMutModel::Running(state) => state,
            ThreadRefMutModel::Sleeping(state) => state,
            ThreadRefMutModel::Interrupted(state) => state,
            ThreadRefMutModel::Zombie(state) => state,
        }
    }
}

//==================================================================================================
// ReadyThread Implementation (Boundary)
//==================================================================================================

impl ReadyThread {
    /// Creates a new ready thread (boundary model).
    ///
    /// # Parameters
    ///
    /// - `id`: Thread identifier.
    /// - `kernel_stack`: Optional abstract kernel stack resource token.
    /// - `user_stack`: Optional abstract user stack resource token.
    /// - `user_tda`: Optional base address for user-space thread data area.
    ///
    /// # Omitted Parameters
    ///
    /// The original constructor also takes `context: ContextInformation` and
    /// `fpu_state: FpuState` (opaque HAL types, out of verification scope).
    ///
    /// # Returns
    ///
    /// A new, well-formed, drop-safe ReadyThread.
    ///
    /// # Cross-Module Verification Obligations
    ///
    /// CROSS-MODULE-CHECK: The real `ReadyThread::new` in `ready.rs`
    /// (verus/split/kernel/pm/thread/ready.rs:259–280) has been verified
    /// with postconditions that imply all of the below. Confirm these
    /// remain consistent if ready.rs changes:
    /// - `result.spec_id() == id.spec_value()`
    /// - `result.spec_kernel_stack() == kernel_stack`
    /// - `result.spec_user_stack() == user_stack`
    /// - `result.spec_user_tda() == user_tda`
    /// - `result.wf()`
    /// - `result.spec_drop_safe()`
    /// - `!result.spec_is_interrupted()`
    /// - `result.spec_locked_mutex_count() == 0`
    pub fn new(
        id: ThreadIdentifier,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        ensures
            result.spec_id() == id.spec_value(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
    {
        proof {
            reveal(ReadyThread::spec_id);
            reveal(ReadyThread::spec_kernel_stack);
            reveal(ReadyThread::spec_user_stack);
            reveal(ReadyThread::spec_user_tda);
            reveal(ReadyThread::wf);
            reveal(ReadyThread::spec_drop_safe);
            reveal(ReadyThread::spec_is_interrupted);
            reveal(ReadyThread::spec_locked_mutex_count);
        }
        ReadyThread {
            state: ThreadState::new(id, kernel_stack, user_stack, user_tda),
        }
    }
}

//==================================================================================================
// ThreadManager Implementation
//==================================================================================================

impl ThreadManager {
    /// Creates a new thread manager and initializes the kernel thread.
    ///
    /// The kernel thread is assigned ID 0. The manager starts with
    /// next_id=1, ready to assign to the first user thread.
    ///
    /// # Returns
    ///
    /// A tuple containing the kernel thread (ID 0) and a new ThreadManager
    /// (next_id=1).
    fn new() -> (result: (ReadyThread, ThreadManager))
        ensures
            result.0.spec_id() == 0,
            result.0.wf(),
            result.0.spec_drop_safe(),
            !result.0.spec_is_interrupted(),
            result.0.spec_locked_mutex_count() == 0,
            result.1.spec_next_id() == 1,
            result.1.wf(),
    {
        proof {
            reveal(ThreadManager::wf);
            reveal(ThreadManager::spec_next_id);
            reveal(ReadyThread::spec_id);
            reveal(ReadyThread::wf);
            reveal(ReadyThread::spec_drop_safe);
            reveal(ReadyThread::spec_is_interrupted);
            reveal(ReadyThread::spec_locked_mutex_count);
        }
        let kernel: ReadyThread = ReadyThread::new(
            ThreadIdentifier::from_i32(0),
            None,
            None,
            None,
        );
        let manager: ThreadManager = ThreadManager {
            next_id: ThreadIdentifier::from_i32(1),
        };
        (kernel, manager)
    }

    /// Creates a new thread with the specified parameters.
    ///
    /// Assigns the current next_id to the new thread and increments
    /// next_id by 1.
    ///
    /// # Parameters
    ///
    /// - `kernel_stack`: Optional kernel stack for the thread.
    /// - `user_stack`: Optional user stack for the thread.
    /// - `user_tda`: Optional base address to user-space thread data area.
    ///
    /// # Omitted Parameters
    ///
    /// The original also takes `context: ContextInformation` (opaque HAL
    /// type, out of verification scope). FpuState is created internally.
    ///
    /// # Returns
    ///
    /// A new ReadyThread with the assigned thread identifier.
    ///
    /// # Divergence from Original
    ///
    /// The `requires next_id.value < i32::MAX` precondition is a
    /// **strengthening** over the original, which has no overflow check.
    /// The original wraps silently in release mode, producing a negative
    /// TID. This precondition catches a latent overflow bug.
    pub fn create_thread(
        &mut self,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        requires
            old(self).wf(),
            old(self)@.next_id < i32::MAX as int,
        ensures
            result.spec_id() == old(self).spec_next_id(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            self.spec_next_id() == old(self).spec_next_id() + 1,
            self.wf(),
    {
        proof {
            reveal(ThreadManager::wf);
            reveal(ThreadManager::spec_next_id);
            reveal(ReadyThread::spec_id);
            reveal(ReadyThread::spec_kernel_stack);
            reveal(ReadyThread::spec_user_stack);
            reveal(ReadyThread::spec_user_tda);
            reveal(ReadyThread::wf);
            reveal(ReadyThread::spec_drop_safe);
            reveal(ReadyThread::spec_is_interrupted);
            reveal(ReadyThread::spec_locked_mutex_count);
        }
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from_i32(self.next_id.into_i32() + 1);
        ReadyThread::new(id, kernel_stack, user_stack, user_tda)
    }
}

//==================================================================================================
// Standalone Functions
//==================================================================================================

/// Initializes the thread manager.
///
/// # Returns
///
/// A tuple containing the kernel thread (ID 0) and a new ThreadManager.
///
/// # Single-Initialization Assumption
///
/// The original code has a `TODO: check for double initialization`
/// comment indicating the intent to prevent multiple calls. This
/// verification model does not enforce single-call semantics: each
/// call produces a fresh manager with a new kernel thread (ID 0).
/// If `init()` were called more than once, the system would have
/// duplicate kernel threads with ID 0, violating global ID uniqueness.
///
/// **System-level assumption:** `init()` is called exactly once during
/// boot. Enforcing this requires a global ghost flag or module-level
/// state, which is outside the scope of this module's verification.
/// Callers are responsible for ensuring single-initialization.
pub fn init() -> (result: (ReadyThread, ThreadManager))
    ensures
        result.0.spec_id() == 0,
        result.0.wf(),
        result.0.spec_drop_safe(),
        !result.0.spec_is_interrupted(),
        result.0.spec_locked_mutex_count() == 0,
        result.1.spec_next_id() == 1,
        result.1.wf(),
{
    ThreadManager::new()
}

} // verus!
