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
// - Mutex guard store inserts address into concrete set; take removes it.
// - Drop safety: newly constructed state is drop-safe (no locked mutexes).
// - Interrupt reason set/take follows Option semantics with non-vacuous round-trip.
// - Resource tracking: taking both stacks removes all resources.

// NOTE: imports are in state.rs (this file is included via include!()).

verus! {

impl ThreadState {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed ThreadState is well-formed.
    pub proof fn lemma_new_is_wf(s: &ThreadState)
        requires
            s.locked_mutex_count == 0usize,
            s.locked_mutex_set@.len() == 0,
            s.locked_mutex_set@.no_duplicates(),
            s.interrupt_reason.is_none(),
        ensures
            s.wf(),
    {
    }

    /// Lemma: A newly constructed ThreadState is drop-safe (no locked mutexes).
    pub proof fn lemma_new_is_drop_safe(s: &ThreadState)
        requires
            s.locked_mutex_count == 0usize,
            s.locked_mutex_set@.len() == 0,
        ensures
            s.spec_drop_safe(),
    {
    }

    /// Lemma: A newly constructed ThreadState has no interrupt reason.
    pub proof fn lemma_new_not_interrupted(s: &ThreadState)
        requires
            s.interrupt_reason.is_none(),
        ensures
            !s.spec_is_interrupted(),
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
                    kernel_stack: None,
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
                    user_stack: None,
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
                    kernel_stack: None,
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
                    user_stack: None,
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

    /// Lemma: set then take interrupt_reason round-trips correctly.
    /// After set(reason) then take, the interrupt reason is cleared (None)
    /// and the intermediate state held exactly the given reason.
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
                post.spec_interrupt_reason().is_none()
                && mid.spec_interrupt_reason() == Some(reason)
                && post.spec_id() == self.spec_id()
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

    /// Lemma: Pushing a new address to the Vec inserts it into the abstract set.
    pub proof fn lemma_store_mutex_guard_increments(&self, address: u64)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
            !self.spec_has_mutex(address as int),
        ensures
            seq_to_set(self.locked_mutex_set@.push(address)).contains(address as int),
            self.locked_mutex_set@.push(address).no_duplicates(),
            seq_to_set(self.locked_mutex_set@.push(address)).len()
                == seq_to_set(self.locked_mutex_set@).len() + 1,
    {
        // push(v) gives s.push(v), and seq_to_set(s.push(v))
        // = seq_to_set(s).insert(v as int) by definition.
        assert(self.locked_mutex_set@.push(address).drop_last() =~= self.locked_mutex_set@);
        if self.locked_mutex_set@.contains(address) {
            lemma_seq_to_set_contains_fwd(self.locked_mutex_set@, address);
        }
        assert(!self.locked_mutex_set@.contains(address));
        assert(self.locked_mutex_set@.push(address).no_duplicates());
        lemma_seq_to_set_len(self.locked_mutex_set@.push(address));
        lemma_seq_to_set_len(self.locked_mutex_set@);
    }

    /// Lemma: Pushing a new address preserves membership of other addresses.
    pub proof fn lemma_store_mutex_guard_preserves_others(&self, address: u64, other: int)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
            !self.spec_has_mutex(address as int),
            other != address as int,
        ensures
            seq_to_set(self.locked_mutex_set@.push(address)).contains(other)
                == seq_to_set(self.locked_mutex_set@).contains(other),
    {
        assert(self.locked_mutex_set@.push(address).drop_last() =~= self.locked_mutex_set@);
    }

    /// Lemma: Taking a mutex guard removes the address and decrements count.
    pub proof fn lemma_take_mutex_guard_decrements(&self, address: u64)
        requires
            self.wf(),
            self.spec_has_mutex(address as int),
        ensures
            ({
                // Find the index of address in the seq.
                let idx: int = choose |k: int|
                    0 <= k < self.locked_mutex_set@.len()
                    && self.locked_mutex_set@[k] == address;
                let new_seq: Seq<u64> = self.locked_mutex_set@.remove(idx);
                !seq_to_set(new_seq).contains(address as int)
                && new_seq.no_duplicates()
            }),
    {
        lemma_seq_to_set_contains_rev(self.locked_mutex_set@, address);
        let idx: int = choose |k: int|
            0 <= k < self.locked_mutex_set@.len()
            && self.locked_mutex_set@[k] == address;
        lemma_seq_to_set_remove(self.locked_mutex_set@, idx);
    }

    /// Lemma: Taking a mutex guard preserves membership of other addresses.
    pub proof fn lemma_take_mutex_guard_preserves_others(&self, address: u64, other: int)
        requires
            self.wf(),
            self.spec_has_mutex(address as int),
            other != address as int,
        ensures
            ({
                let idx: int = choose |k: int|
                    0 <= k < self.locked_mutex_set@.len()
                    && self.locked_mutex_set@[k] == address;
                let new_seq: Seq<u64> = self.locked_mutex_set@.remove(idx);
                seq_to_set(new_seq).contains(other)
                    == seq_to_set(self.locked_mutex_set@).contains(other)
            }),
    {
        lemma_seq_to_set_contains_rev(self.locked_mutex_set@, address);
        let idx: int = choose |k: int|
            0 <= k < self.locked_mutex_set@.len()
            && self.locked_mutex_set@[k] == address;
        lemma_seq_to_set_remove(self.locked_mutex_set@, idx);
    }

    /// Lemma: A well-formed state with zero locked mutexes is drop-safe.
    pub proof fn lemma_zero_mutexes_is_drop_safe(&self)
        requires
            self.wf(),
            self.spec_locked_mutex_count() == 0,
        ensures
            self.spec_drop_safe(),
    {
    }

    /// Lemma: A well-formed state with nonzero locked mutexes is not drop-safe.
    pub proof fn lemma_nonzero_mutexes_not_drop_safe(&self)
        requires
            self.wf(),
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
                    kernel_stack: None,
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
                    user_stack: None,
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
    pub proof fn lemma_store_mutex_guard_preserves_wf(&self, address: u64)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
            !self.spec_has_mutex(address as int),
        ensures
            self.locked_mutex_set@.push(address).no_duplicates(),
            self.locked_mutex_set@.push(address).len()
                == self.locked_mutex_count as nat + 1,
    {
        if self.locked_mutex_set@.contains(address) {
            lemma_seq_to_set_contains_fwd(self.locked_mutex_set@, address);
        }
    }

    /// Lemma: take_mutex_guard preserves well-formedness (when address is held).
    pub proof fn lemma_take_mutex_guard_preserves_wf(&self, address: u64)
        requires
            self.wf(),
            self.spec_has_mutex(address as int),
        ensures
            ({
                let idx: int = choose |k: int|
                    0 <= k < self.locked_mutex_set@.len()
                    && self.locked_mutex_set@[k] == address;
                let new_seq: Seq<u64> = self.locked_mutex_set@.remove(idx);
                new_seq.no_duplicates()
                && new_seq.len() == self.locked_mutex_count as nat - 1
            }),
    {
        lemma_seq_to_set_contains_rev(self.locked_mutex_set@, address);
        let idx: int = choose |k: int|
            0 <= k < self.locked_mutex_set@.len()
            && self.locked_mutex_set@[k] == address;
        lemma_seq_to_set_remove(self.locked_mutex_set@, idx);
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
    // Resource Tracking Lemmas
    //==============================================================================================

    /// Lemma: After taking both stacks, the thread has no resources.
    pub proof fn lemma_take_stacks_removes_resources(&self)
        ensures
            ({
                let post: ThreadState = ThreadState {
                    kernel_stack: None,
                    user_stack: None,
                    ..*self
                };
                !post.spec_has_resources()
            }),
    {
    }

    //==============================================================================================
    // Stack Identity Lemmas
    //==============================================================================================

    /// Lemma: take_kernel_stack returns exactly what was stored.
    pub proof fn lemma_take_kernel_stack_identity(&self, ks: int)
        requires
            self.spec_kernel_stack() == Some(ks),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    kernel_stack: None,
                    ..*self
                };
                // The taken value is the original.
                self.spec_kernel_stack() == Some(ks)
                // After take, it is gone.
                && post.spec_kernel_stack().is_none()
            }),
    {
    }

    /// Lemma: take_user_stack returns exactly what was stored.
    pub proof fn lemma_take_user_stack_identity(&self, us: int)
        requires
            self.spec_user_stack() == Some(us),
        ensures
            ({
                let post: ThreadState = ThreadState {
                    user_stack: None,
                    ..*self
                };
                // The taken value is the original.
                self.spec_user_stack() == Some(us)
                // After take, it is gone.
                && post.spec_user_stack().is_none()
            }),
    {
    }

    //==============================================================================================
    // Drop Safety Lemmas
    //==============================================================================================

    /// Lemma: A well-formed, drop-safe state has an empty mutex set.
    /// This connects `spec_drop_safe()` to the underlying set emptiness,
    /// formalizing the verification-side encoding of the original
    /// `Drop::drop()` check `self.locked_mutexes.is_empty()`.
    pub proof fn lemma_drop_safe_means_no_mutexes(&self)
        requires
            self.wf(),
            self.spec_drop_safe(),
        ensures
            self.spec_locked_mutex_count() == 0,
            forall|addr: int| !self.spec_has_mutex(addr),
    {
        // spec_drop_safe: locked_mutex_set@.len() == 0, so seq is empty.
        assert(self.locked_mutex_set@ =~= Seq::<u64>::empty());
        // seq_to_set of empty is empty.
        assert(seq_to_set(self.locked_mutex_set@) =~= Set::<int>::empty());
    }

    /// Lemma: `check_drop_safe()` faithfully models `Drop::drop()`.
    ///
    /// In the original code, `Drop::drop()` checks
    /// `!self.locked_mutexes.is_empty()` and logs an error if true.
    /// This lemma proves that under well-formedness, the runtime
    /// `check_drop_safe()` (count == 0) is equivalent to the spec-level
    /// `spec_drop_safe()` (Vec empty), which in turn is
    /// equivalent to the original `locked_mutexes.is_empty()` check.
    pub proof fn lemma_check_drop_safe_models_drop(&self)
        requires
            self.wf(),
        ensures
            (self.locked_mutex_count == 0) == self.spec_drop_safe(),
    {
    }

    //==============================================================================================
    // Mutex Roundtrip Lemmas
    //==============================================================================================

    /// Lemma: store then take of the same mutex address is a no-op on the
    /// mutex set (returns to the original set state).
    ///
    /// This proves the internal consistency of the mutex guard operations
    /// within the trust boundary established by T1/T2.
    pub proof fn lemma_mutex_store_take_roundtrip(&self, address: u64)
        requires
            self.wf(),
            self.locked_mutex_count < usize::MAX,
            !self.spec_has_mutex(address as int),
        ensures
            ({
                let pushed: Seq<u64> = self.locked_mutex_set@.push(address);
                // The pushed seq has address at the last position.
                // Removing the last element restores the original seq.
                pushed.remove(pushed.len() as int - 1) =~= self.locked_mutex_set@
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
            a.kernel_stack == b.kernel_stack,
            a.user_stack == b.user_stack,
            a.user_tda == b.user_tda,
            a.interrupt_reason == b.interrupt_reason,
            a.locked_mutex_count == b.locked_mutex_count,
            seq_to_set(a.locked_mutex_set@) =~= seq_to_set(b.locked_mutex_set@),
        ensures
            a@ == b@,
    {
    }
}

} // verus!
