# Review: runnable (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(None)

### High

- **Location:** `wakeup()` in exec (runnable.rs:483–596)
- **Description:** The original `wakeup()` has two-level branching: first it checks whether `sleeping_threads` is `Some` (i.e., the list exists at all), and only then searches within it via `remove_if`. The verified version collapses this into a single `found` oracle that checks `spec_seq_contains(sleeping_thread_ids, tid)`. This is semantically equivalent because in the verification model, `Option<NonEmptyVecDeque<T>>` maps to `Seq<int>` where `None` → empty seq. When there are no sleeping threads, the seq is empty, so `spec_seq_contains` returns false, matching the original's early `Err(self)` return. However, the `found` oracle parameter is an exec-level `bool` whose value cannot be derived from the ghost-only sleeping list. The precondition `found == spec_seq_contains(...)` effectively delegates correctness to the caller. This is documented and justified, but it means the verification does **not** prove that the search itself is correct — only that _given_ a correct search result, the state transition is correct.
- **Suggested Fix:** Document this as an explicit trust boundary in the verification model. Consider adding an exec-level sleeping count check (already available via `sleeping_count`) to at least verify the `sleeping_count == 0 => !found` case without an oracle.

### Medium

- **Location:** `run()` in exec (runnable.rs:317–366) vs. original (runnable.rs:109–149)
- **Description:** The original `run()` returns a 4-tuple `(RunningProcess, Option<InterruptReason>, *mut ContextInformation, Option<VirtualAddress>)`. The verified version only returns `RunningProcess`, omitting the `InterruptReason`, `ContextInformation` pointer, and `VirtualAddress`. This is documented as "HAL boundary" but means the verification does not track what interrupt reason is propagated alongside the state transition. While these are indeed hardware-level concerns, `InterruptReason` is a kernel-level concept (not purely HAL) that could affect process scheduling decisions downstream.
- **Suggested Fix:** Consider adding a ghost `interrupt_reason` field to the boundary `RunningProcess` model, even if its value is unconstrained, to signal to downstream verifiers that this data flows through the transition.

- **Location:** `wf()` in spec (runnable.spec.rs:330–341)
- **Description:** The well-formedness predicate does not enforce thread ID uniqueness/disjointness across the four thread lists. While the spec file documents this as a trust assumption inherited from Rust's ownership model (which is sound reasoning), this means the verification cannot detect a hypothetical bug where the same thread ID ends up in two lists simultaneously. The content-level postconditions on operations partially compensate by tracking exact list contents.
- **Suggested Fix:** This is acceptable as-is given the trust model, but consider adding a `spec fn ids_disjoint(&self) -> bool` predicate (not in `wf()` but available for cross-module proofs) that downstream modules could use to assert disjointness when needed.

- **Location:** `find_thread()` / `find_thread_mut()` — omitted from exec
- **Description:** These two public functions are omitted from exec-level modeling because they return reference types (`ThreadRef<'_>`, `ThreadRefMut<'_>`) that Verus cannot express. A spec-only model `spec_find_thread` is provided. This is a reasonable limitation of Verus, but these are public API functions that callers rely on. The spec model only returns an `Option<int>` tag indicating which list, not the thread data itself.
- **Suggested Fix:** No immediate fix needed given Verus limitations, but document that callers of `find_thread`/`find_thread_mut` should have their own verification of the returned reference's properties.

- **Location:** `state()` / `state_mut()` — omitted from exec
- **Description:** These accessors are omitted because they return `&ProcessState` / `&mut ProcessState`. The `pid_i32()` method partially compensates. However, `state_mut()` allows arbitrary mutation of the inner `ProcessState`, which could affect invariants that downstream code relies on (e.g., vmem mapping). The verification cannot track what happens through `state_mut()`.
- **Suggested Fix:** Acceptable for this module's scope. Cross-module verification of `ProcessState` mutations should be addressed when verifying callers.

### Low

- **Location:** `EXIT_STATUS_INTERRUPTED()` in spec (runnable.spec.rs:155)
- **Description:** The constant is hardcoded as `4` with a comment referencing `src/libs/sysapi/src/errno.rs:21`. If the error code numbering ever changes, this would silently become incorrect. The `exit_status_interrupted_value()` external_body function ensures the exec value matches the spec constant, but the spec constant itself is a magic number.
- **Suggested Fix:** Add a cross-module verification TODO or a CI check that validates this constant against the actual `ErrorCode::Interrupted` value.

- **Location:** `from_state()` in exec (runnable.rs:256–291)
- **Description:** The original `from_state()` takes `Box<ProcessState>` as first parameter. The verified version takes `ProcessIdentifier` directly. This is fine for the abstraction level, but the original signature is `pub(super)` meaning it's used by sibling state modules. The verification model's `from_state` is `pub`, which is a minor visibility discrepancy.
- **Suggested Fix:** No functional impact. Verus proof ergonomics require `pub` fields/methods. Document the visibility difference.

- **Location:** `earliest_admission_time()` — spec-only (runnable.spec.rs:311–315)
- **Description:** The original function has a fallback `unwrap_or(clock::now())` for the case when the iterator returns no minimum. The spec model's `spec_earliest_admission_time()` has a `recommends` clause requiring `ready_thread_ids.len() > 0` but no fallback. Since `wf()` ensures at least one ready thread, this is sound, but the fallback behavior difference is not documented.
- **Suggested Fix:** Add a comment noting that the `unwrap_or` fallback in the original is dead code given the `NonEmptyVecDeque` invariant, and that the spec model correctly omits it.

## Positive Observations

- **Verification passes cleanly:** All 43 verification conditions pass with no errors.
- **No `assume` statements:** The proof files contain zero `assume` calls. All properties are proven from first principles.
- **Minimal `external_body`:** Only two `external_body` functions (`clock_now` and `exit_status_interrupted_value`), both with tight postconditions and clear justifications.
- **Strong postconditions:** The `run()`, `terminate()`, `wakeup()`, and `add_thread()` functions all have content-level postconditions specifying exact list contents after the operation, not just count-level properties.
- **Oracle elimination:** The `terminate()` function eliminates its oracle parameter entirely by using exec-level counters. The `run()` function derives its min-index via proven lemma. Only `wakeup()` retains a `found` oracle, which is well-justified.
- **Comprehensive proof library:** The proof file contains 25+ lemmas covering construction, PID immutability, thread count preservation, well-formedness preservation, minimum-finding correctness, and content preservation.
- **Clear trust boundary documentation:** The spec file contains extensive documentation of what is and isn't verified, trust assumptions, and cross-module verification TODOs.
- **`spec_min_index_rec` correctness:** The minimum-index finding algorithm is verified via inductive proof (`lemma_min_index_rec_bounds`), proving both bounds and minimality — this is the most algorithmically interesting part of the verification.
- **Well-structured split:** Spec, proof, and exec are cleanly separated with appropriate concerns in each file.

## Summary

This is a high-quality verification of a non-trivial OS kernel component. The verification successfully captures the essential correctness properties: PID immutability, thread list non-emptiness invariant (NonEmptyVecDeque), correct state transitions (ready→running, ready→zombie, sleeping→interrupted, sleeping→ready), and content-level list manipulation correctness.

The abstraction model is well-chosen: threads are abstracted to IDs with admission times, and `Option<NonEmptyVecDeque<T>>` is soundly modeled as `Seq<int>`. The main gap is the `wakeup()` oracle parameter, which is a pragmatic necessity given Verus's inability to evaluate `Seq::contains()` at exec level, but it shifts some correctness burden to callers. The omission of `find_thread`/`find_thread_mut` from exec modeling is a Verus limitation, not a design flaw.

The verification would benefit from: (1) an optional `ids_disjoint` spec for cross-module use, (2) ghost tracking of `InterruptReason` through `run()`, and (3) a cross-module check on the `EXIT_STATUS_INTERRUPTED` constant. These are enhancements rather than corrections — the current verification is sound within its stated trust boundary.
