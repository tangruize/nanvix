# Review: sleeping_process Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **`spec_pid()` returns concrete `u64` instead of abstract `int`** (sleeping.spec.rs:143, 269, 290):
  `SleepingProcess::spec_pid()`, `RunnableProcess::spec_pid()`, and `InterruptedProcess::spec_pid()`
  all return `u64`. Per the methodology (Step 1), View-level reasoning should use abstract types.
  The View types correctly use `int` for `pid`, but the exec-level spec functions leak the concrete
  `u64` into public method postconditions (e.g., `result.spec_pid() == self.spec_pid()` in
  `terminate()` at sleeping.rs:212). Public method specs should use `self@.pid` / `result@.pid`
  (the View-level `int` fields) rather than `self.spec_pid()` (concrete `u64`). This does not
  break soundness but violates the abstraction boundary guideline.

- **Public method specs reference `self.sleeping_thread_ids@` and `self.zombie_thread_ids@` directly**
  (sleeping.rs:147–149, 213–220, 276–285, 409–419, 429–437, 491–504):
  The methodology (Step 3) states: "never talk directly about the fields of a `Self` parameter.
  For instance, don't say `self.x`; instead say things like `self@.y`." Nearly all public method
  `requires`/`ensures` clauses use `self.sleeping_thread_ids@`, `self.zombie_thread_ids@`,
  `result.interrupted_thread_ids@`, etc. These should instead use View-level fields
  (`self@.sleeping_thread_ids`, `result@.interrupted_thread_ids`, etc.) to maintain the
  abstraction boundary. The bridging lemmas and View-level transition functions are already in
  place to support this refactoring.

- **Public method specs call `Self::spec_seq_contains`, `Self::spec_no_duplicates`, etc.**
  (sleeping.rs:148–149, 263, 409–419, 491–492):
  The methodology (Step 3) states: "never call any `Self` specification functions other than `inv`
  and `view`." Functions like `spec_seq_contains`, `spec_no_duplicates`, `spec_seqs_disjoint`,
  and `spec_is_subsequence` are internal helpers that should not appear in public method
  signatures. These should be replaced with equivalent predicates on View types (some already
  exist as `SleepingProcessView::spec_no_duplicates` and `spec_seqs_disjoint`) or folded into
  `wf()`.

### Medium

- **`view()` is `open` instead of `closed`** (sleeping.spec.rs:320, 334, 350):
  The methodology (Step 1) recommends `pub closed spec fn view()`. The code includes a comment
  explaining that the Verus `View` trait requires `open`. This is a reasonable justification and
  abstraction is partially preserved because the View types use abstract types. However, since
  `view()` is `open`, callers can see the conversion logic (`spec_u64_seq_as_int`) and reach
  through to concrete fields, weakening the abstraction. Consider whether a wrapper approach
  or trait-level change could restore closedness in future iterations.

- **`SleepingProcessView::wf()` is `open`, not `closed`** (sleeping.spec.rs:385):
  The exec-level `SleepingProcess::wf()` is correctly `closed`, but the View-level
  `SleepingProcessView::wf()` is `open`. Per the methodology, invariant specs should be `closed`
  to hide implementation details from downstream consumers. Since downstream modules use the
  View types for composition, this `open` predicate exposes internal invariant structure. The
  bridging lemma `lemma_view_wf_equiv` already connects the two, so the View-level `wf()` could
  be `closed` without loss of usability.

- **`spec_pid()`, `spec_sleeping_count()`, `spec_zombie_count()`, etc. are `open`**
  (sleeping.spec.rs:143–178):
  The methodology (Step 3) says not to write additional `pub` spec fns beyond `inv`/`view` in
  `impl MyType`. These helpers (`spec_pid`, `spec_sleeping_count`, `spec_zombie_count`,
  `spec_has_sleeping_thread`, `spec_has_zombie_thread`, `spec_has_thread`, `spec_find_thread`)
  are public and open. While some are useful internally, making them public exposes
  implementation details. Consider moving reusable predicates to `SleepingProcessView` and
  keeping exec-level helpers private.

### Low

- **`mutation_frame_preserved` is `open`** (sleeping.spec.rs:256):
  This spec function references concrete fields (`spec_pid()`, `sleeping_thread_ids@`,
  `zombie_thread_ids@`). It is used in `state_mut()`'s `external_body` postcondition. Per
  methodology, it should ideally be `closed` or moved to View-level reasoning.

- **`state()` and `state_mut()` are `external_body` without explicit justification comment**
  (sleeping.rs:172, 188):
  Both are marked `#[verifier::external_body]` and documented as modeling reference-returning
  functions. The trust boundary section in the module header covers this, but inline
  `// TRUST:` comments at each `external_body` annotation would improve auditability.

- **No `assume` or `admit` found**: Confirmed — no unjustified assumptions remain.

## Summary

The sleeping_process verification module is well-structured with comprehensive proofs (29/29
verified, 0 errors). The View types correctly use abstract types (`int`, `Seq<int>`), `wf()` is
properly `pub closed spec fn`, verification passes cleanly, and there are no `assume`/`admit`
statements. The bridging lemmas and View-level transition functions demonstrate strong
methodology awareness.

The primary gap is that public method specifications do not yet follow the methodology's
abstraction boundary rules: they reference concrete struct fields (`self.sleeping_thread_ids@`)
and internal spec helpers (`Self::spec_seq_contains`) instead of using View-level fields
(`self@.sleeping_thread_ids`) and View-level predicates. The View infrastructure to support this
refactoring is already in place (View types, bridging lemmas, and View-level transition
functions), so the remaining work is mechanical: rewriting `requires`/`ensures` clauses to use
`self@` notation and removing internal helpers from public signatures.

The `external_body` usage on `state()`/`state_mut()` is justified by the trust boundary
documentation. Overall, this is a high-quality verification module that needs one systematic
refactoring pass to achieve full methodology compliance.
