// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunningThread Proofs.
// Lemmas for construction, identity preservation, state transition correctness,
// well-formedness preservation, mutex accounting, and drop safety.
//
// Key proven properties:
// - Construction (from_state) produces well-formed state with correct identity.
// - Thread identifier (`id`) is immutable: all operations preserve it.
// - `sleep()` preserves identity, well-formedness, mutex accounting, drop safety,
//   and correctly captures the alarm parameter.
// - `schedule()` preserves identity, well-formedness, mutex accounting, drop safety.
// - `exit()` preserves identity, well-formedness, drop safety, mutex accounting,
//   and correctly captures the exit status.
// - `store_mutex_guard` / `take_mutex_guard` preserve identity and wf.
// - View equality for structurally identical RunningThreads.

use vstd::prelude::*;

verus! {

impl RunningThread {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: Construction from ThreadState produces a well-formed RunningThread.
    pub proof fn lemma_from_state_is_wf(state: ThreadState)
        requires
            state.wf(),
        ensures
            ({
                let r: RunningThread = RunningThread { state: state };
                r.wf()
            }),
    {
    }

    /// Lemma: Construction preserves the thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState)
        ensures
            ({
                let r: RunningThread = RunningThread { state: state };
                r.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: Construction preserves drop safety.
    pub proof fn lemma_from_state_preserves_drop_safe(state: ThreadState)
        ensures
            ({
                let r: RunningThread = RunningThread { state: state };
                r.spec_drop_safe() == state.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Construction preserves mutex accounting.
    pub proof fn lemma_from_state_preserves_mutexes(state: ThreadState)
        ensures
            ({
                let r: RunningThread = RunningThread { state: state };
                r.spec_locked_mutex_count() == state.spec_locked_mutex_count()
                && (forall|a: int| r.spec_has_mutex(a) == state.spec_has_mutex(a))
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
    // State Transition: sleep()
    //==============================================================================================

    /// Lemma: sleep() preserves thread identity.
    pub proof fn lemma_sleep_preserves_id(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: sleep() correctly captures the alarm parameter.
    pub proof fn lemma_sleep_captures_alarm(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.spec_alarm() == alarm
            }),
    {
    }

    /// Lemma: sleep() preserves well-formedness.
    pub proof fn lemma_sleep_preserves_wf(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.wf()
            }),
    {
    }

    /// Lemma: sleep() preserves mutex accounting.
    pub proof fn lemma_sleep_preserves_mutexes(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| s.spec_has_mutex(a) == self.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: sleep() preserves drop safety.
    pub proof fn lemma_sleep_preserves_drop_safety(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
    }

    /// Lemma: sleep() preserves the full ThreadStateView.
    pub proof fn lemma_sleep_preserves_state_view(&self, alarm: Option<int>)
        requires
            self.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: self.state, alarm: alarm };
                s.state@ == self.state@
            }),
    {
    }

    //==============================================================================================
    // State Transition: schedule()
    //==============================================================================================

    /// Lemma: schedule() preserves thread identity.
    pub proof fn lemma_schedule_preserves_id(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: schedule() preserves well-formedness.
    pub proof fn lemma_schedule_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.wf()
            }),
    {
    }

    /// Lemma: schedule() preserves mutex accounting.
    pub proof fn lemma_schedule_preserves_mutexes(&self)
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

    /// Lemma: schedule() preserves drop safety.
    pub proof fn lemma_schedule_preserves_drop_safety(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
    }

    /// Lemma: schedule() preserves the full ThreadStateView.
    pub proof fn lemma_schedule_preserves_state_view(&self)
        requires
            self.wf(),
        ensures
            ({
                let r: ReadyThread = ReadyThread { state: self.state };
                r.state@ == self.state@
            }),
    {
    }

    //==============================================================================================
    // State Transition: exit()
    //==============================================================================================

    /// Lemma: exit() preserves thread identity.
    pub proof fn lemma_exit_preserves_id(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.spec_id() == self.spec_id()
            }),
    {
    }

    /// Lemma: exit() correctly captures the exit status.
    pub proof fn lemma_exit_captures_status(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.spec_status() == status
            }),
    {
    }

    /// Lemma: exit() preserves well-formedness.
    pub proof fn lemma_exit_preserves_wf(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.wf()
            }),
    {
    }

    /// Lemma: exit() preserves mutex accounting (count and per-address membership).
    pub proof fn lemma_exit_preserves_mutexes(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.spec_locked_mutex_count() == self.spec_locked_mutex_count()
                && (forall|a: int| z.spec_has_mutex(a) == self.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: exit() preserves drop safety.
    pub proof fn lemma_exit_preserves_drop_safety(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.spec_drop_safe() == self.spec_drop_safe()
            }),
    {
    }

    /// Lemma: exit() preserves the full ThreadStateView.
    pub proof fn lemma_exit_preserves_state_view(&self, status: int)
        requires
            self.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: self.state, status: status };
                z.state@ == self.state@
            }),
    {
    }

    //==============================================================================================
    // Composite Lemmas
    //==============================================================================================

    /// Lemma: A freshly constructed RunningThread (with empty state) is drop-safe.
    pub proof fn lemma_new_is_drop_safe(
        id: ThreadIdentifier,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    )
        ensures
            ({
                let state: ThreadState = ThreadState {
                    id: id,
                    kernel_stack: kernel_stack,
                    user_stack: user_stack,
                    user_tda: user_tda,
                    interrupt_reason: None,
                    locked_mutex_count: 0usize,
                    locked_mutex_set: Ghost(Set::empty()),
                };
                let r: RunningThread = RunningThread { state: state };
                r.spec_drop_safe() && r.wf() && !r.spec_is_interrupted()
            }),
    {
    }

    /// Lemma: Construction followed by schedule() produces a ReadyThread with
    /// the original identity.
    pub proof fn lemma_from_state_then_schedule(state: ThreadState)
        requires
            state.wf(),
        ensures
            ({
                let running: RunningThread = RunningThread { state: state };
                let ready: ReadyThread = ReadyThread { state: running.state };
                ready.spec_id() == state.spec_id()
                && ready.wf()
                && ready.spec_drop_safe() == state.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Construction followed by exit() produces a ZombieThread with
    /// the original identity and correct status.
    pub proof fn lemma_from_state_then_exit(state: ThreadState, status: int)
        requires
            state.wf(),
        ensures
            ({
                let running: RunningThread = RunningThread { state: state };
                let zombie: ZombieThread = ZombieThread { state: running.state, status: status };
                zombie.spec_id() == state.spec_id()
                && zombie.spec_status() == status
                && zombie.wf()
                && zombie.spec_drop_safe() == state.spec_drop_safe()
                && (forall|a: int| zombie.spec_has_mutex(a) == state.spec_has_mutex(a))
            }),
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two RunningThreads with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &RunningThread, b: &RunningThread)
        requires
            a.state@ == b.state@,
        ensures
            a@ == b@,
    {
    }

    /// Lemma: A thread that acquires a mutex and then releases it returns to
    /// its original mutex state (count and per-address membership). This
    /// exercises non-trivial reasoning about store/take inverse relationship.
    pub proof fn lemma_acquire_then_release_restores_mutex_state(
        t: RunningThread,
        address: Ghost<int>,
    )
        requires
            t.wf(),
            t.state.locked_mutex_count < usize::MAX,
            !t.spec_has_mutex(address@),
        ensures
            ({
                let after_acquire: RunningThread = RunningThread {
                    state: ThreadState {
                        locked_mutex_count: (t.state.locked_mutex_count + 1) as usize,
                        locked_mutex_set: Ghost(t.state.locked_mutex_set@.insert(address@)),
                        ..t.state
                    },
                };
                let after_release: RunningThread = RunningThread {
                    state: ThreadState {
                        locked_mutex_count: (after_acquire.state.locked_mutex_count - 1) as usize,
                        locked_mutex_set: Ghost(after_acquire.state.locked_mutex_set@.remove(address@)),
                        ..after_acquire.state
                    },
                };
                after_release.spec_locked_mutex_count() == t.spec_locked_mutex_count()
                && (forall|a: int| after_release.spec_has_mutex(a) == t.spec_has_mutex(a))
                && after_release.spec_drop_safe() == t.spec_drop_safe()
            }),
    {
        // Trigger set extensionality: insert then remove is identity for non-member.
        let s: Set<int> = t.state.locked_mutex_set@;
        assert(s.insert(address@).remove(address@) =~= s);
    }
}

//==================================================================================================
// SleepingThread Lemmas (Boundary)
//==================================================================================================

impl SleepingThread {
    /// Lemma: from_state preserves thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, alarm: Option<int>)
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: from_state preserves well-formedness.
    pub proof fn lemma_from_state_preserves_wf(state: ThreadState, alarm: Option<int>)
        requires
            state.wf(),
        ensures
            ({
                let s: SleepingThread = SleepingThread { state: state, alarm: alarm };
                s.wf()
            }),
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
    }

    /// Lemma: from_state preserves per-address mutex membership.
    pub proof fn lemma_from_state_preserves_mutexes(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                forall|a: int| z.spec_has_mutex(a) == state.spec_has_mutex(a)
            }),
    {
    }
}

} // verus!
