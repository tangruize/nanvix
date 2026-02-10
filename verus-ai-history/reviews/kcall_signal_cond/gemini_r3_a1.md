# Review: kcall_signal_cond (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Argument Order Mismatch**: The `signal_cond_model` function reorders arguments compared to the original `signal_cond`. Original: `(pid, tid, cond_addr, broadcast)`. Model: `(cond_addr, broadcast, pid, tid)`. This reduces traceability.
  - *Fix*: Reorder arguments in `signal_cond_model` to match the original function signature.
- **Type Abstraction**: The model uses `u32` for `cond_addr` instead of a wrapper type mirroring `ConditionAddress`. While `lemma_architecture_guard` ensures safety, using a newtype wrapper would better reflect the original source structure.
  - *Fix*: Introduce a `ConditionAddress` ghost type or wrapper in the spec.

## Positive Observations
- **Faithful Resource Modeling**: The verification explicitly models and proves the behavior where `ProcessManager::put_cond` is skipped if `notify` fails (`lemma_notify_error_skips_put_cond`). This correctly reflects a potential resource leak in the implementation while ensuring the model remains faithful.
- **Robust Error Propagation**: The 3-step pipeline (get, notify, put) with short-circuiting is rigorously modeled and proven (`lemma_short_circuit_get_cond`, etc.).
- **Architecture Guards**: The inclusion of `lemma_architecture_guard` to explicitly verify x86-32 assumptions (`u32` vs `usize`) is a best practice.
- **Trust Boundary Definition**: The external bodies (`get_cond_model`, `notify_model`, etc.) are clearly defined with appropriate pre/post-conditions, cleanly separating the kernel call logic from the complex PM/Condvar internals.

## Summary
The verification of `kcall_signal_cond` is excellent. It provides a highly faithful model of the original code, including edge cases like the resource cleanup behavior on error. The specifications are strong, covering error propagation, broadcast semantics, and safety preconditions. The separation of concerns between the kernel call pipeline and the underlying subsystems is handled correctly via external bodies.
