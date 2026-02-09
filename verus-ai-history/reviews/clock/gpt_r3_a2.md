# Review: clock (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence — public now() control flow still not modeled (exec/proof)**: The new lemmas (`lemma_now_fallback_valid`, `lemma_now_pit_valid`, `lemma_now_always_valid`) are spec-only and do not introduce an exec-level wrapper that mirrors the real `now()` signature (`pub fn now() -> SystemTime`) or the `#[cfg(feature = "pit")]`/fallback selection. `standalone_now()` still takes `timer_freq` as a parameter, so the verified model does not prove the actual API’s control flow or the `SystemTime::new()` construction. **Suggested Fix:** Add a model function that selects `timer_freq` via the PIT/fallback branches and ties the result to a modeled `SystemTime::new()`; then prove it using the new lemmas.
- **Soundness/Assumption — snapshot consistency remains axiomatic (spec/proof)**: `get()`, `now()`, `standalone_ticks()`, and `standalone_now()` still require `spec_no_concurrent_writer_assumption()` obtained only via the `external_body` axiom. There is no verified wrapper that disables interrupts or otherwise establishes this assumption, so the proof excludes torn-read behaviors without system-level justification. **Suggested Fix:** Prove or model the interrupt-disable discipline around reads, or weaken specs to admit torn reads when the assumption is not established.
- **Equivalence/Trust boundary — timer_handler side effects still unproved (exec/proof)**: `timer_handler_model()` continues to assume that the VM pause check and `ProcessManager::giveup()` cannot modify the counter, but there is no proof or encapsulation preventing such side effects. **Suggested Fix:** Prove non-interference or encapsulate `TIMER_TICKS` access behind verified APIs.
- **Spec strength — PIT frequency still only constrained by positivity (proof/spec)**: `axiom_pit_timer_freq_valid()` still guarantees only `freq > 0`, so correctness of elapsed time relative to the actual PIT frequency is not proved. The new lemmas only propagate this weak guarantee. **Suggested Fix:** Strengthen the axiom to relate the returned value to `pit::get_timer_frequency()` or to a verified frequency range if time correctness matters.

### Low
- **Encapsulation divergence — public fields remain (exec/spec)**: `TimerTicks` fields are still `pub`, allowing arbitrary mutation not possible in the original code. This weakens the guarantee that only `increment()` mutates the counter. **Suggested Fix:** Hide fields and expose spec accessors, or add a ghost capability to enforce mutation discipline.

## Positive Observations
- Added lemmas (`lemma_now_fallback_valid`, `lemma_now_pit_valid`, `lemma_now_always_valid`) clarify the PIT vs fallback paths and formally justify the `SystemTime::new()` precondition for each path at the spec level.
- Verification still proves the core arithmetic safety, wraparound behavior, and nanosecond bounds.

## Summary
The update adds useful spec-level lemmas for the `now()` branches, but the core gaps from the previous review remain: the real public API control flow is still not modeled, snapshot consistency remains purely axiomatic, and handler side effects are still trusted. Addressing these with exec-level wrappers or system-level invariants would materially strengthen equivalence and soundness.
