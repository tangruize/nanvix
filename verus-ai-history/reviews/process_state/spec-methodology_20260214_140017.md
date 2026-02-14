# Review: process_state Spec Methodology (claude-opus-4.6)

## Grade: B-

## Issues Found

### Critical

1. **Verification fails with 4 errors.** Running `./verus-ai/scripts/verify.sh process_state` produces `43 verified, 4 errors`. The failing postconditions are:
   - `new()`: `result.wf()` not satisfied (line 129).
   - `set_capability()`: `self.capabilities.spec_has(capability)` not satisfied (line 160).
   - `clear_capability()`: `!self.capabilities.spec_has(capability)` not satisfied (line 177).
   - `has_capability()`: `result == self.capabilities.spec_has(capability)` not satisfied (line 192), plus precondition failure on `self.capabilities.has(capability)` because `self.wf()` (which implies `self.capabilities.wf()`) is not required.
   
   Root cause: `has_capability()` lacks a `requires self.wf()` (or at minimum `self.capabilities.wf()`), and `new()` cannot prove `wf()` because it cannot establish `self.capabilities.wf()` for the freshly constructed `Capabilities { bits: 0u8 }` without invoking the Capabilities wf lemma properly. The capability methods reference `self.capabilities.spec_has(capability)` which is a private-field drill-down; the verifier cannot resolve these without establishing `wf()` on the Capabilities sub-component.

### High

2. **`view()` and `wf()` are `pub open spec fn`, not `pub closed spec fn`.** The guidelines (Step 1, Step 2) require both to be `pub closed spec fn` to hide implementation internals. All 20+ spec functions in `process_state.spec.rs` are `pub open`, exposing concrete field structure (`self.mutex_addrs@`, `self.mutex_count`, `self.pid`, `self.capabilities`, etc.) to downstream modules. This defeats the purpose of the View abstraction.

3. **ProcessStateView uses concrete types.** The View type uses `Seq<u64>` for mutex/condvar addresses and ref counts, and `Seq<u16>` for PMIO ports. Per the guidelines (Step 1), View types should use abstract types: `int` instead of concrete integer widths, `Seq<int>` instead of `Seq<u64>`, `nat` fields instead of bounded-width fields. The `capabilities_bits: u8` field is a concrete type leaked into the view; it should be abstracted or removed (the `capabilities_granted: Set<Capability>` field already provides the abstract representation).

4. **Public method specs reference `self.field` directly instead of `self@.field`.** The guidelines (Step 3) require public method specs to use `self.view()` (i.e., `self@`) rather than accessing struct fields. Multiple violations:
   - `set_capability` ensures: `self.capabilities.spec_has(capability)` — accesses `self.capabilities` (a struct field).
   - `clear_capability` ensures: `!self.capabilities.spec_has(capability)` — same.
   - `has_capability` ensures: `result == self.capabilities.spec_has(capability)` — same.
   - `get_mutex` requires: `old(self).mutex_addrs@[idx as int]`, `old(self).mutex_ref_counts@[idx as int]` — accesses concrete Vec fields.
   - `put_mutex` requires: `old(self).mutex_addrs@[idx as int]`, `old(self).mutex_ref_counts@[idx as int]` — same.
   - `get_cond` / `put_cond`: same pattern for `cond_addrs@` and `cond_ref_counts@`.
   - `remove_pmio` requires: `old(self).pmio_ports@[found_idx as int]` — accesses concrete field.
   
   All of these should reference `self@.mutex_addrs`, `self@.capabilities_granted`, etc., or use spec helper functions on `ProcessStateView`.

5. **`has_capability()` missing `requires self.wf()`.** This public method calls `self.capabilities.has(capability)` which requires `self.capabilities.wf()`, but `has_capability` has no `requires` clause at all. Per the guidelines (Step 3), all public methods with `Self` parameters should require `inv()`/`wf()`.

### Medium

6. **All struct fields are `pub`.** The ProcessState struct has all fields public (`pub pid`, `pub capabilities`, `pub mutex_count`, etc.). The guidelines and Nanvix coding standards require struct fields to be private, accessible only via getter/setter methods. Public fields allow downstream code to bypass the well-formedness invariant by directly mutating fields.

7. **No `inv()` function — uses `wf()` instead.** The guidelines (Step 2) specify the invariant function should be named `inv` with signature `pub closed spec fn inv(&self) -> bool`. The implementation uses `wf()` instead. While functionally equivalent, this deviates from the naming convention. Additionally, `wf()` is `pub open` rather than `pub closed`.

8. **18 `external_body` stubs in process_state.rs.** Lines 869–1080 contain 18 `#[verifier::external_body]` functions. While the header documentation explains these are frame-condition stubs for omitted HAL/IPC functions, several have only `ensures true` (lines 871, 879, 995, 1021, 1051, 1077, 1116, 1131), providing no meaningful contract. The guidelines (Step 5) require removing `external_body` before declaring success, or at minimum justifying each one. The stubs with `ensures true` are particularly concerning as they provide zero verification value.

9. **Proof lemmas reference `self.field` directly.** The proof file (`process_state.proof.rs`) extensively uses `self.mutex_addrs@`, `self.mutex_ref_counts@`, `self.pid.spec_value()`, `self.capabilities.spec_bits()`, etc. While the guidelines allow private field access in private method specs (Step 4), proof lemmas that are `pub` (e.g., `lemma_new_is_wf`, `lemma_capability_change_preserves_wf`, `lemma_pmio_push_preserves_wf`) expose internal field structure through their requires/ensures.

### Low

10. **`ProcessStateView` has redundant representation.** Both `capabilities_bits: u8` (concrete) and `capabilities_granted: Set<Capability>` (abstract) are in the View type. The abstract `capabilities_granted` is the correct representation per the guidelines; `capabilities_bits` leaks implementation detail.

11. **Spec constants `MUTEX_MAX()` and `COND_MAX()` return `usize` (concrete).** Per abstract-types guidance, spec-level constants should use `nat` or `int`, not `usize`.

## Summary

The process_state verification model demonstrates a solid protocol-level approach to verifying ProcessState management (mutex/condvar capacity, ref-count cleanup, PID immutability, PMIO consistency). The parallel-Vec modeling of BTreeMap is creative and well-documented. However, the implementation has significant methodology deviations from the guidelines: `view()` and `wf()` are `open` instead of `closed`, public method specs access struct fields directly rather than going through the View, the View type uses concrete types, and verification currently fails with 4 errors in the capability-related methods. The 18 `external_body` stubs (8 with trivial `ensures true`) represent a large trusted computing base. Fixing the capability wf propagation and closing the spec functions would bring this to a B+ or A-range.
