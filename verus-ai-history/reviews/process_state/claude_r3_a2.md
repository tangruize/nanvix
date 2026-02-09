# Review: process_state (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### High Issues

1. **Oracle parameter documentation (T5)** — **FIXED.** Trust Assumption T5 was added (lines 84–95 of exec) with explicit "Caller burden" paragraph explaining that callers must derive oracle values from runtime data structures and that T2 (BTreeMap↔ghost map correspondence) is the critical correctness link. This directly addresses the review feedback. Additionally, the `get_mutex`/`get_cond` error-path postconditions were strengthened to include `old(self).spec_mutexes_full()` / `old(self).spec_conditions_full()`, proving errors only occur at capacity.

2. **`put_mutex` semantic mismatch potential** — **No issue (confirmed).** Original review noted "No issue here, just noting the pattern is correct." No change needed or made.

### Medium Issues

1. **Missing `Vmem` parameter documentation** — **FIXED.** Lines 182–184 of exec now state: "The original constructor takes `(pid, vmem)`. The `Vmem` parameter is omitted here because Vmem is an opaque HAL boundary type outside the verification scope (see Trust Assumption T4)."

2. **`remove_pmio` return value not captured** — **FIXED.** Postcondition at line 659 now includes `old(self).ghost_pmio@[found_idx@] == port_number@`, explicitly capturing the identity of the removed element. Additionally, the precondition was strengthened with a first-occurrence constraint (`forall|j: int| 0 <= j < found_idx@ ==> old(self).ghost_pmio@[j] != port_number@`), matching `iter().position()` semantics. The proof file adds `lemma_pmio_first_occurrence_exists` with a non-trivial recursive helper to show this precondition is satisfiable. Well done.

3. **`receive_message_stub` TODO comment** — **FIXED.** Lines 807–808 add: `// TODO: If mailbox semantics are brought into verification scope, // this stub would need a ghost message queue and dequeue postconditions.`

4. **PMIO uniqueness comment in `wf()`** — **FIXED.** Spec file lines 126–128 now include: "Note: PMIO port uniqueness is NOT enforced, matching the original's `LinkedList` semantics which allows duplicate port numbers."

### Low Issues

1. **Trivially true lemmas** — **Acknowledged, retained.** The prover kept these as documentation/regression tests. Acceptable.

2. **Lifetime elision in `ProcessRefMut`/`ProcessRef`** — **Not addressed (correctly).** Original review said "None needed." Lifetimes are outside Verus's model.

3. **`get_pmio_mut_stub` frame condition** — **Not addressed (correctly).** Original review said "None needed." The abstraction level makes the condition correct.

## New Changes Beyond Previous Issues

The prover also made several improvements not directly requested:

- **Added `get_pmio_stub` and `get_pmio_mut_stub`** private helper stubs, covering the original's `get_pmio` and `get_pmio_mut` private methods. Good for coverage.
- **Added `debug_fmt_stub`** covering the `Debug` trait implementation. Appropriate.
- **Added `ProcessRefMut`/`ProcessRef` opaque types** with accessor stubs, covering the dispatch enums from the original. These were previously listed as out-of-scope; they now have basic frame-condition stubs.
- **Strengthened error-path frame conditions** across `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, `remove_pmio` to include `spec_capabilities_bits()` and `spec_pmio_ports()` preservation. Previously these were missing from error paths, meaning a caller couldn't prove these were unchanged after an error. This is a meaningful correctness improvement.
- **Added `lemma_get_mutex_error_implies_full` and `lemma_get_cond_error_implies_full`** proof lemmas. While trivially true (the ensures is a restatement of the requires), they serve as explicit documentation of the capacity-error relationship.
- **Added SOUNDNESS NOTE on `&self` stubs** (lines 712–718) explaining why `ensures true` is sufficient for `&self` methods — Verus's `&self` is truly immutable with no interior mutability support.

## Issues Found

### Critical

_None._

### High

_None._

### Medium

- **Location:** `get_mutex` / `get_cond` Ok path postcondition (exec)
  - **Description:** When `!already_present`, the postcondition does not explicitly state `self.spec_mutex_count() == old(self).spec_mutex_count() + 1`. This is derivable from the combination of `self.wf()`, `self.spec_has_mutex(mutex_addr@)`, `!old(self).spec_has_mutex(mutex_addr@)`, and the frame condition `forall|a: int| a != mutex_addr@ ==> self.spec_has_mutex(a) == old(self).spec_has_mutex(a)` — together these imply the domain grew by exactly one, and `wf()` ties domain size to `mutex_count`. However, downstream consumers must perform this multi-step derivation themselves. The `already_present` case is similarly missing `self.spec_mutex_count() == old(self).spec_mutex_count()`.
  - **Suggested Fix:** Add explicit postconditions: `!already_present ==> self.spec_mutex_count() == old(self).spec_mutex_count() + 1` and `already_present ==> self.spec_mutex_count() == old(self).spec_mutex_count()`. Same for `get_cond`. This makes the spec easier to consume without requiring re-derivation from `wf()`.

### Low

- **Location:** `set_capability` / `clear_capability` postconditions (exec)
  - **Description:** `set_capability` ensures `self.capabilities.spec_has(capability)` but does not state that other capabilities are preserved. Similarly, `clear_capability` ensures `!self.capabilities.spec_has(capability)` without stating others are preserved. This depends on the `Capabilities` module's specs for `set`/`clear` — if those specs guarantee non-interference for other bits, this is fine. If not, a caller couldn't prove other capabilities survived a `set` call.
  - **Suggested Fix:** Verify the `Capabilities::set`/`clear` postconditions guarantee non-interference for other capability bits. If they do, this is a non-issue. If they don't, add explicit frame conditions here (e.g., `forall|c: Capability| c != capability ==> self.capabilities.spec_has(c) == old(self).capabilities.spec_has(c)`).

- **Location:** Proof file — `lemma_get_mutex_error_implies_full` and `lemma_get_cond_error_implies_full`
  - **Description:** These lemmas' ensures are directly restatable from their requires (`spec_mutexes_full()` is defined as `mutex_count >= MUTEX_MAX()`). The lemma body is empty because Verus trivially discharges the obligation. While harmless, they add no verification value beyond documentation.
  - **Suggested Fix:** Retain as documentation but consider adding a comment noting these are "documentation lemmas."

## Positive Observations

- **All previous review items addressed.** Every actionable item from the A- review was properly fixed with verified code changes, not just documentation claims. The verification still passes cleanly (47/47).
- **No `assume` statements.** Zero `assume` calls in the entire module — excellent for soundness.
- **No regressions.** The changes preserved the existing 47 verification conditions while adding new stubs and lemmas. No existing postconditions were weakened.
- **First-occurrence proof is rigorous.** The `lemma_pmio_first_occurrence_exists` with its recursive helper `lemma_pmio_min_index_helper` is a well-structured inductive proof with correct `decreases` clause. It properly establishes that the `remove_pmio` precondition is always satisfiable when a port exists.
- **Error-path frame conditions now complete.** All error paths specify full frame conditions for all verified fields. This was a real gap in the previous version.
- **Comprehensive documentation.** Trust assumptions T1–T5 are thorough, the SOUNDNESS NOTE on `&self` stubs is well-reasoned, and the "Known over-approximation" annotations on `get_mutex`/`get_cond` are honest about the original's behavior.
- **Coverage is strong.** All 22 public functions and 3 private functions from the original source have corresponding verified functions or frame-condition stubs. The `ProcessRefMut`/`ProcessRef` enums and `Debug` impl are also covered.

## Summary

The prover has thoroughly addressed all items from the previous A- review. The oracle-parameter trust assumption (T5) is now well-documented with explicit caller-burden explanation. The `remove_pmio` spec was meaningfully strengthened with first-occurrence semantics and a return-value postcondition backed by a non-trivial proof. Error-path frame conditions are now complete across all functions. No regressions were introduced.

The remaining medium issue (`get_mutex`/`get_cond` not explicitly stating mutex count change in Ok path) is a usability concern rather than a correctness issue — the information is derivable from `wf()` and frame conditions. The low issues are minor observations about proof organization and capability frame conditions.

This is a well-executed protocol-level verification with strong soundness properties: zero `assume` statements, zero `external_body` on core logic, comprehensive frame conditions, and faithful modeling of the original's behavior including its known over-approximation.
