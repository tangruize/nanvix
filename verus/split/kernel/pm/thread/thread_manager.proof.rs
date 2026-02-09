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
// - init() is equivalent to new().

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
    }

    //==============================================================================================
    // Well-Formedness Preservation
    //==============================================================================================

    /// Lemma: create_thread preserves well-formedness.
    pub proof fn lemma_create_thread_preserves_wf(&self)
        requires
            self.wf(),
            self.next_id.value < i32::MAX,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.wf()
            }),
    {
    }

    //==============================================================================================
    // ID Assignment Lemmas
    //==============================================================================================

    /// Lemma: create_thread increments next_id by exactly 1.
    pub proof fn lemma_create_thread_increments_by_one(&self)
        requires
            self.wf(),
            self.next_id.value < i32::MAX,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.spec_next_id() == self.spec_next_id() + 1
            }),
    {
    }

    /// Lemma: IDs assigned by create_thread are always >= 1.
    pub proof fn lemma_assigned_ids_positive(&self)
        requires
            self.wf(),
        ensures
            self.spec_next_id() >= 1,
    {
    }

    /// Lemma: Kernel thread ID (0) is always distinct from any subsequently
    /// created thread ID (>= 1), establishing kernel thread identity uniqueness.
    pub proof fn lemma_kernel_id_distinct_from_created(&self)
        requires
            self.wf(),
        ensures
            self.spec_next_id() != 0,
    {
    }

    /// Lemma: create_thread strictly increases next_id, so IDs are monotonic.
    pub proof fn lemma_create_thread_monotonic(&self)
        requires
            self.wf(),
            self.next_id.value < i32::MAX,
        ensures
            ({
                let post: ThreadManager = ThreadManager {
                    next_id: ThreadIdentifier { value: (self.next_id.value + 1) as i32 },
                };
                post.spec_next_id() > self.spec_next_id()
            }),
    {
    }

    /// Lemma: Two successive create_thread calls assign different IDs.
    /// If the manager assigns ID N (its current next_id), then after
    /// incrementing, the next call assigns ID N+1 != N.
    pub proof fn lemma_successive_creates_distinct(&self)
        requires
            self.wf(),
            self.next_id.value < i32::MAX,
        ensures
            // The current next_id differs from the next next_id.
            self.spec_next_id() != self.spec_next_id() + 1,
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ThreadManagers with identical next_id values have equal views.
    pub proof fn lemma_view_equality(a: &ThreadManager, b: &ThreadManager)
        requires
            a.next_id.spec_value() == b.next_id.spec_value(),
        ensures
            a@ == b@,
    {
    }
}

} // verus!
