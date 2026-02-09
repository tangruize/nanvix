# Review: running_process (claude-opus-4.6) — Round 2

## Grade: A

## Verification Result

44 verified, 0 errors (up from 43 in round 1). All proof obligations discharged.

## Previous Issue Disposition

### High: Oracle parameters in `wakeup()` / `try_join_thread()` — RESOLVED (Documentation)

**Previous:** Oracle parameters create a trust gap if callers are `external_body`.
**Fix:** Added a dedicated "Oracle Parameters" section to the exec file header (lines 68–73) with explicit bold warning: **"All callers of these functions must be verified (not `external_body` or `assume`) for the oracle contracts to hold."**
**Verdict:** Correctly addressed. The structural concern is inherent to Verus's ghost/exec boundary and cannot be eliminated within this module. The documentation is precise and actionable. The oracle pattern itself is sound — Verus verifies the precondition at every non-`external_body` call site, so the constraint is machine-checked where it matters.

### Medium: `state_mut()` / `running_mut()` external_body frame conditions — RESOLVED (Documentation)

**Previous:** Frame condition postconditions are trusted, not proven.
**Fix:** Added "Discharged when `ProcessState` is independently verified" and "Discharged when `RunningThread` is independently verified" to the trust boundary documentation.
**Verdict:** Correctly addressed. These are genuine trust boundaries that can only be discharged by verifying sibling modules. The documentation now explicitly names the discharge condition. The `mutation_frame_preserved()` spec provides the hook for future cross-module proofs.

### Medium: `find_thread()` / `find_thread_mut()` spec-only modeling — RESOLVED (Documentation)

**Previous:** Exec-level search correctness is a trust assumption; `find_thread_mut()` mutation capability is unmodeled.
**Fix:** Expanded trust boundary documentation (lines 51–55) to explicitly note: exec-level search correctness is a trust assumption, `find_thread_mut()` permits mutation, and callers must preserve thread identity and list membership after mutation.
**Verdict:** Correctly addressed. The frame condition on `find_thread_mut()` (postcondition lines 569–575) already ensures modeled fields are unchanged by the search itself. The mutation-through-returned-reference concern is orthogonal and properly documented as a caller obligation.

### Medium: `interrupted_resume()` external_body — RESOLVED (Documentation)

**Previous:** Postconditions are assumed; sibling module changes could invalidate them.
**Fix:** Added "Original source: `src/kernel/src/pm/process/state/interrupted.rs::resume()`. This external_body is discharged when that function is independently verified." (lines 242–243).
**Verdict:** Correctly addressed. The source file reference creates a concrete traceability link.

### Low: Redundant `sleeping_count > 0 || !found` precondition — RESOLVED (Code Fix)

**Previous:** Redundant precondition in `wakeup()` could mask proof weaknesses.
**Fix:** (1) Added `lemma_wf_and_found_implies_sleeping_positive()` to proof file (lines 329–338). (2) Removed the redundant precondition from `wakeup()`. (3) Call the lemma in the proof block (line 1058).
**Verified:** The lemma body is empty (Verus auto-proves it), confirming the derivation is straightforward from `wf()` + `spec_seq_contains()`. Verification passes at 44 items (+1 from new lemma). This is the strongest fix in the batch — a real code improvement backed by a machine-checked proof.

### Low: `ready_count < u64::MAX` modeling assumption — RESOLVED (Documentation)

**Previous:** Minor specification strengthening vs. original.
**Fix:** Added to "Modeling Assumptions" section (lines 77–78): "prevents arithmetic overflow on `ready_count + 1`. Real systems never approach 2^64 threads per process."
**Verdict:** Correctly addressed.

### Low: Condvar / ContextInformation elision — RESOLVED (Documentation)

**Previous:** Elided HAL/sync types not captured in verification.
**Fix:** Added to "Modeling Assumptions" section (lines 79–81): explicitly lists what is elided and notes "If HAL or sync correctness is ever verified, these elisions must be revisited."
**Verdict:** Correctly addressed.

### Low: `wf()` vs `wf_strict()` — NOT EXPLICITLY ADDRESSED (Acceptable)

**Previous:** Suggested proving `wf_strict()` preservation across operations.
**Status:** No change. This was marked "acceptable as-is" in round 1 and remains so. `wf_strict()` is an optional downstream predicate; proving its preservation is a cross-module concern beyond this module's scope.

## New Issues Introduced

None. The only code change (removing redundant precondition + adding lemma) is verified and sound. Documentation changes are purely additive.

## Remaining Issues

### Medium

- **Location:** `state_mut()`, `running_mut()`, `interrupted_resume()` (exec file)
- **Description:** Three `external_body` annotations with assumed postconditions remain. These are inherent trust boundaries that cannot be discharged within this module alone — they require verification of `ProcessState`, `RunningThread`, and `InterruptedProcess::resume()` respectively. All are now explicitly documented with discharge conditions.
- **Status:** Irreducible within this module. Properly documented. No action needed here.

- **Location:** `find_thread()`, `find_thread_mut()` (exec file)
- **Description:** Exec-level search correctness (linear scan matches spec existential) remains a trust assumption due to Verus limitations with reference-returning functions. The frame condition for mutation through `find_thread_mut()` is documented as a caller obligation.
- **Status:** Irreducible within current Verus capabilities. Properly documented.

### Low

- **Location:** `wf_strict()` (spec file)
- **Description:** `wf_strict()` preservation across operations is not proven. Only `wf()` preservation is verified. This means downstream proofs that need thread ID uniqueness must re-derive it for each operation.
- **Status:** Optional enhancement. Not a soundness issue.

## Positive Observations

- **Clean remediation:** The prover addressed all 8 issues from round 1 — 1 with a code fix, 6 with documentation improvements, and 1 acknowledged as acceptable. No issues were dismissed without justification.
- **New lemma is well-crafted:** `lemma_wf_and_found_implies_sleeping_positive()` has an empty body (auto-proved by Verus), confirming the derivation is genuinely trivial and the original precondition was indeed redundant. This increases confidence in the proof architecture.
- **Documentation quality is exemplary:** The exec file header now has clearly delineated sections (Trust Boundary, Oracle Parameters, Modeling Assumptions, Bug Fix) that make every trust assumption explicit and traceable.
- **All prior positive observations still hold:** Complete function coverage (13/13), content-level specifications, bug discovery in `exit_thread()`, thread count conservation, PID immutability, clean spec/proof/exec split.

## Summary

The prover addressed all issues from round 1 with appropriate responses. The one code fix (redundant precondition removal + lemma addition) is the strongest improvement — it's machine-verified and reduces the API surface. The documentation improvements create explicit traceability for all trust assumptions and discharge conditions. The remaining issues are inherent to the verification approach (trust boundaries at module edges) and are properly documented. The verification is sound, complete within its stated trust boundaries, and well-structured for future cross-module integration.
