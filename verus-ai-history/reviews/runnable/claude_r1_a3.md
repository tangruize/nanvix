# Review: runnable (claude-opus-4.6)

## Grade: A

## Previous Issues Status (R2 → R3)

### Medium Issues (from R2)

1. **`wakeup()` Ok branch postcondition style inconsistency** — ✅ **FIXED.** The element-wise specification (R2 lines 481–484) has been replaced with sequence equality: `r.ready_thread_ids@ == self.ready_thread_ids@.push(tid@)` (exec line 481). This is now consistent with `add_thread()` (exec line 590) and more compositionally useful.

### Low Issues (from R2)

2. **`RunningProcess::wf()` is `true`** — ✅ **RESOLVED.** The `wf()` still returns `true` (spec line 304) but the documentation has been enhanced with an explicit `TODO (cross-module)` annotation (spec lines 300–302) indicating that a cross-module linking assertion should be added when the RunningProcess module's own verification is complete. This is the correct approach for modular verification — the boundary model's weakness is now explicitly tracked.

3. **`wakeup()` admission times not content-specified** — ✅ **FIXED.** The Ok branch now includes `exists|t: int| t >= 0 && r.ready_admission_times@ == self.ready_admission_times@.push(t)` (exec lines 483–484), exactly matching the suggested fix. This exposes that admission times grow by exactly one non-negative element, enabling downstream reasoning.

## Issues Found

### Critical

_None._

### High

_None._

### Medium

_None._

### Low

- **Location:** `RunningProcess::wf()` (spec: `runnable.spec.rs:303-304`)
  - **Description:** Still returns `true`. This is an acknowledged design decision with a `TODO` for cross-module linking. No action needed in this module — tracked for future work.
  - **Status:** Accepted — deferred to cross-module verification phase.

## Positive Observations

- **All R2 issues addressed:** The 3 remaining issues from R2 (1 medium, 2 low) have all been resolved — 2 fully fixed and 1 properly tracked with a TODO.
- **Verification passes cleanly:** 38 verified, 0 errors. No assumes, no trusted functions, only 2 justified `external_body` declarations.
- **Complete function coverage:** All 11 original functions are accounted for — 7 with exec-level verified implementations, 3 with spec-only models (documented justification for each), and 1 pair of trivial accessors intentionally elided.
- **Strong postconditions throughout:** Every state-transition function (`run`, `terminate`, `wakeup`, `add_thread`) specifies exact sequence contents via `==`, `push`, `add`, or `spec_remove_at`. No function relies on length-only specifications.
- **Consistent postcondition style:** After the R2→R3 fix, `wakeup()` and `add_thread()` now both use `Seq::push` for the ready list specification, eliminating the previous inconsistency.
- **Admission time tracking:** The existentially-quantified admission time in `wakeup()` correctly models the non-deterministic nature of `clock_now()` while preserving the structural relationship.
- **Sound trust boundary:** All trust assumptions are explicit and documented — ownership semantics (Rust type system), oracle parameters (algorithmic correctness), HAL boundary elisions, and boundary models. No overclaiming.
- **Comprehensive proof library:** The proof file contains 38 verified lemmas covering construction, PID immutability, state transitions, content preservation, find-thread consistency, sequence minimum existence, and view equality.
- **`spec_find_thread` model faithfully captures the original** search order (ready → interrupted → sleeping → zombie) with proven consistency properties.
- **Clean three-file separation:** Spec (pure specifications and view types), proof (lemmas), and exec (implementations) are cleanly separated with minimal coupling.

## Summary

The verification has reached maturity. All issues from the R1 and R2 review cycles have been addressed. The module now provides content-level specifications for all state-transition operations, honest trust boundary documentation, and comprehensive proof coverage. The only remaining item is the `RunningProcess::wf()` boundary model weakness, which is correctly deferred to cross-module verification and explicitly tracked with a TODO.

The verification captures the essential correctness properties of the `RunnableProcess` state machine: construction well-formedness, PID immutability, non-empty ready thread invariant, earliest-admission-time scheduling semantics, correct thread movement across lists during terminate/wakeup, and exhaustive thread search semantics. The abstraction level — modeling threads as ID sequences rather than full thread objects — is appropriate for this module's role as a process-level thread collection manager.

Grade improves from A- to A. No further iteration is needed.
