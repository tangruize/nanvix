# Review: interrupted Exec Consistency (claude-opus-4.6)

## Grade: A

## Function Coverage

| Original Function | Exec Function | Status | Notes |
|---|---|---|---|
| `from_state(Box<ThreadState>, InterruptReason) -> Self` | `from_state(ThreadState, int) -> InterruptedThread` | ✅ Equivalent | Type abstractions documented (Box→direct, enum→int). |
| `id(&self) -> ThreadIdentifier` | `id(&self) -> ThreadIdentifier` | ✅ Equivalent | Body identical: `self.state.id()`. Ghost annotations only. |
| `thread_state(&self) -> &ThreadState` | `thread_state(&self) -> &ThreadState` | ✅ Equivalent | Body identical: `&self.state`. Ghost annotations only. |
| `thread_state_mut(&mut self) -> &mut ThreadState` | `thread_state_mut(&mut self) -> &mut ThreadState` | ✅ `#[verifier::external]` | Correct: Verus cannot express `&mut T` returns. Trust boundary documented. |
| `resume(mut self) -> ReadyThread` | `resume(self) -> ReadyThread` | ✅ Equivalent | Decomposition of `self` fields required by Verus (no `mut self` with partial field access). Semantically identical sequence. |
| `join_cond(&self) -> Condvar` | `join_cond(&self) -> Condvar` | ✅ `#[verifier::external]` | Stub `Condvar`. Read-only accessor, consistent with other thread modules. |

## Issues Found

### Critical

- None.

### Minor

1. **`join_cond` returns a dummy `Condvar` stub, not `self.state.join_cond()`.**
   The original delegates to `self.state.join_cond()`, but the Verus exec stub returns `Condvar` (a unit struct). This is intentional — the `Condvar` field is elided from the Verus `ThreadState` model entirely — and the function is `#[verifier::external]`, so it has no verification effect. However, any attempt to actually *call* this Verus version at runtime would produce incorrect behavior. This is acceptable only because the Verus split files are not compiled for execution. The trust boundary documentation (lines 263–288) clearly explains this. **No action needed.**

2. **`from_state` visibility change: `pub(super)` → `pub`.**
   The original uses `pub(super)` for `from_state`, restricting it to the parent module. The Verus version uses `pub` because Verus proof/spec code in other modules needs access. This is a standard Verus modeling concession and does not affect verification soundness. Documented in the struct-level comment (lines 64–69). **No action needed.**

3. **`ReadyThread` boundary model omits `admission_time` field.**
   The real `ReadyThread::from_state` sets `admission_time = clock::now()`. The boundary model omits this. This is explicitly documented (lines 86–91, spec lines 41–45) as a scheduling property outside scope. The omission is sound — `admission_time` is not a safety, identity, or well-formedness property. **No action needed.**

### Observations

- No `assume`, `admit`, or `external_body` annotations found — clean verification.
- Only two `#[verifier::external]` uses, both with thorough trust boundary documentation.
- Proof file includes 14 lemmas covering construction, identity preservation, resume correctness, mutex accounting, drop safety, stack ownership, reason variant distinctness, and view equality — comprehensive coverage.
- The `resume()` postcondition `result@.spec_interrupt_reason() == Some(self@.spec_reason())` directly proves the key safety property: the interrupt reason is correctly propagated to the ReadyThread's state.

## Consistency Verdict

| Criterion | Pass/Fail |
|---|---|
| All MISMATCH functions restored or equivalence documented | ✅ Pass |
| All MISSING functions added with proper verification | ✅ Pass |
| Equivalence justifications sound | ✅ Pass |
| Exec code faithfully represents original source | ✅ Pass |
| Verification passes (21 verified, 0 errors) | ✅ Pass |

## Summary

The exec consistency fix for `interrupted.rs` is thorough and well-executed. All six original functions are accounted for: four are verified within `verus!` blocks with correct exec bodies matching the original logic, and two (`thread_state_mut`, `join_cond`) are correctly placed outside verification with `#[verifier::external]` and detailed trust boundary documentation. The type abstractions (`Box<ThreadState>` → `ThreadState`, `InterruptReason` enum → `int` tag) are standard Verus modeling patterns, properly constrained by the `wf()` invariant and `spec_valid_reason` predicate. The `resume()` decomposition of `self` into separate field bindings is a necessary Verus workaround that preserves semantic equivalence. The `ReadyThread` boundary model is minimal but sufficient to verify the state transition. Verification is clean: 21 items verified with no errors, no `assume`/`admit`, and no `external_body`.
