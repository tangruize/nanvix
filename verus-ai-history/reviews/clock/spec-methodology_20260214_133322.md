# Review: clock Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Public fields in TimerTicks struct (clock.rs:166–169):** The `minor` and `major` fields are `pub`, which the module header acknowledges weakens encapsulation. The guidelines document (Step 1) says the View type should "hide internal fields that aren't important to users." While the deviation is justified (Verus requires `pub` fields for `pub open spec fn` access), it means many spec functions (e.g., `spec_minor()`, `spec_major()`) and public method postconditions reference `self.minor`/`self.major` (concrete struct fields) rather than `self@.ticks` (the abstract View field). This is a systematic deviation from criterion 4.

- **Public method specs reference `self.field` instead of `self@.field` (clock.rs:223–228, 259, 471–472):** The `get()` method ensures `result.0 == self.major`, `result.1 == self.minor`; `increment()` ensures `result == self.minor`; `now()` ensures `result.0 as nat == Self::spec_compute_seconds(self.major, self.minor, ...)`. Per the guidelines (Step 3): "never talk directly about the fields of a `Self` parameter. For instance, don't say `self.x`; instead say things like `self@.y`." The View type `TimerTicksView` has only `ticks: nat`, so the specs should be expressible using `self@.ticks` and helper specs on `TimerTicksView`, but currently they leak the (major, minor) representation into the public API contracts. The ensures `result@ == TimerTicks::spec_new_view()` in `new()` (line 192) and `result.spec_ticks()` usage are good counterexamples that do use the abstraction properly, but they coexist with concrete-field references.

- **Several public methods lack `wf()` in requires/ensures (criterion 5):** `ticks()` (line 326), `is_max()` (line 345), `is_zero()` (line 364), `compute_nanoseconds()` (line 399), `compute_seconds()` (line 429) do not require `self.wf()` (for instance methods) or ensure `wf()` on any returned `Self`. The guidelines (Step 3) state: "for any input `Self` parameters, one of the preconditions should be that `inv` holds." While `lemma_always_wf` proves `wf()` is universally true (making the omission harmless), the guideline asks for it as documentation and forward-compatibility. `new()`, `increment()`, `get()`, and `now()` do include `wf()` properly.

### Medium
- **`view()` is `open` not `closed` (clock.spec.rs:333):** The guidelines specify `pub closed spec fn view()`. The code includes a justification note (lines 324–329) explaining that the `View` trait in vstd requires `open spec fn view()`. This is an accepted deviation, clearly documented. Grade impact is minimal given the justification.

- **Spec functions that could live on `TimerTicksView` live on `TimerTicks` (clock.spec.rs):** The guidelines (Step 3) suggest: "create a `pub open spec fn` in `MyTypeView` for each such common expression." Currently, `TimerTicksView` has no spec helper methods. Functions like `spec_is_max()`, `spec_is_zero()`, `spec_next_ticks()` operate on the abstract tick count and could be defined on `TimerTicksView`, promoting abstraction in public method postconditions.

- **Many spec functions are `pub open` that could be `pub closed` (clock.spec.rs):** Functions like `spec_minor()`, `spec_major()`, `spec_ticks()`, `spec_is_max()`, `spec_is_zero()`, `spec_next_ticks()` are all `pub open`. The guidelines (Step 4) say internal spec helpers should be private or `pub closed`. Since these expose the (major, minor) representation, making `spec_minor()` and `spec_major()` non-public would improve encapsulation. `spec_ticks()` is the abstract view and could remain open if moved to `TimerTicksView`.

### Low
- **`external_body` axioms are justified (clock.proof.rs:712, 745):** Two `external_body` items exist:
  - `axiom_pit_timer_freq_valid()` — justified by PIT hardware guarantees; well-documented (lines 678–718).
  - `axiom_no_concurrent_writer()` — justified by the single-writer system-level invariant; well-documented (lines 726–751).
  Both are clearly marked as trust boundaries (T1, T5) and are the minimum necessary. No unjustified `external_body`.

- **No `assume` or `admit` found.** Clean.

## Summary

The clock verification is thorough and well-documented, with 59 verified properties and zero errors. The `TimerTicksView` abstraction exists and uses the abstract type `nat` (criterion 1: pass). `wf()` is `pub closed spec fn` (criterion 3: pass). No `assume`/`admit` remain (criterion 6: pass). Verification passes (criterion 7: pass).

The main gap is criterion 4 (public method specs use `self.minor`/`self.major` directly instead of `self@.ticks`) and a partial gap in criterion 5 (several public methods omit `wf()` in preconditions). These are interrelated: since the View type has only a single `ticks` field but the specs need to talk about the (major, minor) decomposition (especially for `get()` and `now()`), the public API surface leaks representation details. Fixing this would require either enriching `TimerTicksView` or moving decomposition-dependent specs to helper functions on the View type.

The `view()` openness deviation is well-justified (vstd trait requirement). The two `external_body` axioms are well-justified trust boundaries with clear documentation. Overall, this is a high-quality verification with minor methodology deviations that are mostly forced by Verus constraints.
