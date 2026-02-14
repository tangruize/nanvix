# Review: interrupted_process Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Public method specs reference `self.field` directly instead of `self@.field`.**
  The methodology (Step 3) states: "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." Multiple public methods violate this:
  - `new()` ensures: `result.sleeping_thread_ids@.len() == 0`, `result.interrupted_thread_ids@ == interrupted_ids@`, `result.zombie_thread_ids@ == zombie_ids@` — all reference concrete struct fields directly rather than using `result@.sleeping_thread_ids`, etc.
  - `from_sleeping()` ensures: same pattern — `result.sleeping_thread_ids@`, `result.interrupted_thread_ids@`, `result.zombie_thread_ids@`.
  - `state_mut()` ensures: `self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@`, etc.
  - `resume()` ensures: `result.ready_thread_ids@.len() == 1`, `result.ready_thread_ids@[0] == self.interrupted_thread_ids@[0]`, `result.sleeping_thread_ids@ == self.sleeping_thread_ids@`, etc. (extensively uses concrete fields on both `self` and `result`).
  - `resume_with_valid_clock()`: same pattern as `resume()`.
  - `find_thread_mut()` ensures: `self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@`, etc.

  These should use `self@.interrupted_thread_ids`, `result@.ready_thread_ids`, etc. to go through the View abstraction. The View types (`InterruptedProcessView`, `RunnableProcessView`) exist with the correct abstract field types (`Seq<int>`), but the public method specs bypass them entirely and operate on the concrete `Vec<u64>` fields via `@` (Verus deep-view of `Vec`).

  **Impact:** Clients writing specs against `InterruptedProcess` are coupled to the concrete `Vec<u64>` representation rather than the abstract `Seq<int>` View. If the implementation changes (e.g., using a different collection type), all client specs break. This defeats the purpose of the View abstraction layer.

### Medium
- **`spec_pid()` returns concrete `u64` instead of abstract `int`.** Per Step 1, View types should use abstract types. The View type `InterruptedProcessView` correctly has `pid: int`, but `InterruptedProcess::spec_pid()` (line 149) returns `u64`. Public specs like `result.spec_pid() == pid` operate on concrete types. These should use `result@.pid == pid as int` or the spec function should return `int`. This is a minor abstraction leak — `u64` is close enough to `int` for practical purposes, but the methodology is clear about preferring abstract types in public-facing specs.

- **`mutation_frame_preserved()` references concrete fields.** Line 259-264: This `pub open spec fn` on `InterruptedProcess` uses `new_self.interrupted_thread_ids@` etc. (concrete fields) rather than `new_self@.interrupted_thread_ids` (view fields). As a public spec function, it should operate on Views.

- **`state()` lacks `wf()` in precondition.** Step 3 says "for any input `Self` parameters, one of the preconditions should be that `inv` holds." `state()` (line 249) has no `requires` clause at all. While `state()` is a simple getter that doesn't need `wf()` to function correctly, the methodology mandates it for consistency. Compare with `state_mut()` which correctly requires `old(self).wf()`.

- **`find_thread()` lacks `wf()` in precondition.** Same issue — `find_thread()` (line 509) has no `requires self.wf()`. The function computes `spec_find_thread()` which works without `wf()`, but the methodology requires `inv()` for all public methods with `Self` parameters.

### Low
- **`view()` is `open spec fn` instead of `closed`.** Lines 562-577. This is justified in the code comment: Verus `View` trait requires `open`. The justification is correct and the note is properly documented. Not a real issue.

- **View type fields are `pub`.** `InterruptedProcessView` and `RunnableProcessView` have all public fields. The methodology doesn't explicitly prohibit this for View types (Step 1 says to "hide internal fields that aren't important"), and all fields here are semantically relevant. Acceptable.

- **Helper spec functions (`spec_no_duplicates`, `spec_seqs_disjoint`) are `pub open` on the concrete type.** These are implementation-level helpers exposed publicly. The methodology says "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." However, these are utility predicates used in preconditions, making them necessary for callers. Could be moved to the View type or made standalone functions to be more methodology-compliant.

## Checklist Summary

| Criterion | Status | Notes |
|-----------|--------|-------|
| 1. View types use abstract types | ✅ PASS | `InterruptedProcessView` uses `int`, `Seq<int>` correctly |
| 2. `view()` is `pub closed spec fn` | ✅ PASS (justified) | Must be `open` due to Verus `View` trait; documented |
| 3. `wf()` exists and is `pub closed spec fn` | ✅ PASS | Both `InterruptedProcess::wf()` and `RunnableProcess::wf()` are `pub closed spec fn` |
| 4. Public specs avoid `self.field` | ❌ FAIL | Extensively uses concrete fields instead of `self@.field` |
| 5. Public methods require/ensure `wf()` | ⚠️ PARTIAL | `state()` and `find_thread()` missing `wf()` precondition |
| 6. No remaining assume/admit/external_body | ✅ PASS | None found in any of the three files |
| 7. Verification passes | ✅ PASS | 37 verified, 0 errors |

## Summary

The `interrupted_process` verification module is well-structured with thorough documentation, clean proofs (37 verified, 0 errors), and no trust gaps (no assume/admit/external_body). View types correctly use abstract types (`int`, `Seq<int>`), and `wf()` is properly `pub closed spec fn`. The main methodology gap is that **public method specs extensively reference concrete struct fields** (`self.interrupted_thread_ids@`, `result.ready_thread_ids@`) instead of going through the View abstraction (`self@.interrupted_thread_ids`, `result@.ready_thread_ids`). This couples clients to the concrete `Vec<u64>` representation, undermining the abstraction that the View types were designed to provide. Two public methods (`state()`, `find_thread()`) also omit the `wf()` precondition required by the methodology. Fixing the `self.field` → `self@.field` migration in public specs is the most impactful improvement needed.
