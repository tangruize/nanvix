// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Fence Implementation
//!
//! A synchronization primitive that allows a thread to wait for a number of
//! signals to be delivered.
//!
//! ## Verified Properties
//!
//! - A new fence has count == 0 and the given total, and is well-formed.
//! - `signal()` increments count by exactly 1 and preserves well-formedness.
//! - `signal()` is only valid when the fence is waiting (count < total).
//! - `wait()` postcondition guarantees the fence is satisfied (count >= total).
//! - `spec_is_satisfied` and `spec_is_waiting` are complementary predicates.
//! - Well-formedness (`wf()`) enforces: count <= total.
//! - Satisfaction is monotone: once satisfied, a fence stays satisfied.
//! - After exactly `total` signals from a new fence, it is satisfied.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicUsize` for the
//! signal count with `Ordering::Acquire`/`Ordering::Release` memory ordering.
//! For verification, we model the count as a plain `usize` field and use
//! `&mut self` for state transitions. This is a sequential model that verifies
//! the fence protocol (state machine correctness) without reasoning about
//! atomicity or memory ordering.
//!
//! **This verified code is a specification model, not a runtime replacement.**
//! The kernel uses the original `src/kernel/src/pm/sync/fence.rs` (with
//! `AtomicUsize` and spin-wait loop) at runtime. The verified model proves
//! the state machine protocol is correct: every reachable state satisfies
//! `wf()`, and signal/wait transitions are sound.
//!
//! ## Verification Scope
//!
//! This verification proves **sequential state machine correctness** of the
//! fence protocol. The following are explicitly **out of scope**:
//! - **Concurrency and atomicity**: The sequential `&mut self` model does not
//!   capture concurrent thread access or atomic memory ordering. A future
//!   extension could use Verus atomic modeling (`vstd::atomic*`) or a
//!   rely/guarantee framework to verify concurrent correctness.
//! - **Liveness under concurrency**: Termination of `wait()` depends on
//!   concurrent signalers making progress, which is not modeled.
//! - **Overflow**: The model uses `usize` for count; overflow protection
//!   is provided by the `count < total` precondition on `signal()`.
//!
//! ## API Divergence
//!
//! The original `wait(&self)` uses `&self` with `AtomicUsize` interior mutability.
//! The verified `wait(&self)` keeps `&self` since it is a pure observer that
//! does not modify state (it only reads `count` and `total`). The verified
//! `signal(&mut self)` uses `&mut self` because Verus requires exclusive
//! references for state mutation. In the concurrent original, `&self` access
//! is safe due to `AtomicUsize` interior mutability.
//!
//! **`signal()` precondition strengthening:** The original `signal(&self)`
//! unconditionally calls `fetch_add(1, Ordering::Release)` with no guard
//! against over-signaling (calling `signal` more than `total` times). The
//! verified model intentionally adds `spec_is_waiting()` (`count < total`)
//! as a precondition, enforcing a strict protocol where each fence receives
//! exactly `total` signals. This is a deliberate strengthening that prevents
//! over-signaling and `usize` overflow. Note: the runtime tolerates
//! over-signaling benignly (the atomic increment simply exceeds `total`, and
//! `wait()` still terminates because `count >= total`). The startup fence in
//! `kmain.rs` creates `Fence::new(ncores - 1)` but starts `ncores`
//! application cores, each calling `signal()` once — the last signal
//! over-shoots by one, which is harmless at runtime. The verified model's
//! stricter precondition would flag this as a protocol violation, making it
//! a conscious divergence from runtime behavior in favor of tighter
//! verification guarantees.
//!
//! ## Trust Boundaries
//!
//! - `wait()`: **Key verification gap.** Modeled as a no-op with a precondition
//!   that the fence is already satisfied. The original `wait()` is a *blocking*
//!   operation: it spins until concurrent signalers establish satisfaction. The
//!   sequential model cannot express this blocking behavior, so instead the
//!   caller must prove satisfaction before calling `wait()`. This means the
//!   verified `wait()` does not capture the fundamental purpose of the original:
//!   waiting for concurrent signals. The trust boundary is that concurrent
//!   signalers will eventually satisfy the fence—this assumption is implicit
//!   in the precondition rather than modeled as an explicit concurrency axiom.
//! - `signal(&mut self)`: Uses `&mut self` (exclusive access) instead of the
//!   original `&self` with `AtomicUsize`. This means the verified model cannot
//!   represent the concurrent case where multiple threads call `signal()`
//!   simultaneously. See `lemma_signal_commutativity` for a spec-level proof
//!   that signal ordering does not affect the final state.
//! - `arch::cpu::pause()`: CPU hint with no semantic effect on fence state.
//!
//! ## Trust Assumptions
//!
//! - **T1: No overflow.** The count is bounded by total, which is a `usize`.
//!   Since `signal()` requires `count < total`, overflow cannot occur.

use vstd::prelude::*;

// Include specifications.
include!("fence.spec.rs");

// Include proofs.
include!("fence.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A synchronization primitive that allows a thread to wait for a number of
/// signals to be delivered.
///
/// # Description
///
/// Wraps a signal counter and a total. In the original implementation, the
/// counter is an `AtomicUsize`; here it is a plain `usize` for verification.
///
/// # Representation
///
/// The fields are `pub` as required by Verus for `pub open spec fn` access.
pub struct Fence {
    /// Number of signals received so far.
    pub count: usize,
    /// Total number of signals to wait for.
    pub total: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Fence {
    /// Instantiates a new fence.
    ///
    /// # Description
    ///
    /// Note: The original `new` is a `const fn`. Verus does not currently
    /// support `const fn` verification, so this is modeled as a plain `fn`.
    ///
    /// # Parameters
    ///
    /// - `total`: Total number of signals to wait for.
    ///
    /// # Returns
    ///
    /// A new `Fence` with zero signals received and the given total.
    pub fn new(total: usize) -> (result: Self)
        ensures
            result.count == 0,
            result.total == total,
            result.spec_count() == 0,
            result.spec_total() == total as nat,
            result@ == Fence::spec_new_view(total as nat),
            result.wf(),
            total == 0 ==> result.spec_is_satisfied(),
            total > 0 ==> result.spec_is_waiting(),
    {
        Fence { count: 0, total }
    }

    /// Waits for all signals to be received.
    ///
    /// # Description
    ///
    /// In the original implementation, this spins in a loop with
    /// `Ordering::Acquire` loads until `count >= total`. In the sequential
    /// verification model, the precondition requires the fence to already be
    /// satisfied, making this a no-op that establishes the postcondition.
    ///
    /// # Precondition
    ///
    /// The fence must be satisfied (all signals received). In the concurrent
    /// runtime, this is eventually established by other threads calling
    /// `signal()`. In the sequential model, the caller must ensure it.
    pub fn wait(&self)
        requires
            self.wf(),
            self.spec_is_satisfied(),
        ensures
            self.spec_is_satisfied(),
            self.count as nat >= self.total as nat,
    {
        // In the sequential model, the precondition guarantees satisfaction,
        // so the spin loop body is never entered.
    }

    /// Signals the fence by incrementing the count.
    ///
    /// # Description
    ///
    /// Models the atomic `fetch_add(1, Ordering::Release)` from the original.
    /// Increments the signal count by one.
    ///
    /// # Precondition
    ///
    /// The fence must be waiting (not yet satisfied). This prevents signaling
    /// past the total and ensures count does not overflow.
    pub fn signal(&mut self)
        requires
            old(self).wf(),
            old(self).spec_is_waiting(),
        ensures
            self.count == old(self).count + 1,
            self.total == old(self).total,
            self.spec_count() == old(self).spec_count() + 1,
            self.spec_total() == old(self).spec_total(),
            self.wf(),
            old(self).spec_remaining() > 0 ==> self.spec_remaining() == old(self).spec_remaining() - 1,
            self.spec_remaining() == 0 ==> self.spec_is_satisfied(),
    {
        self.count = self.count + 1;
    }

    /// Checks if the fence is satisfied.
    ///
    /// # Description
    ///
    /// Verification-only accessor not present in the original runtime code.
    ///
    /// # Returns
    ///
    /// `true` if all signals have been received, `false` otherwise.
    pub fn is_satisfied(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_satisfied(),
            result == (self.count as nat >= self.total as nat),
    {
        self.count >= self.total
    }

    /// Returns the number of signals received so far.
    ///
    /// # Description
    ///
    /// Verification-only accessor not present in the original runtime code.
    ///
    /// # Returns
    ///
    /// The current signal count.
    pub fn get_count(&self) -> (result: usize)
        ensures
            result == self.count,
            result as nat == self.spec_count(),
    {
        self.count
    }

    /// Returns the total number of signals required.
    ///
    /// # Description
    ///
    /// Verification-only accessor not present in the original runtime code.
    ///
    /// # Returns
    ///
    /// The total signal count.
    pub fn get_total(&self) -> (result: usize)
        ensures
            result == self.total,
            result as nat == self.spec_total(),
    {
        self.total
    }
}

} // verus!
