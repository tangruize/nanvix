# Review: kcall_terminate Exec Consistency (claude-opus-4.6)

## Grade: A

## Summary

The exec consistency fix is sound and minimal. The single change — adding a
`terminate` wrapper that delegates to `terminate_model` — correctly restores
the missing function name from the original source. The exec model faithfully
reproduces the original two-step pipeline (PID parse → PM terminate), with
appropriate parameter abstraction (`&mut ProcessManager` → `Ghost`,
`&KcallArgs` → `arg0: u32`). Verification passes cleanly: 21 verified, 0
errors, no `assume`/`admit` introduced, no new `external_body` added.

## Review Criteria

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**PASS.** The consistency report states 0 mismatches; there were none to fix.
The existing `terminate_model` already contained the full exec logic matching
the original control flow.

### 2. Were MISSING functions added with proper verification?

**PASS.** One missing function was identified: `terminate` (the wrapper
matching the original `pub fn terminate(pm, args) -> KcallResult`). The added
wrapper (lines 561–577) delegates to `terminate_model` and returns only the
`KcallResultModel`, discarding the ghost witnesses. The wrapper has its own
postconditions (result exhaustiveness, kernel PID rejection) that are
inherited from `terminate_model`'s ensures clauses. This pattern matches the
`sleep` module's wrapper (which also delegates to its `_model` variant).

### 3. Are equivalence justifications sound?

**PASS.** No equivalence justifications were needed (0 documented
equivalences). The API mapping table in the module-level docs (lines 140–149)
clearly maps each original API call to its model function with `external_body`
annotations. Parameter abstraction is documented and justified.

### 4. Does the exec code now faithfully represent the original source?

**PASS.** Line-by-line comparison of the original `terminate` function
(src/kernel/src/pm/kcall/terminate.rs:21–34) against `terminate_model`
(lines 490–539):

| Original Step | Exec Model Step | Match |
|---|---|---|
| `ProcessIdentifier::try_from(args.arg0)` | `try_from_process_identifier(arg0)` | ✓ |
| `Err(error) => return KcallResult::Error(error.code.into())` | `PidError { error_code } => KcallResultModel::Error { error_code }` | ✓ |
| `error!("{error:?}")` (logging) | Omitted, documented in module docs (line 137) | ✓ |
| `pm.terminate(pid)` | `process_manager_terminate(pid, Ghost(pm_pre))` | ✓ |
| `Ok(()) => KcallResult::ok()` | `TmOk => KcallResultModel::Ok` | ✓ |
| `Err(e) => KcallResult::Error(e.code.into())` | `TmError { error_code } => KcallResultModel::Error { error_code }` | ✓ |

The control flow is structurally identical. The `match` nesting is preserved.
The error short-circuit on PID parse failure (early return) is faithfully
modeled by the outer match arm returning immediately.

### 5. Does verification still pass?

**PASS.** Verification output: `21 verified, 0 errors`.

- No `assume` or `admit` in any of the three files (exec, spec, proof).
- 2 `external_body` annotations on dependency models (T1: `try_from_process_identifier`,
  T2: `process_manager_terminate`) — pre-existing, not added by this fix.
- 1 `external_body` on `axiom_kernel_pid_is_valid` (A1) — pre-existing axiom,
  documented as discharged by the `sys::pid` module.

## Issues Found

### Critical

None.

### Minor

1. **Wrapper postconditions are weaker than `terminate_model`.** The `terminate`
   wrapper only ensures result exhaustiveness and kernel PID rejection, while
   `terminate_model` has ~20 ensures clauses covering state transitions, error
   code preservation, liveness, etc. This is acceptable because the wrapper is
   a convenience function and callers needing full guarantees should use
   `terminate_model` directly. However, if `terminate` becomes the primary API,
   consider forwarding more postconditions (e.g., state preservation on error,
   success-implies-terminatable).

2. **Wrapper does not expose PM post-state.** The `terminate` wrapper discards
   `Ghost<ProcessManagerStateView>` post-state, so callers cannot reason about
   state transitions. This is consistent with the `sleep` wrapper pattern
   (which also discards ghost state), but limits composability. This is a
   design choice, not a bug.

## Trust Boundary Assessment

The three trust boundaries (T1, T2, A1) are well-documented with clear
cross-module discharge paths. Postconditions on `process_manager_terminate`
are comprehensive (12 ensures clauses covering kernel rejection, running
rejection, non-existent rejection, state preservation, liveness, frame
conditions, and well-formedness preservation). The `spec_pm_wf` invariant
prevents postcondition inconsistency, which is proven by
`lemma_wf_prevents_inconsistency`.

## Proof Quality

The 19 proof lemmas cover error propagation, success requirements,
exhaustiveness, error code preservation, kernel/running PID protection,
state transitions, liveness, and well-formedness. The proof structure is
clean with no unnecessary complexity. All lemmas have clear doc comments
explaining their role in the verification.
