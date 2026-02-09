# Review: interrupted_process (claude-opus-4.6)

## Grade: A-

## Previous Issue Disposition

### High #1: `resume()` thread state mutation not modeled (`interrupt_reason`)
- **Status: Addressed (documentation).**
- The prover added explicit trust gap documentation in the exec module header (lines 44–51), the spec trust assumptions (lines 38–44), and the `resume()` docstring (lines 265–271). The documentation clearly states: "This per-thread mutation is NOT modeled because threads are abstracted to integer IDs in this module. If downstream code relies on `interrupt_reason` being set in the ready thread's state, that property must be verified in the thread module's own verification."
- **Verdict:** This is the correct resolution. At this abstraction level (threads = integer IDs), modeling per-thread state fields would violate the abstraction boundary. The documentation properly defers the obligation to the thread module. No longer an issue.

### High #2: `admission_time` oracle decouples from `clock::now()`
- **Status: Addressed (documentation).**
- Updated documentation in exec (lines 52–60, 275–280) and spec (lines 70–74) now explicitly states: "The link is advisory at this module level and must be enforced at the integration proof level." The `spec_admission_time_valid()` predicate and `lemma_valid_admission_time_satisfies_resume_precondition` remain as-is. The `resume()` precondition still only requires `admission_time@ >= 0`.
- **Verdict:** The oracle pattern is a sound design choice for modular verification — the clock is a HAL boundary. Making `spec_admission_time_valid()` a *precondition* of `resume()` would be wrong since this module cannot observe the clock. The documentation now clearly communicates the integration obligation. **Downgraded from High to residual Low.** Residual risk: no *automated* enforcement that callers provide a valid clock reading; relies on manual integration proof discipline.

### Medium #3: Boundary model inconsistency (missing `sleeping_thread_ids` in `runnable.spec.rs`)
- **Status: Partially addressed.**
- The prover added `lemma_interrupted_view_subsumes_runnable_boundary` (proof lines 325–351) and improved documentation (exec lines 40–42, spec lines 46–49).
- **Critical assessment of the bridging lemma:** The lemma is defined on `impl RunnableProcess` (the *boundary type* in the interrupted module) and takes an `InterruptedProcess` (the *primary type* from this module). It proves: `ip@.pid == ip.pid@`, `ip@.interrupted_thread_ids == ip.interrupted_thread_ids@`, etc. These are trivially true from the `View` implementation. The lemma does NOT reference the runnable module's separate `InterruptedProcess` type (which is a *different struct* in `runnable.rs:168–175` with only `pid`, `interrupted_thread_ids`, `zombie_thread_ids`). It cannot, because Verus modules are verified independently and cannot cross-reference each other's types.
- **Verdict: The bridging lemma does not actually bridge the two incompatible InterruptedProcess types across modules.** It restates trivial view properties. The documentation improvement is the real value. The structural gap remains — the runnable module's boundary `InterruptedProcess` still lacks `sleeping_thread_ids`, and its `wf()` (runnable.spec.rs:429–431) only checks `interrupted_thread_ids@.len() >= 1` without uniqueness or disjointness. **Remains Medium** but documentation now properly acknowledges the gap.

### Medium #4: `find_thread()` spec-only models — tag for trust-boundary inventory
- **Status: Addressed.**
- Added "Tagged for trust-boundary inventory" (exec line 74, spec line 58). No code change needed.

### Medium #5: No upper bound on thread counts in `wf()`
- **Status: Not addressed (implicitly rejected).**
- **Verdict: Rejection is reasonable.** My original review rated this "Low priority" and suggested "accept as out-of-scope for functional correctness." Ghost sequences are inherently unbounded; thread count limits are implementation constraints, not functional correctness properties. This module's purpose is to verify state transition logic, not resource bounds.

### Low #6: `state_mut()` frame condition trivially satisfied
- **Status: Already acceptable.** No changes needed.

### Low #7: Duplicated helper functions (`spec_no_duplicates`, `spec_seqs_disjoint`)
- **Status: Not addressed.**
- **Verdict: Reasonable deferral.** Cosmetic issue with no correctness impact.

## Issues Found

### Critical

_None._

### High

_None._

### Medium

- **Location:** `lemma_interrupted_view_subsumes_runnable_boundary` in proof (`interrupted.proof.rs:335–351`)
  - **Description:** The bridging lemma claims to map between this module's `InterruptedProcess` (which includes `sleeping_thread_ids`) and the runnable module's boundary `InterruptedProcess` (which omits it). However, the lemma only proves trivial `View` equalities (`ip@.pid == ip.pid@`, etc.) that are tautological from the `View for InterruptedProcess` implementation. It does not actually reference or constrain the runnable module's separate `InterruptedProcess` type (`runnable.rs:168–175`), which is a structurally different type missing `sleeping_thread_ids`. A real cross-module integration proof would need to manually project this module's `InterruptedProcess` to the runnable module's boundary type, and this lemma provides no meaningful assistance.
  - **Suggested Fix:** Either (a) acknowledge in documentation that this lemma is a documentation artifact, not a functional bridge, or (b) add a postcondition projecting to a tuple `(int, Seq<int>, Seq<int>)` representing the runnable module's boundary model fields, so downstream proofs have a concrete projection to work with. Alternatively, add `sleeping_thread_ids` to the runnable module's boundary `InterruptedProcess` to eliminate the structural mismatch.

- **Location:** `find_thread()` / `find_thread_mut()` in exec (`interrupted.rs:397–449`) — inherited trust gap
  - **Description:** (Carried from previous review, properly documented.) These are spec-only models that bypass the executable `iter().find()` logic. The `lemma_find_thread_refinement_assumption` (proof lines 226–245) is a documentation-only lemma that restates the `spec_find_thread` definition — it does not introduce any `assume` or `axiom`, so it doesn't weaken soundness, but it also doesn't *prove* anything about the real implementation. A bug in the original's search predicate or collection ordering would not be caught. Properly tagged for trust-boundary inventory.
  - **Suggested Fix:** Accept as Verus limitation. Already properly documented and tagged.

### Low

- **Location:** `resume()` in exec — clock oracle remains advisory
  - **Description:** (Downgraded from previous High.) The `spec_admission_time_valid()` predicate exists but is never required by any `requires` clause in this module. Integration proofs must manually establish the clock link. Now properly documented as intentional.
  - **Suggested Fix:** No change needed at this module level. When integration proofs exist, verify they reference `spec_admission_time_valid()`.

- **Location:** `spec_no_duplicates` / `spec_seqs_disjoint` — duplicated across types
  - **Description:** (Carried from previous review.) Identical spec helper functions are defined on both `InterruptedProcess` (spec lines 197–207) and `RunnableProcess` (spec lines 304–314). Divergence risk if one is modified.
  - **Suggested Fix:** Extract to shared spec utility module. Low priority.

## Positive Observations

- **Verification passes:** 23 verified, 0 errors (up from 22 — the new bridging lemma). No `assume`, `external_body`, or `trusted` annotations in any executable or proof code.
- **Trust boundary documentation is now excellent:** Every trust gap from the previous review (interrupt_reason mutation, clock oracle, find_thread spec-only model, ProcessState abstraction, boundary model inconsistency) is now explicitly documented with clear scope and remediation paths. This is a model for how to document verification boundaries.
- **All original functions are covered:** `new`, `from_sleeping`, `state`, `state_mut`, `resume`, `find_thread`, `find_thread_mut`, and standalone `interrupt` — all have verified models with appropriate contracts.
- **`resume()` proof remains rigorous:** Proves all six pairwise disjointness conditions, no-duplicates preservation through subrange, front-not-in-tail, and singleton well-formedness. PID preservation is explicit.
- **`wf()` correctly models Rust ownership semantics:** Intra-list uniqueness and inter-list disjointness faithfully capture the type system guarantees of the original `NonEmptyVecDeque` collections.
- **Clean spec/proof/exec split:** Spec contains only `open spec fn` and view types. Proof contains only lemmas. Exec has inline proof only in `resume()` where needed. No bleed between concerns.
- **No new issues introduced:** The documentation changes and bridging lemma addition did not weaken any existing guarantees or introduce unsoundness.

## Summary

The prover meaningfully addressed the two previous High issues through thorough trust boundary documentation. Both were fundamentally design decisions (abstracting threads to IDs, using clock oracles for modular verification) that were already sound — the fix was making the documentation match the intent. The Medium boundary model inconsistency (`sleeping_thread_ids` missing in runnable's `InterruptedProcess`) received a bridging lemma that is unfortunately trivial and doesn't functionally bridge the two separate types, but the improved documentation properly acknowledges the gap and its scope. No previous issues were worsened, no new issues were introduced, and the verification remains clean (23/0, no trusted primitives).

The verification captures the essential correctness properties of the `InterruptedProcess` module: well-formedness preservation, PID immutability, correct thread list manipulation in `resume()`, and proper state transition contracts. The remaining issues are either Verus limitations (find_thread), cross-module structural mismatches (boundary model), or intentionally advisory patterns (clock oracle) — all properly documented and scoped.
