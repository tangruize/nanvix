# Review: runnable (claude-opus-4.6)

## Grade: A

## Previous Issues Assessment

### High: wakeup() oracle parameter (PARTIALLY FIXED)

- **Previous:** The `found` oracle delegates search correctness to the caller.
- **What changed:** A `sleeping_count == 0` early return was added (line 554) that handles the "no sleeping threads" case purely from exec-level state, without using the oracle. The trust boundary for the oracle is now explicitly documented (lines 489–498).
- **Verdict:** Genuinely improved. The `sleeping_count == 0` case is now fully verified without the oracle (Verus proves that `wf()` + `sleeping_count == 0` ⟹ empty ghost seq ⟹ `found == false`). The oracle is only consulted when `sleeping_count > 0`. This was exactly the suggested fix. The remaining oracle usage is inherent to the abstraction (ghost-only data structure) and properly documented.
- **Residual status:** Acceptable. Demoted to Low — the oracle is correctly constrained and only used when unavoidable.

### Medium: run() omitting InterruptReason (FIXED)

- **Previous:** The `InterruptReason` from the original's 4-tuple return was entirely absent.
- **What changed:** A ghost `interrupt_reason: Ghost<int>` field was added to `RunningProcess` (line 167), its `View` (line 131), and the `RunningProcessView` (line 483). It's populated with `Ghost(0int)` in `run()` (line 376).
- **Verdict:** Fixed as requested. The field signals to downstream verifiers that this data flows through the transition. The value is hardcoded to `0int` in the exec body, but since the postcondition of `run()` makes no claims about `interrupt_reason`, downstream code correctly treats it as unconstrained.
- **Minor note:** The comment "Unconstrained; downstream may refine" is slightly misleading since the ghost value IS deterministically 0 within the function. But from a verification perspective, since no postcondition constrains it, this is functionally equivalent to unconstrained. No action needed.

### Medium: wf() missing thread ID disjointness (FIXED)

- **Previous:** No disjointness predicate existed.
- **What changed:** `spec_seqs_disjoint()` (line 358) and `spec_ids_disjoint()` (line 369) were added as optional spec predicates. Correctly covers all 6 pairwise combinations of 4 lists. NOT added to `wf()` as designed — the trust assumption documentation in `wf()` now references the new predicate (line 342).
- **Verdict:** Fixed exactly as suggested. The predicate is available for cross-module proofs without over-constraining `wf()`.

### Medium: find_thread/find_thread_mut omitted (DOCUMENTED)

- **Previous:** Omitted from exec, spec-only model provided.
- **What changed:** Documentation improved (spec lines 61–64): "Callers of these functions should independently verify the correctness of the returned reference's properties against the `spec_find_thread` model."
- **Verdict:** Appropriately addressed. This is a Verus limitation, not a design choice. The added documentation clarifies the trust boundary for callers.

### Medium: state()/state_mut() omitted (DOCUMENTED)

- **Previous:** Mutation through `state_mut()` could break invariants.
- **What changed:** Documentation improved (spec lines 58–60): "Note: `state_mut()` allows arbitrary mutation of the inner `ProcessState`, which could affect invariants (e.g., vmem mapping). Cross-module verification of `ProcessState` mutations should be addressed when verifying callers."
- **Verdict:** Appropriately addressed for this module's scope.

### Low: EXIT_STATUS_INTERRUPTED magic number (DOCUMENTED)

- **Previous:** Hardcoded `4` with no cross-module check.
- **What changed:** TODO comment added (spec lines 164–166): "TODO (cross-module/CI): Add a CI check or cross-module assertion that validates this constant against the actual `ErrorCode::Interrupted` value."
- **Verdict:** Acceptably addressed with a TODO. The constant is still a magic number, but the risk is now tracked.

### Low: from_state() visibility difference (DOCUMENTED)

- **Previous:** `pub(super)` vs `pub` visibility mismatch.
- **What changed:** Documentation added (exec lines 251–253): "Note: The original is `pub(super)` (used by sibling state modules); here it is `pub` for Verus proof ergonomics (lemma construction). This visibility difference has no functional impact."
- **Verdict:** Fixed with documentation.

### Low: earliest_admission_time() fallback behavior (DOCUMENTED)

- **Previous:** Original has `unwrap_or(clock::now())` fallback not documented as dead code.
- **What changed:** Documentation added (spec lines 67–70): "The original has a fallback `unwrap_or(clock::now())` for the case when the iterator returns no minimum. This fallback is dead code given the `NonEmptyVecDeque` invariant... The spec model correctly omits this unreachable fallback."
- **Verdict:** Fixed with documentation.

## Issues Found

### Critical

(None)

### High

(None)

### Medium

(None)

### Low

- **Location:** `run()` in exec (runnable.rs:376)
- **Description:** The `interrupt_reason` is set to `Ghost(0int)` with a comment saying "Unconstrained." While this is effectively unconstrained from the caller's perspective (no postcondition mentions it), the value is deterministically 0 within the function body. This is a cosmetic distinction — consider rewording the comment to "Not constrained by postcondition; value is placeholder."
- **Suggested Fix:** Change comment to `// Value not constrained by postcondition; downstream may refine.`

- **Location:** `spec_ids_disjoint()` in spec (runnable.spec.rs:369)
- **Description:** The predicate checks between-list disjointness but not within-list uniqueness. In the original code, thread IDs are system-wide unique, meaning no list should contain duplicates either. This is a very minor gap since within-list uniqueness is also a Rust ownership invariant (you can't have two `ReadyThread` structs with the same ID if each thread struct is moved into the deque).
- **Suggested Fix:** No change needed — within-list uniqueness is covered by the same ownership trust assumption. Could optionally add a comment noting this.

## Positive Observations

- **All previous issues addressed:** Every issue from the A1 review was either fixed with code changes or appropriately documented with clear justification. No issues were dismissed without evidence.
- **Verification still clean:** 43 verified, 0 errors. No regressions from the changes.
- **No assume statements:** Zero `assume` calls across all three files.
- **`sleeping_count == 0` early return:** This is a genuinely clever improvement — it narrows the oracle's scope to only the case where sleeping threads actually exist, and the verification proves the empty case automatically.
- **`spec_ids_disjoint` design:** Correctly placed outside `wf()` as an optional predicate, maintaining the lean invariant while providing the predicate for downstream use. All 6 pairwise combinations are covered.
- **`interrupt_reason` tracking:** The ghost field in `RunningProcess` correctly signals data flow without over-constraining the abstraction. The `RunningProcessView` includes it for downstream reasoning.
- **Documentation quality:** Trust boundaries, oracle justifications, and exec coverage notes are thorough and precise.
- **Strong content-level postconditions:** All operations specify exact list contents (not just counts), which is the gold standard for collection-manipulating verified code.

## Summary

The prover has substantively addressed all issues from the A1 review. The most significant improvement is the `sleeping_count == 0` early return in `wakeup()`, which narrows the oracle's scope and adds a verified path that previously relied entirely on the caller. The `interrupt_reason` ghost field, `spec_ids_disjoint` predicate, and documentation improvements all directly address the requested changes. The verification remains clean at 43/0. The only remaining items are cosmetic (comment wording) and theoretical (within-list uniqueness), both at Low priority. This is a well-executed verification of a non-trivial kernel component.
