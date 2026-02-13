# Review: runnable Abstraction (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Missing wakeup bridging lemma.**
  - Location: `runnable.proof.rs` — absent `lemma_wakeup_view_eq`.
  - Description: Every other state-changing operation (`run`, `terminate` (both branches), `add_thread`, `new`) has a bridging lemma proving `result@ == self@.spec_foo(...)`. There is no `lemma_wakeup_view_eq` proving that the exec-level `wakeup()` result view equals `self@.spec_wakeup(tid, time)`. This means downstream modules cannot use `spec_wakeup` as a one-shot postcondition without re-deriving the relationship themselves.
  - Suggested Fix: Add `lemma_wakeup_view_eq(&self, result: &RunnableProcess, tid: i64, time: i64)` to the proof file with preconditions mirroring the wakeup postconditions and an ensures of `result@ == self@.spec_wakeup(tid, time)`. The proof may require establishing that `spec_find_index` selects the same index as the concrete `vec_search`.

### Medium
- **`spec_run` hardcodes `interrupt_reason: 0i64`.**
  - Location: `runnable.spec.rs:566` — `RunnableProcessView::spec_run()`.
  - Description: The original `run()` returns `Option<InterruptReason>` from `next_thread.run()`, which is an opaque value from the thread's previous state. The spec transition hardcodes `0i64`, which is not necessarily the value the real implementation produces. While the exec-level postcondition does not constrain `interrupt_reason` (it's unconstrained at this abstraction level), the `spec_run()` function *does* produce a concrete `0i64`, creating a potential mismatch if anyone uses `spec_run()` and then inspects `interrupt_reason`.
  - Suggested Fix: Either (a) leave `interrupt_reason` unconstrained in `spec_run()` by using an `arbitrary()` / `choose` expression, or (b) add a comment noting that `interrupt_reason` in the `spec_run()` result is a placeholder and must not be relied upon by downstream proofs. The bridging lemma `lemma_run_view_eq` already has `result.interrupt_reason == 0i64` as a precondition, tying the two together, so this is safe but fragile.

- **`RunningProcessView` omits `ready_admission_times`.**
  - Location: `runnable.spec.rs:133-149` — `RunningProcessView` struct.
  - Description: The `RunningProcess` exec model (lines 137-154 of `runnable.rs`) does not carry `ready_admission_times`, and neither does `RunningProcessView`. If the downstream `RunningProcess` module needs to transition back to `RunnableProcess` (e.g., after the running thread blocks), the admission times for remaining ready threads would be lost at the view level. This is acceptable as a boundary model but should be documented as a limitation.
  - Suggested Fix: Add a brief comment in `RunningProcessView` noting that `ready_admission_times` is intentionally omitted and must be re-supplied by the downstream module when transitioning back.

- **`EXIT_STATUS_INTERRUPTED` constant is fragile.**
  - Location: `runnable.spec.rs:184` and `runnable.spec.rs:597`.
  - Description: The magic number `4` is used in both the spec constant and `spec_terminate_to_zombie()`. The existing TODO comment (lines 182-183) acknowledges this but no cross-module assertion exists yet. If `ErrorCode::Interrupted` changes its numeric value, the spec silently diverges from the real implementation.
  - Suggested Fix: Prioritize the CI check mentioned in the TODO, or add a `#[test]` in the non-Verus codebase that asserts `i32::from(ErrorCode::Interrupted) == 4`.

### Low
- **`spec_find_thread` uses magic number tags (0, 1, 2, 3).**
  - Location: `runnable.spec.rs:292-304`.
  - Description: The spec encodes thread list membership as integer tags. While documented, this is less abstract than using a proper enum-like discriminant at the spec level. A downstream consumer must remember the tag assignment.
  - Suggested Fix: Consider defining spec constants (e.g., `pub open spec fn THREAD_REF_READY() -> int { 0 }`) for clarity. Not urgent since this is spec-only and well-documented.

- **Duplicated helper functions between `RunnableProcess` and `RunnableProcessView`.**
  - Location: `runnable.spec.rs` — `spec_seq_contains`, `spec_remove_at`, `spec_min_index_rec` appear on both `RunnableProcess` (lines 307-344) and `RunnableProcessView` (lines 496-522).
  - Description: These are identical spec functions duplicated across the exec-level and view-level types. While the bridging lemma `lemma_view_min_index_eq` proves equivalence for `spec_min_index_rec`, the duplication adds maintenance burden.
  - Suggested Fix: Consider extracting these into a standalone spec module or using free functions. Low priority since Verus may require them on different impl blocks.

- **No `spec_from_state` on `RunnableProcessView`.**
  - Location: `runnable.spec.rs` — absent.
  - Description: The `from_state` constructor is modeled at exec level but has no view-level counterpart. Unlike `new()` which has `spec_new()`, `from_state()` has no `spec_from_state()`. This is defensible since `from_state` is a `pub(super)` internal constructor, but it means cross-module proofs about state reconstitution after transitions (e.g., returning from running to runnable) cannot use a one-shot spec function.
  - Suggested Fix: Add a `spec_from_state` on `RunnableProcessView` if downstream modules need to reason about round-trip transitions. Low priority if only sibling modules call `from_state`.

## Positive Observations
- **Comprehensive view-level spec transitions.** The seven spec transition functions (`spec_new`, `spec_run`, `spec_terminate_has_interrupted`, `spec_terminate_to_interrupted`, `spec_terminate_to_zombie`, `spec_wakeup`, `spec_add_thread`) cover all state-changing operations with clear, abstract semantics.
- **Bridging lemmas connect exec to view.** The proof file establishes that exec postconditions imply view-level spec transition equality for `new`, `run`, `terminate` (both branches), and `add_thread`. This is the key enabler for downstream compositional verification.
- **Well-formedness is cleanly factored.** Both exec-level `wf()` and view-level `wf()` are defined, with `lemma_wf_implies_view_wf` bridging them. The invariants are minimal and well-motivated.
- **Trust assumptions are thoroughly documented.** Thread ID disjointness (ownership semantics), HAL boundary types, and `state_mut()` mutation obligations are all clearly explained with forward-looking cross-module verification obligations.
- **Spec functions use proper abstractions.** `Seq`, `Seq::push`, `Seq::add`, `spec_remove_at` via subrange composition — no bit operations or raw index manipulation leak into the spec level.
- **Content-level postconditions.** All exec functions specify exact sequence contents (not just lengths), enabling strong compositional reasoning.
- **Verification passes cleanly.** 65 verified, 0 errors — all proofs discharge without issues.

## Summary
The abstraction layer is well-designed and nearly complete. The spec transition functions on `RunnableProcessView` provide clean, abstract state machine descriptions that downstream modules can use for compositional verification. The one significant gap is the missing `lemma_wakeup_view_eq` bridging lemma, which breaks the pattern established by all other operations. The `interrupt_reason` hardcoding in `spec_run` is a minor modeling concern. Overall this is strong work that establishes a solid foundation for cross-module verification.
