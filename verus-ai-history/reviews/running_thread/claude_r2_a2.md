# Review: running_thread (claude-opus-4.6) — Round 2

## Grade: A-

## Verification Result

- **All obligations discharge successfully** (no errors reported).
- Zero `assume` / `admit` statements. Zero `#[verifier::external_body]` annotations.
- Single `#[verifier::external]` on `thread_state_mut`, justified by Verus `&mut T` return limitation.

## Disposition of Previous Review Issues

### Issue 1 (was High): `take_mutex_guard` precondition strengthening

**Previous concern:** Original returns `Option<MutexGuard>`; verified model requires `spec_has_mutex(address@)` as a precondition, eliminating the `None` path. Suggested a bridging lemma or `Option<()>` return.

**Current state:** Still uses precondition strengthening. **No bridging lemma added.** However, documentation was significantly improved (lines 425–430) explicitly naming this as "trust assumption T2: callers release only what they hold."

**Verdict: PARTIALLY ADDRESSED (documentation only).** The structural suggestion (bridging lemma) was not implemented, but the documentation honestly frames this as a trust assumption rather than a proven consequence. This is acceptable because:
1. The underlying `BTreeMap::remove` is outside the verification boundary, so a bridging lemma would require either modeling BTreeMap in Verus or wrapping it in `external_body` — both of which just push the trust boundary elsewhere.
2. The protocol invariant (callers only release held mutexes) is a legitimate design contract.
3. The documentation is clear and actionable for future integrators.

**Remaining risk:** Low. Callers outside the verification boundary who violate this precondition will not be caught by the model.

### Issue 2 (was Medium): `thread_state_mut` external escape hatch

**Previous concern:** `#[verifier::external]` allows unverified mutation of `ThreadState`, bypassing `wf()` and `spec_id()` invariants.

**Current state:** Still `#[verifier::external]` (unavoidable). Documentation expanded (lines 466–481) with:
- Explicit trust obligations: preserve `wf()` and `spec_id()`.
- Recommendation to prefer verified forwarding methods (`put_mutex_guard`/`take_mutex_guard`).
- Intended postconditions documented (not machine-checked).

**Verdict: ADDRESSED (within Verus limitations).** The prover correctly identifies that this is a language limitation and provides the best available mitigation: clear trust obligations and forwarding-method recommendations.

### Issue 3 (was Medium): `sleep`/`schedule`/`exit` omit `*mut ContextInformation`

**Previous concern:** The pairing of context pointer to thread identity is a safety-relevant property not captured.

**Current state:** Modeling Notes added on each function (lines 279–282, 305–307, 360–361) documenting the omission. No ghost context token was added.

**Verdict: PARTIALLY ADDRESSED (documentation only).** The suggestion to add a ghost return value was not implemented. However, the context pointer is a HAL-boundary raw pointer that Verus cannot model, so the documentation-only approach is pragmatically acceptable. The omission is clearly marked and does not affect the state-transition properties being verified.

### Issue 4 (was Medium): Boundary model divergence risk

**Previous concern:** Boundary models for `SleepingThread`, `ReadyThread`, `ZombieThread` could silently diverge from real sibling module specs.

**Current state:** `CROSS-MODULE-CHECK` annotations now appear on all three boundary types (lines 85–87, 174–181, 215–224) with **explicit postcondition lists** that must be confirmed when sibling modules are verified.

**Verdict: ADDRESSED.** The explicit postcondition checklists are a significant improvement. While still not automated, they provide a concrete, actionable verification obligation that a future reviewer or CI check can enforce.

### Issue 5 (was Low): `put_mutex_guard` `usize::MAX` precondition

**Previous concern:** Modeling artifact not present in original code.

**Current state:** Documented in Modeling Note (lines 396–400) as a modeling limitation.

**Verdict: FULLY ADDRESSED** as suggested.

### Issue 6 (was Low): `join_cond` omission

**Previous concern:** Condvar identity not modeled.

**Current state:** Still omitted, documented in trust boundary section (line 42).

**Verdict: ACKNOWLEDGED.** Appropriate for current scope.

### Issue 7 (was Low): `from_state` visibility (`pub` vs `pub(super)`)

**Previous state:** Documented as Verus ergonomic requirement.

**Current state:** Unchanged. Comment at line 74 notes construction should only occur via `from_state()`.

**Verdict: ACKNOWLEDGED.** No action needed.

## New Observations (This Round)

### Positive Changes

- **Additional spec functions:** `spec_interrupt_reason`, `spec_user_tda`, `spec_kernel_stack`, `spec_user_stack` (spec lines 99–117) were added, providing broader state visibility for downstream consumers.
- **`from_state` postcondition expanded:** Now includes `spec_is_interrupted` (exec line 260), improving state transparency at construction.
- **`exit()` design note** (exec lines 363–367): Explicitly documents that `spec_drop_safe()` is NOT a precondition, faithfully mirroring the original code's permissive behavior (Drop only logs an error). This is a thoughtful design decision that avoids precondition strengthening.
- **Module-level documentation** (exec lines 1–49): Comprehensive verification model and trust boundary sections provide excellent context.

### New Issues Found

#### Low

- **Location:** `take_mutex_guard` postconditions (exec, lines 439–445)
  **Description:** `take_mutex_guard` does not explicitly postcondition `spec_drop_safe()`. After removing the last mutex, the thread becomes drop-safe, but this property must be derived by the caller through `spec_locked_mutex_count() == 0 && wf()` rather than being stated directly. By contrast, `put_mutex_guard` explicitly ensures `!self.spec_drop_safe()` (line 411). The asymmetry means callers proving drop-safety after unlock need an extra reasoning step.
  **Suggested Fix:** Add `self.spec_locked_mutex_count() == 0 ==> self.spec_drop_safe()` as a postcondition, or add a standalone lemma `lemma_empty_mutex_set_is_drop_safe`. This makes the drop-safety inference explicit and symmetric with `put_mutex_guard`.

- **Location:** `lemma_acquire_then_release_restores_mutex_state` (proof, lines 380–412)
  **Description:** The lemma constructs intermediate states via direct struct literals (`RunningThread { state: ThreadState { ... } }`) rather than referencing the exec functions' postconditions. This is a proof-level pattern (exec functions can't be called in proof mode), so it's structurally correct. However, the connection between the struct literals and the actual exec behavior relies on the reader understanding that the exec postconditions guarantee the same field updates. A brief comment noting this would improve readability.
  **Suggested Fix:** Add a one-line comment: `// Struct literals mirror put_mutex_guard/take_mutex_guard postconditions.`

## Summary of Remaining Issues

| # | Severity | Issue | Status |
|---|----------|-------|--------|
| 1 | Low | `take_mutex_guard` precondition strengthening (no bridging lemma) | Documented as trust assumption T2. Acceptable. |
| 2 | Low | `thread_state_mut` `#[verifier::external]` escape hatch | Verus limitation. Well-documented. |
| 3 | Low | Context pointer omission in `sleep`/`schedule`/`exit` | HAL boundary. Documented. |
| 4 | Info | Boundary model divergence (manual cross-module check needed) | CROSS-MODULE-CHECK annotations added. |
| 5 | Low | `take_mutex_guard` missing explicit `drop_safe` postcondition | New. Derivable but asymmetric with `put_mutex_guard`. |

## Overall Assessment

The prover addressed the previous review primarily through documentation improvements rather than structural changes. This is a reasonable approach: the previous High issue (take_mutex_guard strengthening) and Medium issues (thread_state_mut, context pointer, boundary models) all stem from inherent Verus limitations or deliberate modeling scope decisions, not from bugs or unsound reasoning. The prover's documentation is honest about what is trusted vs. proven, and the CROSS-MODULE-CHECK annotations provide clear integration obligations.

The core verification remains sound and comprehensive: 8 of 10 original methods have machine-checked contracts, identity/wf/mutex/drop-safety are preserved across all state transitions, and the proof library provides 30+ lemmas with good compositional coverage. The single `#[verifier::external]` is justified and the single trust assumption (T2) is clearly documented.

The grade remains A- because the structural gaps (precondition strengthening without bridging lemma, missing context token, asymmetric drop_safe postcondition) persist, but all are Low severity and well within acceptable bounds for a verification of this scope and complexity.
