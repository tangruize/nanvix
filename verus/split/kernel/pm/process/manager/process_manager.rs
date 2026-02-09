// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessManagerInner Implementation
//!
//! Verified model of the kernel process manager state machine.
//!
//! ## Verified Properties
//!
//! - Construction produces a well-formed state with the kernel (PID 0) running.
//! - PID allocation is monotonic: new PIDs are always fresh.
//! - Schedule preserves wf: swaps running↔ready without losing processes.
//! - Full schedule preserves wf: composes resume_all_interrupted + schedule to
//!   match the original scheduler's semantics (alarm + resume + swap).
//! - Sleep preserves wf: non-kernel running→suspended, ready→running.
//! - Exit preserves wf: non-kernel running→zombie, ready→running.
//! - Wakeup preserves wf: suspended→ready, plus no-op variants for running/ready.
//! - Resume preserves wf: all interrupted→ready (batch transition).
//! - Terminate preserves wf: ready→zombie or suspended→interrupted.
//! - Harvest preserves wf: removes from zombie queue.
//! - Kernel safety: the kernel process (PID 0) is always alive (running or ready).
//!   The kernel process cannot be slept, exited, or terminated.
//! - Process partitioning: every PID is in exactly one queue at any time.
//! - Overflow safety: arithmetic on counts and PIDs is proven within bounds.
//! - Query/sync/thread stubs: all remaining original functions are modeled as
//!   verified no-ops that preserve wf() with appropriate preconditions.
//!
//! ## Verification Model
//!
//! The original `ProcessManagerInner` contains complex kernel types
//! (`RunningProcess`, `RunnableProcess`, `SleepingProcess`, `InterruptedProcess`,
//! `ZombieProcess`, `ThreadManager`, `LinkedList`) from various kernel subsystems.
//! For verification, we abstract these into:
//! - `running_pid: i32` — the PID of the single running process.
//! - `ready_count`, `suspended_count`, `interrupted_count`, `zombie_count` — runtime
//!   counts of processes in each queue.
//! - Ghost `Set<int>` for each queue — tracks PID membership for spec reasoning.
//! - `next_pid: i32` — the next PID to allocate, always > all existing PIDs.
//! - `number_buffered_messages: usize` — count of undelivered IPC messages.
//!
//! The `wf()` predicate ties the ghost sets to the runtime state and encodes:
//! - Finiteness and cardinality matching.
//! - Pairwise disjointness of all queues (including running).
//! - Kernel liveness: PID 0 is running or ready, never elsewhere.
//! - PID bounds: all PIDs in [0, next_pid).
//! - Overflow bounds on all counts.
//!
//! ## Trust Boundaries
//!
//! - **T1: Scheduler choice.** The `schedule`, `sleep_running`, `exit_running`, and
//!   `exit_thread_running` functions accept a `chosen_next` parameter modeling the PID
//!   selected by the scheduler (originally `take_earliest_ready`). The precondition
//!   requires it to be a valid ready PID.
//! - **T2: RefCell borrow.** The outer `ProcessManager` wraps `ProcessManagerInner`
//!   in `Rc<RefCell<_>>`. Runtime borrow checking (try_borrow/try_borrow_mut) is not
//!   modeled; it is an external boundary.
//! - **T3: Thread-level details.** The original manager tracks per-process thread
//!   sets. Thread-level state transitions (create_thread, exit_thread, etc.) affect
//!   whether a process goes to ready vs. suspended vs. zombie. We model the outcome
//!   as parameters (e.g., `to_zombie: bool`), trusting the thread-level logic.

use vstd::prelude::*;

// Include specifications.
include!("process_manager.spec.rs");

// Include proofs.
include!("process_manager.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Verification model of the kernel process manager.
pub struct ProcessManagerInner {
    /// PID of the currently running process.
    pub running_pid: i32,
    /// Number of processes in the ready queue.
    pub ready_count: usize,
    /// Number of processes in the suspended (sleeping) queue.
    pub suspended_count: usize,
    /// Number of processes in the interrupted queue.
    pub interrupted_count: usize,
    /// Number of processes in the zombie queue.
    pub zombie_count: usize,
    /// Next PID to allocate (monotonically increasing).
    pub next_pid: i32,
    /// Whether the platform supports interrupts.
    pub interrupt_capable: bool,
    /// Number of buffered IPC messages (not yet consumed).
    pub number_buffered_messages: usize,
    /// Ghost: set of PIDs in the ready queue.
    pub ghost_ready: Ghost<Set<int>>,
    /// Ghost: set of PIDs in the suspended queue.
    pub ghost_suspended: Ghost<Set<int>>,
    /// Ghost: set of PIDs in the interrupted queue.
    pub ghost_interrupted: Ghost<Set<int>>,
    /// Ghost: set of PIDs in the zombie queue.
    pub ghost_zombies: Ghost<Set<int>>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessManagerInner {

    //==============================================================================================
    // Construction
    //==============================================================================================

    /// Creates a new process manager with the kernel process (PID 0) running.
    pub fn new(interrupt_capable: bool) -> (result: Self)
        ensures
            result.wf(),
            result.spec_running_pid() == 0,
            result.ready_count == 0,
            result.suspended_count == 0,
            result.interrupted_count == 0,
            result.zombie_count == 0,
            result.next_pid == 1i32,
            result.interrupt_capable == interrupt_capable,
            result.number_buffered_messages == 0,
    {
        ProcessManagerInner {
            running_pid: 0i32,
            ready_count: 0usize,
            suspended_count: 0usize,
            interrupted_count: 0usize,
            zombie_count: 0usize,
            next_pid: 1i32,
            interrupt_capable,
            number_buffered_messages: 0usize,
            ghost_ready: Ghost(Set::empty()),
            ghost_suspended: Ghost(Set::empty()),
            ghost_interrupted: Ghost(Set::empty()),
            ghost_zombies: Ghost(Set::empty()),
        }
    }

    //==============================================================================================
    // Queries
    //==============================================================================================

    /// Returns the PID of the running process.
    pub fn get_running_pid(&self) -> (result: i32)
        requires self.wf(),
        ensures
            result as int == self.spec_running_pid(),
            result >= 0i32,
    {
        self.running_pid
    }

    /// Returns whether the ready queue is non-empty.
    pub fn has_ready(&self) -> (result: bool)
        requires self.wf(),
        ensures result == self.spec_has_ready(),
    {
        self.ready_count > 0
    }

    /// Returns whether the zombie queue is non-empty.
    pub fn has_zombies(&self) -> (result: bool)
        requires self.wf(),
        ensures result == self.spec_has_zombies(),
    {
        self.zombie_count > 0
    }

    /// Returns whether interrupts are supported.
    pub fn is_interrupt_capable(&self) -> (result: bool)
        requires self.wf(),
        ensures result == self.interrupt_capable,
    {
        self.interrupt_capable
    }

    /// Returns the number of buffered messages.
    pub fn get_buffered_message_count(&self) -> (result: usize)
        requires self.wf(),
        ensures result as nat == self.number_buffered_messages as nat,
    {
        self.number_buffered_messages
    }

    //==============================================================================================
    // Process Creation
    //==============================================================================================

    /// Creates a new process and adds it to the ready queue.
    pub fn create_process(&mut self) -> (result: i32)
        requires
            old(self).wf(),
            old(self).next_pid < i32::MAX,
        ensures
            self.wf(),
            result as int == old(self).next_pid as int,
            self.spec_running_pid() == old(self).spec_running_pid(),
            self.next_pid as int == old(self).next_pid as int + 1,
            self.ready_count == old(self).ready_count + 1,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(old(self).next_pid as int),
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.spec_process_exists(result as int),
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let pid: i32 = self.next_pid;

        proof {
            self.lemma_next_pid_is_fresh();
            // PID is fresh: not in any ghost set, so insert adds exactly 1.
            assert(!self.ghost_ready@.contains(pid as int));
            assert(self.ghost_ready@.insert(pid as int).len()
                == self.ghost_ready@.len() + 1);
            // The new PID won't violate disjointness since it's not in any set.
            assert(!self.ghost_suspended@.contains(pid as int));
            assert(!self.ghost_interrupted@.contains(pid as int));
            assert(!self.ghost_zombies@.contains(pid as int));
            // Running PID not in new ready set (it wasn't before, and it's != pid
            // because running_pid < next_pid = pid, so running_pid != pid).
            assert(self.running_pid as int != pid as int);
            assert(!self.ghost_ready@.insert(pid as int).contains(self.running_pid as int));
            // All PIDs in the new ready set are < pid + 1.
            assert(forall |p: int| self.ghost_ready@.insert(pid as int).contains(p)
                ==> 0 <= p && p < (pid + 1) as int);
            // PID bounds for other sets still hold with new next_pid.
            assert(forall |p: int| self.ghost_suspended@.contains(p)
                ==> p < (pid + 1) as int);
            assert(forall |p: int| self.ghost_interrupted@.contains(p)
                ==> p < (pid + 1) as int);
            assert(forall |p: int| self.ghost_zombies@.contains(p)
                ==> p < (pid + 1) as int);
            // Running PID is still < new next_pid.
            assert((self.running_pid as int) < (pid + 1) as int);
            // Counts bounded: old total + 1 ≤ old next_pid + 1 = new next_pid.
            assert((self.ready_count + 1) as int + (self.suspended_count as int)
                + (self.interrupted_count as int) + (self.zombie_count as int) + 1
                <= (pid + 1) as int);
        }

        self.ghost_ready = Ghost(self.ghost_ready@.insert(pid as int));
        self.ready_count = self.ready_count + 1;
        self.next_pid = pid + 1;

        pid
    }

    //==============================================================================================
    // Scheduling
    //==============================================================================================

    /// Reschedules: moves the running process to ready and runs chosen_next.
    pub fn schedule(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).spec_ready_with_running().contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(
                old(self).running_pid as int
            ).remove(chosen_next as int),
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready = Ghost(
            self.ghost_ready@.insert(old_running as int).remove(chosen_next as int)
        );
        self.running_pid = chosen_next;
    }

    //==============================================================================================
    // Sleep
    //==============================================================================================

    /// Suspends the running process and runs chosen_next from ready.
    pub fn sleep_running(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).ghost_ready@.contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_suspended@ =~= old(self).ghost_suspended@.insert(
                old(self).running_pid as int
            ),
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(chosen_next as int),
            self.ready_count == old(self).ready_count - 1,
            self.suspended_count == old(self).suspended_count + 1,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        self.ghost_suspended = Ghost(self.ghost_suspended@.insert(old_running as int));
        self.ghost_ready = Ghost(self.ghost_ready@.remove(chosen_next as int));
        self.running_pid = chosen_next;
        self.suspended_count = self.suspended_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Running thread sleeps but process still has runnable threads → stays ready.
    pub fn sleep_thread_running(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).spec_ready_with_running().contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(
                old(self).running_pid as int
            ).remove(chosen_next as int),
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready = Ghost(
            self.ghost_ready@.insert(old_running as int).remove(chosen_next as int)
        );
        self.running_pid = chosen_next;
    }

    //==============================================================================================
    // Exit
    //==============================================================================================

    /// Terminates the running process (moves to zombie) and runs chosen_next.
    pub fn exit_running(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).ghost_ready@.contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_zombies@ =~= old(self).ghost_zombies@.insert(
                old(self).running_pid as int
            ),
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(chosen_next as int),
            self.ready_count == old(self).ready_count - 1,
            self.zombie_count == old(self).zombie_count + 1,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        self.ghost_zombies = Ghost(self.ghost_zombies@.insert(old_running as int));
        self.ghost_ready = Ghost(self.ghost_ready@.remove(chosen_next as int));
        self.running_pid = chosen_next;
        self.zombie_count = self.zombie_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Running thread exits but process still has runnable threads → stays ready.
    pub fn exit_thread_running(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).spec_ready_with_running().contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(
                old(self).running_pid as int
            ).remove(chosen_next as int),
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready = Ghost(
            self.ghost_ready@.insert(old_running as int).remove(chosen_next as int)
        );
        self.running_pid = chosen_next;
    }

    /// Running thread exits; only sleeping threads remain → process to suspended.
    pub fn exit_thread_to_suspended(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).ghost_ready@.contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_suspended@ =~= old(self).ghost_suspended@.insert(
                old(self).running_pid as int
            ),
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(chosen_next as int),
            self.ready_count == old(self).ready_count - 1,
            self.suspended_count == old(self).suspended_count + 1,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        self.ghost_suspended = Ghost(self.ghost_suspended@.insert(old_running as int));
        self.ghost_ready = Ghost(self.ghost_ready@.remove(chosen_next as int));
        self.running_pid = chosen_next;
        self.suspended_count = self.suspended_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Running thread exits; all threads now zombies → process to zombie.
    pub fn exit_thread_to_zombie(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).running_pid as int != 0int,
            old(self).ghost_ready@.contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_zombies@ =~= old(self).ghost_zombies@.insert(
                old(self).running_pid as int
            ),
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(chosen_next as int),
            self.ready_count == old(self).ready_count - 1,
            self.zombie_count == old(self).zombie_count + 1,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        self.ghost_zombies = Ghost(self.ghost_zombies@.insert(old_running as int));
        self.ghost_ready = Ghost(self.ghost_ready@.remove(chosen_next as int));
        self.running_pid = chosen_next;
        self.zombie_count = self.zombie_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    //==============================================================================================
    // Wakeup
    //==============================================================================================

    /// Wakes up a suspended process and moves it to the ready queue.
    pub fn wakeup_to_ready(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_suspended@ =~= old(self).ghost_suspended@.remove(pid as int),
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(pid as int),
            self.ready_count == old(self).ready_count + 1,
            self.suspended_count == old(self).suspended_count - 1,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
        self.ghost_ready = Ghost(self.ghost_ready@.insert(pid as int));
        self.suspended_count = self.suspended_count - 1;
        self.ready_count = self.ready_count + 1;
    }

    //==============================================================================================
    // Resume Interrupted
    //==============================================================================================

    /// Resumes all interrupted processes by moving them to the ready queue.
    pub fn resume_all_interrupted(&mut self)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_ready@ =~= old(self).ghost_ready@.union(old(self).ghost_interrupted@),
            self.ghost_interrupted@ =~= Set::<int>::empty(),
            self.ready_count as int == old(self).ready_count as int
                + old(self).interrupted_count as int,
            self.interrupted_count == 0,
            self.suspended_count == old(self).suspended_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        proof {
            Self::lemma_union_disjoint_len(self.ghost_ready@, self.ghost_interrupted@);
        }

        self.ghost_ready = Ghost(self.ghost_ready@.union(self.ghost_interrupted@));
        self.ghost_interrupted = Ghost(Set::empty());
        self.ready_count = self.ready_count + self.interrupted_count;
        self.interrupted_count = 0;
    }

    //==============================================================================================
    // Terminate
    //==============================================================================================

    /// Terminates a ready process by moving it to the zombie queue.
    pub fn terminate_ready(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_ready@.contains(pid as int),
            pid as int != 0int,
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(pid as int),
            self.ghost_zombies@ =~= old(self).ghost_zombies@.insert(pid as int),
            self.ready_count == old(self).ready_count - 1,
            self.zombie_count == old(self).zombie_count + 1,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.ghost_ready = Ghost(self.ghost_ready@.remove(pid as int));
        self.ghost_zombies = Ghost(self.ghost_zombies@.insert(pid as int));
        self.ready_count = self.ready_count - 1;
        self.zombie_count = self.zombie_count + 1;
    }

    /// Terminates a ready process that still has threads; process stays in ready.
    ///
    /// Models `ProcessManagerInner::terminate()` for a process in the ready queue
    /// that has surviving threads. In the original code (mod.rs:1054-1058), the
    /// process goes through `terminate() → Ok(interrupted) → resume() → push_back(ready)`.
    /// The net queue-level effect is a no-op: the process remains in the ready queue.
    /// Internal thread state changes (marking the running thread for termination) are
    /// abstracted away as part of trust boundary T3.
    ///
    /// # Note on Verification Power
    ///
    /// This function takes `&self` (immutable reference), so the proof that `wf()`
    /// is preserved is trivially correct. The actual verification value lies in the
    /// *preconditions*: the function documents that only non-kernel PIDs in the ready
    /// queue reach this code path. The internal mutations (thread termination, process
    /// state transitions) are entirely within trust boundary T3.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the ready process to terminate (must not be kernel PID 0).
    pub fn terminate_ready_stays_ready(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_ready@.contains(pid as int),
            pid as int != 0int,
        ensures
            self.wf(),
    {
        // No queue-level state change: the process stays in ready after
        // terminate + resume. Internal thread state changes are out of scope (T3).
    }

    /// Terminates a suspended process by moving it to the interrupted queue.
    pub fn terminate_suspended(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_suspended@ =~= old(self).ghost_suspended@.remove(pid as int),
            self.ghost_interrupted@ =~= old(self).ghost_interrupted@.insert(pid as int),
            self.suspended_count == old(self).suspended_count - 1,
            self.interrupted_count == old(self).interrupted_count + 1,
            self.ready_count == old(self).ready_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
        self.ghost_interrupted = Ghost(self.ghost_interrupted@.insert(pid as int));
        self.suspended_count = self.suspended_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }

    //==============================================================================================
    // Zombie Harvesting
    //==============================================================================================

    /// Harvests (removes) a zombie process from the zombie queue.
    pub fn harvest_zombie(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_zombies@.contains(pid as int),
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_zombies@ =~= old(self).ghost_zombies@.remove(pid as int),
            self.zombie_count == old(self).zombie_count - 1,
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            !self.spec_process_exists(pid as int),
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.ghost_zombies = Ghost(self.ghost_zombies@.remove(pid as int));
        self.zombie_count = self.zombie_count - 1;
    }

    //==============================================================================================
    // Message Tracking
    //==============================================================================================

    /// Increments the buffered message count.
    pub fn post_message(&mut self)
        requires
            old(self).wf(),
            old(self).number_buffered_messages < usize::MAX - 1,
        ensures
            self.wf(),
            self.number_buffered_messages == old(self).number_buffered_messages + 1,
            self.running_pid == old(self).running_pid,
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.number_buffered_messages = self.number_buffered_messages + 1;
    }

    /// Decrements the buffered message count.
    ///
    /// Models the implicit decrement that occurs when messages are consumed
    /// through `ProcessState::receive_message()`. The original `ProcessManagerInner`
    /// does not have a direct `recv_message` method; instead, the decrement happens
    /// in `unsafe::try_recv()` (unsafe.rs:650-658) which calls
    /// `running.state_mut().receive_message(tid)` and then decrements the counter.
    pub fn recv_message(&mut self)
        requires
            old(self).wf(),
            old(self).number_buffered_messages > 0,
        ensures
            self.wf(),
            self.number_buffered_messages == old(self).number_buffered_messages - 1,
            self.running_pid == old(self).running_pid,
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.number_buffered_messages = self.number_buffered_messages - 1;
    }

    //==============================================================================================
    // Capability Control (no state machine change)
    //==============================================================================================

    /// Models capctl: no process state change.
    pub fn capctl(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Alarm Check (suspended → interrupted)
    //==============================================================================================

    /// Moves a suspended process to the interrupted queue due to alarm expiry.
    pub fn alarm_interrupt(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_suspended@ =~= old(self).ghost_suspended@.remove(pid as int),
            self.ghost_interrupted@ =~= old(self).ghost_interrupted@.insert(pid as int),
            self.suspended_count == old(self).suspended_count - 1,
            self.interrupted_count == old(self).interrupted_count + 1,
            self.ready_count == old(self).ready_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
        self.ghost_interrupted = Ghost(self.ghost_interrupted@.insert(pid as int));
        self.suspended_count = self.suspended_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }

    //==============================================================================================
    // Full Schedule (composition matching original schedule())
    //==============================================================================================

    /// Full scheduling cycle: resume all interrupted, then swap running↔ready.
    ///
    /// Models the complete `ProcessManagerInner::schedule()` (mod.rs:640-678).
    /// The original schedule performs three steps in sequence:
    /// 1. Move the running process to the ready queue.
    /// 2. Call `check_alarm()` (moves expired-alarm suspended→interrupted).
    /// 3. Resume all interrupted processes (interrupted→ready).
    /// 4. Select the next process from ready (`take_earliest_ready`).
    ///
    /// Step 2 (`check_alarm`) is modeled by zero or more preceding calls to
    /// `alarm_interrupt` (trust boundary T1: alarm expiry is a runtime decision).
    /// This function composes steps 1, 3, and 4 into a single verified operation
    /// that first merges all interrupted PIDs into ready, then performs the
    /// running↔ready swap.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID selected by the scheduler from the extended ready set
    ///   (after merging interrupted). Must be a valid PID in the combined set.
    pub fn full_schedule(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            // chosen_next must be in the ready+interrupted+running pool after merging.
            old(self).ghost_ready@.union(old(self).ghost_interrupted@).insert(
                old(self).running_pid as int
            ).contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.interrupted_count == 0,
            self.ghost_interrupted@ =~= Set::<int>::empty(),
            self.suspended_count == old(self).suspended_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        // Step 1+3: Resume all interrupted into ready.
        self.resume_all_interrupted();

        // Step 4: Schedule running↔ready swap (step 1 is incorporated here).
        self.schedule(chosen_next);
    }

    //==============================================================================================
    // Wakeup Variants (covering all wakeup/try_wakeup outcomes)
    //==============================================================================================

    /// Wakeup no-op: target thread is in the running process.
    ///
    /// Models `ProcessManagerInner::wakeup()` (mod.rs:786-802) when the thread
    /// belongs to the running process. The original calls `running_process.wakeup(tid)`
    /// which transitions the thread internally but does not change the process queue.
    /// This is a T3 boundary: thread-level state changes are not modeled.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the running process (must equal running_pid).
    pub fn wakeup_running_noop(&self, pid: i32)
        requires
            self.wf(),
            self.running_pid as int == pid as int,
        ensures
            self.wf(),
    {
        // Thread wakeup within the running process: no queue-level change (T3).
    }

    /// Wakeup no-op: target thread is in a ready process.
    ///
    /// Models `ProcessManagerInner::try_wakeup()` (mod.rs:848-872) when the thread
    /// belongs to a process already in the ready queue. The original calls
    /// `process.wakeup(tid)` and pushes back to ready. Net queue effect: no change.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the ready process containing the target thread.
    pub fn wakeup_ready_noop(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_ready@.contains(pid as int),
        ensures
            self.wf(),
    {
        // Thread wakeup within a ready process: no queue-level change (T3).
    }

    //==============================================================================================
    // Query Operations (no state change)
    //==============================================================================================

    /// Models `find_process`: looks up a process by PID across all queues.
    ///
    /// Verifies the precondition that the process must exist, and that the
    /// operation does not mutate state.
    pub fn find_process(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }

    /// Models `find_process_mut`: mutable lookup of a process by PID.
    ///
    /// Although the original returns a mutable reference, the lookup itself
    /// does not change queue membership. Mutations through the returned
    /// reference are thread-level or state-level (T3) and do not affect queues.
    pub fn find_process_mut(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }

    /// Models `interrupt_reason`: returns and clears the interrupt reason.
    ///
    /// The original takes `&mut self` but only modifies the `interrupt_reason`
    /// field, which is not part of the queue state machine. No queue change.
    pub fn take_interrupt_reason(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // interrupt_reason is not part of the queue state machine model.
    }

    //==============================================================================================
    // Thread-Level Operations (T3 boundary, no queue change)
    //==============================================================================================

    /// Models `create_thread`: creates a new thread in an existing process.
    ///
    /// The queue-level effect depends on the process state:
    /// - If the process is sleeping, it moves to ready (modeled by `wakeup_to_ready`).
    /// - If the process is ready, it stays in ready (no change modeled here).
    /// The actual thread creation is trust boundary T3.
    ///
    /// This stub models the ready-process case (no queue change).
    pub fn create_thread_in_ready(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_ready@.contains(pid as int),
        ensures
            self.wf(),
    {
        // Thread creation in a ready process: no queue-level change (T3).
    }

    /// Models `set_thread_data_area`: sets the TDA for a thread in a sleeping process.
    ///
    /// No queue-level state change. The original requires the process to be
    /// sleeping and the thread to be sleeping within it.
    pub fn set_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
        // Thread metadata update: no queue-level change (T3).
    }

    /// Models `get_thread_data_area`: reads the TDA for a thread in a sleeping process.
    ///
    /// Pure query: no state change.
    pub fn get_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
    }

    /// Models `try_join_thread`: attempts to join a thread.
    ///
    /// The original returns either a zombie thread (success) or a condvar/error.
    /// No queue-level state change occurs.
    pub fn try_join_thread(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
        // Thread join: no queue-level change (T3).
    }

    //==============================================================================================
    // Synchronization Primitives (T3 boundary, no queue change)
    //==============================================================================================

    /// Models `get_mutex`: retrieves or creates a mutex for the running process.
    ///
    /// Mutex state is per-process, not per-queue. No queue change.
    pub fn get_mutex(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `get_cond`: retrieves or creates a condition variable.
    ///
    /// Condvar state is per-process, not per-queue. No queue change.
    pub fn get_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `put_cond`: releases a condition variable.
    ///
    /// Condvar state is per-process, not per-queue. No queue change.
    pub fn put_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `put_mutex_guard`: stores a mutex guard in the running thread.
    ///
    /// Mutex guard tracking is per-thread, not per-queue. No queue change.
    pub fn put_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `take_mutex_guard`: removes a mutex guard from a thread.
    ///
    /// Mutex guard tracking is per-thread, not per-queue. No queue change.
    pub fn take_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `handle_fpu_exception`: saves/restores FPU state.
    ///
    /// FPU state management is per-thread, not per-queue. No queue change.
    pub fn handle_fpu_exception(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
}

} // verus!
