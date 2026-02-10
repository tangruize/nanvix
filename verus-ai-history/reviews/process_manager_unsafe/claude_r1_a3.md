# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: A

## Previous Issue Disposition

### R2 High Issue

1. **`update_inner()` + `switch()` quantum reset bug** → **Fixed.** The prover combined inner mutation and atomic updates into a single `switch()` function (lines 240–293). `self.inner = new_inner` (line 277) does NOT modify `self.current_pid`, so the subsequent `next_pid != self.current_pid` comparison (line 285) correctly compares against the OLD PID — matching the original code's `CURRENT_PID.load(ORDER)` behavior.

   Verified by tracing `exit()` with PID change (old=1, next=2):
   - `switch()` line 277: `self.inner = new_inner` (current_pid still 1)
   - `switch()` line 285: `next_pid(2) != self.current_pid(1)` → TRUE → quantum reset ✓

   The postconditions (lines 265–266) correctly specify that a hard switch with PID change results in `self.current_pid == next_pid && self.remaining_quantum == self.scheduler_freq`. Verus verified this (32/32). **Confirmed fixed.**

### R2 Medium Issues

2. **`ghost_borrow_count` cosmetic** → **Addressed by removal.** The `ghost_borrow_count` field has been removed entirely from the struct. The `get_mut()` function now simply requires `wf()` without a `spec_no_borrows()` precondition. The exclusive access guarantee is documented as T7 trust boundary (lines 44–48) with clear rationale: Nanvix is single-core cooperative, exclusive access ensured by disabling interrupts. This is a cleaner design than a non-functional ghost counter. **Resolved.**

3. **`giveup()` no-switch postcondition missing `self.inner == old(self).inner`** → **Fixed.** Line 378 now includes `&& self.inner == old(self).inner` in the no-switch path postcondition. Callers can now prove inner state is preserved. **Confirmed fixed.**

### R2 Low Issues

4. **`sleep_post_wakeup()` trust-boundary comment** → **Fixed.** Lines 434–437 now explicitly note `was_interrupted` as a "trust-boundary input (T3)". **Confirmed fixed.**

5. **`exit()`/`exit_thread()` missing PID/TID/quantum postconditions** → **Fixed.** `exit()` (lines 490–494) and `exit_thread()` (lines 534–538) now include postconditions for `self.current_pid`, `self.current_tid`, and `self.remaining_quantum`, mirroring `switch()`'s postconditions. **Confirmed fixed.**

## Issues Found

### Critical

_None._

### High

_None._

### Medium

_None._

### Low

- **Location:** `get()` / `get_mut()` (exec — lines 169–196)
  - **Description:** These functions are modeled as no-ops with `wf()` pre/post. They do not return a value or model the reference-borrowing semantics. While the T7 trust boundary documentation is clear, a caller cannot use these functions to establish or verify any property that isn't already in `wf()`. The functions serve purely as documentation checkpoints.
  - **Suggested Fix:** Acceptable as-is. These functions' main role in the original is accessing the global static; the interesting verification happens in the operations that use the reference (exit, sleep, etc.), which are all properly modeled.

- **Location:** `sleep()` / `exit()` / `exit_thread()` — `try_borrow_mut()` error path (exec)
  - **Description:** In the original code, every public function calls `Self::get_mut().try_borrow_mut()?` which can fail with `Err(ResourceBusy)`. The model only captures the success path — callers provide `new_inner` (the post-mutation state). The error path (borrow failure, state unchanged) is implicitly handled by the caller not invoking the model function, but there's no explicit error-path model.
  - **Suggested Fix:** Minor: could add a `try_borrow_fails()` stub that requires/ensures `wf()` to explicitly document the no-state-change error path. But this is low priority since the error path is trivial (state unchanged).

## Positive Observations

- **Correct quantum reset semantics.** The combined `switch()` function correctly models the original code's separation between inner state mutation (which changes `inner.running_pid`) and atomic loads (which read the OLD `CURRENT_PID`). The PID change detection and quantum reset are now semantically equivalent to the original.
- **Clean combined design.** Merging inner update and atomic updates into one `switch()` function is actually more faithful to the original code flow than the R2 two-function approach. The original's `exit()` calls inner mutation and `switch()` sequentially, and the model's `switch()` captures both steps atomically with correct ordering.
- **Comprehensive postconditions on `exit()`/`exit_thread()`.** These now expose PID/TID/quantum state after the operation, giving callers meaningful information to reason about despite T10 (divergence).
- **No `assume`, `external_body`, or `trusted` markers** anywhere in the three files. All 32 verification conditions are proven from first principles.
- **All 17 original functions modeled.** Complete coverage: `init`, `get`, `get_mut`, `exit`, `exit_thread`, `join_thread` (3 paths), `sleep` (2 phases), `giveup` (unified + 2 helpers), `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `try_recv` (2 paths), `wakeup`, `take_mutex_guard`, `is_kernel_running`, `switch`.
- **Well-documented trust boundaries.** T5–T10 are clearly described with rationale, covering raw pointers, atomics, singleton access, interrupts, TID-to-PID mapping, and divergence.
- **Streamlined proof file.** Unnecessary lemmas from R2 (hard switch variants, delegation preserves wf) were properly removed since the new `switch()` design makes them redundant.
- **Spec file is clean and well-commented.** The context switch model documentation (spec lines 29–49) clearly explains the stale-atomic reading pattern and how the model captures it.

## Summary

All previous issues have been genuinely addressed. The critical R2 quantum reset bug was fixed by combining inner state mutation and atomic updates into a single `switch()` function, which correctly preserves the old-PID comparison semantics. The ghost borrow count was cleanly removed in favor of a well-documented T7 trust boundary. The `giveup()` and `exit()`/`exit_thread()` postconditions are now complete.

The verification is sound (no assumes/external_body), complete (all original functions modeled), and semantically equivalent to the original code for the properties verified. The two remaining Low issues (documentary get/get_mut, implicit error paths) are reasonable design trade-offs that don't affect correctness.
