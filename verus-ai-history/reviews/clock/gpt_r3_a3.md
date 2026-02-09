# Review: clock (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence — public `now()` still not modeled end-to-end (exec)**: New exec-level models (`now_fallback_model`, `now_pit_model`) improve branch coverage, but there is still no verified function with the real signature `pub fn now() -> SystemTime` nor a single wrapper that selects the `#[cfg]` branch and constructs a modeled `SystemTime`. The PIT path remains parameterized by `timer_freq` rather than tied to the HAL call. **Suggested Fix:** Add a top-level model that mirrors the real API signature and branch selection, invoking `axiom_pit_timer_freq_valid()` to justify the PIT frequency and proving `SystemTime::new()` success for the constructed value.
- **Soundness/Assumption — snapshot consistency remains axiomatic (spec/proof)**: `get()`, `now()`, `standalone_ticks()`, and the new `now_*_model()` functions still require `spec_no_concurrent_writer_assumption()` from an `external_body` axiom, with no verified wrapper that disables interrupts or otherwise establishes the assumption. **Suggested Fix:** Prove or model the interrupt-disable discipline around reads, or weaken specs to admit torn reads without the assumption.
- **Equivalence/Trust boundary — timer_handler side effects still unproved (exec/proof)**: `timer_handler_model()` still assumes that VM pause checks and `ProcessManager::giveup()` cannot modify the clock counter, with no proof or encapsulation enforcing non-interference. **Suggested Fix:** Prove non-interference or encapsulate `TIMER_TICKS` behind verified APIs.
- **Spec strength — PIT frequency still only constrained by positivity (proof/spec)**: `axiom_pit_timer_freq_valid()` still guarantees only `freq > 0`, so correctness of elapsed time relative to the actual PIT frequency is not proved. **Suggested Fix:** Strengthen the axiom to relate the value to `pit::get_timer_frequency()` or to a verified frequency range if time correctness matters.

### Low
- **Encapsulation divergence — public fields remain (exec/spec)**: `TimerTicks` fields are still `pub`, allowing arbitrary mutation not possible in the original code. This weakens the guarantee that only `increment()` mutates the counter. **Suggested Fix:** Hide fields and expose spec accessors, or add a ghost capability to enforce mutation discipline.

## Positive Observations
- The new `now_fallback_model` and `now_pit_model` functions add exec-level coverage for the PIT vs fallback branches, and their postconditions align with the existing `SystemTime::new()` precondition proofs.
- Core arithmetic, wraparound behavior, and nanosecond bounds remain fully verified and stable.

## Summary
The update improves branch-level modeling of `now()`, but the key soundness and equivalence gaps remain: the public API is still not modeled end-to-end, snapshot consistency is axiomatic, handler side effects are trusted, and PIT frequency correctness is only a positivity guarantee. Verification is stronger than before but still incomplete for full API-level equivalence.
