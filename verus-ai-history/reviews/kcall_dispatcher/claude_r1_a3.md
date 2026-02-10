# Review: kcall_dispatcher (claude-opus-4.6) — Round 3

## Grade: A

## Previous Issues Assessment (from Round 2)

### Medium #1 (R2): Tautological `spec_do_kcall_result_category` postcondition — FIXED

**Previous:** `do_kcall` postcondition #3 used `spec_do_kcall_result_category(args@) =~= spec_classify_kcall(args.number)`, which was definitionally `spec_classify_kcall(x) == spec_classify_kcall(x)`.

**Updated (exec:487-489):** The tautological postcondition has been replaced with two substantive ones:
1. `spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal ==> !result.is_success` — Exit/ExitThread always produce error results.
2. `spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@)` — a per-category constraint function.

**Verification of fix:** I confirmed `spec_dispatch_result_constrained` (spec:502-511) is non-trivially defined: it requires `result.is_success` for `LocalImmediate` and `!result.is_success` for `LocalTerminal`, and is permissive (true) for other categories. This is genuinely constraining for the two categories that have deterministic success/failure semantics.

**Cross-check against original source:** The original `do_kcall` (dispatcher.rs:69-73, 81-85) calls `ProcessManager::exit().unwrap_err()` and `ProcessManager::exit_thread().unwrap_err()` for Exit/ExitThread, confirming they always produce `KcallResult::Error`. The postcondition `!result.is_success` correctly captures this.

**Verdict:** Fully fixed. The tautology is eliminated and replaced with a meaningful per-category constraint.

### Medium #2 (R2): `do_kcall` signature mismatch with original ABI — FIXED (documented)

**Previous:** The representation gap between `DispatchArgs/DispatchResult` and the C ABI `(u32×5) → i64` was undocumented.

**Updated (exec:72-78):** New trust boundary T5 explicitly documents the ABI representation gap, explaining that `DispatchResult.is_success` is a modeling abstraction not present in the raw i64, and that the caller interprets the result based on kcall semantics. The `do_kcall` doc comment (exec:472-479) also describes this gap.

**Verdict:** Adequately addressed through documentation. The representation gap is inherent to the modeling approach and cannot be eliminated without changing the verification model fundamentally.

### Medium #3 (R2): Trivial `DispatchArgs::wf()` — FIXED (documented)

**Previous:** `DispatchArgs::wf()` was trivially `true` without explanation.

**Updated (spec:420-428):** The `wf()` predicate now has a doc comment explaining the design intent: "Intentionally `true` for all u32 values: the dispatcher's wildcard match arm handles any kcall number (including Invalid/undefined) by dispatching to the scoreboard. No u32 value is rejected at the argument level."

**Verdict:** Adequately addressed. The trivially-true `wf()` is justified because the dispatcher does indeed accept all u32 values — the wildcard arm routes unknown numbers to the scoreboard. Adding a constraint would be over-specification.

### Low #1 (R2): Dead Killed branch in handle_sleep_error — UNCHANGED (accepted)

The `DispatchResult::error(-1i32)` on line 410 of the Killed branch remains. This is dead code due to the precondition and is required for Rust's exhaustive matching. The comment "Unreachable: excluded by precondition" adequately documents this.

**Verdict:** Accepted as-is. No change needed.

### Low #2 (R2): Trivial encoding lemmas — IMPROVED

**Previous:** Three lemmas (`lemma_encode_preserves_value`, `lemma_error_encoding_fits_i32`, `lemma_encode_injective_on_value`) were all tautological.

**Updated (proof:405-439):** The prover removed `lemma_encode_preserves_value` (the most trivially tautological), kept the genuinely useful `lemma_error_encoding_fits_i32`, and replaced `lemma_encode_injective_on_value` with `lemma_large_success_not_error` (proof:429-439). The new lemma proves that success values outside i32 range cannot coincide with any well-formed error encoding — this is a non-trivial property about the encoding's distinguishability. Renamed `spec_could_be_error` to `spec_in_error_range` (spec:490-492) for clarity.

**Verification of `lemma_large_success_not_error`:** The lemma states: if `r.is_success` and `!spec_in_error_range(r.value)`, then for all well-formed error results `e`, `spec_encode_result(e) != spec_encode_result(r)`. This works because `spec_result_wf(e) && !e.is_success` implies `e.value` is in i32 range, while `r.value` is outside i32 range, so they cannot be equal. This is a genuine (if simple) property about the encoding.

**Verdict:** Improved. The encoding lemmas now include at least one non-trivially useful property.

## New Issues Introduced

### Low

- **Location:** `do_kcall()` postcondition redundancy (exec: dispatcher.rs:485-489)
  **Description:** The `do_kcall` postcondition has three constraints: (1) `LocalImmediate ==> result.is_success` (line 485), (2) `LocalTerminal ==> !result.is_success` (line 487), and (3) `spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@)` (line 489). Postconditions #1 and #2 are fully subsumed by postcondition #3, since `spec_dispatch_result_constrained` returns `result.is_success` for `LocalImmediate` and `!result.is_success` for `LocalTerminal`. The explicit conditions are redundant with the general constraint.
  **Suggested Fix:** No code change strictly needed — the redundancy improves readability by making the two most important guarantees explicit. Could add a comment noting the redundancy is intentional for clarity.

- **Location:** `lemma_dispatch_constraint_other` (proof: dispatcher.proof.rs:527-538)
  **Description:** This lemma proves `spec_dispatch_result_constrained` is trivially true for `LocalSleepable`, `LocalFallible`, `LocalDirect`, and `Remote`. While sound, it documents the limitation that the constraint provides no guarantees for 4 of 6 categories. For these categories, the `external_body` postcondition degenerates to just `result.wf()`.
  **Suggested Fix:** Informational only. Strengthening the constraint for other categories (e.g., `LocalSleepable` results are either success or error with a valid error code) would require modeling the subsystem behavior, which is outside this module's scope.

## Positive Observations

- **Meaningful category-specific postconditions on `do_kcall`.** The external_body now has three non-trivial guarantees: well-formedness, immediate success, and terminal failure. These correctly capture the deterministic semantics of `GetPid/GetTid` (always succeed) and `Exit/ExitThread` (always return error since the process terminates).
- **`spec_dispatch_result_constrained` is well-designed.** It cleanly separates the per-category behavioral contract from the dispatch classification, allowing the constraint to be extended for additional categories in the future without changing `do_kcall`'s postcondition.
- **New proof lemmas are non-trivial.** `lemma_terminal_is_exit_exitthread` (proof:496-500) proves the biconditional that LocalTerminal classifies exactly Exit and ExitThread. `lemma_large_success_not_error` (proof:429-439) proves encoding distinguishability for large success values. These are genuine properties.
- **Trust boundary documentation is thorough.** T5 (exec:74-78) explicitly documents the ABI representation gap, and the `do_kcall` doc comment (exec:472-479) explains the trust assumption about result interpretation. This is important for auditors evaluating the verification's assurance level.
- **Clean verification: 51 conditions, 0 errors.** Up from 42 (R1) → 49 (R2) → 51 (R3), with each increase representing genuine new verified properties.
- **No `assume` statements anywhere.** Zero assumes in spec, proof, and exec files. The only trust assumptions are the two justified `external_body` annotations.
- **Spec/proof/exec separation remains clean.** No spec or proof logic has leaked into the exec file beyond what's needed for contracts.

## Summary

The verification has reached a mature and well-balanced state. All issues from Round 1 and Round 2 have been either substantively fixed or adequately justified through documentation. The key improvements across three rounds are:

1. **R1→R2:** Divergent Killed path properly separated; `do_kcall` gained initial postconditions; error handling uses typed constructors; encoding model added.
2. **R2→R3:** Tautological postcondition replaced with category-specific behavioral constraints; ABI gap documented as trust boundary T5; trivial `DispatchArgs::wf()` justified; encoding lemmas improved.

The remaining limitations are inherent to the verification approach:
- `do_kcall` is necessarily `external_body` due to unsafe global state, limiting end-to-end verification.
- The ABI representation gap between `DispatchResult` and raw `i64` is a modeling choice that trades precision for expressiveness.
- Per-category constraints are only meaningful for `LocalImmediate` and `LocalTerminal`; other categories have no structural guarantees beyond well-formedness.

These are acceptable trade-offs for an OS kernel dispatcher that interacts with multiple subsystems through unsafe FFI. The verification successfully captures the essential correctness properties that *can* be verified at the dispatcher level: correct routing of every kernel call number, correct error handling for non-divergent sleep errors, and deterministic success/failure for the two categories with deterministic semantics.
