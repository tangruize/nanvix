# Review: spinlock (claude-opus-4.6) — Round 2, Assessment 2

## Grade: A

## Verification Status

20 verified, 0 errors. No `assume` statements. One `external_body` (on `lock()`, justified).
No cheating patterns detected.

## Previous Issues — Disposition

### M1: `wf()` invariant is one-directional; missing reachability invariant

**Status: FIXED — Verified.**

The `wf()` predicate was strengthened from the one-directional implication
`!self.locked ==> !self.token_issued()` to the biconditional
`self.locked == self.token_issued()` (spec.rs:82-84). This is exactly the fix
requested.

**Verification of correctness:**
- `new()`: `locked=false, token_issued=false` → `false == false` → `true` ✓
- `try_lock()` success: `locked=true, token_issued=true` → `true == true` → `true` ✓
- `try_lock()` failure: state unchanged, preconditions (`wf() && !token_issued`) imply
  `!locked` via biconditional, making the failure path provably unreachable by
  contradiction. Postconditions hold vacuously. ✓
- `lock()`: postconditions include `self.locked`, `self@.token_issued`, `self.wf()` → sound ✓
- `unlock()`: `locked=false, token_issued=false` → `false == false` → `true` ✓

The spec-level documentation for `wf()` (spec.rs:71-81) was also updated to accurately
describe the biconditional semantics, including the state machine transitions and what it
prevents. This is thorough.

### L1: `LockToken` forgeable in proof mode

**Status: Acknowledged — No change needed.** Token still has `pub ghost view`
(spec.rs:52). The previous review correctly identified this as following the established
`KernelRedZoneGhost` pattern and requiring no action. The prover's non-fix is justified.

### L2: Proof lemmas remain trivially auto-discharged

**Status: Acknowledged — No change needed.** All 16 proof lemma bodies remain empty
and are auto-discharged by Verus. The previous review correctly identified this as
inherent to a single-boolean state machine. One new lemma was added
(`lemma_new_is_wf`, proof.rs:198-204). The prover's non-fix is justified.

### L3: Token affinity (not linearity) allows token leaks

**Status: Acknowledged — No change needed.** Inherent Verus limitation, properly
documented in the module header (spinlock.rs:57-64). The prover's non-fix is justified.

### L4: `try_lock()` failure path unreachable from well-constructed spinlocks

**Status: FIXED — Verified.**

Documentation was added to `try_lock()` (spinlock.rs:153-156):
```
/// With the strengthened `wf()` biconditional, `wf() && !token_issued` implies
/// `!locked`, so `try_lock()` always succeeds on well-formed spinlocks. The
/// failure path exists for completeness but is unreachable from `new()`-constructed
/// spinlocks.
```

This accurately explains the situation. With the M1 fix making `wf()` a biconditional,
the analysis is now explicit: `wf() ∧ ¬token_issued → ¬locked → success path always
taken`. The documentation update is precisely what was suggested.

## New Issues Found

### Low

- **N1: Module header describes old one-directional `wf()` invariant**
  - **Location:** exec (`spinlock.rs:18`)
  - **Description:** The module-level header comment says:
    ```
    //! - Well-formedness (`wf()`) enforces: unlocked implies no token outstanding.
    ```
    This describes only one direction (`¬locked → ¬token_issued`) of the now-biconditional
    `wf()` predicate (`locked ↔ token_issued`). The missing direction is "locked implies
    token outstanding" (`locked → token_issued`). The spec-level documentation (spec.rs:71-81)
    correctly describes the biconditional, so this is a stale header comment, not a spec
    error.
  - **Suggested Fix:** Update line 18 to:
    ```
    //! - Well-formedness (`wf()`) enforces: locked if and only if token outstanding.
    ```

## Positive Observations

- **M1 fix is clean and correct.** The biconditional change is minimal (one line in
  spec.rs) with cascading benefits: it tightens the invariant, makes `try_lock()`'s
  success provably guaranteed on well-formed spinlocks, and the documentation updates
  in both spec.rs and the `try_lock()` docstring are thorough.

- **Ghost instance identity continues to provide strong token isolation.**
  `lemma_token_instance_isolation` (proof.rs:173-184) formalizes that tokens from
  different lock instances cannot satisfy each other's unlock preconditions. This
  addresses the primary architectural concern from the r1_a3 review.

- **New `lemma_new_is_wf` lemma is a reasonable addition.** (proof.rs:198-204) It
  documents that `spec_new_view` produces a view with `!locked && !token_issued`, which
  together with the biconditional `wf()` confirms new spinlocks are well-formed. While
  trivially auto-discharged, it serves as regression documentation for the strengthened
  invariant.

- **All positive observations from the previous review remain valid:** excellent module
  documentation, clean spec/proof/exec separation, justified `external_body` on `lock()`,
  protocol round-trip proof, honest trust assumptions, and meaningful machine-checked
  token-based enforcement.

## Summary

The prover correctly addressed both actionable issues from the previous review:

1. **M1 (Medium)**: `wf()` was strengthened to a biconditional — properly implemented,
   all transitions verified, documentation updated in spec.rs and try_lock() docstring.
2. **L4 (Low)**: try_lock() failure path unreachability was documented with clear
   explanation of why the strengthened wf() makes the success path provably guaranteed.

The remaining low-priority issues (L1, L2, L3) were correctly identified as inherent
limitations requiring no code changes, and the prover's rejections are justified.

One new minor issue was found: the module-level header comment (spinlock.rs:18) still
describes the old one-directional `wf()` rather than the new biconditional. This is a
documentation-only issue with no impact on verification soundness.

The grade is upgraded from A- to A. The primary gap (weak `wf()`) is resolved, the
documentation is comprehensive (with one minor stale line), and the verification model
is sound. The single remaining issue is a trivial header update.
