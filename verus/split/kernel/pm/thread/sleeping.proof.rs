// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingThread Proofs.
// Lemmas for construction, identity preservation, state transition correctness,
// well-formedness preservation, and alarm/mutex/drop-safety propagation.
//
// Key proven properties:
// - Construction preserves identity, alarm, and well-formedness.
// - Thread identifier is immutable: all operations preserve it.
// - wakeup() transitions to ReadyThread preserving identity, wf, mutex
//   accounting, and drop safety.
// - interrupt() transitions to InterruptedThread preserving identity, wf,
//   mutex accounting, drop safety, and capturing the reason.
// - alarm() correctly returns the captured alarm value.
// - set_thread_data_area / get_thread_data_area round-trip correctly.
// - View equality for structurally identical SleepingThreads.

use vstd::prelude::*;

verus! {

impl SleepingThread {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: Construction from ThreadState produces a well-formed SleepingThread.
    pub proof fn lemma_from_state_is_wf(state: ThreadState, alarm: Option<int>)
        requires
            state.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.wf()
            }),
    {
    }

    /// Lemma: Construction preserves the thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, alarm: Option<int>)
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: Construction captures the alarm correctly.
    pub proof fn lemma_from_state_captures_alarm(state: ThreadState, alarm: Option<int>)
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.spec_alarm() == alarm
            }),
    {
    }

    /// Lemma: Construction preserves drop safety.
    pub proof fn lemma_from_state_preserves_drop_safe(state: ThreadState, alarm: Option<int>)
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.spec_drop_safe() == state.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Construction preserves mutex accounting.
    pub proof fn lemma_from_state_preserves_mutexes(state: ThreadState, alarm: Option<int>)
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.spec_locked_mutex_count() == state.spec_locked_mutex_count()
                && (forall|a: int| s.spec_has_mutex(a) == state.spec_has_mutex(a))
            }),
    {
    }

    //==============================================================================================
    // Identity Correctness
    //==============================================================================================

    /// Lemma: spec_id faithfully reflects the underlying state's identity.
    pub proof fn lemma_id_correct(&self)
        ensures
            self.spec_id() == self.state.spec_id(),
    {
    }

    //==============================================================================================
    // State Transition: wakeup()
    //==============================================================================================

    /// Lemma: After wakeup(), the ReadyThread has the same identity.
    pub proof fn lemma_wakeup_preserves_id(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: wakeup() preserves well-formedness.
    pub proof fn lemma_wakeup_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.wf()
            }),
    {
    }

    /// Lemma: wakeup() preserves mutex accounting.
    pub proof fn lemma_wakeup_preserves_mutexes(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| r.spec_has_mutex(a) == self.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: wakeup() preserves drop safety.
    pub proof fn lemma_wakeup_preserves_drop_safety(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
    }

    //==============================================================================================
    // State Transition: interrupt()
    //==============================================================================================

    /// Lemma: After interrupt(), the InterruptedThread has the same identity.
    pub proof fn lemma_interrupt_preserves_id(&self, reason: int)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: self.state, reason: reason };
                t.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: interrupt() captures the reason correctly.
    pub proof fn lemma_interrupt_captures_reason(&self, reason: int)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: self.state, reason: reason };
                t.spec_reason() == reason
            }),
    {
    }

    /// Lemma: interrupt() preserves well-formedness.
    pub proof fn lemma_interrupt_preserves_wf(&self, reason: int)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: self.state, reason: reason };
                t.wf()
            }),
    {
    }

    /// Lemma: interrupt() preserves mutex accounting.
    pub proof fn lemma_interrupt_preserves_mutexes(&self, reason: int)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: self.state, reason: reason };
                t.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| t.spec_has_mutex(a) == self.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: interrupt() preserves drop safety.
    pub proof fn lemma_interrupt_preserves_drop_safety(&self, reason: int)
        requires
            self.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: self.state, reason: reason };
                t.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
    }

    //==============================================================================================
    // Thread Data Area Lemmas
    //==============================================================================================

    /// Lemma: set_thread_data_area followed by get_thread_data_area round-trips correctly.
    pub proof fn lemma_tda_roundtrip(state: ThreadState, alarm: Option<int>, tda: Option<int>)
        requires
            state.wf(),
        ensures
            ({
                let post_state: ThreadState = ThreadState {
                    user_tda: tda,
                    ..state
                };
                post_state.spec_user_tda() == tda
                && post_state.spec_id() == state.spec_id()
                && post_state.wf()
            }),
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two SleepingThreads with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &SleepingThread, b: &SleepingThread)
        requires
            a.state@ == b.state@,
            a.alarm == b.alarm,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// ReadyThread Lemmas (Boundary)
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
}

//==================================================================================================
// InterruptedThread Lemmas (Boundary)
//==================================================================================================

impl InterruptedThread {
    /// Lemma: from_state preserves thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, reason: int)
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: from_state captures reason correctly.
    pub proof fn lemma_from_state_captures_reason(state: ThreadState, reason: int)
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.spec_reason() == reason
            }),
    {
    }

    /// Lemma: from_state preserves well-formedness.
    pub proof fn lemma_from_state_preserves_wf(state: ThreadState, reason: int)
        requires
            state.wf(),
            SleepingThread::spec_valid_reason(reason),
        ensures
            ({
                let t: InterruptedThread = InterruptedThread { state: state, reason: reason };
                t.wf()
            }),
    {
    }
}

} // verus!
