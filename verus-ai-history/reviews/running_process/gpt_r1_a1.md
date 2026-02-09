# Review: running_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage gap for multiple public APIs (exec)**
  - **Location:** `RunningProcess` impl (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The verified exec module does not implement or model `state()`, `state_mut()`, `running_mut()`, `try_join_thread()`, `find_thread()`, or `find_thread_mut()`. These are public in the original source and are only partially modeled via spec-only helpers (or not at all), so criterion (1) is not satisfied and behavioral equivalence is incomplete.
  - **Suggested Fix:** Add verified wrappers or `external_body` stubs with precise specs for these functions (including reference-returning ones), or re-express them in a Verus-friendly form to directly verify their semantics against the original.

- **API/semantic mismatch in `wakeup()` via oracle parameter**
  - **Location:** `RunningProcess::wakeup` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The verified signature adds a `found: bool` parameter with a precondition tying it to ghost state. This is not present in the original API and shifts responsibility to callers, making the spec stronger than the implementation and unsound if callers pass inconsistent values.
  - **Suggested Fix:** Remove the `found` parameter and internally compute the branch from ghost state (e.g., with a `spec_seq_contains`-driven proof and `choose`), or provide a public wrapper that enforces the correct value so callers cannot violate it.

- **Non-equivalence in `exit_thread()` interrupted branch**
  - **Location:** `RunningProcess::exit_thread` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The model intentionally passes `new_zombie_ids` into `InterruptedProcess` while the original passes `self.zombie.take()` (which is always `None` after line 261). This changes observable behavior by preserving the exited thread’s zombie state, so criterion (4) fails.
  - **Suggested Fix:** Either fix the original source and re-verify against the corrected code, or adjust the model to match the original behavior and document the bug separately.

- **Weak specification of `InterruptedProcess::resume()`**
  - **Location:** `interrupted_resume` external_body (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The `external_body` only constrains counts and non-empty ready list, but does not preserve thread IDs or establish that the result is a permutation of the interrupted set. This allows IDs to change or be dropped, weakening safety and equivalence for `sleep()`, `exit()`, and `exit_thread()` interrupted branches.
  - **Suggested Fix:** Strengthen the spec to preserve the multiset of interrupted thread IDs, specify that exactly one interrupted thread becomes ready, and that the remaining IDs are unchanged.

### Medium
- **Invariants are too weak for ownership/uniqueness guarantees**
  - **Location:** `wf()` in `running.spec.rs`.
  - **Description:** `wf()` only checks count/length equality and does not ensure disjointness of thread lists or that the running thread ID is absent from other lists. This permits models with duplicated IDs, undermining correctness for `find_thread`, `try_join_thread`, and `wakeup` semantics.
  - **Suggested Fix:** Strengthen `wf()` (or require `wf_strict()` in public APIs) to enforce uniqueness/disjointness of thread IDs, matching the Rust ownership model.

- **Unmodeled mutability of `state_mut()` / `running_mut()`**
  - **Location:** Missing exec stubs; trust notes in `running.spec.rs`.
  - **Description:** The model assumes PID/running TID immutability but provides no verified contract for these mutable accessors, leaving a gap in the proof of PID and TID stability.
  - **Suggested Fix:** Add specs that constrain permissible mutations (e.g., PID and running thread ID must remain unchanged) or model the accessors as `external_body` with explicit frame conditions.

### Low
- **Interrupted branch of `sleep()` is underspecified in the public postcondition**
  - **Location:** `RunningProcess::sleep` (exec), `verus/split/kernel/pm/process/state/running.rs`.
  - **Description:** The postcondition for the interrupted branch does not state the exact ready/interrupted contents or counts, relying implicitly on `interrupted_resume`’s (currently weak) spec.
  - **Suggested Fix:** Once `interrupted_resume` is strengthened, propagate those guarantees into `sleep()`’s ensures to make the state transition explicit at the API level.

## Positive Observations
- The split between exec/spec/proof is clean and well documented, with clear trust boundaries and an explicit list of modeled abstractions.
- Core state-machine transitions (`schedule`, `sleep`, `exit`, `exit_thread`, `wakeup`) have detailed postconditions and dedicated proof lemmas.
- The spec file documents known divergence and limitations, which helps auditability.

## Summary
The verification captures many core transition properties, but it is incomplete and not fully equivalent to the source due to missing public API coverage, an oracle-based `wakeup`, and a documented divergence in `exit_thread`. Strengthening invariants and the `InterruptedProcess::resume()` contract, and adding verified stubs for the remaining public methods, would significantly improve soundness and equivalence.
