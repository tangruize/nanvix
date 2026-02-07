// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedThread Proofs.
// Lemmas for construction, identity preservation, state transition correctness,
// well-formedness preservation, and mutex/drop-safety propagation.
//
// Key proven properties:
// - Construction produces well-formed state with correct identity and reason.
// - Thread identifier is immutable: all operations preserve it.
// - resume() correctly stamps the interrupt reason onto the ThreadState.
// - resume() preserves thread identity across the state transition.
// - resume() preserves well-formedness, mutex accounting, drop safety, and stacks.
// - Killed and TimedOut reason constants are distinct.
// - Well-formed reasons are exhaustively one of the two valid variants.
// - ReadyThread::from_state preserves identity and well-formedness.
// - View equality: structurally identical InterruptedThreads have equal views.

use vstd::prelude::*;

verus! {

impl InterruptedThread {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: Construction produces a well-formed InterruptedThread.
    pub proof fn lemma_from_state_wf(state: ThreadState, reason: int)
        requires
            state.wf(),
            InterruptedThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.wf()
            }),
    {
    }

    /// Lemma: Construction preserves the thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, reason: int)
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: Construction captures the reason correctly.
    pub proof fn lemma_from_state_captures_reason(state: ThreadState, reason: int)
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.spec_reason() == reason
            }),
    {
    }

    //==============================================================================================
    // Identity Correctness Lemma
    //==============================================================================================

    /// Lemma: spec_id faithfully reflects the underlying state's identity.
    pub proof fn lemma_id_correct(&self)
        ensures
            self.spec_id() == self.state.spec_id(),
    {
    }

    //==============================================================================================
    // Resume State Transition Lemmas
    //==============================================================================================

    // NOTE: The resume lemmas below construct the post-state manually as
    //   `ThreadState { interrupt_reason: Some(self.reason), ..self.state }`
    // mirroring the exec `resume` implementation. If the exec `resume` code
    // changes (e.g., modifies additional fields), these lemmas must be
    // updated in sync to remain meaningful.

    /// Lemma: After resume, the resulting state has the interrupt reason set
    /// to the InterruptedThread's reason. This is the key safety property:
    /// the interrupt reason is correctly propagated from the InterruptedThread
    /// to the underlying ThreadState during the resume transition.
    pub proof fn lemma_resume_sets_reason(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.spec_interrupt_reason() == Some(self.spec_reason())
                && post_state.spec_is_interrupted()
            }),
    {
    }

    /// Lemma: After resume, the thread identity is preserved.
    pub proof fn lemma_resume_preserves_id(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: After resume, well-formedness is preserved.
    pub proof fn lemma_resume_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.wf()
            }),
    {
    }

    /// Lemma: Resume preserves the mutex accounting (count and per-address membership).
    pub proof fn lemma_resume_preserves_mutexes(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| #![auto] post_state.spec_has_mutex(a) == self.state.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: Resume preserves drop safety when the source state is drop-safe.
    pub proof fn lemma_resume_preserves_drop_safety(&self)
        requires
            self.wf(),
            self.spec_drop_safe(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Resume preserves kernel and user stack ownership.
    pub proof fn lemma_resume_preserves_stacks(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: Some(self.reason),
                    ..self.state
                };
                post_state.spec_kernel_stack() == self.spec_kernel_stack()
                && post_state.spec_user_stack() == self.spec_user_stack()
            }),
    {
    }

    //==============================================================================================
    // Reason Variant Lemmas
    //==============================================================================================

    /// Lemma: Killed and TimedOut reason constants are distinct.
    pub proof fn lemma_reasons_distinct()
        ensures
            INTERRUPT_REASON_KILLED() != INTERRUPT_REASON_TIMED_OUT(),
    {
    }

    /// Lemma: A valid reason is either Killed or TimedOut (exhaustive and exclusive).
    pub proof fn lemma_valid_reason_exhaustive(reason: int)
        requires
            InterruptedThread::spec_valid_reason(reason),
        ensures
            reason == INTERRUPT_REASON_KILLED() || reason == INTERRUPT_REASON_TIMED_OUT(),
            !(reason == INTERRUPT_REASON_KILLED() && reason == INTERRUPT_REASON_TIMED_OUT()),
    {
    }

    //==============================================================================================
    // View Equality Lemma
    //==============================================================================================

    /// Lemma: Two InterruptedThreads with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &InterruptedThread, b: &InterruptedThread)
        requires
            a.state@ == b.state@,
            a.reason == b.reason,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// ReadyThread Lemmas
//==================================================================================================

impl ReadyThread {
    /// Lemma: from_state preserves thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState)
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state };
                r.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: from_state preserves well-formedness.
    pub proof fn lemma_from_state_preserves_wf(state: ThreadState)
        requires
            state.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state };
                r.wf()
            }),
    {
    }

    /// Lemma: from_state preserves interrupt reason.
    pub proof fn lemma_from_state_preserves_interrupt_reason(state: ThreadState)
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state };
                r.spec_interrupt_reason() == state.spec_interrupt_reason()
            }),
    {
    }
}

} // verus!
