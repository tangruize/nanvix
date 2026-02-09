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
//! - Sleep preserves wf: non-kernel running→suspended, ready→running.
//! - Exit preserves wf: non-kernel running→zombie, ready→running.
//! - Wakeup preserves wf: suspended→ready.
//! - Resume preserves wf: all interrupted→ready (batch transition).
//! - Terminate preserves wf: ready→zombie or suspended→interrupted.
//! - Harvest preserves wf: removes from zombie queue.
//! - Kernel safety: the kernel process (PID 0) is always alive (running or ready).
//!   The kernel process cannot be slept, exited, or terminated.
//! - Process partitioning: every PID is in exactly one queue at any time.
//! - Overflow safety: arithmetic on counts and PIDs is proven within bounds.
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
            // Counts bounded: total + 1 still < i32::MAX since total was < i32::MAX - 1.
            assert((self.ready_count + 1) as int + (self.suspended_count as int)
                + (self.interrupted_count as int) + (self.zombie_count as int) + 1
                < i32::MAX as int);
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
    {
        self.ghost_ready = Ghost(self.ghost_ready@.remove(pid as int));
        self.ghost_zombies = Ghost(self.ghost_zombies@.insert(pid as int));
        self.ready_count = self.ready_count - 1;
        self.zombie_count = self.zombie_count + 1;
    }

    /// Terminates a ready process that still has threads; moves to interrupted.
    pub fn terminate_ready_to_interrupted(&mut self, pid: i32)
        requires
            old(self).wf(),
            old(self).ghost_ready@.contains(pid as int),
            pid as int != 0int,
        ensures
            self.wf(),
            self.running_pid == old(self).running_pid,
            self.ghost_ready@ =~= old(self).ghost_ready@.remove(pid as int),
            self.ghost_interrupted@ =~= old(self).ghost_interrupted@.insert(pid as int),
            self.ready_count == old(self).ready_count - 1,
            self.interrupted_count == old(self).interrupted_count + 1,
            self.suspended_count == old(self).suspended_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
    {
        self.ghost_ready = Ghost(self.ghost_ready@.remove(pid as int));
        self.ghost_interrupted = Ghost(self.ghost_interrupted@.insert(pid as int));
        self.ready_count = self.ready_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
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
    {
        self.number_buffered_messages = self.number_buffered_messages + 1;
    }

    /// Decrements the buffered message count.
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
    {
        self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
        self.ghost_interrupted = Ghost(self.ghost_interrupted@.insert(pid as int));
        self.suspended_count = self.suspended_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }
}

} // verus!
