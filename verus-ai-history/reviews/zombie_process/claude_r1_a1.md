# Review: zombie_process (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

*None.*

### High

- **Location:** `state_mut()` (exec — zombie.rs:156)
  **Description:** `state_mut()` is marked `external_body` and its postcondition asserts that `self.spec_pid() == old(self).spec_pid()` — i.e., the PID is preserved. However, the original `state_mut()` returns `&mut ProcessState`, giving the caller unrestricted write access to *all* ProcessState fields, including `pid`. The postcondition claims PID immutability, but there is no Rust-level enforcement of this invariant: nothing prevents a caller from writing `state_mut().pid = ...`. This `external_body` annotation hides an **unverified trust assumption** about caller discipline. If any caller mutates the PID through this reference, the entire verification is unsound.
  **Suggested Fix:** Document this as a trust assumption more prominently, and ideally add a runtime assertion or architectural note that ProcessState does not expose a public setter for `pid`. Alternatively, model `state_mut()` without the PID-preservation postcondition and require callers to re-establish PID equality as a proof obligation.

- **Location:** `find_thread()` and `find_thread_mut()` (exec — zombie.rs:209–240)
  **Description:** These functions are modeled as pure spec computations (`Ghost(self.spec_find_thread(tid@))`) rather than executable search logic. The original code performs `iter().find(|t| t.id() == tid)` and returns `Option<ThreadRef>` / `Option<ThreadRefMut>`. The verified version returns `Ghost<Option<int>>`, collapsing the rich return type into a ghost integer tag. This means the verification does **not** prove that the linear search correctly finds the thread, nor that the returned reference actually points to the matching thread. The `lemma_find_thread_refinement_assumption` in the proof file is labeled as such but is actually a trivially true tautology (it just restates the spec definition), not a genuine refinement link.
  **Suggested Fix:** Acknowledge this explicitly as an unverified trust boundary. The integration obligation spec (`spec_find_thread_integration_obligation`) is correctly placed but should be more prominently flagged as **unproven**. Consider adding an executable ghost loop that iterates over `zombie_thread_ids` to verify the search logic at the ghost level, even if the real iterator cannot be modeled.

### Medium

- **Location:** `state()` (exec — zombie.rs:139)
  **Description:** `state()` is `external_body` and returns `Ghost<int>`. The original returns `&ProcessState`, which contains ~9 fields (pid, capabilities, vmem, events, mailbox, mmio, pmio, mutexes, conditions). Modeling it as a single `int` (PID) loses all information about the other fields. If any caller of `state()` reads capabilities, vmem, or other fields, those reads are outside the verification model entirely.
  **Suggested Fix:** This is acceptable for PID-focused verification but should be documented as a deliberate abstraction boundary. If future verification needs to reason about capabilities or vmem through ZombieProcess, the model will need extension.

- **Location:** `wf()` invariant (spec — zombie.spec.rs:121)
  **Description:** The well-formedness predicate enforces `zombie_count as nat == zombie_thread_ids@.len()`, non-emptiness, and no-duplicates. However, it does **not** require that thread IDs are valid (e.g., non-negative, within some range). The original `ThreadIdentifier` is a structured type with constraints. The ghost model uses unbounded `int`, which is more permissive.
  **Suggested Fix:** Consider adding a validity constraint on thread IDs (e.g., non-negative) to tighten the model, or document that thread ID validity is outside scope.

- **Location:** `new()` (exec — zombie.rs:107)
  **Description:** The verified `new()` takes `zombie_count: u64` as an exec-level parameter separate from the ghost sequence, requiring the caller to maintain `zombie_count as nat == zombie_ids@.len()` as a precondition. The original `new()` has no such parameter — it takes `NonEmptyVecDeque<ZombieThread>` which implicitly knows its own length. This introduces an additional proof obligation on callers that doesn't exist in the original API.
  **Suggested Fix:** This is a reasonable modeling choice but adds a potential source of mismatch. Document clearly that callers at integration boundaries must establish this length-consistency invariant.

- **Location:** `bury()` return type (exec — zombie.rs:175)
  **Description:** The original `bury()` returns `(NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus)` — returning actual ownership of the thread objects and process state. The verified version returns `(Ghost<Seq<int>>, Ghost<int>, Ghost<int>)` — purely ghost data. This means the verification does not prove anything about resource transfer or ownership semantics of `bury()`, which is its primary purpose (releasing resources for the parent to collect).
  **Suggested Fix:** Document that ownership/resource transfer is outside the ghost model scope. This is the key semantic purpose of `bury()` and is not captured.

### Low

- **Location:** `lemma_find_thread_refinement_assumption` (proof — zombie.proof.rs:152)
  **Description:** Despite its name suggesting a refinement link, this lemma is a trivial consequence of the `spec_find_thread` definition. It proves nothing beyond what the spec already states. Its name is misleading — it is not actually an "assumption" in the verification sense.
  **Suggested Fix:** Rename to `lemma_find_thread_completeness` or similar, and add a comment clarifying it's a spec-level property, not a refinement proof.

- **Location:** `ZombieProcessView` (spec — zombie.spec.rs:55)
  **Description:** The `View` trait is implemented but `ZombieProcessView` is never used in any postcondition or proof lemma. It exists but serves no verification purpose.
  **Suggested Fix:** Either use it in specifications (e.g., `ensures result@ == ...`) or remove it to reduce unused abstraction.

- **Location:** `spec_seq_contains` (spec — zombie.spec.rs:105)
  **Description:** This helper is defined but never referenced in any spec, proof, or exec function. It duplicates the logic already in `spec_has_zombie_thread`.
  **Suggested Fix:** Remove the dead spec function or use it to define `spec_has_zombie_thread` for consistency.

## Positive Observations

- **Clean separation of concerns.** The three-file split (exec/spec/proof) is well-organized. Specifications are clearly separated from proofs and executable code.
- **Thorough documentation.** The module-level documentation in all three files clearly states the verification scope, trust boundary, and modeling decisions. The "Design-Level Verification" framing is honest and accurate.
- **Integration obligations are explicit.** The `spec_find_thread_integration_obligation` and `spec_process_state_pid_integration_obligation` specs provide clear contracts for future integration proofs.
- **All 16 verification conditions pass.** The module verifies cleanly with no errors.
- **Immutability properties are well-captured.** PID and exit status immutability across all operations is correctly specified and proven.
- **No-duplicates invariant.** Enforcing thread ID uniqueness in `wf()` is a good modeling choice that captures an important ownership property from the original Rust code.
- **`find_thread_mut()` frame conditions.** The postconditions correctly ensure that finding a thread mutably does not modify the zombie process state, which mirrors the original's semantics.

## Summary

The verification is a solid design-level (ghost model) proof of the ZombieProcess abstraction. It correctly captures the key structural invariants: non-empty zombie thread list, PID/status immutability, thread ID uniqueness, and decomposition via `bury()`. The three-file split is clean and the documentation is exemplary in its transparency about trust boundaries.

The main weaknesses are: (1) the `state_mut()` external_body trusts PID immutability without Rust-level enforcement, (2) `find_thread()`/`find_thread_mut()` are spec-level pass-throughs that don't verify the actual search logic, and (3) `bury()`'s resource-transfer semantics (its primary purpose) are not captured in the ghost model. These are inherent limitations of ghost-model verification and are honestly documented, but they represent real gaps in assurance for the most safety-critical properties (correct resource cleanup on process death).

Overall, this is a well-executed design-level verification that would benefit from tightening the `state_mut()` trust boundary and eventually moving toward executable verification of the search logic.
