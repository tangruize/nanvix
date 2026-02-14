# Review: semaphore Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`view()` is `open spec fn` instead of `pub closed spec fn` (Step 1 deviation).** The `View` trait implementation at `semaphore.spec.rs:344` uses `open spec fn view()`. The methodology document (Step 1) recommends `pub closed spec fn` to hide implementation internals. The code includes a justification comment (lines 340–343) explaining that the Verus `View` trait requires `open` — this is a genuine Verus tooling constraint, not an oversight. However, since `view()` is open, the concrete field `self.value` is exposed through the view body, which weakens abstraction. **Mitigated** by `wf()` being `pub closed`, preventing clients from depending on internal field layout without revealing the invariant.

- **`Semaphore.value` field is `pub` (Step 1 / coding standards deviation).** The struct at `semaphore.rs:193-196` declares `pub value: usize`. The Nanvix coding standards require private fields with getter/setter access. The code includes a justification comment (lines 190–192) attributing this to Verus tooling constraints for `pub open spec fn` access. This is acceptable but means public method specs reference `self.value` directly (a concrete field) in several places — see next issue.

### Medium

- **Public method specs reference `self.value` (concrete field) instead of `self@.value` (view field) in some places (Step 3 deviation).** The methodology (Step 3) states: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." Several public methods mix both:
  - `new()` ensures: `result.value == value` (line 214) — should use `result@.value`.
  - `down_available()` ensures: `self.value == old(self).value - 1` (line 253) — should use `self@.value`.
  - `down_or_block()` ensures: `self.value == old(self).value - 1` (line 296) — should use `self@.value`.
  - `try_down()` ensures: `result == old(self).spec_is_available()`, `result ==> self.value == old(self).value - 1` (lines 330-331) — should use `self@.value`.
  - `up()` ensures: `self.value == old(self).value + 1` (line 379) — should use `self@.value`.
  - `get_value()` ensures: `result == self.value` (line 404) — should use `self@.value`.
  - `is_available()` ensures: `result == (self.value > 0)` (line 425) — should use `self@.value`.

  The view-based equivalents (`self@.value`) are also present alongside the concrete ones, so the specs are "doubly stated." The concrete `self.value` clauses are redundant given the view clauses plus `wf()`, but they leak implementation details into public contracts. This is a methodology deviation, not a correctness issue.

- **`spec_wf()` is `pub open` instead of `pub closed` (Step 2 nuance).** At `semaphore.spec.rs:142`, `spec_wf(view: SemaphoreView)` is declared `pub open spec fn`. The methodology says the invariant should be `pub closed`. However, this is a *static* helper on `SemaphoreView` (not `&self`), used for ghost-state reasoning in proofs. The instance method `wf(&self)` at line 131 is correctly `pub closed spec fn`. This is a minor style deviation — `spec_wf` could be closed or private since it's only used in proofs.

- **Several spec helper functions are `pub open` that could be `pub closed` or private.** Functions like `spec_value()`, `spec_waiters()`, `spec_is_available()`, `spec_is_exhausted()`, `spec_new_view()`, `spec_drop_safe()`, `spec_down_blocking()`, `spec_wake()`, etc. are all `pub open`. The methodology (Step 3) says: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." These are convenience specs on the view layer, which the methodology suggests placing on `MyTypeView` instead (Step 3, last paragraph). Not a correctness issue, but a structural deviation.

### Low

- **`CallerContext` fields are all `pub`.** At `semaphore.spec.rs:40-49`, all fields of `CallerContext` are public. Since this is a ghost-only struct used in spec-level reasoning, this is pragmatically acceptable, but per Nanvix coding standards, fields should be private with getters.

- **`CallerContext::inv()` is trivially `true`.** At `semaphore.spec.rs:64`, the invariant for `CallerContext` is `true`. This is correctly documented as intentional (no internal consistency constraints), but it means the inv/wf pattern adds no value for this type.

- **`SemaphoreView` fields are `pub`.** At `semaphore.spec.rs:23-28`, `value` and `waiters` are public. For a View type this is expected — the methodology says the view exposes observable state — so this is correct.

## Verification

- **Verification passes:** 44 verified, 0 errors. ✅
- **No `assume`, `admit`, or `#[verifier::external_body]`** in any code. ✅

## Methodology Compliance Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| View uses abstract types (`nat`) | ✅ Pass | `SemaphoreView` uses `nat` for both fields. |
| `view()` is `pub closed spec fn` | ⚠️ Justified deviation | Must be `open` due to Verus `View` trait constraint. Documented. |
| `inv()`/`wf()` exists and is `pub closed` | ✅ Pass | `wf()` is `pub closed spec fn` at line 131. |
| Public specs use `self@.field` not `self.field` | ⚠️ Partial | View-based specs present, but concrete `self.value` also appears in public ensures. |
| Public methods require/ensure `wf()` | ✅ Pass | All public methods have `wf()` in requires (for `old(self)`) and ensures. |
| No assume/admit/external_body | ✅ Pass | None found. |
| Verification passes | ✅ Pass | 44 verified, 0 errors. |

## Summary

The semaphore verification is well-structured with thorough spec coverage including blocking protocol reasoning, caller context modeling, error mapping, and round-trip properties. The View type correctly uses abstract types (`nat`), `wf()` is properly `pub closed`, all public methods maintain the invariant, and verification passes cleanly with no assumptions. The main methodology deviation is that public method specifications redundantly include concrete field references (`self.value`) alongside the proper view-based ones (`self@.value`). Removing the concrete field clauses from public ensures would bring the specs into full compliance with the specifying-and-proving-types methodology. The `open` visibility of `view()` is a justified Verus tooling constraint. Overall, this is a high-quality verification with minor style deviations.
