# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: A

## Previous Issue Resolution

### High #1: exit()/exit_thread() divergence not machine-checked — **FIXED ✓**

The prover added a `ghost_diverged: Ghost<bool>` field to `ProcessManagerUnsafeState`. After `exit()`/`exit_thread()`, the field is set to `Ghost(true)`. Since `wf()` now includes `spec_not_diverged()` (which requires `ghost_diverged@ == false`), all subsequent operations that require `wf()` as a precondition will fail to verify after a diverging call. I verified:
- `wf()` includes `self.spec_not_diverged()` (spec.rs:163)
- `spec_not_diverged()` requires `self.ghost_diverged@ == false` (spec.rs:116)
- `exit()` body: `self.switch(...); self.ghost_diverged = Ghost(true);` (exec:573-574)
- `exit_thread()` body: `self.switch(...); self.ghost_diverged = Ghost(true);` (exec:620-621)
- `switch()` ensures `self.wf()`, so the switch succeeds, then ghost_diverged breaks wf()
- `exit()/exit_thread()` postconditions no longer claim `self.wf()` — they claim `self.ghost_diverged@ == true`

This is a clean, elegant solution that provides genuine machine-checked divergence prevention. The init lemma and proof file were also correctly updated to include `ghost_diverged: Ghost(false)`.

### High #2: exit() implicit hard-switch assumption — **FIXED ✓**

Both `exit()` (line 559) and `exit_thread()` (line 606) now have explicit precondition `chosen_next_tid != old(self).current_tid`. This makes the hard-switch requirement machine-checked rather than relying on implicit reasoning about inner module semantics. The postconditions are simplified accordingly (e.g., `self.current_tid == chosen_next_tid` is now unconditional).

Note: The precondition `chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid` is kept but is now vacuously true given the new `!=` precondition. This is harmless (commented as "vacuously true given above").

### Medium #1: Delegation functions don't model inner mutations — **ACCEPTABLY DEFERRED**

No change made. The delegation functions remain `&self` with `succeeds: bool`. This is acceptable: T12 trust boundary was already documented, and sync object correctness is explicitly deferred to the inner module. The prover's choice to keep these as thin stubs is reasonable given the scope boundary.

### Medium #2: join_thread_harvest() is a no-op — **ACCEPTABLY DEFERRED**

No change made. The harvesting logic (user stack page unmapping) is purely a memory management operation that doesn't affect queue-level state. Since this module's verification boundary is the process/thread queue model, this remains within the acceptable trust boundary.

### Medium #3: sleep() doesn't model alarm parameter — **ACCEPTABLY DEFERRED**

No change made. Since the inner model doesn't distinguish timed vs. untimed sleep queues (both map to `ghost_suspended`), adding the alarm parameter here without inner model changes wouldn't add verification value. Reasonable deferral.

### Medium #4: giveup() missing error path — **FIXED ✓**

Added `giveup_error()` (lines 449-461) modeling the `try_borrow_mut()` failure case. Requires `wf()`, ensures `wf()`, no state change. This correctly models the original's `?` error propagation path. Verification count increased from 32 to 33.

### Low #1: switch() user_tda not documented in T5 — **FIXED ✓**

T5 documentation updated to mention `user_tda`: "The `user_tda` parameter (user-space thread data area virtual address) is also abstracted away; it affects address space setup for the next thread but not queue-level state." (exec lines 42-44)

### Low #2: try_recv_some() struct update fragility — **N/A (no change needed)**

Correctly left as-is per the original review's suggestion.

### Low #3: spec_tid_valid() lacks upper bound — **NOT ADDRESSED**

Still only `current_tid >= 0i32`. Acceptable for current scope since TIDs are only compared, not used arithmetically in this module.

### Low #4: Performance counter modeling — **NOT ADDRESSED**

Still omitted. Acceptable for single-core Nanvix.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- **Location:** Delegation functions `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` (exec file, lines 628-726)
  **Description:** (Carried from R1, acceptably deferred.) These remain pure identity functions with `&self`. The original `get_mutex()` can create a new mutex in the inner table, and `put_cond()` removes a condvar — both are mutations. The T12 trust boundary documentation is adequate, but this means the verification proves nothing about these functions' actual behavior beyond wf() preservation. This is the largest unverified behavioral gap in the module.
  **Suggested Fix:** No immediate fix needed. If the inner module's sync object verification is ever extended, these should be updated to `&mut self` with `new_inner` parameters.

### Low

- **Location:** `exit()` and `exit_thread()` preconditions (exec lines 561, 608)
  **Description:** The precondition `chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid` is vacuously true given the earlier `chosen_next_tid != old(self).current_tid`. It is commented as "(vacuously true given above)" which is accurate, but redundant preconditions add noise and could confuse future readers.
  **Suggested Fix:** Consider removing the vacuous precondition to simplify the contract, or leave as-is for consistency with `switch()`.

- **Location:** `exit()` / `exit_thread()` — no error path model (exec file)
  **Description:** The original `exit()` calls `Self::get_mut().try_borrow_mut()?.exit(status)` where the `?` can fail. Similarly for `exit_thread()`. A `giveup_error()` was added for giveup's error path, but there are no corresponding `exit_error()` or `exit_thread_error()` functions for consistency. The error paths are trivial (no state change, return Err), but the asymmetry with giveup_error is notable.
  **Suggested Fix:** Consider adding `exit_error()` and `exit_thread_error()` for consistency, or document that error paths for all functions are covered by the T2 trust boundary.

- **Location:** spec_tid_valid (spec.rs:130-132)
  **Description:** (Carried from R1.) No upper bound on `current_tid`. Low risk since TIDs are only compared in this module.
  **Suggested Fix:** Consider `current_tid < i32::MAX` for defense in depth.

## Positive Observations

- **Machine-checked divergence (ghost_diverged):** The new `ghost_diverged` flag is an elegant solution to the divergence problem. It cleanly invalidates wf() after exit/exit_thread, preventing any post-exit operations from being verified. This elevates T10 from a documentation-only trust boundary to a machine-checked property — a significant improvement.

- **Zero assumes/external_body:** The module still has no `assume` or `external_body` annotations. All 33 verified functions pass cleanly.

- **Explicit hard-switch preconditions:** Making `chosen_next_tid != old(self).current_tid` an explicit precondition on exit/exit_thread strengthens the contract and removes implicit reasoning about inner module behavior.

- **Complete function coverage with error paths:** All 17 original functions have verified counterparts, and error paths are now modeled for giveup. The total verified function count is 33.

- **Faithful switch() modeling:** Unchanged from R1 — correctly captures stale-atomic PID comparison semantics.

- **Well-documented trust boundaries:** T5-T13 are comprehensive. T10 was upgraded from a trust boundary to a machine-checked property.

## Summary

The prover addressed both High issues effectively. The ghost_diverged flag provides genuine machine-checked divergence prevention — a meaningful improvement over the documentation-only approach. The explicit hard-switch precondition on exit/exit_thread removes implicit assumptions. The giveup error path and T5 user_tda documentation were also addressed as requested.

The remaining deferred items (delegation function mutations, join_thread_harvest detail, sleep alarm parameter) are well-documented under existing trust boundaries and represent reasonable scope choices. The module now has 33 verified functions with 0 errors, 0 assumes, and 0 external_body annotations. The verification quality has improved from A- to A.
