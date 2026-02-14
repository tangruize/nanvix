# Review: condvar Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **CondvarView uses concrete types `i32` instead of abstract `int`.** The `CondvarView` struct declares `pub sleeping: Seq<(i32, i32)>` (line 22, condvar.spec.rs). Per the guidelines (Step 1), the View type should use abstract types: `Seq<(int, int)>` instead of `Seq<(i32, i32)>`. This leaks the concrete representation choice (i32 for pid/tid) into the abstraction layer. The same concrete `(i32, i32)` type propagates throughout spec functions (`spec_front`, `spec_back`, `spec_remove_at_seq`, etc.) and proof lemmas, all of which should use `(int, int)` in the View.

### Medium
- **Several `pub open spec fn` helpers on `Condvar` that should be on `CondvarView` or private.** The guidelines (Step 3) say: "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`. Any other specification functions in `impl MyType` should be private and used only internally." Functions like `spec_is_empty`, `spec_is_nonempty`, `spec_len`, `spec_contains_pid`, `spec_contains_tid`, `spec_contains_entry`, `spec_all_unique`, `spec_no_kernel_pid`, `spec_front`, `spec_back`, `spec_drop_safe`, `spec_kernel_pid`, `spec_notify_all_result`, and `spec_remove_at_seq` (lines 49–173, condvar.spec.rs) are all `pub open spec fn` on `Condvar`. Per the methodology, common spec helpers for public use should be `pub open spec fn` on `CondvarView`, while implementation-internal predicates like `spec_all_unique` and `spec_no_kernel_pid` should be private (non-pub) on `Condvar`.

- **Public method specs reference `self.len` (a concrete field) directly.** Several public method ensures clauses reference `self.len` or `old(self).len` directly (e.g., `enqueue` line 252, `dequeue_first` line 365, `remove_at` line 406, `clear` line 679, `is_empty` line 704, `get_len` line 716). Per Step 3: "never talk directly about the fields of a `Self` parameter." These should use `self@`-based expressions or spec functions like `self.spec_len()` instead of `self.len`.

- **`get_len` does not require `wf()`.** The `get_len` method (line 714) has no `requires` clause and does not require `self.wf()`. Per Step 3, all public methods with `&self` parameters should require `inv()`/`wf()`. Without it, the postcondition `result == self.len` is trivially true but unhelpful since `self.len` may be inconsistent with the actual queue.

### Low
- **`view()` is `open` instead of `closed`, with justification.** The `view()` implementation (line 188, condvar.spec.rs) is `open spec fn` with a comment explaining this is required by the Verus `View` trait. This is acceptable and properly justified — the trait requires matching openness. No action needed.

- **`wf()` uses correct pattern.** The `wf()` function (line 42) is `pub closed spec fn` as required by the guidelines (Step 2). It correctly encapsulates implementation invariants (length consistency, uniqueness, no-kernel-pid).

- **Proof lemma `lemma_wf_len_consistency` (line 62, condvar.proof.rs) references `self.len` directly in ensures.** This is a proof lemma, not a public method spec, so it is less critical, but it does expose the concrete field. Similarly, `lemma_empty_is_drop_safe` (line 797) references `self.len` in requires.

- **`remove_at` requires `idx < old(self).len` — concrete field in public requires.** Line 403 references `old(self).len` directly. Should use `old(self).spec_len()` or `old(self)@.sleeping.len()`.

## Verification Status

✅ **Verification passes**: 53 verified, 0 errors.

## No assume/admit/external_body

✅ No `assume`, `admit`, or unjustified `#[verifier::external_body]` found in any of the three files.

## Summary

The condvar spec methodology is well-structured with comprehensive proofs covering FIFO ordering, uniqueness preservation, wait protocol safety, and drop safety. The `wf()` function is properly `pub closed` and the `view()` openness is justified by the Verus trait requirement. Verification passes cleanly with 53 verified obligations and no assumptions.

The main methodology gap is that **CondvarView uses concrete `i32` types instead of abstract `int`**, which is a clear deviation from Step 1 of the guidelines. This means the abstraction layer leaks the concrete pid/tid representation. Additionally, public spec helper functions are placed on `Condvar` rather than `CondvarView` (contrary to Step 3), and several public method specs reference concrete fields like `self.len` instead of using `self@`-based access. These are structural methodology issues that don't affect soundness but reduce the abstraction benefit of the View pattern: changing pid/tid from `i32` to another type would require updating the View and all specs, which the methodology is designed to prevent.
