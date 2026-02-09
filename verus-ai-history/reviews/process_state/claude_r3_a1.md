# Review: process_state (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `get_mutex` / `get_cond` (exec, lines 306 and 465)
  - **Description:** The original `get_mutex` takes a concrete `MutexAddress` and uses `BTreeMap::entry().or_insert_with().clone()`, which does NOT reject existing keys at capacity — `entry()` returns the existing entry without growing the map. The original only fails when a truly new insertion would exceed capacity. The verified model's comment acknowledges this as a "known over-approximation (inherited from original)" but this is incorrect: the original code **does** over-approximate (checks `len() >= MUTEX_OPEN_MAX` before `entry()`), and the verified model faithfully replicates this. However, the verified model's documentation in the spec comment at line 289-291 states this is "inherited from original" — which is accurate. This is not actually an issue in the verified code, but rather in the original source. The verified model correctly captures the original's behavior. **Downgraded to informational.**
  - **Revised description:** The `get_mutex` and `get_cond` signatures differ materially from the original: they take `Ghost<int>` addresses and `bool` oracle parameters instead of concrete `MutexAddress`/`ConditionAddress` types. While this is the standard Verus oracle-parameter pattern and is well-documented (Trust Assumption T5), it means the **caller** must correctly compute the `already_present` boolean. The precondition `already_present == old(self).spec_has_mutex(mutex_addr@)` constrains correctness, but in practice the gap between ghost state and the runtime `BTreeMap` is bridged only by trust assumption T2.
  - **Suggested Fix:** Document in the trust assumptions that the oracle-parameter pattern shifts the burden of correctness to the caller, and that the BTreeMap↔ghost map correspondence (T2) is the critical link. This is already partially done but could be more explicit.

- **Location:** `put_mutex` (exec, line 385) — `ref_count_at_threshold` semantic mismatch potential
  - **Description:** In the original `put_mutex`, the `extract_if` predicate checks `mutex_addr == addr && mutex.reference_count() <= 2`. The `extract_if` iterates over **all** entries but the predicate filters to only the matching address. The verified model correctly captures this single-entry semantics. However, the `ref_count_at_threshold` oracle parameter is only constrained when `contains` is true (line 394). If `contains` is false, `ref_count_at_threshold` is unconstrained — which is fine since it's unused in the error path. No issue here, just noting the pattern is correct.
  - **Suggested Fix:** None needed.

### Medium

- **Location:** `new()` (exec, line 186) — Missing `Vmem` parameter
  - **Description:** The original `ProcessState::new(pid, vmem)` takes a `Vmem` parameter. The verified model's `new(pid)` omits it since Vmem is abstracted away. This is a deliberate modeling choice (Vmem is an opaque boundary type), but it means the verified constructor is not signature-equivalent to the original. Any caller porting to use the verified model would need adaptation.
  - **Suggested Fix:** Add a brief note in the documentation that the `Vmem` parameter is omitted because it is an opaque boundary type outside the verification scope. _(Already partially addressed in the trust assumptions section.)_

- **Location:** `remove_pmio` (exec, line 632) — Original returns `Result<AnyIoPort, Error>`, verified returns `Result<(), Error>`
  - **Description:** The original `remove_pmio` returns the removed `AnyIoPort` value (`Ok(self.pmio.remove(index))`). The verified model returns `Ok(())`, losing the return value. This means the spec cannot express properties about what was removed (e.g., the returned port has the requested port number).
  - **Suggested Fix:** Consider adding a ghost postcondition like `result is Ok ==> old(self).ghost_pmio@[found_idx@] == port_number@` to capture the return-value identity. This is already implied by the precondition but making it explicit as a postcondition would strengthen the spec.

- **Location:** `receive_message_stub` (exec, line 798) — Takes `&mut self` but original returns `Option<Message>`
  - **Description:** The original `receive_message` dequeues from the mailbox, which is a meaningful state change (message is consumed). The stub correctly claims frame conditions on verified fields, but does not model the mailbox state change at all. If mailbox state is ever brought into scope, this stub would need revision. The `&mut self` is correct (mailbox is modified), but the ensures clause makes no mention of any mailbox-related ghost state.
  - **Suggested Fix:** Acceptable as-is given mailbox is out of scope. Add a TODO comment noting that bringing mailbox into verification scope would require revisiting this stub.

- **Location:** Spec file — `wf()` invariant does not include PMIO uniqueness
  - **Description:** The original `remove_pmio` uses `iter().position()` which finds the first matching port, implying ports can be duplicated in the list. The `wf()` predicate does not enforce uniqueness of PMIO port numbers. This is actually correct (the original LinkedList allows duplicates), but it means the model faithfully captures a potentially surprising behavior: adding the same port twice creates two entries, and removing it removes only the first.
  - **Suggested Fix:** Consider adding a comment in `wf()` explicitly noting that PMIO port uniqueness is not enforced, matching the original's LinkedList semantics.

### Low

- **Location:** Proof file — Some lemmas are trivially true
  - **Description:** Several proof lemmas (e.g., `lemma_set_capability_preserves_pid`, `lemma_mutex_change_preserves_pid`, `lemma_get_existing_mutex_no_change`, `lemma_get_mutex_error_implies_full`, `lemma_mutex_count_bounded`) have empty proof bodies and are direct consequences of the definitions. While not harmful, they add bulk without significant verification value — Verus auto-proves them.
  - **Suggested Fix:** These serve as documentation and regression tests. Acceptable to keep, but could add a note that these are "trivial-by-construction" lemmas for documentation purposes.

- **Location:** `ProcessRefMut` / `ProcessRef` (exec, lines 1006, 1034) — Lifetime elision
  - **Description:** The original types are `ProcessRefMut<'a>` and `ProcessRef<'a>` with lifetime parameters. The verified model drops lifetimes entirely, using `_phantom: ()`. This is expected since Verus doesn't model Rust lifetimes, but worth noting for completeness.
  - **Suggested Fix:** None needed. Document that lifetime-based safety is outside Verus's model.

- **Location:** `get_pmio_mut_stub` (exec, line 936) — Frame condition may be too strong
  - **Description:** The stub claims `self.spec_pmio_ports() == old(self).spec_pmio_ports()`, asserting the PMIO sequence is unchanged. The original `get_pmio_mut` returns `&mut AnyIoPort`, which allows the caller to mutate the port's internal state (e.g., cached values). However, since `ghost_pmio` only tracks port numbers (not port state), this frame condition is correct: the port number sequence doesn't change even if an individual port is mutated via the returned reference.
  - **Suggested Fix:** None needed. The abstraction level (port numbers only) makes this correct.

## Positive Observations

- **No `assume` statements.** The core module contains zero `assume` calls, which is excellent for soundness. All 47 verification conditions are proved.
- **Comprehensive frame conditions.** Every `&mut self` function specifies what it does NOT change, providing strong non-interference guarantees across operations. This is particularly well done for the stub functions.
- **Well-formedness preservation.** All operations (core and stubs) ensure `self.wf()` in their postconditions, maintaining the global invariant.
- **Faithful modeling of the original's over-approximation.** The capacity check bug in the original (rejecting existing keys at capacity) is accurately captured and documented rather than "fixed" in the model.
- **Thorough trust assumptions.** The T1-T5 trust assumptions are clearly enumerated, explaining exactly what is trusted and why. The Arc reference-counting model (T3) is particularly well-explained.
- **Clean spec/proof/exec separation.** Spec functions, proof lemmas, and executable code are properly separated into their respective files with clear organization.
- **Verification passes cleanly.** All 47 verification conditions pass with no errors in ~5 seconds.
- **PMIO first-occurrence proof.** The `lemma_pmio_first_occurrence_exists` with its recursive helper `lemma_pmio_min_index_helper` is a non-trivial proof that correctly establishes the precondition for `remove_pmio` is satisfiable.
- **Constants validated.** MUTEX_MAX (32) and COND_MAX (32) match `build/kernel_config.toml` values.

## Summary

This is a solid protocol-level verification of the ProcessState module. The core state management logic — PID immutability, capability delegation, bounded mutex/condvar collection management with reference-counted cleanup, and PMIO port tracking — is fully verified with no `assume` statements and comprehensive frame conditions.

The verification correctly models the original's behavior including its over-approximation in capacity checks. The oracle-parameter pattern for bridging ghost/exec boundaries is well-documented. All `external_body` annotations are confined to HAL/IPC boundary stubs (not core logic), which is appropriate.

The main areas for improvement are: (1) the `remove_pmio` return value is not captured in the spec, (2) some proof lemmas are trivially true and serve only as documentation, and (3) the mailbox abstraction could benefit from a TODO noting future verification scope expansion. None of these are correctness issues — they are opportunities to strengthen an already strong verification.
