// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadState Specification.
// This file contains spec functions and View types for the ThreadState type.
//
// ## Verification Model
//
// ThreadState manages per-thread state in the kernel. For verification we model:
// - `id` as a ThreadIdentifier (verified dependency).
// - `kernel_stack` and `user_stack` as `Option<int>` (abstract resource tokens
//   with identity preservation via Option::take semantics).
// - `user_tda` as Option<int> (abstract virtual address).
// - `interrupt_reason` as Option<int> (abstract reason tag).
// - `locked_mutexes` as a ghost `Set<int>` (abstract mutex address set) with a
//   runtime `locked_mutex_count` counter. The ghost set faithfully models
//   `BTreeMap::insert`/`BTreeMap::remove` per-key semantics, while the counter
//   provides the exec-level size. `wf()` ties them together.
// - `join_cond`, `context`, `fpu_state` as opaque (HAL/sync boundary types).
//
// The spec focuses on the state management protocol: ID immutability,
// Option take/store semantics, mutex guard set consistency, and drop safety.

// NOTE: imports are in state.rs (this file is included via include!()).

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ThreadState.
///
/// Models the logical state of a thread: its identity, resource ownership,
/// interrupt status, and locked mutex set.
#[verifier::ext_equal]
pub struct ThreadStateView {
    /// Thread identifier value.
    pub id: int,
    /// Abstract kernel stack resource token.
    pub kernel_stack: Option<int>,
    /// Abstract user stack resource token.
    pub user_stack: Option<int>,
    /// Optional user thread data area address.
    pub user_tda: Option<int>,
    /// Optional interrupt reason tag.
    pub interrupt_reason: Option<int>,
    /// Number of locked mutexes held.
    pub locked_mutex_count: nat,
    /// Ghost set of locked mutex addresses.
    pub locked_mutex_set: Set<int>,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ThreadState {
    /// Spec function: returns the thread identifier value.
    pub open spec fn spec_id(&self) -> int {
        self.id.spec_value()
    }

    /// Spec function: returns the abstract kernel stack token.
    pub open spec fn spec_kernel_stack(&self) -> Option<int> {
        self.kernel_stack
    }

    /// Spec function: returns whether a kernel stack is present.
    pub open spec fn spec_has_kernel_stack(&self) -> bool {
        self.kernel_stack.is_some()
    }

    /// Spec function: returns the abstract user stack token.
    pub open spec fn spec_user_stack(&self) -> Option<int> {
        self.user_stack
    }

    /// Spec function: returns whether a user stack is present.
    pub open spec fn spec_has_user_stack(&self) -> bool {
        self.user_stack.is_some()
    }

    /// Spec function: returns the user thread data area.
    pub open spec fn spec_user_tda(&self) -> Option<int> {
        self.user_tda
    }

    /// Spec function: returns the interrupt reason.
    pub open spec fn spec_interrupt_reason(&self) -> Option<int> {
        self.interrupt_reason
    }

    /// Spec function: returns the number of locked mutexes.
    pub open spec fn spec_locked_mutex_count(&self) -> nat {
        self.locked_mutex_count as nat
    }

    /// Spec function: returns whether a specific mutex address is locked.
    pub open spec fn spec_has_mutex(&self, address: int) -> bool {
        seq_to_set(self.locked_mutex_set@).contains(address)
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ThreadState is well-formed when:
    /// - The underlying Vec has no duplicate entries.
    /// - The Vec length equals the runtime counter.
    pub closed spec fn wf(&self) -> bool {
        self.locked_mutex_set@.no_duplicates()
        && self.locked_mutex_set@.len() == self.locked_mutex_count as nat
    }

    /// Spec function: checks if the thread holds no locked mutexes.
    ///
    /// Defined directly on the Vec length so the predicate is self-contained
    /// and meaningful even without `wf()`. Under `wf()`, this is equivalent
    /// to `self.locked_mutex_count == 0` (proven by
    /// `lemma_check_drop_safe_models_drop`).
    pub open spec fn spec_drop_safe(&self) -> bool {
        self.locked_mutex_set@.len() == 0
    }

    /// Spec function: checks if the thread has been interrupted.
    pub open spec fn spec_is_interrupted(&self) -> bool {
        self.interrupt_reason.is_some()
    }

    /// Spec function: checks if the thread has resources (stacks) to release.
    pub open spec fn spec_has_resources(&self) -> bool {
        self.kernel_stack.is_some() || self.user_stack.is_some()
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ThreadState {
    type V = ThreadStateView;

    // NOTE: view() must remain `open` because the Verus `View` trait requires
    // implementations to use `open spec fn`. This is a justified exception to
    // the guideline that view() should be `pub closed spec fn`.
    open spec fn view(&self) -> ThreadStateView {
        ThreadStateView {
            id: self.id.spec_value(),
            kernel_stack: self.kernel_stack,
            user_stack: self.user_stack,
            user_tda: self.user_tda,
            interrupt_reason: self.interrupt_reason,
            locked_mutex_count: self.locked_mutex_count as nat,
            locked_mutex_set: seq_to_set(self.locked_mutex_set@),
        }
    }
}

} // verus!
