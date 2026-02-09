# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **External-body stubs still dominate core API (not fixed)** (exec: `process_state.rs` stubs `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`, `debug_fmt_stub`, `ProcessRefMut::state_mut_stub`, `ProcessRef::state_stub`).
  - **Description:** These functions remain `#[verifier::external_body]` with `ensures true` or frame-only conditions, so large parts of the original module’s behavior (VMem copies, mailbox/event/MMIO semantics, PMIO I/O, accessor dispatch) are still unverified. This does not satisfy the “no unjustified external_body in core module” criterion and leaves core functionality outside verification.
  - **Suggested Fix:** Replace stubs with modeled exec code or stronger specs (including functional postconditions), or move them behind a clearly documented abstraction boundary module with explicit trust assumptions.

- **API/semantics still non-equivalent to original (not fixed)** (exec: `process_state.rs` functions `get_mutex`, `get_cond`, `remove_pmio`, plus stubs for copy/IO methods).
  - **Description:** The verified interface still diverges from the real API: `get_mutex`/`get_cond` return `Ghost<nat>` ref counts instead of `Mutex`/`Condvar` handles, `remove_pmio` returns `Result<(), Error>` instead of `Result<AnyIoPort, Error>`, and copy/IO stubs omit parameters and values. This breaks semantic equivalence and leaves correctness of returned handles/values unverified.
  - **Suggested Fix:** Introduce abstract token types for returned handles and model the relevant return values, or provide coupling lemmas that relate ghost results to concrete return values/signatures.

### Medium
- **Ref-count decrement/drop modeling still absent (not fixed)** (exec/spec: `process_state.rs` comments around ref-counts and `put_mutex`/`put_cond`, plus `process_state.spec.rs` ref-count model).
  - **Description:** The model still only increments ref counts and relies on a `ref_count_at_threshold` oracle for cleanup, without any explicit representation of clone drops. This prevents reasoning about when ref counts decrease and weakens cleanup/liveness guarantees.
  - **Suggested Fix:** Model handle lifetimes or add explicit spec/exec decrement hooks and tie the threshold oracle to that updated state.

### Low
- **Hard-coded capacity constants remain** (spec: `ProcessState::MUTEX_MAX`/`COND_MAX`; exec: `MUTEX_MAX_EXEC`/`COND_MAX_EXEC`).
  - **Description:** Constants are still fixed at `32` and only documented as matching configuration. If the kernel config changes, the verification can silently diverge.
  - **Suggested Fix:** Import or generate the constants from the build config to keep spec and exec aligned.

## Positive Observations
- The protocol-level invariants (`wf`) and collection semantics are still clearly specified and internally consistent.
- PMIO removal continues to model “first occurrence” semantics correctly.
- The proof file still cleanly separates lemmas from exec logic.

## Summary
No substantive changes addressing the prior review issues were found: the same external-body stubs, API mismatches, missing ref-count decrements, and hard-coded constants remain. The verification still serves as a protocol model rather than a semantically equivalent, fully sound verification of the original module. Significant modeling work is needed to close the remaining gaps.
