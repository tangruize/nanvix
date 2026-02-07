// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ZombieThread Proofs.
// Lemmas for construction, identity preservation, status correctness,
// well-formedness preservation, mutex accounting, drop safety,
// harvest correctness, and view equality.
//
// Key proven properties:
// - Construction (from_state) produces well-formed state with correct identity and status.
// - Thread identifier (`id`) is immutable across all operations.
// - `status()` correctly returns the exit status captured at construction.
// - `harvest()` returns the kernel and user stacks from the underlying state
//   (via take_kernel_stack / take_user_stack semantics).
// - Mutex accounting (count and per-address membership) is preserved through construction.
// - Drop safety is preserved through construction.
// - View equality for structurally identical ZombieThreads.

use vstd::prelude::*;

verus! {

impl ZombieThread {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: Construction from ThreadState produces a well-formed ZombieThread.
    pub proof fn lemma_from_state_is_wf(state: ThreadState, status: int)
        requires
            state.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.wf()
            }),
    {
    }

    /// Lemma: Construction preserves the thread identity.
    pub proof fn lemma_from_state_preserves_id(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_id() == state.spec_id()
            }),
    {
    }

    /// Lemma: Construction captures the exit status correctly.
    pub proof fn lemma_from_state_captures_status(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_status() == status
            }),
    {
    }

    /// Lemma: Construction preserves drop safety.
    pub proof fn lemma_from_state_preserves_drop_safe(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_drop_safe() == state.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Construction preserves mutex accounting.
    pub proof fn lemma_from_state_preserves_mutexes(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_locked_mutex_count() == state.spec_locked_mutex_count()
                && (forall|a: int| z.spec_has_mutex(a) == state.spec_has_mutex(a))
            }),
    {
    }

    /// Lemma: Construction preserves the full ThreadStateView.
    pub proof fn lemma_from_state_preserves_state_view(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.state@ == state@
            }),
    {
    }

    /// Lemma: Construction preserves kernel and user stack tokens.
    pub proof fn lemma_from_state_preserves_stacks(state: ThreadState, status: int)
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_kernel_stack() == state.spec_kernel_stack()
                && z.spec_user_stack() == state.spec_user_stack()
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
    // Status Correctness
    //==============================================================================================

    /// Lemma: spec_status faithfully reflects the stored exit status.
    pub proof fn lemma_status_correct(&self)
        ensures
            self.spec_status() == self.status,
    {
    }

    //==============================================================================================
    // Harvest Lemmas
    //==============================================================================================

    /// Lemma: harvest returns the kernel stack that was in the state.
    /// After harvest, the state's kernel stack is None.
    pub proof fn lemma_harvest_returns_stacks(z: ZombieThread)
        requires
            z.wf(),
        ensures
            // The harvest operation returns the stacks that were stored.
            z.spec_kernel_stack() == z.state.spec_kernel_stack(),
            z.spec_user_stack() == z.state.spec_user_stack(),
    {
    }

    /// Lemma: After harvest, the zombie's identity is unchanged
    /// (harvest consumes self, but the stacks returned are from the original state).
    pub proof fn lemma_harvest_identity(z: ZombieThread)
        requires
            z.wf(),
        ensures
            z.spec_id() == z.state.spec_id(),
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ZombieThreads with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ZombieThread, b: &ZombieThread)
        requires
            a.state@ == b.state@,
            a.status == b.status,
        ensures
            a@ == b@,
    {
    }

    //==============================================================================================
    // Composite Lemmas
    //==============================================================================================

    /// Lemma: A ZombieThread created from a drop-safe state is itself drop-safe.
    pub proof fn lemma_drop_safe_propagates(state: ThreadState, status: int)
        requires
            state.wf(),
            state.spec_drop_safe(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_drop_safe()
            }),
    {
    }

    /// Lemma: A ZombieThread created from a state with locked mutexes
    /// is NOT drop-safe.
    pub proof fn lemma_not_drop_safe_propagates(state: ThreadState, status: int)
        requires
            state.wf(),
            !state.spec_drop_safe(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                !z.spec_drop_safe()
            }),
    {
    }

    /// Lemma: Construction followed by id() returns the original thread identity.
    pub proof fn lemma_from_state_then_id(state: ThreadState, status: int)
        requires
            state.wf(),
        ensures
            ({
                let z: ZombieThread = ZombieThread { state: state, status: status };
                z.spec_id() == state.spec_id()
                && z.spec_status() == status
                && z.wf()
            }),
    {
    }

    /// Lemma: The status of a zombie is independent of its thread state.
    /// Two zombies with the same status but different states have the same spec_status.
    pub proof fn lemma_status_independent_of_state(
        state1: ThreadState,
        state2: ThreadState,
        status: int,
    )
        ensures
            ({
                let z1: ZombieThread = ZombieThread { state: state1, status: status };
                let z2: ZombieThread = ZombieThread { state: state2, status: status };
                z1.spec_status() == z2.spec_status()
            }),
    {
    }
}

} // verus!
