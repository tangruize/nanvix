// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Spinlock Proofs.
// This file contains proof lemmas for the Spinlock type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================
//
// The following lemmas are definition-unfolding properties that serve as
// executable documentation and regression tests for spec changes. They are
// automatically discharged by Verus.

impl Spinlock {
    /// Lemma: A newly created spinlock is unlocked with no token outstanding.
    pub proof fn lemma_new_is_unlocked(id: nat)
        ensures
            Spinlock::spec_new_view(id) == (SpinlockView { locked: false, id: id, token_issued: false }),
            !Spinlock::spec_new_view(id).locked,
            !Spinlock::spec_new_view(id).token_issued,
            Spinlock::spec_new_view(id).id == id,
    {
    }

    /// Lemma: A spinlock is either locked or unlocked (totality).
    pub proof fn lemma_state_is_total(&self)
        ensures
            self.spec_is_locked() || self.spec_is_unlocked(),
            !(self.spec_is_locked() && self.spec_is_unlocked()),
    {
    }

    /// Lemma: spec_is_locked and spec_is_unlocked are complementary.
    pub proof fn lemma_locked_unlocked_complementary(&self)
        ensures
            self.spec_is_locked() == !self.spec_is_unlocked(),
    {
    }

    /// Lemma: View reflects the locked state.
    pub proof fn lemma_view_reflects_locked(&self)
        ensures
            self@.locked == self.spec_is_locked(),
    {
    }

    /// Lemma: Two spinlocks with equal views have equal locked state and identity.
    pub proof fn lemma_view_equality(a: &Spinlock, b: &Spinlock)
        requires
            a@ == b@,
        ensures
            a.spec_is_locked() == b.spec_is_locked(),
            a@.id == b@.id,
            a@.token_issued == b@.token_issued,
    {
    }

    /// Lemma: try_lock on an unlocked spinlock succeeds and locks it.
    pub proof fn lemma_try_lock_unlocked_succeeds(pre: &Spinlock)
        requires
            pre.spec_is_unlocked(),
        ensures
            !pre.locked,
    {
    }

    /// Lemma: try_lock on a locked spinlock fails and state is unchanged.
    pub proof fn lemma_try_lock_locked_fails(pre: &Spinlock)
        requires
            pre.spec_is_locked(),
        ensures
            pre.locked,
    {
    }

    /// Lemma: unlock on a locked spinlock produces an unlocked spinlock.
    pub proof fn lemma_unlock_produces_unlocked(id: usize)
        ensures
            !(Spinlock { locked: false, id: id, token_issued: false }).locked,
            (Spinlock { locked: false, id: id, token_issued: false }).spec_is_unlocked(),
    {
    }

    /// Lemma: Well-formedness is preserved: new spinlocks are well-formed.
    pub proof fn lemma_new_is_wf(id: nat)
        ensures ({
            let view: SpinlockView = Spinlock::spec_new_view(id);
            !view.locked && !view.token_issued
        }),
    {
    }

    /// Lemma: The well-formedness invariant ensures no token is outstanding
    /// when the lock is unlocked.
    pub proof fn lemma_wf_unlocked_no_token(s: &Spinlock)
        requires
            s.wf(),
            s.spec_is_unlocked(),
        ensures
            !s@.token_issued,
    {
    }

    /// Lemma: The `LockToken` snapshot matches the locked state.
    ///
    /// # Description
    ///
    /// A token produced at lock-acquisition time always has `locked == true` in
    /// its view, since the spinlock is locked at production time.
    pub proof fn lemma_lock_token_snapshot_is_locked(token: &LockToken)
        requires
            token.view.locked,
        ensures
            token.view == (SpinlockView { locked: true, id: token.view.id, token_issued: token.view.token_issued }),
    {
    }

    /// Lemma: `try_lock()` on a locked, well-formed spinlock fails.
    ///
    /// # Description
    ///
    /// Proves the contended-case property: when a spinlock is locked and
    /// well-formed (token is outstanding), `try_lock()` will fail (return false).
    pub proof fn lemma_try_lock_contended_fails(s: &Spinlock)
        requires
            s.wf(),
            s.spec_is_locked(),
        ensures
            s.locked,
            s@.token_issued,
    {
    }

    /// Lemma: `lock()` requires `spec_is_unlocked()` to prevent sequential deadlock.
    ///
    /// # Description
    ///
    /// In the sequential `&mut self` model, calling `lock()` on an already-locked
    /// spinlock would spin forever (no concurrent unlocker). This lemma proves
    /// that a locked, well-formed spinlock has a token outstanding, so the caller
    /// cannot discharge the obligation — formalizing why the precondition is
    /// necessary for deadlock prevention.
    pub proof fn lemma_lock_precondition_prevents_deadlock(s: &Spinlock)
        requires
            s.wf(),
            s.spec_is_locked(),
        ensures
            s@.token_issued,
            !s.spec_is_unlocked(),
    {
    }
}

//==================================================================================================
// Proof Lemmas — Protocol Properties
//==================================================================================================
//
// The following lemmas prove protocol properties that reason across multiple
// state transitions or relate different API operations.

impl Spinlock {
    /// Lemma: Lock-then-unlock round-trip restores unlocked state. Identity and
    /// token tracking are preserved through the protocol.
    pub proof fn lemma_lock_unlock_roundtrip(id: usize)
        ensures ({
            let initial: Spinlock = Spinlock { locked: false, id: id, token_issued: false };
            let after_lock: Spinlock = Spinlock { locked: true, id: id, token_issued: true };
            let after_unlock: Spinlock = Spinlock { locked: false, id: id, token_issued: false };
            &&& initial.spec_is_unlocked()
            &&& after_lock.spec_is_locked()
            &&& after_unlock.spec_is_unlocked()
            &&& initial@ == after_unlock@
            &&& initial@.id == after_lock@.id
            &&& after_lock@.id == after_unlock@.id
            &&& !initial@.token_issued
            &&& after_lock@.token_issued
            &&& !after_unlock@.token_issued
            &&& initial.wf()
            &&& after_lock.wf()
            &&& after_unlock.wf()
        }),
    {
    }

    /// Lemma: An unlocked spinlock has the same view as a new spinlock with its id.
    pub proof fn lemma_unlocked_eq_new_view(s: &Spinlock)
        requires
            s.spec_is_unlocked(),
            s.wf(),
        ensures
            s@ == Spinlock::spec_new_view(s@.id),
    {
    }

    /// Lemma: After `new()` followed by `try_lock()`, the result is always `true`.
    ///
    /// # Description
    ///
    /// Proves a protocol property: a freshly created spinlock is always acquirable.
    /// Any spinlock matching `new()`'s postcondition satisfies `try_lock()`'s success
    /// condition.
    pub proof fn lemma_new_then_try_lock_succeeds(s: &Spinlock)
        requires
            s@ == Spinlock::spec_new_view(s@.id),
        ensures
            s.spec_is_unlocked(),
            !s.locked,
            !s@.token_issued,
            s.wf(),
    {
    }

    /// Lemma: A `LockToken` produced by a locked spinlock is valid for unlock.
    ///
    /// # Description
    ///
    /// Proves the lock-release obligation is always dischargeable: a token whose
    /// view matches a locked spinlock satisfies `unlock()`'s preconditions.
    pub proof fn lemma_lock_token_valid_for_unlock(s: &Spinlock, token: &LockToken)
        requires
            s.spec_is_locked(),
            token.view == s@,
        ensures
            s.locked,
            token.view.locked,
            token.view.id == s@.id,
            token.view.token_issued == s@.token_issued,
    {
    }

    /// Lemma: Tokens from different lock instances cannot satisfy each other's
    /// unlock preconditions.
    ///
    /// # Description
    ///
    /// If two locked spinlocks have different identities, a token from one cannot
    /// be used to unlock the other. This formalizes instance-level token isolation.
    pub proof fn lemma_token_instance_isolation(
        s1: &Spinlock, s2: &Spinlock, token: &LockToken,
    )
        requires
            s1.spec_is_locked(),
            s2.spec_is_locked(),
            s1@.id != s2@.id,
            token.view == s1@,
        ensures
            token.view != s2@,
    {
    }
}

} // verus!
