# Review: process_capability Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **Public method specs use `spec_bits()`/`spec_has()`/`spec_set()`/`spec_clear()` instead of `self@` fields.** The methodology (Step 3) says public method ensures/requires should use `self@.field` (i.e., view-level abstractions) and should not call `Self` spec functions other than `inv`/`view`. Currently, `set()` ensures `self.spec_bits() == old(self).spec_set(capability)` and `self.spec_has(capability)` — these are bit-level implementation specs, not view-based abstractions. The public API should use `self@.granted` (the set-level view field) for postconditions, e.g., `self@.granted =~= old(self)@.granted.insert(capability)`. The set-level specs (`spec_set_contains`) are included as *additional* ensures but the primary ensures still expose implementation-level bit operations.

- **Public methods do not require/ensure `wf()` unconditionally.** The methodology (Step 3) says `old(self).inv()` should be in `requires` and `self.inv()` should be in `ensures` for `&mut self` methods. Currently, `set()` and `clear()` use *conditional* preservation (`old(self).wf() ==> self.wf()`) rather than requiring `wf()` as a precondition. `has()` has no `wf()` requirement at all. The `wf()` invariant should be a required precondition on public methods, not an optional conditional.

### Medium

- **`view()` is `closed spec fn` but not declared `pub`.** The View trait implementation at line 259 of `capability.spec.rs` declares `closed spec fn view(...)` without `pub`. While this may be an artifact of the trait impl syntax, the methodology (Step 1) specifies it should be `pub closed spec fn`. Verus trait impls may implicitly inherit visibility from the trait, but this should be verified.

- **Many spec functions are `pub open` that could be `pub closed`.** The bit-level specs (`spec_bits`, `spec_mask`, `spec_has`, `spec_set`, `spec_clear`, `spec_pow2_mask`) are all `pub open spec fn`. Per the methodology, implementation-detail specs should be private or `pub closed` — only the view and invariant need to be `pub`. The set-level specs (`spec_as_set`, `spec_set_contains`, `spec_set_insert`, `spec_set_remove`, `spec_granted`) are also `pub open`, which is more defensible since they represent the abstract interface, but the bit-level ones leak implementation details.

- **`spec_default()` is `pub open spec fn`.** This spec constructor exposes the internal representation (`Capabilities { bits: 0u8 }`). It should be `pub closed` with its properties exposed through lemmas, or return a view-level value.

### Low

- **`CapabilitiesView.bits` retains a concrete-adjacent representation.** The view type uses `bits: int` which, while technically abstract (`int` not `u8`), still mirrors the implementation's bitfield. The methodology (Step 1) says view types should "hide internal fields that aren't important to users." Since the `granted: Set<Capability>` field fully captures the abstract state, the `bits` field is arguably unnecessary in the view and exists only for bridging proofs. Consider keeping it only in internal proof helpers rather than the public view type.

- **No `inv()` function — uses `wf()` instead.** The methodology (Step 2) names the invariant function `inv()`. The implementation uses `wf()`, which is semantically equivalent but a naming deviation. This is a minor style point; the codebase consistently uses `wf()` as documented.

## Verification

- **Status:** PASSED — 98 verified, 0 errors.
- **No `assume`, `admit`, or unjustified `external_body`** found anywhere in the three files.

## Summary

The `process_capability` module is well-verified with comprehensive bit-level and set-level proofs, thorough bridging lemmas, and excellent documentation. The verification passes cleanly with 98 obligations and zero assumptions. The two main methodology gaps are: (1) public method specs primarily use implementation-level spec functions (`spec_bits`, `spec_has`, `spec_set`, `spec_clear`) rather than view-based abstractions (`self@.granted`), and (2) `wf()` is not unconditionally required/ensured on public methods as the methodology prescribes. The set-level abstraction layer is a strong design — it just needs to be promoted to the primary interface in public method contracts, with the bit-level specs demoted to internal/proof use. Overall, this is a solid verification effort that demonstrates careful attention to correctness properties.
