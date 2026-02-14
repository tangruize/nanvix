# Review: sleeping_process Abstraction (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- **Missing explicit error-case spec for `wakeup()`**
  - Location: `SleepingProcessView` in `sleeping.spec.rs`
  - Description: `spec_wakeup()` models only the success case (thread found). The error case
    (thread not found, returning self unchanged) has no corresponding View-level spec function
    such as `spec_wakeup_not_found() -> SleepingProcessView`. While the error case is trivially
    identity (like `spec_wakeup_alarm_none`), the asymmetry means downstream modules cannot
    write `ensures result@ =~= old(self)@.spec_wakeup_not_found()` for the Err branch —
    they must enumerate field equalities manually. This breaks the stated design goal.
  - Suggested Fix: Add `pub open spec fn spec_wakeup_not_found(self) -> SleepingProcessView { self }`
    and a corresponding bridging lemma `lemma_wakeup_not_found_refines_spec` in the proof file.

### Low

- **Cross-type reference in `SleepingProcessView::wf()`**
  - Location: `SleepingProcessView::wf()` in `sleeping.spec.rs` (lines 324–329)
  - Description: The View-level `wf()` calls `SleepingProcess::spec_no_duplicates` and
    `SleepingProcess::spec_seqs_disjoint`, reaching into the exec-level type's namespace.
    This creates a cross-layer dependency that is conceptually unclean — View types should
    be self-contained. The functions operate on `Seq<u64>` so there is no semantic issue,
    but it couples the View layer to the exec-level impl.
  - Suggested Fix: Either add standalone spec helper functions on `SleepingProcessView` (or as
    free functions) wrapping these predicates, or accept the coupling with a comment explaining
    the choice.

- **Duplicated `spec_remove_at` definition**
  - Location: `SleepingProcess::spec_remove_at` (sleeping.spec.rs:187) and
    `SleepingProcessView::spec_remove_at_seq` (sleeping.spec.rs:332)
  - Description: Both have identical bodies (`s.subrange(0, idx).add(s.subrange(idx + 1, ...))`).
    Verus unfolds both and sees extensional equality, so verification passes, but there is no
    explicit lemma establishing `spec_remove_at(s, idx) == spec_remove_at_seq(s, idx)`. This
    could become fragile if one definition is changed without the other.
  - Suggested Fix: Either have `spec_remove_at_seq` delegate to `SleepingProcess::spec_remove_at`,
    or add a trivial bridging lemma proving their equivalence.

- **Boundary type `wf()` is minimal**
  - Location: `RunnableProcess::wf()` (sleeping.spec.rs:254) and
    `InterruptedProcess::wf()` (sleeping.spec.rs:266)
  - Description: Boundary type well-formedness only checks non-empty primary list
    (e.g., `ready_thread_ids@.len() >= 1`). No cross-list disjointness or no-duplicates
    checks are included. This is acceptable as a trust boundary, but it means the
    `result.wf()` postcondition on `terminate()`, `wakeup()`, etc. provides weaker
    guarantees than the real types enforce. Downstream consumers may need to re-derive
    uniqueness/disjointness from the field-level postconditions.
  - Suggested Fix: Consider strengthening boundary `wf()` to include no-duplicates on
    the primary list at minimum. Alternatively, document that callers should use the
    field-level postconditions for stronger properties.

## Positive Observations

- **Exec file unchanged**: The `.rs` exec file has zero diff from the previous verified commit
  (`457abcc41`), fully satisfying the NO EXEC CHANGES criterion.
- **Verification passes**: 28 verified, 0 errors — the abstraction layer does not break any
  existing proofs.
- **Complete coverage of state-changing functions**: All six exec functions that change state
  (`new`, `terminate`, `wakeup`, `wakeup_alarm` ×2, `add_thread`) have matching View-level
  spec transition functions.
- **Bridging lemmas are well-structured**: Each spec transition function has a corresponding
  bridging lemma that proves exec postconditions imply View-level equality. The
  `lemma_wakeup_refines_spec` correctly handles `choose` index uniqueness under no-duplicates.
- **Proper use of `ext_equal`**: All three View types are annotated with `#[verifier::ext_equal]`,
  enabling the `=~=` extensional equality used in bridging lemma postconditions.
- **`spec_wakeup` uses `choose` elegantly**: The abstract `choose`-based index selection
  avoids exposing any concrete search strategy, maintaining a clean abstraction boundary.
- **`spec_wakeup_alarm_none` models identity correctly**: Returns `self` directly,
  cleanly capturing the no-op semantics.
- **Good documentation**: The header comments in the spec file clearly enumerate the
  transition functions and explain the design rationale for bridging lemmas.
- **Type alignment**: The proof file correctly migrated from `int`/`Seq<int>` to
  `u64`/`Seq<u64>`, aligning proof-level types with exec-level types.

## Summary

The abstraction improvements are well-designed and correctly implemented. Six View-level
abstract state transition functions on `SleepingProcessView` enable downstream modules to
express postconditions as single equalities (`result@ =~= old(self)@.spec_terminate()`)
rather than enumerating field changes. All bridging lemmas are sound and the `choose`-index
uniqueness proof in `lemma_wakeup_refines_spec` is the most non-trivial and is handled
correctly. The main gap is the missing error-case spec for `wakeup()`, which breaks
symmetry with the alarm case and prevents downstream use of the View abstraction for the
Err branch. Minor style issues (cross-type references, duplicated definitions) do not
affect soundness. Verification passes at 28/0.
