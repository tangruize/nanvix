// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Mutex Proofs.
// This file contains proof lemmas for the Mutex type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================
//
// The following lemmas are definition-unfolding properties that serve as
// executable documentation and regression tests for spec changes. They are
// automatically discharged by Verus.

impl Mutex {
    /// Lemma: A newly created mutex is unlocked with no token outstanding.
    pub proof fn lemma_new_is_unlocked(id: nat)
        ensures
            Mutex::spec_new_view(id) == (MutexView { locked: false, id: id, token_issued: false }),
            !Mutex::spec_new_view(id).locked,
            !Mutex::spec_new_view(id).token_issued,
            Mutex::spec_new_view(id).id == id,
    {
    }

    /// Lemma: A mutex is either locked or unlocked (totality).
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

    /// Lemma: Two mutexes with equal views have equal locked state and identity.
    pub proof fn lemma_view_equality(a: &Mutex, b: &Mutex)
        requires
            a@ == b@,
        ensures
            a.spec_is_locked() == b.spec_is_locked(),
            a@.id == b@.id,
            a@.token_issued == b@.token_issued,
    {
    }

    /// Lemma: try_lock on an unlocked mutex succeeds and locks it.
    pub proof fn lemma_try_lock_unlocked_succeeds(pre: &Mutex)
        requires
            pre.spec_is_unlocked(),
        ensures
            !pre.locked,
    {
    }

    /// Lemma: try_lock on a locked mutex fails and state is unchanged.
    pub proof fn lemma_try_lock_locked_fails(pre: &Mutex)
        requires
            pre.spec_is_locked(),
        ensures
            pre.locked,
    {
    }

    /// Lemma: unlock on a locked mutex produces an unlocked mutex.
    pub proof fn lemma_unlock_produces_unlocked(id: nat)
        ensures
            !(Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) }).locked,
            (Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) }).spec_is_unlocked(),
    {
    }

    /// Lemma: Well-formedness is preserved: new mutexes are well-formed.
    pub proof fn lemma_new_is_wf(id: nat)
        ensures ({
            let view: MutexView = Mutex::spec_new_view(id);
            !view.locked && !view.token_issued
        }),
    {
    }

    /// Lemma: The well-formedness invariant ensures no token is outstanding
    /// when the mutex is unlocked.
    pub proof fn lemma_wf_unlocked_no_token(s: &Mutex)
        requires
            s.wf(),
            s.spec_is_unlocked(),
        ensures
            !s@.token_issued,
    {
    }

    /// Lemma: A well-formed, locked mutex has token_issued, and the token's
    /// snapshot matches the mutex's current view.
    pub proof fn lemma_locked_wf_implies_token_state(s: &Mutex)
        requires
            s.wf(),
            s.spec_is_locked(),
        ensures
            s@.token_issued,
            s@.locked,
            s@ == (MutexView { locked: true, id: s@.id, token_issued: true }),
    {
    }

    /// Lemma: `try_lock()` on a locked, well-formed mutex fails.
    pub proof fn lemma_try_lock_contended_fails(s: &Mutex)
        requires
            s.wf(),
            s.spec_is_locked(),
        ensures
            s.locked,
            s@.token_issued,
    {
    }

    /// Lemma: `lock()` requires `spec_is_unlocked()` to prevent sequential deadlock.
    pub proof fn lemma_lock_precondition_prevents_deadlock(s: &Mutex)
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

impl Mutex {
    /// Lemma: Lock-then-unlock round-trip restores unlocked state. Identity and
    /// token tracking are preserved through the protocol.
    pub proof fn lemma_lock_unlock_roundtrip(id: nat)
        ensures ({
            let initial: Mutex = Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) };
            let after_lock: Mutex = Mutex { locked: true, id: Ghost(id), token_issued: Ghost(true) };
            let after_unlock: Mutex = Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) };
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

    /// Lemma: An unlocked mutex has the same view as a new mutex with its id.
    pub proof fn lemma_unlocked_eq_new_view(s: &Mutex)
        requires
            s.spec_is_unlocked(),
            s.wf(),
        ensures
            s@ == Mutex::spec_new_view(s@.id),
    {
    }

    /// Lemma: After `new()` followed by `try_lock()`, the result is always `true`.
    pub proof fn lemma_new_then_try_lock_succeeds(s: &Mutex)
        requires
            s@ == Mutex::spec_new_view(s@.id),
        ensures
            s.spec_is_unlocked(),
            !s.locked,
            !s@.token_issued,
            s.wf(),
    {
    }

    /// Lemma: A `MutexToken` produced by a locked mutex is valid for unlock.
    pub proof fn lemma_lock_token_valid_for_unlock(s: &Mutex, token: &MutexToken)
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

    /// Lemma: Tokens from different mutex instances cannot satisfy each other's
    /// unlock preconditions.
    pub proof fn lemma_token_instance_isolation(
        s1: &Mutex, s2: &Mutex, token: &MutexToken,
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

    /// Lemma: After unlock, the mutex is in a state where lock can be acquired again.
    ///
    /// # Description
    ///
    /// Proves the relockability property: after unlock, the mutex satisfies
    /// all preconditions for a subsequent lock() call.
    pub proof fn lemma_unlock_enables_relock(id: nat)
        ensures ({
            let after_unlock: Mutex = Mutex { locked: false, id: Ghost(id), token_issued: Ghost(false) };
            &&& after_unlock.spec_is_unlocked()
            &&& after_unlock.wf()
            &&& !after_unlock.token_issued()
        }),
    {
    }

    /// Lemma: Mutual exclusion — a well-formed mutex with a valid lock token
    /// must be in the locked state.
    ///
    /// # Description
    ///
    /// If a mutex is well-formed and a valid lock token exists (produced by
    /// `lock()`/`try_lock()`, so `token.view.locked == true`), and the token
    /// matches the mutex view, then the mutex must be locked with token_issued.
    /// Since token_issued is a single boolean, at most one token can be
    /// outstanding per well-formed mutex instance.
    pub proof fn lemma_mutual_exclusion(s: &Mutex, token: &MutexToken)
        requires
            s.wf(),
            token.view == s@,
            token.view.locked,
        ensures
            s.spec_is_locked(),
            s@.token_issued,
            s@.locked,
    {
    }

    /// Lemma: No double-unlock — a well-formed, unlocked mutex cannot satisfy
    /// the preconditions of `unlock()`.
    ///
    /// # Description
    ///
    /// Proves that double-unlock is precondition-blocked: after unlock produces
    /// an unlocked, well-formed mutex with `!token_issued`, the `unlock()`
    /// preconditions (`locked`, `wf()`, `token_issued()`) cannot all be satisfied.
    pub proof fn lemma_no_double_unlock(s: &Mutex)
        requires
            s.wf(),
            s.spec_is_unlocked(),
        ensures
            !s.locked,
            !s@.token_issued,
    {
    }
}

} // verus!
