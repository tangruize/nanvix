# Review: running_process (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage still incomplete for public APIs**
  - **Location:** `RunningProcess` impl (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** `state_mut()`, `running_mut()`, `try_join_thread()`, `find_thread()`, and `find_thread_mut()` are still not modeled as exec functions. They remain spec-only or omitted, so criterion (1) is still unmet and equivalence is partial.
  - **Suggested Fix:** Add exec stubs (possibly `external_body`) for each missing function with precise specs, or refactor the API to expose Verus-friendly wrappers that are verified against the original behavior.

- **`wakeup()` still relies on an oracle parameter**
  - **Location:** `RunningProcess::wakeup` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The `found: bool` parameter remains, which is not present in the original API and shifts correctness to callers. This keeps the spec stronger than the implementation and is a soundness gap if callers supply inconsistent values.
  - **Suggested Fix:** Remove the oracle from the public interface and compute the branch internally from ghost state (e.g., a verified search using `choose` + proof), or provide a verified wrapper that enforces `found == spec_seq_contains(...)`.

- **Known divergence in `exit_thread()` remains**
  - **Location:** `RunningProcess::exit_thread` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The model still passes `new_zombie_ids` into the interrupted path while the original passes `self.zombie.take()` (which is always `None`). This changes observable behavior, so criterion (4) is still violated.
  - **Suggested Fix:** Either fix the original source and re-verify against it, or model the original behavior and document the bug separately.

### Medium
- **Well-formedness still does not enforce thread ID uniqueness**
  - **Location:** `wf()` in `running.spec.rs`.
  - **Description:** `wf()` remains only count/length consistency. Without disjointness or exclusion of the running thread, the model allows duplicated IDs, weakening correctness for `find_thread`, `try_join_thread`, and `wakeup` semantics.
  - **Suggested Fix:** Require `wf_strict()` for public APIs (or strengthen `wf()` itself) to match the Rust ownership invariant.

### Low
- **`exit()` runnable branch still underspecifies ready/interrupted contents**
  - **Location:** `RunningProcess::exit` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The postcondition only constrains `ready_thread_ids.len() == 1` and not which ID becomes ready or how the interrupted tail is preserved, leaving observable behavior weaker than the original.
  - **Suggested Fix:** Mirror the strengthened `interrupted_resume()` guarantees in `exit()`’s ensures (e.g., ready is the first interrupted ID; interrupted list is the tail).

## Positive Observations
- `interrupted_resume()`’s spec is now strengthened to preserve IDs and specify exact ready/tail semantics.
- `sleep()` and `exit_thread()` interrupted-branch postconditions now explicitly reflect the resumed thread and tail preservation.
- A `state()` accessor was added with a clear PID-level spec, and mutation-frame helpers were documented.

## Summary
Some prior issues were addressed (notably `interrupted_resume()` strength and interrupted-branch specs), but major gaps remain: missing exec coverage for several public APIs, the oracle-based `wakeup`, and a still-present semantic divergence in `exit_thread()`. The verification is improved but not complete or fully equivalent to the source.
