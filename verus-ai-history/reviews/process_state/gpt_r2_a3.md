# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **External-body stubs still dominate core API (not fixed)** (exec: `process_state.rs` stubs `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`, `debug_fmt_stub`, `ProcessRefMut::state_mut_stub`, `ProcessRef::state_stub`).
  - **Description:** These remain `#[verifier::external_body]` with `ensures true` or frame-only conditions, so VMem copies, mailbox/event/MMIO semantics, PMIO I/O, and accessor dispatch are still unverified. This fails the “no unjustified external_body in core module” requirement and leaves core behavior outside verification.
  - **Suggested Fix:** Replace stubs with modeled exec code or stronger specs with functional postconditions, or move them behind a clearly documented abstraction boundary module with explicit trust assumptions.

- **API/semantics still non-equivalent to original (not fixed)** (exec: `process_state.rs` functions `get_mutex`, `get_cond`, `remove_pmio`, plus copy/IO stubs).
  - **Description:** The verified interface still diverges: `get_mutex`/`get_cond` return `Ghost<nat>` ref counts instead of `Mutex`/`Condvar`, `remove_pmio` returns `Result<(), Error>` instead of `Result<AnyIoPort, Error>`, and copy/IO stubs omit parameters and return values. This breaks semantic equivalence and leaves handle/value correctness unverified.
  - **Suggested Fix:** Introduce abstract token/handle types and model return values, or provide coupling lemmas linking ghost results to concrete return values and signatures.

### Medium
- **Ref-count decrement/drop modeling still absent (not fixed)** (spec/exec: ref-count model plus `put_mutex`/`put_cond`).
  - **Description:** The model still only increments ref counts and uses a `ref_count_at_threshold` oracle for cleanup, with no explicit representation of clone drops. This prevents reasoning about when ref counts decrease and weakens cleanup/liveness arguments.
  - **Suggested Fix:** Model handle lifetimes or add spec/exec decrement hooks and tie the threshold oracle to updated ghost state.

### Low
- **Hard-coded capacity constants remain** (spec: `MUTEX_MAX`/`COND_MAX`; exec: `MUTEX_MAX_EXEC`/`COND_MAX_EXEC`).
  - **Description:** Constants are still fixed at 32 and only documented as matching configuration. If kernel config changes, verification can silently diverge.
  - **Suggested Fix:** Import or generate constants from build configuration to keep spec/exec aligned.

## Positive Observations
- The protocol-level invariants and collection semantics remain clear and consistent.
- PMIO removal still models first-occurrence semantics correctly.
- Spec/proof remain cleanly separated.

## Summary
No substantive fixes were found for the prior review issues: the external stubs, API mismatches, missing ref-count decrements, and hard-coded constants persist. The verification remains a protocol model rather than a semantically equivalent, fully sound verification of the original module. Significant modeling work is still required.
