# Review: interrupted_process Exec Consistency (claude-opus-4.6)

## Grade: A

## Files Reviewed

- Original: `src/kernel/src/pm/process/state/interrupted.rs`
- Exec: `verus/split/kernel/pm/process/state/interrupted.rs`
- Spec: `verus/split/kernel/pm/process/state/interrupted.spec.rs`
- Proof: `verus/split/kernel/pm/process/state/interrupted.proof.rs`
- Consistency report: `verus-ai-history/ast-consistency/interrupted_process_20260214_154057_fix.md`

## Verification Status

**PASSED** — 38 verified, 0 errors.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** All 8 function mismatches are documented with clear justifications in the consistency report. Each mismatch falls into one of two categories:

- **Type abstraction** (`new`, `from_sleeping`, `state`, `state_mut`, `interrupt`): Parameter/return types differ due to the fundamental model abstraction (`Box<ProcessState>` → `u64`, `NonEmptyVecDeque<T>` → `Vec<u64>`). The algorithmic logic is identical.
- **Verus limitation** (`resume`, `find_thread`, `find_thread_mut`): Additional oracle parameters or spec-only return types required because Verus cannot express reference-typed returns or access HAL clock boundaries.

No function was silently dropped or had its algorithm altered without justification.

### 2. Were MISSING functions added with proper verification?

**N/A.** The consistency report states 0 missing functions were added. All original functions have Verus counterparts. The two EXTRA items (`resume_with_valid_clock`, boundary `RunnableProcess`) are verification helpers, not restorations of missing originals, and are properly justified.

### 3. Are equivalence justifications sound?

**Yes, with one minor observation.**

- **`new` / `from_sleeping`**: Straightforward field mapping. `sleeping_threads: None` → `Vec::new()` is correct since empty `Vec` models `None` for `Option<NonEmptyVecDeque<_>>`. Verified by postconditions and `wf()` establishment.
- **`state` / `state_mut`**: PID return models `&ProcessState` / `&mut ProcessState` under the abstraction. Frame condition on `state_mut` is proven (all fields preserved). Integration obligation `spec_process_state_pid_integration_obligation` properly defers the PID linking to construction sites.
- **`resume`**: Core algorithm faithfully reproduced:
  - Original: `pop_front()` → `next_thread.resume()` → `RunnableProcess::from_state(...)`.
  - Verus: `Vec::remove(0)` → identity (ID-preserving) → direct `RunnableProcess` construction.
  - The `admission_time` oracle parameter is well-justified — `clock::now()` is a HAL boundary. The `resume_with_valid_clock` wrapper provides an enforcement path.
  - `lemma_resume_refines_spec` connects exec postconditions to the view-level `spec_resume()`.
- **`find_thread` / `find_thread_mut`**: Spec-only model is the correct approach given Verus limitations. The search order (interrupted → sleeping → zombie) matches the original. Trust gap is explicitly documented with `lemma_find_thread_refinement_assumption`.
- **`interrupt`**: `thread.interrupt(InterruptReason::Killed)` → `(sleeping_tid, 0u64)` is ID-preserving with reason tag. Semantically equivalent.

**Minor observation**: The `state_mut()` in the original returns `&mut ProcessState`, allowing actual mutation of process state fields (e.g., capabilities). The Verus model returns `u64` (immutable PID), which means any downstream code relying on mutating `ProcessState` through this accessor is not modeled. The documentation acknowledges this ("frame condition holds trivially"), but this is worth noting as a scope limitation — not a flaw in the consistency fix.

### 4. Does the exec code now faithfully represent the original source?

**Yes.** Function-by-function comparison confirms:

| Original Function | Verus Function | Algorithm Match |
|---|---|---|
| `InterruptedProcess::new` | `InterruptedProcess::new` | ✅ Identical logic, abstracted types |
| `InterruptedProcess::from_sleeping` | `InterruptedProcess::from_sleeping` | ✅ Identical logic, abstracted types |
| `InterruptedProcess::state` | `InterruptedProcess::state` | ✅ Returns PID (abstraction of `&ProcessState`) |
| `InterruptedProcess::state_mut` | `InterruptedProcess::state_mut` | ✅ Returns PID, frame condition proven |
| `InterruptedProcess::resume` | `InterruptedProcess::resume` | ✅ Pop-front + resume + construct RunnableProcess |
| `InterruptedProcess::find_thread` | `InterruptedProcess::find_thread` | ✅ Spec model, search order preserved |
| `InterruptedProcess::find_thread_mut` | `InterruptedProcess::find_thread_mut` | ✅ Spec model, frame condition proven |
| `interrupt` (standalone) | `interrupt` (standalone) | ✅ ID-preserving with reason tag |

The exec code includes 29 proof functions supporting well-formedness preservation, view equality, refinement, projection, and integration obligations. The proof infrastructure is thorough.

### 5. Does verification still pass?

**Yes.** Verification passes cleanly: `38 verified, 0 errors`.

## Issues Found

### Critical

None.

### Minor

1. **`state_mut()` mutation scope**: The original allows mutation of arbitrary `ProcessState` fields via the returned `&mut ProcessState`. The Verus model abstracts this to an immutable PID return, which means mutations performed through this accessor in other modules are outside the verification scope. This is correctly documented but represents a trust gap for code paths that modify `ProcessState` after calling `state_mut()`.

2. **`find_thread` / `find_thread_mut` executable logic unverified**: The iterator-based search (`iter().find(...)`) across three collections is modeled spec-only. A bug in the real search predicate or collection ordering would not be caught. This is a known Verus limitation, properly documented with `lemma_find_thread_refinement_assumption` and integration obligations. The `lemma_find_thread_result_unique` proof under `wf()` provides confidence that the spec model is at least deterministic.

3. **Per-thread state mutation trust gap in `resume()`**: The original `InterruptedThread::resume()` calls `set_interrupt_reason()` before converting to `ReadyThread`. This per-thread mutation is not modeled. Properly documented with `spec_resume_reason_integration_obligation` and deferred to thread module verification.

### Observations

- The consistency report accurately reflects the state of the code — no discrepancies found between the report and actual files.
- Documentation quality is excellent: module-level doc comments, per-function trust gap documentation, and integration obligations are comprehensive.
- The view-level abstract state transitions (`spec_resume`, `spec_new`, etc.) in the spec file provide clean downstream interfaces.
- The `lemma_project_to_runnable_boundary` projection lemma thoughtfully handles the cross-module boundary shape mismatch (runnable module omits `sleeping_thread_ids`).

## Summary

The exec consistency fix for `interrupted_process` is thorough and well-executed. All 8 function mismatches and 2 extra items are justified with sound equivalence arguments backed by formal integration obligations. The type abstraction strategy (complex kernel types → integer IDs) is consistently applied and well-documented. Trust gaps are explicitly enumerated with machine-readable spec obligations for future integration proofs. Verification passes cleanly at 38/0. The only reason this does not receive A+ is the inherent (and acknowledged) unverified surface area around `find_thread` executable logic, `state_mut` mutation scope, and per-thread state mutation in `resume()` — these are Verus/model limitations, not implementation defects.
