# Review: thread_manager (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `ThreadManager::create_thread` (exec, thread_manager.rs)
  **Description:** The spec still strengthens behavior with `requires old(self).next_id.value < i32::MAX`, while the original code silently wraps on overflow. This semantic divergence remains unchanged; the verified model excludes a real runtime behavior (negative IDs).
  **Suggested Fix:** Either add a runtime overflow check in the original code (and update the spec), or weaken the spec to model wrap-around semantics and prove the resulting properties.

### Medium
- **Location:** `ThreadRefMutModel::thread_state` and per-variant `thread_state_mut()` (exec, thread_manager.rs + other thread modules)
  **Description:** The verification continues to treat mutation through `thread_state_mut()` as a trust boundary, with no proof that mutations preserve `wf()` and `spec_id()`. This leaves a gap for invariant violations by callers.
  **Suggested Fix:** Model mutable access with explicit postconditions (or ghost borrow tokens) and prove callers maintain `wf()`/`spec_id()`.

- **Location:** `init()` (exec, thread_manager.rs)
  **Description:** Single-initialization is still only an assumption. Multiple calls can create multiple kernel threads with ID 0, violating global ID uniqueness.
  **Suggested Fix:** Add a module-level ghost flag or runtime guard to enforce one-time initialization and prove it.

### Low
- **Location:** `ThreadRefModel` / `ThreadRefMutModel` (exec, thread_manager.rs)
  **Description:** Value-based models still omit borrowing/aliasing semantics; exclusivity relies on Rust’s type system rather than verification.
  **Suggested Fix:** Introduce ghost aliasing/ownership invariants if higher-level proofs need these properties; otherwise keep documented reliance on Rust.

## Positive Observations
- Coverage still includes all functions in `mod.rs` and documents HAL boundaries.
- Specs capture ID assignment, monotonicity, and kernel ID uniqueness; proofs remain consistent.
- Split between exec/spec/proof is clean and documented.

## Summary
The updated files do not actually resolve the previously reported issues; the overflow divergence, mutation trust boundary, and single-init assumption remain. Verification is still useful but not fully equivalent or sound for these behaviors. Addressing these gaps is required for a higher grade.
