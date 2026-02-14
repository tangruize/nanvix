// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ReadyThread Proofs.
// Lemmas for construction, identity preservation, state transition correctness,
// well-formedness preservation, and scheduling properties.
//
// Key proven properties:
// - Construction preserves identity and well-formedness.
// - from_state preserves all ThreadState properties.
// - run() extracts interrupt reason, clears it from RunningThread, preserves identity.
// - run() preserves well-formedness, mutex accounting, and drop safety.
// - terminate() sets Interrupted exit status, preserves identity and well-formedness.
// - View equality for structurally identical ReadyThreads.

use vstd::prelude::*;

verus! {

impl ReadyThread {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: Construction from ThreadState produces a well-formed ReadyThread.
    pub proof fn lemma_from_state_is_wf(state: ThreadState, time: int)
        requires
            state.wf(),
            time >= 0,
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state, admission_time: time };
                r.wf()
            }),
    {
        reveal(ReadyThread::wf);
    }

    /// Lemma: Construction preserves the thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, time: int)
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state, admission_time: time };
                r.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: Construction preserves drop safety (view-level).
    pub proof fn lemma_from_state_preserves_drop_safe(state: ThreadState, time: int)
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state, admission_time: time };
                r.spec_drop_safe() == state@.drop_safe()
            }),
    {
    }

    /// Lemma: Construction captures the admission time.
    pub proof fn lemma_from_state_captures_time(state: ThreadState, time: int)
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: state, admission_time: time };
                r.spec_admission_time() == time
            }),
    {
    }

    /// Lemma: A freshly constructed ReadyThread (with empty state) is drop-safe.
    pub proof fn lemma_new_is_drop_safe(r: &ReadyThread)
        requires
            r.state.locked_mutex_count == 0usize,
            r.state.locked_mutex_set@.len() == 0,
            r.state.locked_mutex_set@.no_duplicates(),
            r.state.interrupt_reason.is_none(),
            r.admission_time >= 0,
        ensures
            r.spec_drop_safe() && r.wf() && !r.spec_is_interrupted(),
    {
        reveal(ReadyThread::wf);
        ThreadState::lemma_new_is_wf(&r.state);
    }

    //==============================================================================================

    /// Lemma: spec_id faithfully reflects the underlying state's identity.
    pub proof fn lemma_id_correct(&self)
        ensures
            self.spec_id() == self.state.spec_id(),
    {
    }

    //==============================================================================================
    // State Transition: run()
    //==============================================================================================

    /// Lemma: After run(), the RunningThread has the same identity.
    pub proof fn lemma_run_preserves_id(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..self.state
                };
                post_state.spec_id() == self.spec_id()
            }),
    {
        reveal(ReadyThread::wf);
    }

    /// Lemma: After run(), the RunningThread's state has no interrupt reason.
    pub proof fn lemma_run_clears_interrupt(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..self.state
                };
                !post_state.spec_is_interrupted()
            }),
    {
        reveal(ReadyThread::wf);
    }

    /// Lemma: run() preserves well-formedness through the state transition.
    pub proof fn lemma_run_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..self.state
                };
                post_state.wf()
            }),
    {
        reveal(ReadyThread::wf);
        self.state.lemma_take_interrupt_reason_preserves_wf();
    }

    /// Lemma: run() preserves mutex accounting.
    pub proof fn lemma_run_preserves_mutexes(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..self.state
                };
                post_state.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| post_state.spec_has_mutex(a) == self.spec_has_mutex(a))
            }),
    {
        reveal(ReadyThread::wf);
    }

    /// Lemma: run() preserves drop safety.
    pub proof fn lemma_run_preserves_drop_safety(&self)
        requires
            self.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..self.state
                };
                post_state@.drop_safe() == self.spec_drop_safe()
            }),
    {
        reveal(ReadyThread::wf);
    }

    //==============================================================================================
    // State Transition: terminate()
    //==============================================================================================

    /// Lemma: terminate() produces a ZombieThread with the same identity.
    pub proof fn lemma_terminate_preserves_id(&self)
        ensures
            ({
                let z: ZombieThread = ZombieThread {
                    state: self.state,
                    status: EXIT_STATUS_INTERRUPTED(),
                };
                z.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: terminate() produces a ZombieThread with Interrupted status.
    pub proof fn lemma_terminate_sets_status(&self)
        ensures
            ({
                let z: ZombieThread = ZombieThread {
                    state: self.state,
                    status: EXIT_STATUS_INTERRUPTED(),
                };
                z.spec_status() == EXIT_STATUS_INTERRUPTED()
            }),
    {
    }

    /// Lemma: terminate() preserves well-formedness.
    pub proof fn lemma_terminate_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread {
                    state: self.state,
                    status: EXIT_STATUS_INTERRUPTED(),
                };
                z.wf()
            }),
    {
        reveal(ReadyThread::wf);
        reveal(ZombieThread::wf);
    }

    /// Lemma: terminate() preserves drop safety.
    pub proof fn lemma_terminate_preserves_drop_safety(&self)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread {
                    state: self.state,
                    status: EXIT_STATUS_INTERRUPTED(),
                };
                z.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
        reveal(ReadyThread::wf);
    }

    //==============================================================================================
    // Composite Lemmas
    //==============================================================================================

    /// Lemma: Construction followed by run() produces a RunningThread with the
    /// original identity, no interrupt, and preserved well-formedness.
    ///
    /// Exercises the composition of new() + run() specifications.
    pub proof fn lemma_new_then_run(r: &ReadyThread)
        requires
            r.state.locked_mutex_count == 0usize,
            r.state.locked_mutex_set@.len() == 0,
            r.state.locked_mutex_set@.no_duplicates(),
            r.state.interrupt_reason.is_none(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    interrupt_reason: None,
                    ..r.state
                };
                // Identity preserved through new() + run().
                post_state.spec_id() == r.state.id.spec_value()
                // Interrupt cleared (was already None, stays None).
                && !post_state.spec_is_interrupted()
                // Well-formedness preserved.
                && post_state.wf()
                // Drop safety preserved (new thread has no mutexes).
                && post_state.spec_drop_safe()
            }),
    {
        ThreadState::lemma_new_is_wf(&r.state);
        r.state.lemma_take_interrupt_reason_preserves_wf();
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ReadyThreads with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ReadyThread, b: &ReadyThread)
        requires
            a.state@ == b.state@,
            a.admission_time == b.admission_time,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// RunningThread Lemmas (Boundary)
//==================================================================================================

impl RunningThread {
    /// Lemma: from_state preserves thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState)
        ensures
            ({
                let r: RunningThread = RunningThread { state: state };
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
                let r: RunningThread = RunningThread { state: state };
                r.wf()
            }),
    {
        reveal(RunningThread::wf);
    }
}

//==================================================================================================
// ZombieThread Lemmas (Boundary)
//==================================================================================================

impl ZombieThread {
    /// Lemma: from_state preserves thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: from_state captures status correctly.
    pub proof fn lemma_from_state_captures_status(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_status() == status
            }),
    {
    }

    /// Lemma: from_state preserves well-formedness.
    pub proof fn lemma_from_state_preserves_wf(state: ThreadState, status: int)
        requires
            state.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.wf()
            }),
    {
        reveal(ZombieThread::wf);
    }
}

} // verus!
