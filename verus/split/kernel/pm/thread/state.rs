// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ThreadState Implementation
//!
//! Manages per-thread state in the Nanvix kernel: identity, stacks, execution
//! context, FPU state, interrupt reason, locked mutexes, and thread data area.
//!
//! ## Verified Properties
//!
//! - Construction produces well-formed, drop-safe state with correct initial values.
//! - Thread identifier (`id`) is immutable: all operations preserve it.
//! - `take_kernel_stack` / `take_user_stack` follow Option::take semantics.
//! - `set_interrupt_reason` / `take_interrupt_reason` follow Option set/take semantics.
//! - `store_thread_data_area` / `get_thread_data_area` round-trip correctly.
//! - `store_mutex_guard` inserts an address into the ghost set (no double-lock).
//! - `take_mutex_guard` removes an address from the ghost set (address must be held).
//! - Well-formedness (`wf()`): the runtime counter equals the ghost set size, and
//!   the ghost set is finite. Preserved by all operations.
//! - Drop safety (`spec_drop_safe()`): no locked mutexes remain at destruction.
//!   This is the verification-side encoding of the `Drop` invariant — the original
//!   `Drop::drop()` logs an error if locked mutexes remain. Newly constructed state
//!   is always drop-safe.
//!
//! ## Verification Model
//!
//! The original `ThreadState` contains complex kernel types (`KernelStack`,
//! `UserStack`, `ContextInformation`, `FpuState`, `Condvar`, `MutexGuard`,
//! `BTreeMap`, `Pin<Box<_>>`) from HAL, MM, and sync subsystems. For
//! verification we abstract these away:
//! - Stacks → `has_kernel_stack: bool`, `has_user_stack: bool` (presence flags).
//! - Thread data area → `user_tda: Option<int>` (abstract address).
//! - Interrupt reason → `interrupt_reason: Option<int>` (abstract reason tag).
//! - Locked mutexes → `locked_mutex_count: usize` (runtime counter) paired with
//!   a ghost `locked_mutex_set: Set<int>` that tracks per-address semantics,
//!   faithfully modeling `BTreeMap::insert`/`BTreeMap::remove`. The `wf()`
//!   predicate ties the counter to the set size.
//! - Context, FPU state, join_cond → elided (opaque HAL/sync boundary types).
//!
//! The `context_mut()`, `fpu_state_mut()`, and `join_cond()` functions return
//! opaque pointers or cloned sync primitives that cannot be meaningfully
//! modeled in a pure spec. They are omitted from the verification model.
//!
//! ## Trust Assumption
//!
//! - **T1: No double-locking.** `store_mutex_guard` requires
//!   `!self.spec_has_mutex(address@)` as a precondition. In the original kernel,
//!   double-locking the same mutex causes a deadlock, so this condition always
//!   holds in correct executions. The precondition formalizes this kernel invariant.
//! - **T2: Release-what-you-hold.** `take_mutex_guard` requires
//!   `self.spec_has_mutex(address@)`. In the original kernel, a thread releases
//!   only mutexes it holds. The precondition formalizes this.
//!
//! ## Verification Scope
//!
//! This verification proves the **state management protocol** is correct:
//! field updates follow Option semantics, ID is immutable, mutex guard
//! tracking is consistent with per-key semantics, and drop safety holds.
//! The following are out of scope:
//! - Raw pointer safety for `context_mut()` / `fpu_state_mut()`.
//! - Interior mutability / shared ownership of `Condvar`.
//! - Pin projection safety.

use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("state.spec.rs");

// Include proofs.
include!("state.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents the state of a thread.
///
/// This is the verification model of `src/kernel/src/pm/thread/state.rs::ThreadState`.
/// Complex kernel types are abstracted to boolean/integer/nat fields.
pub struct ThreadState {
    /// Thread identifier (verified dependency).
    pub id: ThreadIdentifier,
    /// Whether a kernel stack is present.
    pub has_kernel_stack: bool,
    /// Whether a user stack is present.
    pub has_user_stack: bool,
    /// Optional base address for the user-space thread data area.
    pub user_tda: Option<int>,
    /// Interrupt reason tag, if any.
    pub interrupt_reason: Option<int>,
    /// Number of locked mutexes held by this thread.
    pub locked_mutex_count: usize,
    /// Ghost set of locked mutex addresses, faithfully modeling the
    /// original `BTreeMap<MutexAddress, MutexGuard>` per-key semantics.
    pub locked_mutex_set: Ghost<Set<int>>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ThreadState {
    /// Creates a new thread state.
    ///
    /// # Parameters
    ///
    /// - `id`: Thread identifier.
    /// - `has_kernel_stack`: Whether a kernel stack is provided.
    /// - `has_user_stack`: Whether a user stack is provided.
    /// - `user_tda`: Optional base address for user-space thread data area.
    ///
    /// # Returns
    ///
    /// A new, well-formed, drop-safe ThreadState with no interrupt reason
    /// and no locked mutexes.
    pub fn new(
        id: ThreadIdentifier,
        has_kernel_stack: bool,
        has_user_stack: bool,
        user_tda: Option<int>,
    ) -> (result: ThreadState)
        ensures
            result.spec_id() == id.spec_value(),
            result.spec_has_kernel_stack() == has_kernel_stack,
            result.spec_has_user_stack() == has_user_stack,
            result.spec_user_tda() == user_tda,
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            result.spec_drop_safe(),
            result.wf(),
    {
        ThreadState {
            id: id,
            has_kernel_stack: has_kernel_stack,
            has_user_stack: has_user_stack,
            user_tda: user_tda,
            interrupt_reason: None,
            locked_mutex_count: 0usize,
            locked_mutex_set: Ghost(Set::empty()),
        }
    }

    /// Returns the identifier of the thread.
    ///
    /// # Returns
    ///
    /// The thread identifier, unchanged from construction.
    pub fn id(&self) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == self.spec_id(),
    {
        self.id
    }

    /// Returns the kernel stack presence and clears it (Option::take).
    ///
    /// # Returns
    ///
    /// Whether a kernel stack was present before the take.
    pub fn take_kernel_stack(&mut self) -> (result: bool)
        ensures
            result == old(self).spec_has_kernel_stack(),
            !self.spec_has_kernel_stack(),
            self.spec_id() == old(self).spec_id(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_interrupt_reason() == old(self).spec_interrupt_reason(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            self.wf(),
    {
        let had: bool = self.has_kernel_stack;
        self.has_kernel_stack = false;
        had
    }

    /// Returns the user stack presence and clears it (Option::take).
    ///
    /// # Returns
    ///
    /// Whether a user stack was present before the take.
    pub fn take_user_stack(&mut self) -> (result: bool)
        ensures
            result == old(self).spec_has_user_stack(),
            !self.spec_has_user_stack(),
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_interrupt_reason() == old(self).spec_interrupt_reason(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            self.wf(),
    {
        let had: bool = self.has_user_stack;
        self.has_user_stack = false;
        had
    }

    /// Sets the interrupt reason for the thread.
    ///
    /// # Parameters
    ///
    /// - `reason`: The interrupt reason tag to set.
    pub fn set_interrupt_reason(&mut self, reason: int)
        ensures
            self.spec_is_interrupted(),
            self.spec_interrupt_reason() == Some(reason),
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            self.wf(),
    {
        self.interrupt_reason = Some(reason);
    }

    /// Takes and returns the interrupt reason, clearing it (Option::take).
    ///
    /// # Returns
    ///
    /// The interrupt reason if one was set, or None.
    pub fn take_interrupt_reason(&mut self) -> (result: Option<int>)
        ensures
            result == old(self).spec_interrupt_reason(),
            !self.spec_is_interrupted(),
            self.spec_interrupt_reason() == None::<int>,
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            self.wf(),
    {
        let reason: Option<int> = self.interrupt_reason;
        self.interrupt_reason = None;
        reason
    }

    /// Stores a mutex guard, adding the address to the locked mutex set.
    ///
    /// # Parameters
    ///
    /// - `address`: Ghost mutex address being locked.
    ///
    /// # Note
    ///
    /// The original uses `BTreeMap::insert(address, guard)`. We model this
    /// with a ghost `Set<int>` for per-key semantics and a runtime counter.
    /// The no-double-lock precondition formalizes the kernel invariant that
    /// double-locking causes a deadlock and therefore never occurs.
    pub fn store_mutex_guard(&mut self, address: Ghost<int>)
        requires
            old(self).wf(),
            old(self).locked_mutex_count < usize::MAX,
            !old(self).spec_has_mutex(address@),
        ensures
            self.spec_has_mutex(address@),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count() + 1,
            !self.spec_drop_safe(),
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_interrupt_reason() == old(self).spec_interrupt_reason(),
            self.wf(),
    {
        self.locked_mutex_count = self.locked_mutex_count + 1;
        self.locked_mutex_set = Ghost(self.locked_mutex_set@.insert(address@));
    }

    /// Takes a mutex guard, removing the address from the locked mutex set.
    ///
    /// # Parameters
    ///
    /// - `address`: Ghost mutex address being released.
    ///
    /// # Returns
    ///
    /// Always returns true (the precondition guarantees the address is held).
    ///
    /// # Note
    ///
    /// The original uses `BTreeMap::remove(address)` which returns
    /// `Option<MutexGuard>`. The precondition `spec_has_mutex(address@)`
    /// formalizes the kernel invariant that a thread only releases mutexes
    /// it holds.
    pub fn take_mutex_guard(&mut self, address: Ghost<int>) -> (result: bool)
        requires
            old(self).wf(),
            old(self).spec_has_mutex(address@),
        ensures
            result == true,
            !self.spec_has_mutex(address@),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count() - 1,
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_user_tda() == old(self).spec_user_tda(),
            self.spec_interrupt_reason() == old(self).spec_interrupt_reason(),
            self.wf(),
    {
        self.locked_mutex_count = self.locked_mutex_count - 1;
        self.locked_mutex_set = Ghost(self.locked_mutex_set@.remove(address@));
        true
    }

    /// Sets the base address for the user-space thread data area.
    ///
    /// # Parameters
    ///
    /// - `user_tda`: Optional thread data area address to set.
    pub fn store_thread_data_area(&mut self, user_tda: Option<int>)
        ensures
            self.spec_user_tda() == user_tda,
            self.spec_id() == old(self).spec_id(),
            self.spec_has_kernel_stack() == old(self).spec_has_kernel_stack(),
            self.spec_has_user_stack() == old(self).spec_has_user_stack(),
            self.spec_interrupt_reason() == old(self).spec_interrupt_reason(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            self.wf(),
    {
        self.user_tda = user_tda;
    }

    /// Returns the base address for the user-space thread data area.
    ///
    /// # Returns
    ///
    /// The user thread data area address, if set.
    pub fn get_thread_data_area(&self) -> (result: Option<int>)
        ensures
            result == self.spec_user_tda(),
    {
        self.user_tda
    }
}

} // verus!
