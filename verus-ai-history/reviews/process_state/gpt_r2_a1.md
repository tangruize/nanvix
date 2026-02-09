# Review: process_state (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Coverage/Soundness gap via external_body stubs** (exec: `process_state.rs` stubs such as `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `add_event_stub`, `remove_event_stub`, `post_message_stub`, `receive_message_stub`, `add_mmio_stub`, `remove_mmio_stub`, `read_pmio_stub`, `write_pmio_stub`, `vmem_stub`, `vmem_mut_stub`, `get_pmio_stub`, `get_pmio_mut_stub`, `debug_fmt_stub`, `ProcessRefMut::state_mut_stub`, `ProcessRef::state_stub`).
  - **Description:** These replace real functions with `#[verifier::external_body]` and mostly `ensures true` or frame-only conditions. This means core behaviors (mailbox, events, MMIO, VMem copies, PMIO IO, accessor dispatch) are unverified and could arbitrarily violate semantics, failing the “no unjustified external_body in core module” requirement and leaving large parts of the API uncovered.
  - **Suggested Fix:** Replace stubs with modeled exec versions or richer specs (at least return/side-effect guarantees), and avoid `ensures true` for any function in this module. If full modeling is infeasible, add an explicit abstraction boundary module and move the external stubs there.

- **Non-equivalent APIs/return values in the verified model** (exec: `process_state.rs` functions `get_mutex`, `get_cond`, `remove_pmio`, plus stubs for copy/IO methods).
  - **Description:** The verified code is not ABI-compatible: `get_mutex`/`get_cond` return ghost reference counts instead of `Mutex`/`Condvar`, `remove_pmio` returns `Result<(), Error>` instead of `Result<AnyIoPort, Error>`, and copy/IO methods have no parameters. This breaks semantic equivalence with the original and leaves correctness of returned handles/values unverified.
  - **Suggested Fix:** Introduce abstract token types for `Mutex`/`Condvar`/`AnyIoPort` and return them in the verified API, or provide coupling lemmas that relate the model’s ghost results to the concrete return values and signatures.

### Medium
- **Ref-count model cannot represent decrements/drops** (spec/exec: `process_state.spec.rs` ref-count model + `get_mutex`/`put_mutex`/`get_cond`/`put_cond` in `process_state.rs`).
  - **Description:** Ghost ref counts only increase (on get) and never decrease, yet `put_mutex`/`put_cond` depend on a `ref_count_at_threshold` oracle tied to the ghost count. This makes it impossible to model clone drops and can over-constrain callers or block proofs of cleanup, weakening equivalence and any liveness/cleanup reasoning.
  - **Suggested Fix:** Add an explicit spec/exec hook to decrement ghost ref counts when clones are dropped (or model returned handles with drop lemmas), and tie `ref_count_at_threshold` to that updated ghost state.

### Low
- **Hard-coded capacity constants** (spec: `MUTEX_MAX`/`COND_MAX` in `process_state.spec.rs`).
  - **Description:** The constants are fixed at 32 and only documented as matching kernel config. If `build/kernel_config.toml` changes, the verification silently becomes unsound.
  - **Suggested Fix:** Import the config constants into the model (or generate them) so spec and code stay aligned.

## Positive Observations
- The protocol-level model is clearly documented and the spec/proof split is clean, with `wf()` invariants and lemmas that preserve PID immutability, capacity bounds, and map/sequence consistency.
- `get_mutex`/`put_mutex` and `get_cond`/`put_cond` explicitly model reference-count thresholds and capacity checks, matching the original control flow.
- The PMIO removal spec correctly encodes “first occurrence” semantics consistent with `LinkedList::iter().position()`.

## Summary
The verification proves a well-structured protocol model for capabilities, bounded resource maps, and PMIO tracking, but it stops short of semantic equivalence to the real kernel implementation. Key behaviors (VMem, mailbox, events, MMIO/PMIO I/O) are left as `external_body` stubs and several APIs return different types or omit parameters, which materially weakens coverage and soundness. Tightening the abstraction boundary, modeling returned handles, and adding ref-count decrement semantics would substantially improve fidelity and completeness.
