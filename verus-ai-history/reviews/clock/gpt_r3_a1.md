# Review: clock (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Coverage/Equivalence — standalone_now vs. real now() (exec)**: The verified model replaces `pub fn now() -> SystemTime` with `standalone_now(timer, timer_freq)` returning `(u64, u32)` and taking `timer_freq` as a parameter, but there is no verified wrapper that models the `#[cfg(feature = "pit")]`/fallback selection or the `SystemTime::new()` construction. This means the verification proves arithmetic safety but does not directly prove the behavior of the public API. **Suggested Fix:** Add an exec-level wrapper (or a proof lemma) that models the actual `now()` control flow, uses `axiom_pit_timer_freq_valid()`/`axiom_fallback_timer_freq_valid()` to justify `timer_freq > 0`, and links the computed pair to a modeled `SystemTime::new()` result.
- **Soundness/Assumption — snapshot consistency for get()/ticks()/now() (spec/proof)**: `get()`, `standalone_ticks()`, and `now()` require `spec_no_concurrent_writer_assumption()` obtained only via the `external_body` axiom. The module does not prove that callers disable interrupts or otherwise prevent concurrent writes, so the spec excludes torn reads that could happen if the assumption is violated. **Suggested Fix:** Connect this assumption to kernel-level invariants (e.g., a verified wrapper that disables interrupts around reads), or weaken specs to admit torn-read behavior when the assumption is not established.
- **Equivalence/Trust boundary — timer_handler side effects (exec/proof)**: `timer_handler_model()` only calls `increment()` and assumes that the VM pause check and `ProcessManager::giveup()` cannot modify the clock state. This is asserted but not proved, so equivalence relies on a trust boundary. **Suggested Fix:** Add a proof obligation or module-level invariant that these HAL/scheduler paths cannot touch `TIMER_TICKS`, or encapsulate `TIMER_TICKS` behind accessors verified to be exclusive.
- **Spec strength — PIT frequency only constrained by positivity (proof/spec)**: `axiom_pit_timer_freq_valid()` guarantees only `freq > 0`, so the verification does not establish correctness of time scaling relative to the actual PIT frequency; it only proves safety of arithmetic. **Suggested Fix:** Strengthen the axiom to relate the returned value to `pit::get_timer_frequency()` or add a separate assumption capturing the expected hardware frequency range if correctness of elapsed time matters.

### Low
- **Encapsulation divergence — public fields in verified TimerTicks (exec/spec)**: `TimerTicks` fields are `pub` in the verified model, allowing arbitrary mutation that the original API forbids. While `wf()` is trivial, this weakens the guarantee that only `increment()` mutates the counter. **Suggested Fix:** Hide fields behind `pub(crate)` with accessor spec fns, or add an invariant/ghost token to enforce mutation discipline in verified callers.

## Positive Observations
- The arithmetic around `increment()`, tick combination, and `now()` nanosecond computation is thoroughly specified and proved, including wraparound at `u64::MAX` and avoidance of `SystemTime::new()` failure.
- Trust boundaries are explicitly documented (T1–T5), with lemmas that quantify torn-read consequences and prove `wrapping_add` equivalence.
- Spec/proof separation is clean: exec code is minimal and defers to `clock.spec.rs` and `clock.proof.rs` for properties and lemmas.

## Summary
The verification is strong for sequential arithmetic correctness and documents its trust boundaries clearly, but it does not fully connect to the concrete public APIs and platform behavior. Tightening the links between the verified models and the real `now()`/`timer_handler()` paths, and justifying the snapshot-consistency and PIT-frequency assumptions at system level, would improve coverage and semantic equivalence.
