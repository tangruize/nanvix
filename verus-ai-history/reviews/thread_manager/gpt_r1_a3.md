# Review: thread_manager (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **ThreadRefMut coverage still incomplete** (thread_manager.rs exec/spec): The new `ThreadRefMutModel` only models the read aspect via `thread_state()` and explicitly leaves mutation as a trust boundary; there is still no modeled/verified equivalent of `thread_state_mut()` with mutation effects. This weakens equivalence and leaves the core mutability contract unverified. **Suggested Fix:** Introduce a verified abstract mutation contract (e.g., a spec function describing allowed mutations) or model `thread_state_mut()` via a separate ghost state/permission token to preserve invariants.
- **Overflow behavior still assumed** (thread_manager.rs exec `create_thread`): The precondition `next_id.value < i32::MAX` remains, but the original code has no overflow check. This is still a semantic gap and an unverifiable system-wide assumption. **Suggested Fix:** Add a runtime check/error path in the exec model or explicitly model wraparound and adjust uniqueness properties.
- **Admission time still omitted** (thread_manager.rs exec boundary ReadyThread): The boundary ReadyThread still omits `admission_time`, which the real constructor sets and scheduling can depend on. This remains a semantic gap for clients relying on that property. **Suggested Fix:** Import the ready module’s ReadyThread model or include admission_time in this boundary model with corresponding ensures.

### Low
- **OOM behavior still elided** (thread_manager.rs exec docs): The model continues to assume allocation success without modeling the original panic-on-OOM behavior. **Suggested Fix:** Document this as an explicit fatality assumption or model an abort path if required by system safety proofs.

## Positive Observations
- **ThreadRef coverage improved:** The new `ThreadRefModel` validates dispatch identity preservation across variants, closing the prior omission for `ThreadRef::thread_state()`.
- Resource preservation and ID allocation properties remain specified and proven, with clean exec/spec/proof separation and no `assume`/`external_body` in this module.

## Summary
The update meaningfully improves coverage for `ThreadRef` dispatch, but `ThreadRefMut` mutation semantics remain unverified, and overflow/admission-time/OOM assumptions persist. Verification is better but still incomplete against full equivalence and soundness expectations. Addressing the remaining medium issues would move this module toward a full pass.
