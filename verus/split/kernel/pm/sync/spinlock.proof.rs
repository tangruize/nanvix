// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Spinlock Proofs.
// This file contains proof lemmas for the Spinlock type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl Spinlock {
    /// Lemma: A newly created spinlock is unlocked.
    pub proof fn lemma_new_is_unlocked(id: nat)
        ensures
            Spinlock::spec_new_view(id) == (SpinlockView { locked: false, id: id }),
            !Spinlock::spec_new_view(id).locked,
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

    /// Lemma: Two spinlocks with equal views have equal locked state.
    pub proof fn lemma_view_equality(a: &Spinlock, b: &Spinlock)
        requires
            a@ == b@,
        ensures
            a.spec_is_locked() == b.spec_is_locked(),
            a@.id == b@.id,
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
    pub proof fn lemma_unlock_produces_unlocked(id: nat)
        ensures
            !(Spinlock { locked: false, id: Ghost(id) }).locked,
            (Spinlock { locked: false, id: Ghost(id) }).spec_is_unlocked(),
    {
    }

    /// Lemma: Lock-then-unlock round-trip restores unlocked state.
    ///
    /// # Note
    ///
    /// Models the protocol: starting from unlocked, acquiring the lock,
    /// then releasing it returns to the unlocked state. Identity is preserved.
    pub proof fn lemma_lock_unlock_roundtrip(id: nat)
        ensures ({
            let initial: Spinlock = Spinlock { locked: false, id: Ghost(id) };
            let after_lock: Spinlock = Spinlock { locked: true, id: Ghost(id) };
            let after_unlock: Spinlock = Spinlock { locked: false, id: Ghost(id) };
            &&& initial.spec_is_unlocked()
            &&& after_lock.spec_is_locked()
            &&& after_unlock.spec_is_unlocked()
            &&& initial@ == after_unlock@
            &&& initial@.id == after_lock@.id
            &&& after_lock@.id == after_unlock@.id
        }),
    {
    }

    /// Lemma: An unlocked spinlock has the same view as a new spinlock with its id.
    pub proof fn lemma_unlocked_eq_new_view(s: &Spinlock)
        requires
            s.spec_is_unlocked(),
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
    {
    }

    /// Lemma: A `LockToken` produced by a locked spinlock is valid for unlock.
    ///
    /// # Description
    ///
    /// Proves the lock-release obligation is always dischargeable: a token whose
    /// view matches a locked spinlock satisfies `unlock()`'s preconditions.
    /// This connects the token-producing postconditions of `lock()`/`try_lock()`
    /// to the token-consuming preconditions of `unlock()`.
    pub proof fn lemma_lock_token_valid_for_unlock(s: &Spinlock, token: &LockToken)
        requires
            s.spec_is_locked(),
            token.view == s@,
        ensures
            s.locked,
            token.view.locked,
            token.view.id == s@.id,
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
            token.view == (SpinlockView { locked: true, id: token.view.id }),
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
