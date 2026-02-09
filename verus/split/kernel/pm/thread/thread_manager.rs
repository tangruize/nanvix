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
//! - Kernel thread ID (0) is distinct from all created thread IDs (>= 1).
//! - Well-formedness (wf): next_id >= 1, preserved by all operations.
//! - Newly created threads are well-formed, drop-safe, not interrupted,
//!   and hold no locked mutexes.
//! - init() is equivalent to new().
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
//! omitted from the verification model. They are dispatch patterns that
//! delegate to the underlying thread type's `thread_state()` /
//! `thread_state_mut()` methods without adding invariants or core logic.
//!
//! ## Trust Boundary
//!
//! - ReadyThread::new is a boundary model: postconditions match the real
//!   ReadyThread::new's verified spec from the ready module.
//!   CROSS-MODULE-CHECK: When ready.rs is verified, confirm the real
//!   ReadyThread::new implies all postconditions listed here.
//! - FpuState::new() and ContextInformation::default() are elided (HAL boundary).
//! - Overflow: create_thread requires next_id.value < i32::MAX. The
//!   original code does not check for overflow; this precondition
//!   formalizes the assumption that the system does not create more than
//!   i32::MAX - 1 threads.

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("thread_manager.spec.rs");

// Include proofs.
include!("thread_manager.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that is ready to run (boundary model).
///
/// Models the original `ReadyThread` from the sibling `ready.rs` module.
/// Only `new()` is modeled — enough to verify ThreadManager operations.
///
/// **Cross-module dependency:** When `ReadyThread` is verified independently,
/// the postconditions of this boundary model's `new` must be confirmed
/// as implied by the real `ReadyThread::new` spec.
pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
}

/// A thread manager responsible for creating and managing threads.
///
/// Verification model of `src/kernel/src/pm/thread/mod.rs::ThreadManager`.
/// Tracks the next thread identifier to assign.
pub struct ThreadManager {
    /// Next thread identifier to be assigned.
    pub next_id: ThreadIdentifier,
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
    /// CROSS-MODULE-CHECK: When `ready.rs` is verified, confirm the real
    /// `ReadyThread::new` implies all of:
    /// - `result.spec_id() == id.spec_value()`
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
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
    {
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
    pub fn create_thread(
        &mut self,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        requires
            old(self).wf(),
            old(self).next_id.value < i32::MAX,
        ensures
            result.spec_id() == old(self).spec_next_id(),
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            self.spec_next_id() == old(self).spec_next_id() + 1,
            self.wf(),
    {
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
