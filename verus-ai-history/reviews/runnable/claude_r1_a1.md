# Review: runnable (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

_None._

### High

- **Location:** `run()` postcondition (exec: `runnable.rs:288-304`)
  - **Description:** The `ensures` clause only constrains the *length* of `result.ready_thread_ids@` (`== self.spec_ready_count() - 1`) but does not specify the *contents* — i.e., that the remaining sequence is exactly the original ready IDs with the selected index removed. A downstream caller cannot derive which thread IDs remain in the ready list from the postcondition alone.
  - **Suggested Fix:** Add an ensures clause: `result.ready_thread_ids@ == self.ready_thread_ids@.subrange(0, selected_idx@).add(self.ready_thread_ids@.subrange(selected_idx@ + 1, self.ready_thread_ids@.len() as int))`.

- **Location:** `terminate()` postcondition, `Interrupted` branch (exec: `runnable.rs:358-367`)
  - **Description:** The postcondition only requires `ip.interrupted_thread_ids@.len() >= 1` but does not specify the exact count or contents. The correct count should be `self.spec_interrupted_count() + self.spec_sleeping_count()`, reflecting the merge of original interrupted threads with sleeping-to-interrupted conversions. This is a weak postcondition that allows implementations that silently lose or duplicate threads.
  - **Suggested Fix:** Replace `ip.interrupted_thread_ids@.len() >= 1` with `ip.interrupted_thread_ids@.len() == self.spec_interrupted_count() + self.spec_sleeping_count()`, and ideally also specify that `ip.interrupted_thread_ids@ == self.interrupted_thread_ids@.add(self.sleeping_thread_ids@)`.

### Medium

- **Location:** `wf()` predicate (spec: `runnable.spec.rs:202-210`)
  - **Description:** The spec file comments (lines 21–22, 28) explicitly claim "Thread IDs across all lists are disjoint" and "All thread IDs are non-negative" as key invariants, but the actual `wf()` predicate only checks three things: non-empty ready list, parallel array lengths, and non-negative admission times. Thread ID uniqueness/disjointness and non-negativity of thread IDs are never enforced. This undermines the claimed ownership semantics guarantee.
  - **Suggested Fix:** Add conjuncts to `wf()` enforcing: (a) no duplicate thread IDs within each list, (b) thread IDs across all four lists are pairwise disjoint, and (c) all thread IDs are non-negative. Alternatively, clearly document that ownership semantics are modeled implicitly and remove the misleading comments.

- **Location:** `earliest_admission_time()` — missing exec function (exec: `runnable.rs`)
  - **Description:** The original has an `earliest_admission_time()` exec function (original line 216–222) that returns `SystemTime`. The verified code only has spec-level `spec_earliest_admission_time()` and proof lemmas, but no exec-level counterpart. This is a coverage gap.
  - **Suggested Fix:** Add an exec function `earliest_admission_time(&self)` that computes the minimum admission time and has postcondition tying to `spec_earliest_admission_time()`, or document the explicit omission rationale.

- **Location:** `find_thread()` and `find_thread_mut()` — omitted (exec: `runnable.rs`)
  - **Description:** Both functions (original lines 238–266, 282–320) are entirely omitted from the verified model. The documentation (line 57–59) notes reference types prevent Verus expression. While the justification is valid, these are public API functions that search across all four thread lists — a non-trivial operation where a spec could verify exhaustive search and correct variant selection.
  - **Suggested Fix:** Model these as returning `Option<int>` (the thread ID if found) with a tag indicating which list it came from, proving mutual exclusion of list membership. Even a spec-only model would strengthen the verification.

- **Location:** Proof lemmas `lemma_terminate_no_interrupted_gives_zombie`, `lemma_terminate_with_interrupted_gives_interrupted`, `lemma_terminate_with_sleeping_gives_interrupted` (proof: `runnable.proof.rs:196-230`)
  - **Description:** These three lemmas have `ensures true` — trivially true postconditions. The comments describe what they should prove (e.g., that the result is a ZombieProcess or InterruptedProcess), but the actual ensures clause is vacuous. They verify successfully but prove nothing useful.
  - **Suggested Fix:** Replace `true` with substantive postconditions. For example, `lemma_terminate_no_interrupted_gives_zombie` should ensure something like: the result of terminate with `has_interrupted == false` is `TerminateResult::Zombie(zp)` with specific properties. Alternatively, remove these lemmas and strengthen the `terminate()` postcondition directly.

- **Location:** `RunningProcess::wf()` (spec: `runnable.spec.rs:229-231`)
  - **Description:** `RunningProcess::wf()` is defined as `true` — no actual well-formedness checking. This means the `run()` postcondition cannot guarantee the output RunningProcess satisfies any structural invariant (even though it doesn't claim to — the ensures don't mention `result.wf()`). When this boundary model is consumed by other modules, `wf()` provides no guarantees.
  - **Suggested Fix:** Add at minimum: the running thread ID is non-negative, and/or the PID matches expectations. This would make the boundary model more useful for cross-module verification.

### Low

- **Location:** Oracle parameters in `run()` and `wakeup()` (exec: `runnable.rs:288, 435`)
  - **Description:** The `run()` function takes `selected_idx: Ghost<int>` as an oracle parameter with a precondition that it has the earliest admission time. Similarly, `wakeup()` takes `found: bool` and `found_idx: Ghost<int>`. This design pushes the correctness proof of the scheduling algorithm (the for-loop that finds the minimum) and the sleeping thread search to the caller. The actual iterative search in the original `run()` is not verified.
  - **Suggested Fix:** This is an acceptable verification strategy, but document explicitly that the scheduling algorithm correctness is a trust assumption. Alternatively, implement and verify the min-finding loop as a standalone function.

- **Location:** `state()` and `state_mut()` — omitted (exec: `runnable.rs`)
  - **Description:** These accessor functions (original lines 88–94) are not modeled. Since `ProcessState` is abstracted to just a PID, the omission is reasonable but means the verified model cannot express operations that access the full process state through these methods.
  - **Suggested Fix:** No action needed given the current abstraction level. Document as an intentional elision.

- **Location:** `EXIT_STATUS_INTERRUPTED` hardcoded constant (spec: `runnable.spec.rs:102`)
  - **Description:** The constant is hardcoded as `4`, which correctly matches `EINTR` in the codebase (`src/libs/sysapi/src/errno.rs:21`). However, there is no mechanical link — if the upstream constant changes, this spec would silently become incorrect.
  - **Suggested Fix:** Add a comment referencing the source file and line, or consider a cross-module spec link if Verus supports it.

## Positive Observations

- **Verification passes cleanly:** 33 verified, 0 errors with no assumes — high confidence in soundness.
- **Clean spec/proof/exec separation:** The three-file split is well-organized. Spec functions are `open spec`, proofs are in separate lemmas, and exec code is readable.
- **Thorough documentation:** The module header (lines 1–59 of `runnable.rs`) clearly describes the verification model, trust boundary, and abstraction decisions. This is exemplary.
- **Sound external_body usage:** Only `clock_now()` and `exit_status_interrupted_value()` use `external_body`, both justified and with reasonable postconditions. No `assume` statements anywhere.
- **Core state transitions verified:** `new`, `from_state`, `run`, `terminate`, `wakeup`, and `add_thread` all have verified implementations with postconditions covering PID preservation, thread count changes, and well-formedness preservation.
- **Non-trivial proof work:** The `lemma_seq_has_min` helper (proof lines 409–445) is a well-structured inductive proof showing existence of a minimum element in a sequence, supporting the `earliest_admission_time` specification.
- **View types for extensional equality:** Proper use of `#[verifier::ext_equal]` View types enables reasoning about structural equality of process states.
- **EXIT_STATUS_INTERRUPTED verified correct:** The hardcoded value 4 matches the codebase definition (EINTR = 4).

## Summary

The verification is a solid effort that captures the essential state-machine transitions of `RunnableProcess` — construction, scheduling (`run`), termination, wakeup, and thread addition. PID immutability, non-empty ready thread invariant, and thread count preservation are all verified. The spec/proof/exec split is clean and well-documented.

The main weaknesses are: (1) **underspecified postconditions** — `run()` and `terminate(Interrupted)` don't fully characterize the resulting thread lists (only lengths, not contents), which limits compositional reasoning by callers; (2) **missing wf() conjuncts** — the claimed thread ID disjointness invariant is documented but not enforced; (3) **coverage gaps** — `earliest_admission_time`, `find_thread`, and `find_thread_mut` lack exec-level verified implementations; (4) **vacuous proof lemmas** — three terminate-related lemmas have `ensures true`.

**Recommendations (priority order):**
1. Strengthen `run()` and `terminate()` postconditions to specify thread list contents, not just lengths.
2. Either add thread ID disjointness to `wf()` or remove the misleading comments.
3. Replace the three `ensures true` terminate lemmas with substantive postconditions.
4. Add an exec-level `earliest_admission_time()` function.
5. Consider a spec-only model for `find_thread` to verify exhaustive search.
