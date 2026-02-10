# Review: kcall_scoreboard (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### High #1: All struct fields `pub` — **RESOLVED (Accepted)**
The prover correctly documented (lines 55-60 of exec) that `pub` fields are standard practice in Verus verification models due to the `verus!{}` macro requiring field access for spec functions and View trait implementations. I verified this claim: every other verified module in the project (Mutex, Semaphore, Spinlock, Condvar, Fence) uses `pub` fields on its verification model structs. The prover's reasoning is sound — these are verification model types, not production API types. The original production structs maintain proper encapsulation.

### High #2: `handle()` `&self → &mut self` missing trust boundary note — **RESOLVED (Fixed)**
The prover added an explicit note in T4 (lines 136-141 of exec): "The original `handle()` takes `&self` and uses atomic `try_down()`... The verified model uses `&mut self`, which means the mutual exclusion between `handle()` and `dispatch()` is *assumed* (via the sequential model), not *proven*." This directly addresses the concern and is clearly stated.

### Medium #1: `abandon_dispatch()` precondition too narrow — **RESOLVED (Fixed)**
The spec `spec_abandon_dispatch` now preserves `view.phase` instead of hardcoding `ScoreBoardPhase::Handled`. The exec precondition changed from `old(self).spec_is_handled()` to `!old(self).spec_is_idle()`, accepting Signaled, Dispatched, or Handled phases. The proof lemmas (`lemma_abandon_dispatch_not_wf`, `lemma_abandon_dispatch_preserves_data`) were correspondingly generalized. The T5 documentation was updated. This is a thorough fix.

### Medium #2: Missing `get_board_mut()` — **RESOLVED (Accepted with minor issue)**
The API mapping table was updated to note that `&mut` return is unsupported by Verus and callers should use slot fields directly. This is reasonable given `pub` fields. However, T1 (line 107) still references "`get_board()` and `get_board_mut()`" — `get_board_mut()` does not exist. See Low #1 below.

### Medium #3: Tautological `sb@ == sb@` ensures — **PARTIALLY RESOLVED**
- `lemma_try_handle_idle_is_noop`: **Fixed properly.** New ensures derive concrete field values (`!sb.locked`, `sb.dispatched_value == 0`, `sb.handled_value == 0`) from the `wf()` + idle preconditions. These are meaningful derived properties.
- `lemma_try_handle_fail_preserves_wf`: **Still tautological.** Renamed but the ensures `sb.phase == sb.phase`, `sb.locked == sb.locked`, etc. are self-comparisons on an immutable `&ScoreBoard` — always trivially true. The only meaningful ensure is `sb.wf()`, which is also trivially preserved since no mutation occurs on `&ScoreBoard`. See Medium #1 below.

### Medium #4: `KcallResult` Copy semantics not documented — **RESOLVED (Fixed)**
Lines 221-224 of exec now document: "The original `KcallResult` derives `Copy`, so `Ok(self.ret)` in `dispatch()` returns a copy without moving. This flat struct model inherently has copy semantics in Verus, matching the original behavior."

### Low #1: `complete_dispatch()` returns Ghost — **RESOLVED (Fixed)**
Now returns concrete `KcallResult` with `result.wf()` in ensures. The construction `KcallResult { is_success: self.result.is_success, value: self.result.value }` correctly copies the fields before state mutation. Ensures include `result@ == old(self).result@`.

### Low #2-4: Module-level init, Debug, spec_n_identical_cycles — **Unchanged (Acceptable)**
These were informational notes, not actionable fixes.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- **Location:** `lemma_try_handle_fail_preserves_wf` (proof: `scoreboard.proof.rs:570-581`)
  - **Description:** The ensures clauses `sb.phase == sb.phase`, `sb.locked == sb.locked`, `sb.dispatched_value == sb.dispatched_value`, `sb.handled_value == sb.handled_value` are self-referential tautologies — trivially true for any value. Since the lemma takes `&ScoreBoard` (immutable reference), no mutation is possible, making even `sb.wf()` trivially preserved from the precondition. The rename from "preserves_state" to "preserves_wf" is more honest, but the field-level ensures remain meaningless. Compare with `lemma_try_handle_idle_is_noop` which correctly derives concrete values from wf() + idle.
  - **Suggested Fix:** Either (a) derive concrete values like the idle lemma does (e.g., for the Dispatched case: `sb.locked`, `sb.dispatched_value == 0`, `sb.handled_value == 0`), (b) remove the tautological field ensures and keep only `sb.wf()` with a comment noting it's trivial for `&` references, or (c) remove the lemma entirely since `try_handle()`'s ensures already proves state preservation on the failure path (`!success ==> self@ == old(self)@`).

### Low

- **Location:** T1 trust boundary documentation (exec: `scoreboard.rs:107`)
  - **Description:** Line 107 references "`get_board()` and `get_board_mut()`" but `get_board_mut()` does not exist as a method. This is stale documentation from before the API table was updated.
  - **Suggested Fix:** Change to "`get_board()`" or "`get_board()` (and direct field access for mutation)".

## Positive Observations

- **Thorough fix of `abandon_dispatch()`:** The generalization from Handled-only to any-active-phase is well-executed. The spec, exec, and proof files were all consistently updated. The documentation in T5 was also updated. This is the most impactful improvement.
- **Concrete `KcallResult` return from `complete_dispatch()`:** Now faithfully mirrors the original `Ok(self.ret)` return, with `result.wf()` ensuring the copy is well-formed.
- **Honest trust boundary documentation for `handle(&self)`:** The explicit note in T4 that the `&self → &mut self` change means mutual exclusion is assumed, not proven, is exactly what was needed. This is important for downstream consumers of the verification.
- **Field visibility justification is well-researched:** Citing the existing convention across mutex, semaphore, slab, etc. is convincing and verifiable.
- **`lemma_try_handle_idle_is_noop` properly fixed:** The new ensures clauses derive concrete field values from the invariant, making the lemma actually useful.
- **62 verification conditions pass** with no `assume`, `external_body`, or `trusted` annotations in the code.
- **All previous positives still hold:** Excellent module-level documentation, sound state machine model, comprehensive error path modeling, clean spec/proof/exec separation, ghost state for induction, semaphore infallibility proofs.

## Summary

The prover addressed 7 of 8 actionable issues from R1. The `abandon_dispatch()` generalization, concrete `complete_dispatch()` return, and `handle(&self)` trust boundary documentation are all well-executed fixes. The `pub` fields justification is verified against the project-wide convention. One proof lemma (`lemma_try_handle_fail_preserves_wf`) still has tautological ensures, and there's a stale `get_board_mut()` reference in T1 documentation. Neither affects soundness. The verification is mature and provides strong sequential correctness guarantees for the scoreboard dispatch protocol.
