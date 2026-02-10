# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### High Issues — Both Fixed

1. **switch() quantum not reset on same-PID hard switch** — **FIXED.** The prover added a clear design note (exec lines 329–333) explaining that same-PID hard switches intentionally inherit quantum because the scheduler treats quantum as per-process, not per-thread. This matches the original code (`unsafe.rs:774-777` only resets quantum on `next_pid != previous_pid`). The documentation is accurate and the design rationale is sound.

2. **exit()/exit_thread() kernel protection asymmetry** — **FIXED.** `exit()` now requires BOTH `current_pid != KERNEL_PID_RAW` AND `current_tid != KERNEL_TID_RAW` (exec lines 594–595), with a comment (lines 588–593) explaining why both checks are needed: PID check prevents exiting the kernel process, TID check matches the original safety contract. `exit_thread()` retains only the TID check (line 659), which is correct — thread exit only needs to verify the calling thread isn't the kernel thread. The asymmetry is now intentional and documented.

### Medium Issues — All Fixed

3. **sleep() missing alarm parameter** — **FIXED.** A `Ghost(alarm): Ghost<int>` parameter was added (exec line 490) with clear documentation (lines 481–487) encoding `alarm < 0` as None (indefinite sleep) and `alarm >= 0` as Some(time). `join_thread_wait()` correctly passes `Ghost(-1int)` (line 927) to model `join_cond.wait(None)`. The ghost alarm is not constrained in postconditions (it doesn't change state), but it enables callers to reason about sleep variants.

4. **join_thread() loop model single-iteration** — **FIXED.** A new `lemma_join_thread_loop_invariant` (proof.rs lines 183–214) was added. The lemma proves that for any iteration outcome (0/1/2), a wf()-satisfying state exists after the iteration. **Caveat:** The lemma is somewhat trivial for the harvest/error cases (outcome != 1 just restates `self.wf()` from `self.wf()`), and for the wait case (outcome == 1) it only asserts `new_inner.wf()` which is already a precondition. The true loop composition (that switch restores wf consistency between inner and atomics) is proven by the `join_thread` exec function itself. This is acceptable but the lemma is more of a documentation artifact than a genuinely new proof.

5. **wakeup() no TID parameter** — **FIXED.** A `Ghost(woken_tid): Ghost<int>` parameter was added (exec line 775) with precondition `woken_tid >= 0` (line 780) and documentation (lines 771–774) explaining it models the original `tid: ThreadIdentifier`.

6. **spec_tid_valid() weak validity** — **FIXED.** `spec_tid_valid` now checks `current_tid >= 0i32 && current_tid < i32::MAX` (spec lines 134–137). The `switch()` function and all its callers consistently require `next_tid < i32::MAX` (exec line 313, and propagated to giveup_with_switch:402, giveup:432, sleep:501, exit:587, exit_thread:657, join_thread_wait:916, join_thread:998).

### Low Issues — Mostly Fixed

7. **Delegation functions over-abstracted** — **FIXED.** All delegation functions now have `Ghost(addr): Ghost<int>` parameters with `addr >= 0` preconditions: `get_mutex` (line 709), `put_mutex_guard` (line 724), `get_cond` (line 739), `put_cond` (line 754), `take_mutex_guard` (line 796).

8. **try_recv_some() message content not modeled** — **Acknowledged.** Remains as T11 trust boundary. Acceptable at current scope.

9. **join_thread_harvest() user stack unmapping not modeled** — **FIXED.** New T14 trust boundary documentation (exec lines 872–880) explains the re-borrow risk and why the harvest is modeled as a no-op (queue model doesn't track page mappings, and the original code handles failures with best-effort `warn!` logging).

10. **exit_thread() condvar notify_all not modeled** — **NOT FIXED.** The original `exit_thread()` (unsafe.rs:285–297) extracts `join_cond: Condvar` and calls `join_cond.notify_all()?` before the context switch. This is the mechanism that wakes threads blocked in `join_thread()`. The verified `exit_thread()` does not model the notification. This remains a gap for cross-function reasoning.

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `exit_thread()` (exec, line 645) — condvar notify_all still unmodeled
  - **Description:** The original `exit_thread()` extracts a `join_cond: Condvar` from the inner exit_thread call and invokes `join_cond.notify_all()?` before context-switching. This notify is the causal link between a thread exiting and `join_thread()` waiters being woken. The verified model does not represent this — `exit_thread()` goes directly to `switch()` then sets `ghost_diverged`. For any future whole-system reasoning about join_thread termination, this missing link would need to be addressed.
  - **Suggested Fix:** Add a ghost postcondition (e.g., `self.inner.spec_join_notified()`) or add a `Ghost(notified): Ghost<bool>` with `ensures notified == true` to indicate the condvar was signaled. Alternatively, document as trust boundary T15.

- **Location:** `lemma_join_thread_loop_invariant` (proof.rs, line 198) — lemma is trivial
  - **Description:** The lemma's ensures clauses are: (a) `outcome != 1 ==> self.wf()` (trivially from `requires self.wf()` and no state change), and (b) `outcome == 1 ==> new_inner.wf()` (trivially from `requires new_inner.wf()`). Neither clause composes the switch/sleep path that actually transforms state. The real loop invariant proof is embedded in the `join_thread` exec function's postconditions, which prove `self.wf()` after the wait path through `join_thread_wait → sleep → switch`. The lemma adds a named proof point but no new proof content.
  - **Suggested Fix:** Either strengthen the lemma to prove something non-trivial (e.g., that after N iterations the state remains wf), or acknowledge it as a documentation-only lemma. This is cosmetic, not a soundness issue.

### Low

- **Location:** `exit_thread()` (exec, line 645) — no `current_pid != KERNEL_PID_RAW` check
  - **Description:** Unlike `exit()` which now checks both PID and TID against kernel values, `exit_thread()` only checks `current_tid != KERNEL_TID_RAW`. If the kernel process (PID 0) were to create non-kernel threads (TID != 0), `exit_thread` would allow exiting them. This is likely correct behavior (only the kernel thread 0 is special), but the asymmetry should be documented.
  - **Suggested Fix:** Add a brief comment explaining why PID check is not needed for thread exit (only the kernel *thread* is protected, not all threads in the kernel process).

- **Location:** Documentation (exec, line 99) — stale verification count
  - **Description:** Line 99 states "32 verified functions, 0 errors" but the module now has 38 verified functions. This header comment was not updated after adding new functions and the loop invariant lemma.
  - **Suggested Fix:** Update to "38 verified functions, 0 errors".

## Positive Observations

- **Comprehensive issue resolution:** 9 of 10 previous issues were addressed, with substantive code changes (not just comment additions). The prover added ghost parameters, strengthened preconditions, added trust boundary documentation, and created a new proof lemma.
- **Consistent `next_tid < i32::MAX` propagation:** The TID upper bound was added to `switch()` and correctly propagated to all 7 callers. This is thorough.
- **Well-reasoned alarm encoding:** The `Ghost(alarm): Ghost<int>` with negative-as-None encoding is clean and the `join_thread_wait` correctly uses `Ghost(-1int)` for the indefinite wait case.
- **Zero assume/external_body:** Still no `assume` statements or `external_body` annotations across all 38 verified functions. Full machine-checking.
- **T14 trust boundary:** The new trust boundary documenting the harvest re-borrow risk is a genuine improvement — it identifies a subtle failure mode in the original code (re-borrow within the page unmapping loop).
- **Kernel protection now symmetric:** The `exit()` function now correctly guards both PID and TID, matching the original safety documentation more faithfully.

## Summary

The prover made substantive improvements addressing nearly all previous issues. The verification is sound: 38 functions verified with 0 errors, 0 assume/external_body, and comprehensive postconditions. The remaining issues are: (1) `exit_thread()` condvar `notify_all` is still unmodeled, which matters for cross-function reasoning about join/exit interactions, (2) the loop invariant lemma is technically correct but trivially so, and (3) minor documentation staleness. None of these are soundness issues. The verification is upgraded from A- to A.
