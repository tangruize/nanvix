# Review: process_manager_unsafe (claude-opus-4.6)

## Grade: A-

## Previous Issue Disposition

### High Issues (from R1)

1. **get()/get_mut() not modeled** → **Partially Fixed.** Functions `get()` and `get_mut()` are now present (lines 175, 196) with a `ghost_borrow_count` field and `spec_no_borrows()` precondition on `get_mut()`. However, the ghost borrow count is initialized to 0 in `init()` and **never modified by any function** — no function increments or decrements it. The `spec_no_borrows()` precondition (`ghost_borrow_count@ == 0int`) is therefore always trivially satisfied. The "exclusive access" modeling is cosmetic: it doesn't actually prevent reasoning about concurrent mutable access. **Downgraded from High to Medium** since the preconditions/postconditions are at least documented and the T7 trust boundary is explicitly acknowledged.

2. **switch() conflates inner mutation with atomics** → **Fixed with new bug introduced.** The prover correctly separated `switch()` (atomics only, line 283) from `update_inner()` (inner state, line 240). However, `update_inner()` also sets `self.current_pid = next_pid` (line 261), which means when `switch()` subsequently checks `next_pid != self.current_pid` (line 317), the condition is always FALSE — the PID was already updated. This prevents the quantum reset (`self.remaining_quantum = self.scheduler_freq`) from ever firing on PID changes. See new High issue below.

### Medium Issues (from R1)

3. **giveup() split into two functions** → **Fixed.** Unified `giveup()` entry point (line 385) correctly dispatches to `giveup_no_switch()` / `giveup_with_switch()`. Minor spec gap: the no-switch path postcondition omits `self.inner == old(self).inner`, preventing callers from knowing the inner state is preserved. See new Low issue.

4. **join_thread missing condvar-wait path** → **Fixed.** `join_thread_wait()` (line 744) delegates to `sleep()` with proper preconditions. Liveness documented as trust boundary. Good fix.

5. **exit()/exit_thread() divergence** → **Addressed.** Documented as T10 trust boundary with detailed explanation (lines 54-59). Callers warned not to reason about post-exit code. Reasonable resolution.

6. **sleep() post-wakeup not modeled** → **Fixed.** `sleep_post_wakeup()` (line 470) models the interrupt-reason check. The model is thin (passes through a `was_interrupted` bool) but correctly captures the two return paths (Ok vs Err(Interrupted)).

### Low Issues (from R1)

7. **spec_quantum_valid comment** → **Fixed.** Comment added (spec lines 110-114) explaining why `>= 1` is correct. ✓
8. **Delegation functions no parameters** → **Fixed.** Now have `succeeds: bool` parameter and return type. ✓
9. **try_recv_some missing tid** → **Fixed.** Ghost `tid` parameter added with `tid >= 0` precondition. ✓
10. **wf() missing TID-to-PID mapping** → **Addressed.** Documented as T9 trust boundary (spec lines 50-57). ✓

## Issues Found

### Critical

_None._

### High

- **Location:** `update_inner()` + `switch()` interaction (exec — lines 240-325)
  - **Description:** `update_inner()` sets `self.current_pid = next_pid` (line 261) to maintain `wf()` (which requires `spec_pid_consistent`). When `switch()` is subsequently called, it compares `next_pid != self.current_pid` (line 317), but since `update_inner()` already set `current_pid = next_pid`, this comparison is always false. This means the quantum reset branch (`self.remaining_quantum = self.scheduler_freq`, line 319) is **never executed** when there is a PID change.

    In the original code (unsafe.rs:765-777), `switch()` reads `CURRENT_PID` via `CURRENT_PID.load(ORDER)` which still holds the **old** PID (inner mutation does not touch the atomic). So the comparison `next_pid != previous_pid` correctly detects PID changes and resets the quantum.

    Concrete trace for `exit()` with PID change (old_pid=1, next_pid=2):
    - Original: inner.exit() → CURRENT_PID still 1 → switch() sees 2≠1 → quantum reset ✓
    - Model: update_inner() → current_pid=2 → switch() sees 2==2 → NO quantum reset ✗

    This affects `exit()`, `exit_thread()`, `sleep()`, `giveup_with_switch()`, and `join_thread_wait()` — all functions that call `update_inner()` + `switch()`.
  - **Suggested Fix:** `update_inner()` should NOT update `current_pid` — only `self.inner`. To maintain `wf()` between the two calls, either: (a) weaken `wf()` to allow a temporary `spec_pid_consistent` violation (add a `spec_mid_transition` predicate), or (b) pass the old PID to `switch()` so it can detect the change, or (c) combine inner update + atomic update into a single `update_inner_and_switch()` that correctly handles quantum reset — this was essentially the original R1 design but needs the quantum reset added explicitly.

### Medium

- **Location:** `ghost_borrow_count` / `get_mut()` (exec — lines 109, 196; spec — lines 132-134, 182-184)
  - **Description:** The `ghost_borrow_count` field is initialized to `Ghost(0int)` in `init()` and never modified anywhere. No function increments it (to model acquiring a borrow) or decrements it (to model releasing a borrow). The `spec_no_borrows()` precondition on `get_mut()` is trivially satisfied at all program points. The singleton access model therefore provides no actual exclusivity guarantee — a caller could invoke `get_mut()` twice "simultaneously" and the model would not detect a conflict.
  - **Suggested Fix:** Add `borrow_mut()` and `release_borrow()` functions that increment/decrement `ghost_borrow_count`. Have `get_mut()` call `borrow_mut()` as a precondition-establishing step, and have each caller release the borrow after use. Alternatively, if this is genuinely not modelable in Verus (since the original uses RefCell runtime checking), acknowledge that `spec_no_borrows()` is a documentation annotation rather than a verified invariant, and remove it from `wf()`.

- **Location:** `giveup()` no-switch path postcondition (exec — lines 403-411)
  - **Description:** When `remaining_quantum > 1`, the postcondition specifies that quantum is decremented and PID/TID are unchanged, but does **not** assert `self.inner == old(self).inner`. The `giveup_no_switch()` helper does ensure this (line 341), but the information is lost in the composite postcondition. A caller of `giveup()` cannot prove that the inner state is preserved on the no-switch path.
  - **Suggested Fix:** Add `&& self.inner == old(self).inner` to the `remaining_quantum > 1` postcondition branch.

### Low

- **Location:** `sleep_post_wakeup()` (exec — lines 470-480)
  - **Description:** The function takes `was_interrupted: bool` as a parameter and returns it unchanged. This models the existence of two return paths but doesn't connect the interrupt status to the inner state (e.g., `inner.interrupt_reason()` would read from the thread's state). The model trusts the caller to provide the correct `was_interrupted` value.
  - **Suggested Fix:** Acceptable as-is given T3 (thread-level details are external), but a comment noting that `was_interrupted` is a trust-boundary input would improve clarity.

- **Location:** `exit()` / `exit_thread()` postconditions (exec — lines 517-520, 557-560)
  - **Description:** These functions don't specify postconditions about `remaining_quantum` or `current_pid`/`current_tid`. While these are divergent operations (T10) and callers shouldn't reason about post-exit state, the model does return normally. Adding postconditions like `self.current_pid == chosen_next_pid` would make the model more informative without contradicting the T10 boundary.
  - **Suggested Fix:** Add postconditions mirroring `switch()`'s postconditions for the PID/TID/quantum state.

## Positive Observations

- **Strong separation of concerns.** The split of `update_inner()` from `switch()` is the right architectural decision — it correctly models that the original code mutates inner state and atomics in separate steps. The implementation just needs the quantum reset interaction fixed.
- **Unified `giveup()` entry point.** The composite function with sub-function dispatch is clean and matches the original's single-function API surface. Postconditions correctly cover both branches.
- **Comprehensive trust boundary documentation.** T9 (TID-to-PID mapping) and T10 (divergence) are now explicitly documented with rationale. The module header (lines 40-59) clearly lays out all trust boundaries.
- **39 verification conditions pass** (up from 30), reflecting the added functions.
- **Good `join_thread_wait()` model.** Correctly delegates to `sleep()` and documents the liveness argument as a trust boundary with a clear explanation of the termination reasoning.
- **Delegation functions now model success/failure.** The `succeeds: bool` parameter and return type provide a minimal but useful model of the Result type without over-specifying.

## Summary

The prover made genuine improvements: 7 of 10 previous issues were properly fixed. The singleton access modeling (`get()`/`get_mut()`), unified `giveup()`, `join_thread_wait()`, `sleep_post_wakeup()`, and documentation improvements are all real fixes.

However, the separation of `update_inner()` from `switch()` — which was the correct architectural response to the R1 High issue — introduced a new semantic equivalence bug: `update_inner()` prematurely updates `current_pid`, preventing `switch()` from detecting PID changes and resetting the quantum. This is the primary remaining issue.

The `ghost_borrow_count` singleton modeling is cosmetic rather than substantive (never modified after init), but this is a reasonable trade-off given Verus's limitations for modeling runtime borrow checking.

**Recommendation:** Fix the quantum reset bug by restructuring how `update_inner()` and `switch()` interact with `current_pid`. The simplest correct approach may be to not update `current_pid` in `update_inner()` and instead have `switch()` receive an `old_pid` parameter, or to combine both steps into one function that handles the quantum reset explicitly.
