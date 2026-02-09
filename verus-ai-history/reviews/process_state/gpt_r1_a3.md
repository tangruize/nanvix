# Review: process_state (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Coverage/equivalence still incomplete for module APIs (exec)**: New stubs for `ProcessRefMut`/`ProcessRef`, `get_pmio`/`get_pmio_mut`, and `Debug` exist, but their signatures and semantics do not match the originals (e.g., accessors return no `&ProcessState`, helper stubs omit `port_number` and return no port). This does not establish semantic equivalence or a verified version of the original APIs. **Suggested Fix**: Provide stubs with matching signatures (returning references or abstract tokens), or add verified wrappers that refine to the original behavior.
- **Oracle-parameter reliance remains (exec)**: `get_mutex`, `put_mutex`, `get_cond`, `put_cond`, and `remove_pmio` still depend on caller-supplied booleans/indices with no verified wrapper that computes them from an exec model of the collections. The “oracle parameter” explanation is documentation only and does not prove equivalence with the original code. **Suggested Fix**: Add wrappers that compute these values from modeled BTreeMap/LinkedList state or prove a refinement relation between an exec model and the ghost state.

### Medium
- **`external_body` stubs with `ensures true` remain underspecified (exec)**: `copy_from_user_unaligned_stub`, `copy_to_user_unaligned_stub`, `read_pmio_stub`, `vmem_stub`, `get_pmio_stub`, and `debug_fmt_stub` still provide no explicit frame conditions. The new “no interior mutability” note is not a formal proof and does not prevent side effects via opaque types. **Suggested Fix**: Add explicit frame-condition ensures or introduce opaque state tokens with non-interference proofs.
- **Liveness/cleanup still unmodeled in ref-count spec (spec/exec)**: The model still lacks a decrement operation or assumption reflecting dropped clones, so eventual cleanup of mutexes/condvars cannot be proven. The explanatory note does not add a proof obligation. **Suggested Fix**: Introduce a decrement step or explicit assumptions about external drops, then prove eventual removal properties.

### Low
- **Return-value identity still abstracted away (exec/spec)**: `get_mutex`/`get_cond` still return only ghost ref-counts and `remove_pmio` does not model the removed port, limiting reasoning about resource identity/ownership. **Suggested Fix**: Add abstract tokens or ghost values for returned resources and relate them to the map/sequence state.

## Positive Observations
- The previous fixes to capacity error conditions and first-occurrence PMIO removal remain in place.
- Additional stubs were added for previously missing helpers and accessor enums, improving coverage documentation.

## Summary
The update adds stubs and commentary, but the main gaps persist: the new stubs do not match original signatures, oracle parameters still stand in for executable computation, and several `external_body` stubs remain underspecified. Verification is improved but still not complete or fully sound with respect to the original module.
