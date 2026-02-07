// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadState Proofs.
// This file contains proof lemmas for the ThreadState type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - ID is immutable: all operations preserve the thread identifier.
// - Take operations clear the corresponding field (Option::take semantics).
// - Store operations set the corresponding field.
// - Mutex guard store increments count; take decrements count (when present).
// - Drop safety: newly constructed state is drop-safe (no locked mutexes).
// - Interrupt reason set/take follows Option semantics.

use vstd::prelude::*;

verus! {

impl ThreadState {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed ThreadState is well-formed.
    pub proof fn lemma_new_is_wf(
        id: ThreadIdentifier,
        has_kernel_stack: bool,
        has_user_stack: bool,
        user_tda: Option<int>,
    )
        ensures
            ({
                let s: ThreadState = ThreadState {
                    id: id,
                    has_kernel_stack: has_kernel_stack,
                    has_user_stack: has_user_stack,
                    user_tda: user_tda,
                    interrupt_reason: None,
                    locked_mutex_count: 0,
                };
                s.wf()
            }),
    {
    }

    /// Lemma: A newly constructed ThreadState is drop-safe (no locked mutexes).
    pub proof fn lemma_new_is_drop_safe(
        id: ThreadIdentifier,
        has_kernel_stack: bool,
        has_user_stack: bool,
        user_tda: Option<int>,
    )
        ensures
            ({
                let s: ThreadState = ThreadState {
                    id: id,
                    has_kernel_stack: has_kernel_stack,
                    has_user_stack: has_user_stack,
                    user_tda: user_tda,
                    interrupt_reason: None,
                    locked_mutex_count: 0,
                };
                s.spec_drop_safe()
            }),
    {
    }

    /// Lemma: A newly constructed ThreadState has no interrupt reason.
    pub proof fn lemma_new_not_interrupted(
        id: ThreadIdentifier,
        has_kernel_stack: bool,
        has_user_stack: bool,
        user_tda: Option<int>,
    )
        ensures
            ({
                let s: ThreadState = ThreadState {
                    id: id,
                    has_kernel_stack: has_kernel_stack,
                    has_user_stack: has_user_stack,
                    user_tda: user_tda,
                    interrupt_reason: None,
                    locked_mutex_count: 0,
                };
                !s.spec_is_interrupted()
            }),
    {
    }

    //==============================================================================================
    // ID Immutability Lemmas
    //==============================================================================================

    /// Lemma: take_kernel_stack preserves the thread identifier.
    pub proof fn lemma_take_kernel_stack_preserves_id(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_kernel_stack: false,
                    ..*self
                };
                post.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: take_user_stack preserves the thread identifier.
    pub proof fn lemma_take_user_stack_preserves_id(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_user_stack: false,
                    ..*self
                };
                post.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: set_interrupt_reason preserves the thread identifier.
    pub proof fn lemma_set_interrupt_reason_preserves_id(&self, reason: int)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: Some(reason),
                    ..*self
                };
                post.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: take_interrupt_reason preserves the thread identifier.
    pub proof fn lemma_take_interrupt_reason_preserves_id(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..*self
                };
                post.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: store_thread_data_area preserves the thread identifier.
    pub proof fn lemma_store_tda_preserves_id(&self, tda: Option<int>)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    user_tda: tda,
                    ..*self
                };
                post.spec_id() == self.spec_id()
            }),
    {
    }

    //==============================================================================================
    // Option Take/Store Semantics
    //==============================================================================================

    /// Lemma: After take_kernel_stack, the kernel stack is absent.
    pub proof fn lemma_take_kernel_stack_clears(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_kernel_stack: false,
                    ..*self
                };
                !post.spec_has_kernel_stack()
            }),
    {
    }

    /// Lemma: After take_user_stack, the user stack is absent.
    pub proof fn lemma_take_user_stack_clears(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_user_stack: false,
                    ..*self
                };
                !post.spec_has_user_stack()
            }),
    {
    }

    /// Lemma: After set_interrupt_reason, the thread is interrupted.
    pub proof fn lemma_set_interrupt_reason_is_interrupted(&self, reason: int)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: Some(reason),
                    ..*self
                };
                post.spec_is_interrupted()
            }),
    {
    }

    /// Lemma: After take_interrupt_reason, the thread is not interrupted.
    pub proof fn lemma_take_interrupt_reason_clears(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..*self
                };
                !post.spec_is_interrupted()
            }),
    {
    }

    /// Lemma: set then take interrupt_reason round-trips: restores None.
    pub proof fn lemma_interrupt_reason_roundtrip(&self, reason: int)
        ensures
            ({
                let mid: ThreadState = ThreadState {
                    interrupt_reason: Some(reason),
                    ..*self
                };
                let post: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..mid
                };
                post.spec_interrupt_reason() == self.spec_interrupt_reason()
                    || self.spec_interrupt_reason().is_some()
            }),
    {
    }

    /// Lemma: set then take interrupt_reason on a state with no prior reason yields None.
    pub proof fn lemma_interrupt_reason_set_take_on_none(&self, reason: int)
        requires
            !self.spec_is_interrupted(),
        ensures
            ({
                let mid: ThreadState = ThreadState {
                    interrupt_reason: Some(reason),
                    ..*self
                };
                let post: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..mid
                };
                !post.spec_is_interrupted()
            }),
    {
    }

    //==============================================================================================
    // Mutex Guard Lemmas
    //==============================================================================================

    /// Lemma: Storing a mutex guard increments the locked mutex count.
    pub proof fn lemma_store_mutex_guard_increments(&self)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
        ensures
            ({
                let post: ThreadState = ThreadState {
                    locked_mutex_count: (self.locked_mutex_count + 1) as usize,
                    ..*self
                };
                post.spec_locked_mutex_count() == self.spec_locked_mutex_count() + 1
            }),
    {
    }

    /// Lemma: Taking a mutex guard decrements the locked mutex count (when count > 0).
    pub proof fn lemma_take_mutex_guard_decrements(&self)
        requires
            self.wf(),
            self.spec_locked_mutex_count() > 0,
        ensures
            ({
                let post: ThreadState = ThreadState {
                    locked_mutex_count: (self.locked_mutex_count - 1) as usize,
                    ..*self
                };
                post.spec_locked_mutex_count() == self.spec_locked_mutex_count() - 1
            }),
    {
    }

    /// Lemma: A state with zero locked mutexes is drop-safe.
    pub proof fn lemma_zero_mutexes_is_drop_safe(&self)
        requires
            self.spec_locked_mutex_count() == 0,
        ensures
            self.spec_drop_safe(),
    {
    }

    /// Lemma: A state with nonzero locked mutexes is not drop-safe.
    pub proof fn lemma_nonzero_mutexes_not_drop_safe(&self)
        requires
            self.spec_locked_mutex_count() > 0,
        ensures
            !self.spec_drop_safe(),
    {
    }

    //==============================================================================================
    // Well-Formedness Preservation
    //==============================================================================================

    /// Lemma: take_kernel_stack preserves well-formedness.
    pub proof fn lemma_take_kernel_stack_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_kernel_stack: false,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: take_user_stack preserves well-formedness.
    pub proof fn lemma_take_user_stack_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    has_user_stack: false,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: set_interrupt_reason preserves well-formedness.
    pub proof fn lemma_set_interrupt_reason_preserves_wf(&self, reason: int)
        requires
            self.wf(),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: Some(reason),
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: take_interrupt_reason preserves well-formedness.
    pub proof fn lemma_take_interrupt_reason_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: store_thread_data_area preserves well-formedness.
    pub proof fn lemma_store_tda_preserves_wf(&self, tda: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    user_tda: tda,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: store_mutex_guard preserves well-formedness.
    pub proof fn lemma_store_mutex_guard_preserves_wf(&self)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
        ensures
            ({
                let post: ThreadState = ThreadState {
                    locked_mutex_count: (self.locked_mutex_count + 1) as usize,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: take_mutex_guard preserves well-formedness (when count > 0).
    pub proof fn lemma_take_mutex_guard_preserves_wf(&self)
        requires
            self.wf(),
            self.spec_locked_mutex_count() > 0,
        ensures
            ({
                let post: ThreadState = ThreadState {
                    locked_mutex_count: (self.locked_mutex_count - 1) as usize,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    //==============================================================================================
    // Thread Data Area Lemmas
    //==============================================================================================

    /// Lemma: store then get thread data area round-trips.
    pub proof fn lemma_tda_store_get_roundtrip(&self, tda: Option<int>)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    user_tda: tda,
                    ..*self
                };
                post.spec_user_tda() == tda
            }),
    {
    }

    //==============================================================================================
    // View Equality Lemmas
    //==============================================================================================

    /// Lemma: Two ThreadStates with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ThreadState, b: &ThreadState)
        requires
            a.id.spec_value() == b.id.spec_value(),
            a.has_kernel_stack == b.has_kernel_stack,
            a.has_user_stack == b.has_user_stack,
            a.user_tda == b.user_tda,
            a.interrupt_reason == b.interrupt_reason,
            a.locked_mutex_count == b.locked_mutex_count,
        ensures
            a@ == b@,
    {
    }
}

} // verus!
