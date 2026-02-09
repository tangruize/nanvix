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
///
/// This is the abstract state machine that tracks which processes are in which
/// lifecycle state. Complex kernel types (process objects, thread manager, linked
/// lists) are abstracted to PID sets and counts.
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
    ///
    /// # Parameters
    ///
    /// - `interrupt_capable`: Whether the platform supports interrupts.
    ///
    /// # Returns
    ///
    /// A well-formed process manager with PID 0 running and all queues empty.
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
        requires
            self.wf(),
        ensures
            result as int == self.spec_running_pid(),
            result >= 0i32,
    {
        self.running_pid
    }

    /// Returns whether the ready queue is non-empty.
    pub fn has_ready(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_has_ready(),
    {
        self.ready_count > 0
    }

    /// Returns whether the zombie queue is non-empty.
    pub fn has_zombies(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_has_zombies(),
    {
        self.zombie_count > 0
    }

    /// Returns whether interrupts are supported.
    pub fn is_interrupt_capable(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.interrupt_capable,
    {
        self.interrupt_capable
    }

    /// Returns the number of buffered messages.
    pub fn get_buffered_message_count(&self) -> (result: usize)
        requires
            self.wf(),
        ensures
            result as nat == self.number_buffered_messages as nat,
    {
        self.number_buffered_messages
    }

    //==============================================================================================
    // Process Creation
    //==============================================================================================

    /// Creates a new process and adds it to the ready queue.
    ///
    /// Models `ProcessManagerInner::create_process()`. Allocates the next PID,
    /// adds it to the ready set, and increments next_pid.
    ///
    /// # Returns
    ///
    /// The PID of the newly created process.
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
            // The new PID is fresh (not in any existing set).
            self.lemma_next_pid_is_fresh();
            // Cardinality: inserting a fresh element adds 1.
            // Broadcast axiom handles insert len.
            self.ghost_ready = Ghost(self.ghost_ready@.insert(pid as int));
        }

        self.ready_count = self.ready_count + 1;
        self.next_pid = pid + 1;

        pid
    }

    //==============================================================================================
    // Scheduling
    //==============================================================================================

    /// Reschedules: moves the running process to ready and runs chosen_next.
    ///
    /// Models `ProcessManagerInner::schedule()`. The running process is added
    /// to the ready queue, and `chosen_next` (selected by the scheduler) is
    /// removed from the ready queue and set as running.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID selected by the scheduler. Must be in the ready set
    ///   after the running process has been added to it.
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
        self.running_pid = chosen_next;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            let with_old: Set<int> = old_ready.insert(old_running as int);
            let new_ready: Set<int> = with_old.remove(chosen_next as int);

            // Cardinality: insert adds 1 (old_running not in old_ready), remove subtracts 1.
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);

            // Kernel safety: kernel is running or in new ready.
            self.lemma_kernel_alive_after_schedule(chosen_next as int);

            self.ghost_ready = Ghost(new_ready);
        }
    }

    //==============================================================================================
    // Sleep
    //==============================================================================================

    /// Suspends the running process and runs chosen_next from ready.
    ///
    /// Models the case where the running process goes to sleep (all threads sleeping).
    /// The kernel process (PID 0) cannot sleep.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the ready queue to run next.
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
        self.running_pid = chosen_next;

        proof {
            // Old running is not in suspended (by running_exclusive).
            // So insert adds 1.
            // Broadcast axiom handles insert len.
            // Chosen is in ready. Remove subtracts 1.
            // Broadcast axiom handles remove len.

            self.ghost_suspended = Ghost(
                self.ghost_suspended@.insert(old_running as int)
            );
            self.ghost_ready = Ghost(
                self.ghost_ready@.remove(chosen_next as int)
            );
        }

        self.suspended_count = self.suspended_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Puts the running process to sleep but it still has runnable threads,
    /// so it goes back to ready. Runs chosen_next from ready.
    ///
    /// Models the case where the running thread sleeps but the process has
    /// other runnable threads, so the process stays in ready.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the extended ready set to run next.
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
        self.running_pid = chosen_next;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
            self.ghost_ready = Ghost(
                old_ready.insert(old_running as int).remove(chosen_next as int)
            );
        }
    }

    //==============================================================================================
    // Exit
    //==============================================================================================

    /// Terminates the running process (moves to zombie) and runs chosen_next.
    ///
    /// Models the case where the running process has no more runnable threads
    /// and becomes a zombie. The kernel process (PID 0) cannot exit.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the ready queue to run next.
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
        self.running_pid = chosen_next;

        proof {
            // Broadcast axiom handles insert len.
            // Broadcast axiom handles remove len.

            self.ghost_zombies = Ghost(
                self.ghost_zombies@.insert(old_running as int)
            );
            self.ghost_ready = Ghost(
                self.ghost_ready@.remove(chosen_next as int)
            );
        }

        self.zombie_count = self.zombie_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Terminates the running thread but the process still has runnable threads,
    /// so it goes back to ready. Runs chosen_next from ready.
    ///
    /// Models the case where exit_thread is called but the process has other
    /// runnable threads remaining.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the extended ready set to run next.
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
        self.running_pid = chosen_next;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
            self.ghost_ready = Ghost(
                old_ready.insert(old_running as int).remove(chosen_next as int)
            );
        }
    }

    /// Terminates the running thread; the process has only sleeping threads left,
    /// so it goes to suspended. Runs chosen_next from ready.
    ///
    /// Models exit_thread when remaining threads are all sleeping.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the ready queue to run next.
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
        self.running_pid = chosen_next;

        proof {
            // Broadcast axiom handles insert len.
            // Broadcast axiom handles remove len.

            self.ghost_suspended = Ghost(
                self.ghost_suspended@.insert(old_running as int)
            );
            self.ghost_ready = Ghost(
                self.ghost_ready@.remove(chosen_next as int)
            );
        }

        self.suspended_count = self.suspended_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    /// Terminates the running thread; all threads are now zombies,
    /// so the process goes to zombie. Runs chosen_next from ready.
    ///
    /// Models exit_thread when all remaining threads become zombies.
    ///
    /// # Parameters
    ///
    /// - `chosen_next`: PID from the ready queue to run next.
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
        self.running_pid = chosen_next;

        proof {
            // Broadcast axiom handles insert len.
            // Broadcast axiom handles remove len.

            self.ghost_zombies = Ghost(
                self.ghost_zombies@.insert(old_running as int)
            );
            self.ghost_ready = Ghost(
                self.ghost_ready@.remove(chosen_next as int)
            );
        }

        self.zombie_count = self.zombie_count + 1;
        self.ready_count = self.ready_count - 1;
    }

    //==============================================================================================
    // Wakeup
    //==============================================================================================

    /// Wakes up a suspended process and moves it to the ready queue.
    ///
    /// Models `ProcessManagerInner::wakeup()` for the case where the woken
    /// thread makes the process runnable.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the suspended process to wake up.
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
        proof {
            Self::lemma_wakeup_suspended_len(self.ghost_suspended@, pid as int);
            Self::lemma_wakeup_ready_len(self.ghost_ready@, pid as int);

            self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
            self.ghost_ready = Ghost(self.ghost_ready@.insert(pid as int));
        }

        self.suspended_count = self.suspended_count - 1;
        self.ready_count = self.ready_count + 1;
    }

    //==============================================================================================
    // Resume Interrupted
    //==============================================================================================

    /// Resumes all interrupted processes by moving them to the ready queue.
    ///
    /// Models the loop in `ProcessManagerInner::schedule()` that processes
    /// all interrupted processes before selecting the next to run.
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

            self.ghost_ready = Ghost(self.ghost_ready@.union(self.ghost_interrupted@));
            self.ghost_interrupted = Ghost(Set::empty());
        }

        self.ready_count = self.ready_count + self.interrupted_count;
        self.interrupted_count = 0;
    }

    //==============================================================================================
    // Terminate
    //==============================================================================================

    /// Terminates a ready process by moving it to the zombie queue.
    ///
    /// Models `ProcessManagerInner::terminate()` for a process in the ready queue
    /// that has no more runnable threads after termination.
    /// The kernel process (PID 0) cannot be terminated.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the ready process to terminate.
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
        proof {
            // Broadcast axiom handles remove len.
            // Broadcast axiom handles insert len.

            self.ghost_ready = Ghost(self.ghost_ready@.remove(pid as int));
            self.ghost_zombies = Ghost(self.ghost_zombies@.insert(pid as int));
        }

        self.ready_count = self.ready_count - 1;
        self.zombie_count = self.zombie_count + 1;
    }

    /// Terminates a ready process that still has threads; moves to interrupted.
    ///
    /// Models `ProcessManagerInner::terminate()` for a process that has threads
    /// remaining (will be resumed and can then be cleaned up).
    /// The kernel process (PID 0) cannot be terminated.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the ready process to terminate.
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
        proof {
            // Broadcast axiom handles remove len.
            // Broadcast axiom handles insert len.

            self.ghost_ready = Ghost(self.ghost_ready@.remove(pid as int));
            self.ghost_interrupted = Ghost(
                self.ghost_interrupted@.insert(pid as int)
            );
        }

        self.ready_count = self.ready_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }

    /// Terminates a suspended process by moving it to the interrupted queue.
    ///
    /// Models `ProcessManagerInner::terminate()` for a suspended process.
    /// The kernel process (PID 0) cannot be terminated (and cannot be suspended).
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the suspended process to terminate.
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
        proof {
            // Broadcast axiom handles remove len.
            // Broadcast axiom handles insert len.

            self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
            self.ghost_interrupted = Ghost(
                self.ghost_interrupted@.insert(pid as int)
            );
        }

        self.suspended_count = self.suspended_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }

    //==============================================================================================
    // Zombie Harvesting
    //==============================================================================================

    /// Harvests (removes) a zombie process from the zombie queue.
    ///
    /// Models `ProcessManagerInner::harvest_zombies()`. The zombie is removed
    /// from the zombie set; its resources are freed by the caller.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the zombie process to harvest.
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
        proof {
            // Broadcast axiom handles remove len.

            self.ghost_zombies = Ghost(self.ghost_zombies@.remove(pid as int));
        }

        self.zombie_count = self.zombie_count - 1;
    }

    //==============================================================================================
    // Message Tracking
    //==============================================================================================

    /// Increments the buffered message count.
    ///
    /// Models the message buffering in `ProcessManagerInner::post_message()`.
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
    ///
    /// Models message consumption in `try_recv()`.
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

    /// Models `ProcessManagerInner::capctl()`. Capability changes do not
    /// affect the process state machine (queue membership).
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the process whose capabilities to change.
    pub fn capctl(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
        // No state change; capabilities are orthogonal to the state machine.
    }

    //==============================================================================================
    // Alarm Check (suspended → interrupted)
    //==============================================================================================

    /// Moves a suspended process to the interrupted queue due to alarm expiry.
    ///
    /// Models the alarm checking in `ProcessManagerInner::check_alarm()`.
    ///
    /// # Parameters
    ///
    /// - `pid`: PID of the suspended process whose alarm expired.
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
        proof {
            // Broadcast axiom handles remove len.
            // Broadcast axiom handles insert len.

            self.ghost_suspended = Ghost(self.ghost_suspended@.remove(pid as int));
            self.ghost_interrupted = Ghost(
                self.ghost_interrupted@.insert(pid as int)
            );
        }

        self.suspended_count = self.suspended_count - 1;
        self.interrupted_count = self.interrupted_count + 1;
    }
}

} // verus!
