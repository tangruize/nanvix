# Review: zombie_process Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **`view()` is `open`, not `pub closed spec fn` (zombie.spec.rs:472).** The guidelines (Step 1) require `pub closed spec fn view(&self) -> MyTypeView`. The implementation comment says "Verus does not support `closed` for View trait method impls" — this is a legitimate Verus limitation (trait impls cannot use `closed`), so the `open` annotation is justified. However, the View fields on `ZombieProcessView` are `pub` (lines 99-104), which exposes the mapping. If Verus adds `closed` trait support, this should be revisited.

### Medium
- **Four `external_body` functions (zombie.rs:180, 207, 263, 299).** `state()`, `state_mut()`, `find_thread()`, and `find_thread_mut()` are all `external_body`. Each is thoroughly documented with justification:
  - `state()` / `state_mut()`: return reference types to `ProcessState`, which can't be modeled in the ghost model's `u64`-PID abstraction. PID immutability for `state_mut()` is verified cross-module in `process_state.rs` (documented in detail).
  - `find_thread()` / `find_thread_mut()`: return `ThreadRef`/`ThreadRefMut` reference wrappers that Verus cannot express. Three integration obligations are defined and documented. `lemma_ghost_search_correctness` proves the ghost search logic is sound.
  All four have well-justified trust boundaries with formal integration obligations. No unjustified `external_body` usage.
- **No `assume` or `admit` found.** Clean.

- **Some `pub open spec fn` on `ZombieProcess` impl expose internal details (zombie.spec.rs:113-155).** `spec_pid()`, `spec_status()`, `spec_zombie_count()`, `spec_seq_contains()`, `spec_has_zombie_thread()`, `spec_find_thread()`, `spec_no_duplicates()` are all `pub open`. The guidelines (Step 3) say "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." These are used in public method preconditions/postconditions (e.g., `new()` requires `spec_no_duplicates`), so some exposure is needed, but they leak implementation details (direct field access like `self.pid`, `self.zombie_thread_ids@`). Ideally, public method specs should use `self@.field` (View fields) rather than `self.field` (exec fields). The `mutation_frame_preserved` function (line 200) also accesses `self.spec_pid()` and `self.zombie_thread_ids@` directly.

### Low
- **`inv()` is correctly `pub closed spec fn` (zombie.spec.rs:188).** ✅ Compliant with Step 2.
- **`ZombieProcessView` uses abstract types correctly (zombie.spec.rs:98-105).** `pid: int`, `zombie_thread_ids: Seq<int>`, `status: int` — all abstract. ✅ Compliant with Step 1.
- **Public method specs use `self@.field` in postconditions (zombie.rs:155-156, 235-237).** `new()` ensures `result@.pid`, `result@.zombie_thread_ids`, `result@.status`. `bury()` ensures via `self@.zombie_thread_ids`, `self@.pid`, `self@.status`. ✅ Compliant with Step 3.
- **Public methods require/ensure `inv()` for self parameters (zombie.rs).** `new()` ensures `result.inv()`. `state()` requires `self.inv()`. `state_mut()` requires `old(self).inv()`, ensures `self.inv()`. `bury()` requires `self.inv()`. `find_thread()` requires `self.inv()`. `find_thread_mut()` requires `old(self).inv()`, ensures `self.inv()`. ✅ Compliant with Step 3.
- **`new()` precondition uses `Self::spec_no_duplicates(zombie_ids@)` (zombie.rs:153).** This references exec-level `zombie_ids@` (a `Seq<u64>`) rather than View-level abstract types. Since `new()` takes exec parameters and the View doesn't exist yet, this is acceptable — the precondition can't use `self@` on a constructor.
- **`ZombieProcessView` has `wf()` (zombie.spec.rs:454).** The View has `wf()` as a well-formedness predicate, and `lemma_exec_inv_implies_view_wf` (zombie.proof.rs:457) proves exec `inv()` implies view `wf()`. Good design.

## Verification
- **25 verified, 0 errors.** All proofs pass. ✅

## Summary

The zombie_process spec methodology is well-executed. The `ZombieProcessView` correctly uses abstract types (`int`, `Seq<int>`). `inv()` is properly `pub closed spec fn`. Public method specs consistently use `self@.field` in postconditions and require/ensure `inv()` on all self parameters. There are no `assume` or `admit` statements. The four `external_body` functions are each thoroughly justified with detailed trust boundary documentation, cross-module verification references, and formal integration obligations with proof lemmas.

The main area for improvement is that several `pub open spec fn` helpers on `ZombieProcess` (not `ZombieProcessView`) expose exec-level field access (`self.pid`, `self.zombie_thread_ids@`) in the public interface — the guidelines prefer routing all public specs through the View. This is a moderate methodology deviation but doesn't compromise soundness since `inv()` itself is properly closed. The `view()` being `open` is a known Verus limitation, not a methodology violation. Overall, this is a high-quality spec with strong proof coverage.
