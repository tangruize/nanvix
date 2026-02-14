# Review: kredzone Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical

- None.

### High

- **View type uses concrete `usize` instead of abstract `int`**: `KernelRedZoneView.contents` is typed `Seq<usize>` (kredzone.spec.rs:222). Per the guidelines, View types should use abstract types — `Seq<int>` instead of `Seq<usize>`. Similarly, `index()` returns `usize` and `update()` takes `usize` — these should use `int` in the abstract view. The `spec_store_effect` and `spec_load_result` functions also operate on `usize` values rather than `int`.

- **`KernelRedZoneView` spec functions are `pub open` instead of `pub closed`**: The guidelines (Step 1) require `view()` to be `pub closed spec fn`. While `KernelRedZoneGhost.view()` is correctly `pub closed` (kredzone.spec.rs:103), the `KernelRedZoneView` methods `len()`, `index()`, `update()`, `in_bounds()`, and `is_well_formed()` are all `pub open spec fn` (kredzone.spec.rs:26-58). Per Step 3 of the guidelines, common subexpressions on the View type may be `pub open spec fn`, so these are partially justified — they serve as the "common expressions" pattern described. However, `is_well_formed()` (the invariant for the view itself) leaks implementation details about the length equality and arguably should be `pub closed`.

### Medium

- **`KernelRedZoneView.contents` field is `pub`**: The view struct's `contents` field is declared `pub` (kredzone.spec.rs:222 in kredzone.rs). While View types are less constrained than implementation types, exposing internal fields directly means public method specs could bypass the accessor methods and reference `self.contents` directly, which is contrary to the spirit of abstraction.

- **`KernelRedZoneGhost.view` field is `pub ghost`**: The ghost struct's `view` field is `pub ghost` (kredzone.spec.rs:92). This allows direct field access (e.g., `ghost.view` at kredzone.rs:434, kredzone.proof.rs:144) rather than going through `ghost.view()` (the closed spec fn). Public method specs should use `ghost.view()` not `ghost.view` for consistency with the methodology. The `store_with_ghost` ensures clause at kredzone.rs:433 accesses `ghost.view` directly.

- **Public `store`/`load` functions do not require/ensure `inv()`**: The standalone `store()` and `load()` functions (kredzone.rs:270, 354) have no `inv()` preconditions or postconditions. This is understandable since they are standalone functions operating on global state (not `&self` methods), but the ghost-state wrappers `store_with_ghost` and `load_with_ghost` do correctly require/ensure `inv()` on the ghost parameter — so the higher-level API follows the methodology.

### Low

- **`axiom_volatile_read_consistency` is an `external_body` proof function**: This is a justified `external_body` (kredzone.proof.rs:254) — it encapsulates the trust assumption T2 (volatile reads return the last written value), which cannot be proven within Verus. The justification is well-documented in the function's doc comment and the module-level documentation. This is the only `external_body` proof function and serves as a clearly labeled axiom.

- **`raw_store` and `raw_load` are `external_body` exec functions**: These are justified `external_body` functions (kredzone.rs:303, 387) that encapsulate volatile memory operations which Verus cannot reason about. The trust boundary is minimal and well-documented. The bounds checking is verified in the calling `store`/`load` functions.

- **No `assume` or `admit` found**: Good — no unjustified assumptions exist.

## Checklist Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| View uses abstract types | ⚠️ Partial | Uses `Seq<usize>` instead of `Seq<int>` |
| `view()` is `pub closed spec fn` | ✅ Pass | `KernelRedZoneGhost::view()` is `pub closed` |
| `inv()`/`wf()` exists and is `pub closed` | ✅ Pass | `KernelRedZoneGhost::inv()` is `pub closed` |
| Public specs use `self@.field` not `self.field` | ⚠️ Partial | Ghost wrappers access `ghost.view` directly instead of `ghost.view()` |
| Public methods require/ensure `inv()` | ✅ Pass | Ghost wrappers correctly require/ensure `inv()` |
| No unjustified `assume`/`admit`/`external_body` | ✅ Pass | All `external_body` uses are documented and justified |
| Verification passes | ✅ Pass | 36 verified, 0 errors |

## Summary

The kredzone verified specification is well-structured with thorough documentation of trust boundaries (T1-T5) and a clean separation between verified bounds checking and trusted volatile operations. The verification architecture — splitting raw operations (`raw_store`/`raw_load`) from verified wrappers — maximizes verified code while keeping the trust boundary minimal and clearly delineated.

The main methodology gap is the use of concrete `usize` in the `KernelRedZoneView` instead of abstract `int`. Since the kredzone stores raw memory values that are inherently machine-word-sized, there is a pragmatic argument for `usize`, but the guidelines explicitly call for `int` in view types to maintain abstraction. The `pub` fields on both `KernelRedZoneView` and `KernelRedZoneGhost` also weaken encapsulation, allowing specs to bypass accessor methods.

The algebraic property lemmas (read-after-write, non-interference, commutativity, idempotence) are comprehensive and well-proven. The single axiom (`axiom_volatile_read_consistency`) is clearly labeled and minimally scoped. Overall, this is a solid verification with minor methodology deviations that could be tightened.
