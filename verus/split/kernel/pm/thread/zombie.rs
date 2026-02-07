// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ZombieThread Implementation
//!
//! Represents a thread that has terminated and is waiting to be harvested
//! in the Nanvix kernel. A zombie thread holds an exit status and the
//! underlying thread state until its resources (kernel stack, user stack)
//! are reclaimed via `harvest()`.
//!
//! ## Verified Properties
//!
//! - Construction (from_state) produces well-formed state with correct identity and status.
//! - Thread identifier (`id`) is immutable: accessor returns the construction-time value.
//! - `status()` correctly returns the exit status captured at construction.
//! - `thread_state()` returns a reference with the same identity and full state view.
//! - `harvest()` returns the kernel and user stacks from the underlying state
//!   (delegating to ThreadState's take_kernel_stack / take_user_stack).
//! - Well-formedness (`wf()`) is preserved through construction.
//! - Mutex accounting (count and per-address membership) is preserved through construction.
//! - Drop safety is preserved through construction.
//!
//! ## Verification Model
//!
//! The original `ZombieThread` contains complex kernel types. For verification:
//! - `Box<ThreadState>` -> `ThreadState` directly (Box is transparent).
//! - `ExitStatus` -> `int` (abstract status tag).
//! - `KernelStack` / `UserStack` -> `Option<int>` (abstract resource tokens,
//!   modeled in ThreadState with identity preservation via Option::take semantics).
//!
//! ## Trust Boundary
//!
//! - `thread_state_mut()` is `#[verifier::external]`: returns `&mut ThreadState`
//!   which Verus cannot express. See documented trust obligations.
//! - `harvest()` delegates to verified ThreadState methods (`take_kernel_stack`,
//!   `take_user_stack`). The return type is modeled as a tuple of `Option<int>`
//!   tokens matching the abstract resource model. Resource lifecycle tracking
//!   of returned stacks by the caller is out of verification scope.
//! - `Box<ThreadState>` is modeled as `ThreadState` directly. Box deallocation
//!   correctness is out of verification scope.
//! - `ExitStatus` is modeled as unbounded `int`. Bounding to the original
//!   type's range is an intentional simplification (see `wf()` documentation).

use crate::kernel::pm::thread::state::ThreadState;
use crate::kernel::pm::thread::state::ThreadStateView;
use crate::kernel::pm::sys::tid::ThreadIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("zombie.spec.rs");

// Include proofs.
include!("zombie.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A thread that has terminated and is waiting to be harvested.
///
/// Verification model of `src/kernel/src/pm/thread/zombie.rs::ZombieThread`.
/// `Box<ThreadState>` is modeled as `ThreadState` directly.
/// `ExitStatus` is modeled as `int`.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics (spec access,
/// direct construction in lemmas). The original has private fields.
/// INVARIANT: Construction should only occur via `from_state()` which establishes `wf()`.
pub struct ZombieThread {
    /// The exit status of the terminated thread (abstract int tag).
    pub status: int,
    /// The underlying thread state.
    pub state: ThreadState,
}

//==================================================================================================
// ZombieThread Implementation
//==================================================================================================

impl ZombieThread {
    /// Creates a zombie thread from an existing thread state and exit status.
    ///
    /// # Parameters
    ///
    /// - `state`: The thread state.
    /// - `status`: The exit status of the terminated thread.
    ///
    /// # Returns
    ///
    /// A well-formed ZombieThread preserving all ThreadState properties
    /// and capturing the exit status.
    pub fn from_state(state: ThreadState, status: int) -> (result: ZombieThread)
        requires
            state.wf(),
        ensures
            result.state@ == state@,
            result.spec_id() == state.spec_id(),
            result.spec_status() == status,
            result.spec_kernel_stack() == state.spec_kernel_stack(),
            result.spec_user_stack() == state.spec_user_stack(),
            result.spec_user_tda() == state.spec_user_tda(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        ZombieThread { status: status, state: state }
    }

    /// Returns the identifier of the zombie thread.
    ///
    /// # Returns
    ///
    /// The thread identifier, unchanged from construction.
    pub fn id(&self) -> (result: ThreadIdentifier)
        ensures
            result.spec_value() == self.spec_id(),
    {
        self.state.id()
    }

    /// Returns a reference to the thread state.
    ///
    /// # Returns
    ///
    /// A reference to the underlying ThreadState with the same identity
    /// and full state transparency.
    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
            result@ == self.state@,
    {
        &self.state
    }

    /// Harvests the zombie thread and reclaims its resources.
    ///
    /// # Returns
    ///
    /// A tuple containing the optional kernel stack and user stack tokens
    /// of the terminated thread. The stacks are taken from the underlying
    /// state via `take_kernel_stack()` / `take_user_stack()` (Option::take
    /// semantics), matching the original implementation's mutation sequence.
    ///
    /// # Modeling Note
    ///
    /// The original returns `(Option<KernelStack>, Option<UserStack>)`.
    /// These are modeled as `(Option<int>, Option<int>)` — abstract resource
    /// tokens with identity preservation. The returned tokens are exactly
    /// those that were stored in the ThreadState at construction time.
    /// After harvest, the ThreadState's stack fields are None, preserving
    /// wf() through the take-then-drop sequence.
    pub fn harvest(mut self) -> (result: (Option<int>, Option<int>))
        requires
            old(self).wf(),
        ensures
            result.0 == old(self).spec_kernel_stack(),
            result.1 == old(self).spec_user_stack(),
    {
        let kstack: Option<int> = self.state.take_kernel_stack();
        let ustack: Option<int> = self.state.take_user_stack();
        (kstack, ustack)
    }

    /// Returns the exit status of the zombie thread.
    ///
    /// # Returns
    ///
    /// The exit status of the terminated thread.
    pub fn status(&self) -> (result: int)
        ensures
            result == self.spec_status(),
    {
        self.status
    }
}

} // verus!

/// Non-verus impl block for functions that cannot be expressed inside `verus!`
/// due to Verus language limitations (e.g., `&mut T` return types).
impl ZombieThread {
    /// Returns a mutable reference to the thread state.
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying ThreadState.
    ///
    /// # Trust Boundary
    ///
    /// Marked `#[verifier::external]` because Verus does not yet support
    /// `&mut T` return types — neither `external_body` nor normal `verus!`
    /// functions can express the signature.
    ///
    /// Callers that mutate the `ThreadState` through this reference operate
    /// outside the verification boundary. Callers MUST preserve:
    /// - `self.wf()` — the well-formedness invariant.
    /// - `self.spec_id()` — the thread identity must not change.
    /// - `self.spec_status()` — the exit status must not change.
    ///
    /// **Intended postconditions** (not machine-checked):
    /// - `ensures old(self).spec_id() == self.spec_id()` (identity preserved).
    /// - `ensures old(self).wf() ==> self.wf()` (well-formedness preserved).
    /// - `ensures old(self).spec_status() == self.spec_status()` (status preserved).
    #[verifier::external]
    pub fn thread_state_mut(&mut self) -> &mut ThreadState {
        &mut self.state
    }
}
