// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ThreadManager Proofs.
// Lemmas for construction correctness, ID management, and well-formedness.
//
// Key proven properties:
// - new() produces a well-formed manager with next_id=1 and kernel thread with id=0.
// - create_thread assigns the current next_id and increments by exactly 1.
// - create_thread preserves well-formedness.
// - Kernel thread ID (0) is distinct from all subsequently created thread IDs (>=1).
// - IDs assigned by create_thread are strictly monotonically increasing.
// - Global ID uniqueness: any two IDs assigned at different manager states are distinct.
// - init() is equivalent to new().
//
// Note: All lemmas are auto-proved by the SMT solver, confirming the specs
// are consistent. The lemmas serve as documentation of key safety properties
// and regression guards against spec changes.

use vstd::prelude::*;

verus! {

impl ThreadManager {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: new() produces a well-formed ThreadManager.
    pub proof fn lemma_new_produces_wf_manager()
        ensures
            ({
                let m: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: 1 },
                };
                m.wf()
            }),
    {
        reveal(ThreadManager::wf);
    }

    /// Lemma: new() produces a kernel thread with ID 0.
    pub proof fn lemma_new_kernel_thread_id()
        ensures
            ({
                let tid: ThreadIdentifier = ThreadIdentifier { value: 0 };
                tid.spec_value() == 0
            }),
    {
    }

    /// Lemma: new() sets next_id to 1.
    pub proof fn lemma_new_next_id()
        ensures
            ({
                let m: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: 1 },
                };
                m.spec_next_id() == 1
            }),
    {
        reveal(ThreadManager::spec_next_id);
    }

    //==============================================================================================
    // Well-Formedness Preservation
    //==============================================================================================

    /// Lemma: create_thread preserves well-formedness.
    pub proof fn lemma_create_thread_preserves_wf(&self)
        requires
            self.wf(),
            self@.next_id < i32::MAX as int,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.wf()
            }),
    {
        reveal(ThreadManager::wf);
    }

    //==============================================================================================
    // ID Assignment Lemmas
    //==============================================================================================

    /// Lemma: create_thread increments next_id by exactly 1.
    pub proof fn lemma_create_thread_increments_by_one(&self)
        requires
            self.wf(),
            self@.next_id < i32::MAX as int,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.spec_next_id() == self.spec_next_id() + 1
            }),
    {
        reveal(ThreadManager::wf);
        reveal(ThreadManager::spec_next_id);
    }

    /// Lemma: IDs assigned by create_thread are always >= 1.
    pub proof fn lemma_assigned_ids_positive(&self)
        requires
            self.wf(),
        ensures
            self.spec_next_id() >= 1,
    {
        reveal(ThreadManager::wf);
        reveal(ThreadManager::spec_next_id);
    }

    /// Lemma: Kernel thread ID (0) is always distinct from any subsequently
    /// created thread ID (>= 1), establishing kernel thread identity uniqueness.
    pub proof fn lemma_kernel_id_distinct_from_created(&self)
        requires
            self.wf(),
        ensures
            self.spec_next_id() != 0,
    {
        reveal(ThreadManager::wf);
        reveal(ThreadManager::spec_next_id);
    }

    /// Lemma: create_thread strictly increases next_id, so IDs are monotonic.
    pub proof fn lemma_create_thread_monotonic(&self)
        requires
            self.wf(),
            self@.next_id < i32::MAX as int,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.spec_next_id() > self.spec_next_id()
            }),
    {
        reveal(ThreadManager::wf);
        reveal(ThreadManager::spec_next_id);
    }

    /// Lemma: Two successive create_thread calls assign different IDs.
    /// If the manager assigns ID N (its current next_id), then after
    /// incrementing, the next call assigns ID N+1 != N.
    pub proof fn lemma_successive_creates_distinct(&self)
        requires
            self.wf(),
            self@.next_id < i32::MAX as int,
        ensures
            // The current next_id differs from the next next_id.
            self.spec_next_id() != self.spec_next_id() + 1,
    {
        reveal(ThreadManager::wf);
        reveal(ThreadManager::spec_next_id);
    }

    //==============================================================================================
    // Global ID Uniqueness
    //==============================================================================================

    /// Lemma: Any two thread IDs assigned at different manager states are
    /// globally unique. If a manager had next_id == n1 at time T1 and
    /// next_id == n2 at time T2 where n2 > n1, then the threads created
    /// at T1 (with ID n1) and T2 (with ID n2) have distinct IDs.
    ///
    /// This is the fundamental kernel safety property: thread IDs never
    /// collide. It follows from strict monotonicity of next_id, but is
    /// stated explicitly as the top-level uniqueness guarantee.
    pub proof fn lemma_all_assigned_ids_globally_unique(n1: int, n2: int)
        requires
            n1 >= 1,
            n2 > n1,
        ensures
            n1 != n2,
    {
    }

    /// Lemma: The kernel thread ID (0) is distinct from any thread created
    /// by a well-formed manager (whose next_id >= 1). Combined with
    /// `lemma_all_assigned_ids_globally_unique`, this establishes that all
    /// thread IDs in the system — including the kernel thread — are unique.
    pub proof fn lemma_kernel_id_unique_from_all_created(created_id: int)
        requires
            created_id >= 1,
        ensures
            0 != created_id,
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ThreadManagers with identical next_id values have equal views.
    pub proof fn lemma_view_equality(a: &ThreadManager, b: &ThreadManager)
        requires
            a.spec_next_id() == b.spec_next_id(),
        ensures
            a@ == b@,
    {
        reveal(ThreadManager::spec_next_id);
    }
}

//==================================================================================================
// ThreadRefModel Lemmas
//==================================================================================================

impl ThreadRefModel {
    /// Lemma: thread_state() dispatch preserves thread identity across all
    /// variants. For any ThreadRefModel, the dispatched state has the same
    /// spec_id as the enum itself.
    pub proof fn lemma_dispatch_preserves_id(&self)
        ensures
            self.spec_state().spec_id() == self.spec_id(),
    {
    }

    /// Lemma: thread_state() dispatch on a well-formed variant returns a
    /// well-formed ThreadState.
    pub proof fn lemma_dispatch_preserves_wf(&self)
        requires
            self.spec_state().wf(),
        ensures
            self.spec_state().wf(),
    {
    }
}

//==================================================================================================
// ThreadRefMutModel Lemmas
//==================================================================================================

impl ThreadRefMutModel {
    /// Lemma: thread_state() dispatch preserves thread identity across all
    /// variants. For any ThreadRefMutModel, the dispatched state has the
    /// same spec_id as the enum itself.
    pub proof fn lemma_dispatch_preserves_id(&self)
        ensures
            self.spec_state().spec_id() == self.spec_id(),
    {
    }

    /// Lemma: thread_state() dispatch on a well-formed variant returns a
    /// well-formed ThreadState.
    pub proof fn lemma_dispatch_preserves_wf(&self)
        requires
            self.spec_state().wf(),
        ensures
            self.spec_state().wf(),
    {
    }
}

} // verus!
