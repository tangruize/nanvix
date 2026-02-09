// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # SleepingProcess Implementation
//!
//! Represents a process that is waiting for a condition to be satisfied in
//! the Nanvix kernel. A SleepingProcess has at least one sleeping thread
//! and optional zombie threads.
//!
//! ## Verified Properties
//!
//! - Construction (new) produces well-formed state with correct identity.
//! - Process identifier (PID) is immutable: all operations preserve it.
//! - `terminate()` converts all sleeping threads to interrupted, producing
//!   an InterruptedProcess with PID preserved and non-empty interrupted list.
//! - `wakeup(tid)` removes a sleeping thread by ID and transitions to
//!   RunnableProcess with that thread as the only ready thread. Returns Err
//!   if the thread is not found.
//! - `wakeup_alarm()` partitions sleeping threads into expired-alarm
//!   (→interrupted) and remaining (→sleeping). Returns Ok(InterruptedProcess)
//!   if any expired, Err(SleepingProcess) otherwise.
//! - `add_thread()` adds a ready thread and transitions to RunnableProcess.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only via
//!   `spec_find_thread()`.
//! - Well-formedness (including thread ID uniqueness and list disjointness)
//!   is preserved by all operations.
//!
//! ## Verification Model
//!
//! The original `SleepingProcess` contains complex kernel types. For verification:
//! - `Box<ProcessState>` -> PID (int, identity tracking only).
//! - `NonEmptyVecDeque<SleepingThread>` -> `Seq<int>` of thread IDs (ghost).
//! - `Option<NonEmptyVecDeque<ZombieThread>>` -> `Seq<int>` (empty = None).
//! - `InterruptReason` -> elided (always `Killed` in terminate, `TimedOut` in
//!   wakeup_alarm — the reason value does not affect state machine logic).
//! - `alarm: Option<SystemTime>` -> elided. Alarm-based partitioning in
//!   `wakeup_alarm()` is modeled via oracle parameters.
//!
//! ## Trust Boundary
//!
//! - `RunnableProcess`, `InterruptedProcess` are boundary models of sibling modules.
//! - Thread state transitions (interrupt(), wakeup()) are ID-preserving.
//! - `find_thread()` / `find_thread_mut()` return reference types that Verus
//!   cannot express; modeled spec-only.
//! - `state()` / `state_mut()` return references to ProcessState; modeled as
//!   external_body with frame conditions.
//! - `wakeup_alarm()` uses oracle parameters for the alarm-based partition.
//!   The per-thread alarm comparison (`now >= alarm`) is simple arithmetic
//!   that is not modeled. Oracle preconditions ensure conservation.
//!
//! ## Oracle Parameters
//!
//! - `wakeup(tid, found)`: `found` oracle tied to `spec_seq_contains()`.
//! - `wakeup_alarm(has_expired, interrupted_ids, remaining_ids)`: partition
//!   oracle tied to content and length conservation constraints. All partition
//!   elements must come from the original sleeping list, with no duplicates
//!   within or across partitions. Both partitions must be subsequences of the
//!   original sleeping list, capturing the stable partition ordering of the
//!   original implementation.
//!
//! ## Fields
//!
//! All struct fields are `pub` for Verus proof ergonomics. The original has
//! private fields with getter/setter methods.

use vstd::prelude::*;

// Include specifications.
include!("sleeping.spec.rs");

// Include proofs.
include!("sleeping.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A process that is waiting for a condition to be satisfied.
///
/// Verification model of `src/kernel/src/pm/process/state/sleeping.rs::SleepingProcess`.
pub struct SleepingProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: Ghost<int>,
    /// Ghost sequence of sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of zombie thread IDs (may be empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exec-level count of sleeping threads.
    pub sleeping_count: u64,
}

/// A process that is ready to run (boundary model).
///
/// Models `RunnableProcess` from the sibling module.
pub struct RunnableProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that was interrupted (boundary model).
///
/// Models `InterruptedProcess` from the sibling module.
pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

//==================================================================================================
// SleepingProcess Implementation
//==================================================================================================

impl SleepingProcess {
    /// Creates a new SleepingProcess.
    ///
    /// Models the original `SleepingProcess::new(state, sleeping_threads, zombie_threads)`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `sleeping_ids`: Ghost sleeping thread IDs (must be non-empty).
    /// - `zombie_ids`: Ghost zombie thread IDs (may be empty).
    /// - `sleeping_count`: Exec-level count of sleeping threads.
    ///
    /// # Returns
    ///
    /// A new, well-formed SleepingProcess.
    pub fn new(
        pid: Ghost<int>,
        sleeping_ids: Ghost<Seq<int>>,
        zombie_ids: Ghost<Seq<int>>,
        sleeping_count: u64,
    ) -> (result: SleepingProcess)
        requires
            sleeping_count as nat == sleeping_ids@.len(),
            sleeping_ids@.len() >= 1,
            Self::spec_no_duplicates(sleeping_ids@),
            Self::spec_no_duplicates(zombie_ids@),
            Self::spec_seqs_disjoint(sleeping_ids@, zombie_ids@),
        ensures
            result.spec_pid() == pid@,
            result.sleeping_thread_ids@ == sleeping_ids@,
            result.zombie_thread_ids@ == zombie_ids@,
            result.spec_sleeping_count() == sleeping_ids@.len(),
            result.wf(),
    {
        SleepingProcess {
            pid,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
            sleeping_count,
        }
    }

    /// Returns the process state (modeled as PID).
    ///
    /// Models the original `SleepingProcess::state()`.
    ///
    /// # Returns
    ///
    /// The process identifier.
    #[verifier::external_body]
    pub fn state(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
    {
        unimplemented!()
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `SleepingProcess::state_mut()`.
    /// Callers must ensure PID immutability after mutation.
    ///
    /// # Returns
    ///
    /// The process identifier (as a ghost value).
    #[verifier::external_body]
    pub fn state_mut(&mut self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
            self.spec_pid() == old(self).spec_pid(),
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.sleeping_count == old(self).sleeping_count,
    {
        unimplemented!()
    }

    /// Terminates the process by interrupting all sleeping threads.
    ///
    /// Models the original `SleepingProcess::terminate()`.
    /// All sleeping threads are converted to interrupted threads (with
    /// InterruptReason::Killed). Thread IDs are preserved through the transition.
    ///
    /// # Returns
    ///
    /// An InterruptedProcess with all threads interrupted.
    pub fn terminate(self) -> (result: InterruptedProcess)
        requires
            self.wf(),
        ensures
            result.spec_pid() == self.spec_pid(),
            result.wf(),
            // All sleeping threads become interrupted (ID-preserving transition).
            result.interrupted_thread_ids@ == self.sleeping_thread_ids@,
            result.interrupted_thread_ids@.len() == self.spec_sleeping_count(),
            // No sleeping threads remain (all were interrupted).
            result.sleeping_thread_ids@.len() == 0,
            // Zombie threads are preserved.
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        proof {
            assert(self.sleeping_thread_ids@.len() >= 1);
        }

        InterruptedProcess {
            pid: Ghost(self.pid@),
            interrupted_thread_ids: Ghost(self.sleeping_thread_ids@),
            sleeping_thread_ids: Ghost(Seq::empty()),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        }
    }

    /// Wakes up a sleeping thread and transitions to RunnableProcess.
    ///
    /// Models the original `SleepingProcess::wakeup(tid)`.
    /// Searches for the thread with the given ID; if found, removes it from
    /// the sleeping list and creates a RunnableProcess with that thread as
    /// the only ready thread. If not found, returns self unchanged.
    ///
    /// ## Oracle Parameter
    ///
    /// The `found` parameter is required because the sleeping list is ghost
    /// (`Ghost<Seq<int>>`), so `Seq::contains()` cannot be evaluated at exec
    /// time. The precondition ties `found` to `spec_seq_contains()`.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to wake up.
    /// - `found`: Oracle — whether the thread is in the sleeping list.
    ///
    /// # Returns
    ///
    /// Ok(RunnableProcess) if found, Err(SleepingProcess) if not found.
    pub fn wakeup(
        self,
        tid: Ghost<int>,
        found: bool,
    ) -> (result: Result<RunnableProcess, SleepingProcess>)
        requires
            self.wf(),
            found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            match result {
                Ok(rp) => {
                    found
                    && rp.spec_pid() == self.spec_pid()
                    && rp.wf()
                    // Exactly one ready thread: the woken thread.
                    && rp.ready_thread_ids@.len() == 1
                    && rp.ready_thread_ids@[0] == tid@
                    // No interrupted threads.
                    && rp.interrupted_thread_ids@.len() == 0
                    // Sleeping list has the found thread removed.
                    // Under wf() no_duplicates, this existential is uniquely determined.
                    && (exists|idx: int| 0 <= idx < self.sleeping_thread_ids@.len()
                        && self.sleeping_thread_ids@[idx] == tid@
                        && rp.sleeping_thread_ids@ ==
                            Self::spec_remove_at(self.sleeping_thread_ids@, idx))
                    && rp.sleeping_thread_ids@.len() == self.spec_sleeping_count() - 1
                    // The woken thread is no longer in the sleeping list.
                    && !Self::spec_seq_contains(rp.sleeping_thread_ids@, tid@)
                    // Zombie threads preserved.
                    && rp.zombie_thread_ids@ == self.zombie_thread_ids@
                },
                Err(sp) => {
                    !found
                    && sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    && sp.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    && sp.zombie_thread_ids@ == self.zombie_thread_ids@
                    && sp.sleeping_count == self.sleeping_count
                },
            },
    {
        if !found {
            return Err(SleepingProcess {
                pid: Ghost(self.pid@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                sleeping_count: self.sleeping_count,
            });
        }

        // Derive the index via proof using `choose`.
        let ghost found_idx: int = choose|i: int|
            0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid@;

        proof {
            self.lemma_spec_find_sleeping_index(tid);
        }

        let ghost new_sleeping_ids: Seq<int> =
            self.sleeping_thread_ids@.subrange(0, found_idx)
                .add(self.sleeping_thread_ids@.subrange(
                    found_idx + 1,
                    self.sleeping_thread_ids@.len() as int,
                ));

        proof {
            // Prove lengths.
            let s: Seq<int> = self.sleeping_thread_ids@;
            let idx: int = found_idx;
            let left: Seq<int> = s.subrange(0, idx);
            let right: Seq<int> = s.subrange(idx + 1, s.len() as int);
            assert(left.len() == idx as nat);
            assert(right.len() == (s.len() - idx as nat - 1) as nat);
            assert(left.add(right).len() == (s.len() - 1) as nat);

            // Prove new_sleeping_ids matches spec_remove_at.
            assert(new_sleeping_ids =~= Self::spec_remove_at(s, idx));

            // Prove the woken thread is no longer in the sleeping list.
            Self::lemma_remove_at_removes_element(s, idx);

            // Prove single-element ready list properties.
            let ready: Seq<int> = Seq::<int>::empty().push(tid@);
            assert(ready.len() == 1);
            assert(ready[0] == tid@);
        }

        Ok(RunnableProcess {
            pid: Ghost(self.pid@),
            ready_thread_ids: Ghost(Seq::<int>::empty().push(tid@)),
            interrupted_thread_ids: Ghost(Seq::empty()),
            sleeping_thread_ids: Ghost(new_sleeping_ids),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        })
    }

    /// Wakes up sleeping threads with expired alarms.
    ///
    /// Models the original `SleepingProcess::wakeup_alarm(now)`.
    /// Partitions sleeping threads into those with expired alarms (→interrupted)
    /// and those without (→remaining sleeping). Returns Ok(InterruptedProcess)
    /// if any threads had expired alarms, Err(SleepingProcess) otherwise.
    ///
    /// ## Oracle Parameters
    ///
    /// The alarm-based partition is provided as oracle parameters because:
    /// - Per-thread alarm data (`Option<SystemTime>`) is not modeled in this
    ///   verification module (alarm checking is simple comparison logic).
    /// - The partition cannot be computed from ghost state alone.
    /// - Conservation constraint ties the oracle to structural correctness.
    ///
    /// # Parameters
    ///
    /// - `has_expired`: Oracle — whether any thread has an expired alarm.
    /// - `interrupted_ids`: Oracle — thread IDs with expired alarms.
    /// - `remaining_ids`: Oracle — thread IDs without expired alarms.
    ///
    /// # Returns
    ///
    /// Ok(InterruptedProcess) if expired threads exist, Err(SleepingProcess) otherwise.
    pub fn wakeup_alarm(
        self,
        has_expired: bool,
        interrupted_ids: Ghost<Seq<int>>,
        remaining_ids: Ghost<Seq<int>>,
    ) -> (result: Result<InterruptedProcess, SleepingProcess>)
        requires
            self.wf(),
            // Oracle: partition must be a valid decomposition.
            has_expired == (interrupted_ids@.len() > 0),
            // Length conservation.
            interrupted_ids@.len() + remaining_ids@.len()
                == self.sleeping_thread_ids@.len(),
            // Content conservation: all partition elements come from original sleeping list.
            forall|i: int| #![auto] 0 <= i < interrupted_ids@.len() ==>
                Self::spec_seq_contains(self.sleeping_thread_ids@, interrupted_ids@[i]),
            forall|i: int| #![auto] 0 <= i < remaining_ids@.len() ==>
                Self::spec_seq_contains(self.sleeping_thread_ids@, remaining_ids@[i]),
            // Partition integrity: no duplicates within or across partitions.
            Self::spec_no_duplicates(interrupted_ids@),
            Self::spec_no_duplicates(remaining_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, remaining_ids@),
            // Stable partition: both partitions preserve relative order from the original.
            Self::spec_is_subsequence(interrupted_ids@, self.sleeping_thread_ids@),
            Self::spec_is_subsequence(remaining_ids@, self.sleeping_thread_ids@),
            // If not expired, sleeping list is preserved.
            !has_expired ==> remaining_ids@ =~= self.sleeping_thread_ids@,
        ensures
            match result {
                Ok(ip) => {
                    has_expired
                    && ip.spec_pid() == self.spec_pid()
                    && ip.wf()
                    && ip.interrupted_thread_ids@ == interrupted_ids@
                    && ip.interrupted_thread_ids@.len() >= 1
                    && ip.sleeping_thread_ids@ == remaining_ids@
                    && ip.zombie_thread_ids@ == self.zombie_thread_ids@
                    // Conservation: partition sizes sum to original.
                    && ip.interrupted_thread_ids@.len() + ip.sleeping_thread_ids@.len()
                        == self.spec_sleeping_count()
                    // Stable ordering: partitions are subsequences of the original.
                    && Self::spec_is_subsequence(ip.interrupted_thread_ids@, self.sleeping_thread_ids@)
                    && Self::spec_is_subsequence(ip.sleeping_thread_ids@, self.sleeping_thread_ids@)
                },
                Err(sp) => {
                    !has_expired
                    && sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    && sp.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    && sp.zombie_thread_ids@ == self.zombie_thread_ids@
                    && sp.sleeping_count == self.sleeping_count
                },
            },
    {
        if has_expired {
            proof {
                assert(interrupted_ids@.len() >= 1);
            }

            Ok(InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: interrupted_ids,
                sleeping_thread_ids: remaining_ids,
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            })
        } else {
            // No alarm expired: the process remains sleeping with all state unchanged.
            // sleeping_count, sleeping_thread_ids, and zombie_thread_ids are identity-preserved.
            Err(SleepingProcess {
                pid: Ghost(self.pid@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                sleeping_count: self.sleeping_count,
            })
        }
    }

    /// Adds a ready thread and transitions to RunnableProcess.
    ///
    /// Models the original `SleepingProcess::add_thread(ready_thread)`.
    /// The ready thread becomes the only ready thread in the resulting
    /// RunnableProcess. All sleeping and zombie threads are preserved.
    ///
    /// # Parameters
    ///
    /// - `ready_tid`: Ghost thread identifier of the thread to add.
    ///
    /// # Returns
    ///
    /// A RunnableProcess with the added thread as the only ready thread.
    pub fn add_thread(self, ready_tid: Ghost<int>) -> (result: RunnableProcess)
        requires
            self.wf(),
            // The added thread must not collide with existing sleeping or zombie threads.
            // In the original code, Rust ownership prevents this; we enforce it explicitly.
            !Self::spec_seq_contains(self.sleeping_thread_ids@, ready_tid@),
            !Self::spec_seq_contains(self.zombie_thread_ids@, ready_tid@),
        ensures
            result.spec_pid() == self.spec_pid(),
            result.wf(),
            // Exactly one ready thread: the added thread.
            result.ready_thread_ids@.len() == 1,
            result.ready_thread_ids@[0] == ready_tid@,
            // No interrupted threads.
            result.interrupted_thread_ids@.len() == 0,
            // Sleeping threads preserved.
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            // Zombie threads preserved.
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        proof {
            let ready: Seq<int> = Seq::<int>::empty().push(ready_tid@);
            assert(ready.len() == 1);
            assert(ready[0] == ready_tid@);
        }

        RunnableProcess {
            pid: Ghost(self.pid@),
            ready_thread_ids: Ghost(Seq::<int>::empty().push(ready_tid@)),
            interrupted_thread_ids: Ghost(Seq::empty()),
            sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        }
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// Models the original `SleepingProcess::find_thread(tid)`.
    /// Returns the abstract list variant from `spec_find_thread()`:
    /// - `Some(0)`: sleeping thread.
    /// - `Some(1)`: zombie thread.
    /// - `None`: not found.
    ///
    /// The original returns `Option<ThreadRef>` with actual references. Since
    /// Verus cannot express reference-returning functions, callers must
    /// independently verify correct use of the returned reference against
    /// this spec model when reference types become expressible in Verus.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread(&self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self.spec_find_thread(tid@),
    {
        Ghost(self.spec_find_thread(tid@))
    }

    /// Finds a thread by its identifier (mutable variant).
    ///
    /// Models the original `SleepingProcess::find_thread_mut(tid)`.
    /// Same semantics as `find_thread()`. Frame condition: self is unchanged.
    /// The mutable reference in the original allows in-place mutation of
    /// the found thread. Callers must preserve the thread's identity and
    /// list membership after such mutation.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread_mut(&mut self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self).spec_find_thread(tid@),
            self.spec_pid() == old(self).spec_pid(),
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.sleeping_count == old(self).sleeping_count,
            self.wf() == old(self).wf(),
    {
        Ghost(old(self).spec_find_thread(tid@))
    }
}

} // verus!
