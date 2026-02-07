# Review: mutex — Round 3 Attempt 3 (claude-opus-4.6)

## Grade: A

## Previous Review Disposition (r3_a2)

### NEW-M1: Refinement Argument incorrectly claims `Release` ordering for unlock

**Status: Fixed.**

The r3_a2 review identified that the Refinement Argument (point 2) incorrectly stated `store(false, Release)` when the original implementation uses `Ordering::Relaxed` (line 79 of `src/kernel/src/pm/sync/mutex.rs`). The prover corrected this exactly as suggested:

1. **Point 2:** Changed `store(false, Release)` → `store(false, Relaxed)` (line 128). **Verified** against the original: `self.locked.store(false, Ordering::Relaxed)` at line 79 of the original. The documentation now accurately reflects the actual code.

2. **Point 3:** Replaced "guaranteed by x86 TSO and the `Acquire`/`Release` ordering" with "guaranteed by x86 TSO, which upgrades all stores to effective release semantics" and added the architecture-specificity caveat: "the original uses `Relaxed` ordering for the unlock store, so this argument is architecture-specific to x86 TSO and does not hold on weakly-ordered architectures (e.g., ARM, RISC-V)" (lines 131–136). **Verified:** This is technically accurate — x86 TSO provides store-store and load-load ordering, making all stores effectively release stores. The caveat about ARM/RISC-V is correct: `Relaxed` stores on those architectures may be reordered, breaking the linearizability argument.

No other files were modified. The spec and proof files are unchanged from the previous round.

## New Issues Introduced

(none)

The change is purely a documentation correction within the existing Refinement Argument section. No spec, proof, or exec code was modified.

## Remaining Architectural Limitations (Non-actionable)

These are inherent to the sequential verification approach and are well-documented. They have been stable across all review rounds and are not fixable within the current verification model:

1. **Sequential model cannot verify concurrent contention.** The `lock()` precondition models instant success. Documented in Verification Scope (lines 42–54).
2. **Token construction is a trust assumption.** `MutexToken`'s `pub ghost view` is a Verus language constraint. Documented as Trust Assumption T3 (lines 110–117).
3. **`reference_count()` not modeled.** Arc-specific diagnostic, not part of lock protocol. Documented in API mapping (line 64).

## Verification Soundness Assessment

- **Verification conditions:** 27 verified, 0 errors.
- **Escape hatches:** None. No `assume`, `external_body`, `trusted`, or `admit` in any of the three files.
- **Spec/exec separation:** Clean three-file split (`mutex.rs`, `mutex.spec.rs`, `mutex.proof.rs`) via `include!()`.
- **Well-formedness preservation:** `wf()` biconditional (`locked == token_issued`) is established by `new()` and preserved by `try_lock()`, `lock()`, and `unlock()` — confirmed in all postconditions.
- **Token-ownership protocol:** Tokens created only on successful lock acquisition, bound to mutex instance via view identity (`token.view == self@`), consumed by `unlock()`. Double-unlock is precondition-blocked (`lemma_no_double_unlock`). Cross-instance token use is precondition-blocked (`lemma_token_instance_isolation`).
- **Proof coverage:** 27 lemmas: 12 definitional regression tests + 9 protocol properties (round-trip, isolation, mutual exclusion, contention resolution, relockability, no-double-unlock, etc.).
- **Documentation accuracy:** The Refinement Argument now correctly describes the original code's memory orderings (`Acquire`/`Relaxed` for `compare_exchange`, `Relaxed` for `store`) and explicitly scopes the linearizability argument to x86 TSO.

## Positive Observations

- **Precise fix.** The prover corrected the factual error exactly as suggested, with no extraneous changes. The three-line diff is surgical and correct.
- **Zero escape hatches** across all three files throughout all review rounds — this is the strongest quality signal.
- **Documentation is exemplary.** The 142-line module header provides: Verified Properties, Verification Model, Verification Scope, API Mapping, API Divergence, Trust Boundaries, Trust Assumptions (T1–T3), and Refinement Argument. Each section is accurate and internally consistent.
- **Honest about limitations.** The Verification Scope explicitly lists six out-of-scope categories. The Refinement Argument now honestly notes the architecture-specific nature of the linearizability claim.
- **Meaningful protocol proofs.** The proof file goes beyond definitional unfolding to prove round-trip restoration, instance isolation, mutual exclusion, contention resolution, and relockability.
- **Responsive across all rounds.** Over three review rounds, every actionable issue was addressed precisely and correctly. Non-actionable items were appropriately identified and justified.

## Summary

The prover fixed the sole remaining issue from r3_a2: the Refinement Argument's memory ordering now correctly states `Relaxed` (matching the original `unlock_unchecked()`) and includes an explicit x86 TSO architecture-specificity caveat. The fix is accurate and minimal.

No issues remain. The mutex verification is complete and sound within its stated scope: it proves sequential state machine correctness of the lock/unlock protocol with 27/0 verification, zero escape hatches, comprehensive proof coverage, and production-grade documentation that accurately describes both the verified properties and the verification's inherent limitations.

**Recommendation:** Accept. No further changes needed.
