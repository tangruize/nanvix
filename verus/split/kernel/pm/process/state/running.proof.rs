// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proof lemmas for RunningProcess and boundary process types.

use vstd::prelude::*;

verus! {

//==================================================================================================
// RunningProcess Lemmas
//==================================================================================================

impl RunningProcess {
    /// A well-formed RunningProcess satisfies inv().
    ///
    /// This is essentially the definition of inv(), but stated as a lemma for callers
    /// that hold a reference to a well-formed process.
    proof fn lemma_new_is_wf(&self)
        requires
            self.ready_count as nat == self.ready_thread_ids@.len(),
            self.interrupted_count as nat == self.interrupted_thread_ids@.len(),
            self.sleeping_count as nat == self.sleeping_thread_ids@.len(),
            self.zombie_count as nat == self.zombie_thread_ids@.len(),
        ensures
            self.inv(),
    {
        reveal(RunningProcess::inv);
    }

    /// wf_strict implies inv.
    proof fn lemma_wf_strict_implies_wf(&self)
        requires
            self.wf_strict(),
        ensures
            self.inv(),
    {
        reveal(RunningProcess::inv);
    }

    /// Two RunningProcesses with identical views have equal abstract fields.
    proof fn lemma_view_equality(a: &RunningProcess, b: &RunningProcess)
        requires
            a@ == b@,
        ensures
            a@.pid == b@.pid,
            a@.running_thread_id == b@.running_thread_id,
            a@.ready_thread_ids == b@.ready_thread_ids,
            a@.interrupted_thread_ids == b@.interrupted_thread_ids,
            a@.sleeping_thread_ids == b@.sleeping_thread_ids,
            a@.zombie_thread_ids == b@.zombie_thread_ids,
    {
    }

    /// Removing an element from a sequence decreases its length by one.
    proof fn lemma_remove_at_length(s: Seq<u64>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            Self::spec_remove_at(s, idx).len() == s.len() - 1,
    {
        let left: Seq<u64> = s.subrange(0, idx);
        let right: Seq<u64> = s.subrange(idx + 1, s.len() as int);
        assert(left.len() == idx as nat);
        assert(right.len() == (s.len() - idx as nat - 1) as nat);
        assert(left.add(right).len() == (s.len() - 1) as nat);
    }

    /// After removing a zombie thread via try_join_thread, the zombie list shrinks by one.
    proof fn lemma_try_join_zombie_post_shrinks(&self, tid: u64)
        requires
            self.spec_try_join_thread(tid) == JOIN_TAG_ZOMBIE as int,
            self.spec_has_zombie_thread(tid),
        ensures
            ({
                let post_zombies: Seq<u64> = self.spec_try_join_zombie_post(tid);
                post_zombies.len() == self.spec_zombie_count() - 1
            }),
    {
        let s: Seq<u64> = self.zombie_thread_ids@;
        let idx: int = choose|i: int|
            0 <= i < s.len()
            && #[trigger] s[i] == tid;
        Self::lemma_remove_at_length(s, idx);
    }
}

//==================================================================================================
// Sequence Bridging Lemmas (Seq<u64> → Seq<int>)
//
// These lemmas prove that converting a sequence from u64 to int (via Seq::new
// with `as int` mapping) commutes with standard sequence operations (push,
// add, subrange, remove_at). They bridge concrete Vec<u64> results to
// abstract Seq<int> postconditions.
//==================================================================================================

/// Proves that converting a pushed sequence matches pushing onto the converted sequence.
proof fn lemma_seq_as_int_push(s: Seq<u64>, v: u64)
    ensures
        Seq::new(s.push(v).len(), |i: int| s.push(v)[i] as int)
            =~= Seq::new(s.len(), |i: int| s[i] as int).push(v as int),
{
    let lhs: Seq<int> = Seq::new(s.push(v).len(), |i: int| s.push(v)[i] as int);
    let rhs: Seq<int> = Seq::new(s.len(), |i: int| s[i] as int).push(v as int);
    assert(lhs.len() == rhs.len());
    assert forall |i: int| 0 <= i < lhs.len() implies lhs[i] == rhs[i]
    by {
        if i < s.len() as int {
            assert(s.push(v)[i] == s[i]);
        } else {
            assert(s.push(v)[i] == v);
        }
    }
}

/// Proves that converting a concatenated sequence matches concatenating the converted sequences.
proof fn lemma_seq_as_int_add(a: Seq<u64>, b: Seq<u64>)
    ensures
        Seq::new(a.add(b).len(), |i: int| a.add(b)[i] as int)
            =~= Seq::new(a.len(), |i: int| a[i] as int).add(
                Seq::new(b.len(), |i: int| b[i] as int)),
{
    let lhs: Seq<int> = Seq::new(a.add(b).len(), |i: int| a.add(b)[i] as int);
    let rhs: Seq<int> = Seq::new(a.len(), |i: int| a[i] as int).add(
        Seq::new(b.len(), |i: int| b[i] as int));
    assert(lhs.len() == rhs.len());
    assert forall |i: int| 0 <= i < lhs.len() implies lhs[i] == rhs[i]
    by {
        if i < a.len() as int {
            assert(a.add(b)[i] == a[i]);
        } else {
            assert(a.add(b)[i] == b[i - a.len() as int]);
        }
    }
}

/// Proves that converting a sub-ranged sequence matches sub-ranging the converted sequence.
proof fn lemma_seq_as_int_subrange(s: Seq<u64>, lo: int, hi: int)
    requires
        0 <= lo <= hi <= s.len(),
    ensures
        Seq::new(s.subrange(lo, hi).len(), |i: int| s.subrange(lo, hi)[i] as int)
            =~= Seq::new(s.len(), |i: int| s[i] as int).subrange(lo, hi),
{
    let lhs: Seq<int> = Seq::new(s.subrange(lo, hi).len(), |i: int| s.subrange(lo, hi)[i] as int);
    let rhs: Seq<int> = Seq::new(s.len(), |i: int| s[i] as int).subrange(lo, hi);
    assert(lhs.len() == rhs.len());
    assert forall |i: int| 0 <= i < lhs.len() implies lhs[i] == rhs[i]
    by {
        assert(s.subrange(lo, hi)[i] == s[lo + i]);
    }
}

/// Proves that converting a remove_at result matches remove_at on the converted sequence.
proof fn lemma_seq_as_int_remove_at(s: Seq<u64>, idx: int)
    requires
        0 <= idx < s.len(),
    ensures
        Seq::new(
            RunningProcess::spec_remove_at(s, idx).len(),
            |i: int| RunningProcess::spec_remove_at(s, idx)[i] as int
        ) =~= RunningProcessView::seq_remove_at(
            Seq::new(s.len(), |i: int| s[i] as int), idx),
{
    lemma_seq_as_int_subrange(s, 0, idx);
    lemma_seq_as_int_subrange(s, idx + 1, s.len() as int);
    lemma_seq_as_int_add(s.subrange(0, idx), s.subrange(idx + 1, s.len() as int));
}

//==================================================================================================
// RunnableProcess Lemmas
//==================================================================================================

impl RunnableProcess {
    /// A RunnableProcess with non-empty ready list satisfies inv().
    proof fn lemma_new_wf(&self)
        requires
            self.ready_thread_ids@.len() >= 1,
        ensures
            self.inv(),
    {
        reveal(RunnableProcess::inv);
    }
}

//==================================================================================================
// SleepingProcess Lemmas
//==================================================================================================

impl SleepingProcess {
    /// A SleepingProcess with non-empty sleeping list satisfies inv().
    proof fn lemma_new_wf(&self)
        requires
            self.sleeping_thread_ids@.len() >= 1,
        ensures
            self.inv(),
    {
        reveal(SleepingProcess::inv);
    }
}

//==================================================================================================
// InterruptedProcess Lemmas
//==================================================================================================

impl InterruptedProcess {
    /// An InterruptedProcess with non-empty interrupted list satisfies inv().
    proof fn lemma_new_wf(&self)
        requires
            self.interrupted_thread_ids@.len() >= 1,
        ensures
            self.inv(),
    {
        reveal(InterruptedProcess::inv);
    }
}

//==================================================================================================
// ZombieProcess Lemmas
//==================================================================================================

impl ZombieProcess {
    /// A ZombieProcess with non-empty zombie list satisfies inv().
    proof fn lemma_new_wf(&self)
        requires
            self.zombie_thread_ids@.len() >= 1,
        ensures
            self.inv(),
    {
        reveal(ZombieProcess::inv);
    }
}

//==================================================================================================
// Connecting Lemmas — RunningProcessView
//==================================================================================================
//
// These lemmas machine-check that the exec postconditions (field-level) imply
// View-level equality with the corresponding spec transition function.
// Usage:  after an exec call, invoke the lemma in a proof block to obtain
//         `result@ =~= old(self)@.spec_foo(args)`.

impl RunningProcessView {
    /// Connecting lemma for `RunningProcess::schedule()`.
    proof fn lemma_schedule_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.ready_thread_ids =~= pre.ready_thread_ids.push(pre.running_thread_id),
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
        ensures
            post =~= pre.spec_schedule(),
    {
    }

    /// Connecting lemma for `RunningProcess::sleep()` — ready branch.
    proof fn lemma_sleep_ready_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.ready_thread_ids =~= pre.ready_thread_ids,
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids.push(pre.running_thread_id),
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
        ensures
            post =~= pre.spec_sleep_to_runnable_ready(),
    {
    }

    /// Connecting lemma for `RunningProcess::sleep()` — interrupted branch.
    proof fn lemma_sleep_interrupted_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids.push(pre.running_thread_id),
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
            post.ready_thread_ids.len() == 1,
            post.ready_thread_ids[0] == pre.interrupted_thread_ids[0],
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids.subrange(
                1, pre.interrupted_thread_ids.len() as int),
            pre.interrupted_thread_ids.len() > 0,
        ensures
            post =~= pre.spec_sleep_to_runnable_interrupted(),
    {
        assert(post.ready_thread_ids =~=
            Seq::<int>::empty().push(pre.interrupted_thread_ids[0]));
    }

    /// Connecting lemma for `RunningProcess::sleep()` — sleeping branch.
    proof fn lemma_sleep_sleeping_view_matches(
        pre: RunningProcessView,
        post: SleepingProcessView,
    )
        requires
            post.pid == pre.pid,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids.push(pre.running_thread_id),
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
        ensures
            post =~= pre.spec_sleep_to_sleeping(),
    {
    }

    /// Connecting lemma for `RunningProcess::exit()` — runnable branch.
    proof fn lemma_exit_runnable_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.zombie_thread_ids =~=
                pre.zombie_thread_ids.push(pre.running_thread_id).add(pre.ready_thread_ids),
            post.sleeping_thread_ids.len() == 0,
            post.ready_thread_ids.len() == 1,
            post.ready_thread_ids[0] ==
                pre.interrupted_thread_ids.add(pre.sleeping_thread_ids)[0],
            post.interrupted_thread_ids =~=
                pre.interrupted_thread_ids.add(pre.sleeping_thread_ids).subrange(
                    1, (pre.interrupted_thread_ids.len() + pre.sleeping_thread_ids.len()) as int),
            pre.interrupted_thread_ids.len() + pre.sleeping_thread_ids.len() > 0,
        ensures
            post =~= pre.spec_exit_to_runnable(),
    {
        let combined: Seq<int> = pre.interrupted_thread_ids.add(pre.sleeping_thread_ids);
        assert(post.sleeping_thread_ids =~= Seq::<int>::empty());
        assert(post.ready_thread_ids =~= Seq::<int>::empty().push(combined[0]));
    }

    /// Connecting lemma for `RunningProcess::exit()` — zombie branch.
    proof fn lemma_exit_zombie_view_matches(
        pre: RunningProcessView,
        post: ZombieProcessView,
        status: int,
    )
        requires
            post.pid == pre.pid,
            post.status == status,
            post.zombie_thread_ids =~=
                pre.zombie_thread_ids.push(pre.running_thread_id).add(pre.ready_thread_ids),
        ensures
            post =~= pre.spec_exit_to_zombie(status),
    {
    }

    /// Connecting lemma for `RunningProcess::exit_thread()` — ready branch.
    proof fn lemma_exit_thread_ready_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.ready_thread_ids =~= pre.ready_thread_ids,
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids.push(pre.running_thread_id),
        ensures
            post =~= pre.spec_exit_thread_to_runnable_ready(),
    {
    }

    /// Connecting lemma for `RunningProcess::exit_thread()` — interrupted branch.
    proof fn lemma_exit_thread_interrupted_view_matches(
        pre: RunningProcessView,
        post: RunnableProcessView,
    )
        requires
            post.pid == pre.pid,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids.push(pre.running_thread_id),
            post.ready_thread_ids.len() == 1,
            post.ready_thread_ids[0] == pre.interrupted_thread_ids[0],
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids.subrange(
                1, pre.interrupted_thread_ids.len() as int),
            pre.interrupted_thread_ids.len() > 0,
        ensures
            post =~= pre.spec_exit_thread_to_runnable_interrupted(),
    {
        assert(post.ready_thread_ids =~=
            Seq::<int>::empty().push(pre.interrupted_thread_ids[0]));
    }

    /// Connecting lemma for `RunningProcess::exit_thread()` — sleeping branch.
    proof fn lemma_exit_thread_sleeping_view_matches(
        pre: RunningProcessView,
        post: SleepingProcessView,
    )
        requires
            post.pid == pre.pid,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids.push(pre.running_thread_id),
        ensures
            post =~= pre.spec_exit_thread_to_sleeping(),
    {
    }

    /// Connecting lemma for `RunningProcess::exit_thread()` — zombie branch.
    proof fn lemma_exit_thread_zombie_view_matches(
        pre: RunningProcessView,
        post: ZombieProcessView,
        status: int,
    )
        requires
            post.pid == pre.pid,
            post.status == status,
            post.zombie_thread_ids =~= pre.zombie_thread_ids.push(pre.running_thread_id),
        ensures
            post =~= pre.spec_exit_thread_to_zombie(status),
    {
    }

    /// Connecting lemma for `RunningProcess::wakeup()` — success (Ok) case.
    ///
    /// Requires `tid` to appear at most once in `sleeping_thread_ids` so that
    /// the existential index from the exec postcondition matches the `choose`
    /// index in `spec_wakeup_ok`.
    proof fn lemma_wakeup_ok_view_matches(
        pre: RunningProcessView,
        post: RunningProcessView,
        tid: int,
    )
        requires
            post.pid == pre.pid,
            post.running_thread_id == pre.running_thread_id,
            post.ready_thread_ids =~= pre.ready_thread_ids.push(tid),
            exists|idx: int| 0 <= idx < pre.sleeping_thread_ids.len()
                && pre.sleeping_thread_ids[idx] == tid
                && post.sleeping_thread_ids =~=
                    RunningProcessView::seq_remove_at(pre.sleeping_thread_ids, idx),
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
            // Uniqueness: tid appears at most once in sleeping list.
            forall|i: int, j: int|
                0 <= i < pre.sleeping_thread_ids.len()
                && 0 <= j < pre.sleeping_thread_ids.len()
                && pre.sleeping_thread_ids[i] == tid
                && pre.sleeping_thread_ids[j] == tid
                ==> i == j,
        ensures
            post =~= pre.spec_wakeup_ok(tid),
    {
        let s: Seq<int> = pre.sleeping_thread_ids;
        let exec_idx: int = choose|idx: int|
            0 <= idx < s.len()
            && s[idx] == tid
            && post.sleeping_thread_ids =~= RunningProcessView::seq_remove_at(s, idx);
        let spec_idx: int = choose|i: int|
            0 <= i < s.len() && #[trigger] s[i] == tid;
        // Both satisfy s[_] == tid; uniqueness forces them equal.
        assert(s[exec_idx] == tid);
        assert(s[spec_idx] == tid);
        assert(exec_idx == spec_idx);
    }

    /// Connecting lemma for `RunningProcess::wakeup()` — failure (Err) case.
    proof fn lemma_wakeup_err_view_matches(
        pre: RunningProcessView,
        post: RunningProcessView,
    )
        requires
            post.pid == pre.pid,
            post.running_thread_id == pre.running_thread_id,
            post.ready_thread_ids =~= pre.ready_thread_ids,
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
        ensures
            post =~= pre.spec_wakeup_err(),
    {
    }

    /// Connecting lemma for `RunningProcess::try_join_thread()` — zombie case.
    ///
    /// Requires `tid` to appear at most once in `zombie_thread_ids` so that
    /// the existential index from the exec postcondition matches the `choose`
    /// index in `spec_join_zombie_result`.
    proof fn lemma_join_zombie_view_matches(
        pre: RunningProcessView,
        post: RunningProcessView,
        tid: int,
    )
        requires
            post.pid == pre.pid,
            post.running_thread_id == pre.running_thread_id,
            post.ready_thread_ids =~= pre.ready_thread_ids,
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            exists|idx: int| 0 <= idx < pre.zombie_thread_ids.len()
                && pre.zombie_thread_ids[idx] == tid
                && post.zombie_thread_ids =~=
                    RunningProcessView::seq_remove_at(pre.zombie_thread_ids, idx),
            // Uniqueness: tid appears at most once in zombie list.
            forall|i: int, j: int|
                0 <= i < pre.zombie_thread_ids.len()
                && 0 <= j < pre.zombie_thread_ids.len()
                && pre.zombie_thread_ids[i] == tid
                && pre.zombie_thread_ids[j] == tid
                ==> i == j,
        ensures
            post =~= pre.spec_join_zombie_result(tid),
    {
        let s: Seq<int> = pre.zombie_thread_ids;
        let exec_idx: int = choose|idx: int|
            0 <= idx < s.len()
            && s[idx] == tid
            && post.zombie_thread_ids =~= RunningProcessView::seq_remove_at(s, idx);
        let spec_idx: int = choose|i: int|
            0 <= i < s.len() && #[trigger] s[i] == tid;
        // Both satisfy s[_] == tid; uniqueness forces them equal.
        assert(s[exec_idx] == tid);
        assert(s[spec_idx] == tid);
        assert(exec_idx == spec_idx);
    }

    /// Connecting lemma for `RunningProcess::try_join_thread()` — non-zombie case.
    proof fn lemma_join_non_zombie_view_matches(
        pre: RunningProcessView,
        post: RunningProcessView,
    )
        requires
            post.pid == pre.pid,
            post.running_thread_id == pre.running_thread_id,
            post.ready_thread_ids =~= pre.ready_thread_ids,
            post.interrupted_thread_ids =~= pre.interrupted_thread_ids,
            post.sleeping_thread_ids =~= pre.sleeping_thread_ids,
            post.zombie_thread_ids =~= pre.zombie_thread_ids,
        ensures
            post =~= pre.spec_join_non_zombie_result(),
    {
    }
}

} // verus!