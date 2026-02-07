# Review: fence (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Vacuous `wait` verification (exec):** The verified `wait` function has a precondition `requires self.spec_is_satisfied()`. This assumes the postcondition before execution, effectively making the function a no-op that proves "if satisfied, then satisfied". It completely ignores the actual runtime behavior (spin loop) and fails to model the blocking nature of the primitive.
- **Concurrency Model Mismatch (exec):** The verified `signal` function requires `&mut self`, whereas the original implementation uses `&self` with `AtomicUsize`. This means the verification only proves correctness for sequential access, failing to capture the concurrent usage patterns (multiple threads signaling a shared fence) that are central to the component's purpose.

### Medium
- **Over-restrictive `signal` precondition (exec):** The verified `signal` requires `spec_is_waiting()`, forbidding calls on an already-satisfied fence. The original implementation allows over-signaling (using `fetch_add`). This divergence means the verified model rejects valid runtime traces where a fence might receive more signals than `total` (e.g., in `kmain` startup as noted in docs).
- **Missing `const` on `new` (exec):** The verified `new` function is not `const`, unlike the original. This restricts where the verified structure can be initialized.

### Low
- **Type Mismatch (exec):** The verified code uses `usize` instead of `AtomicUsize`. While documented as a model, this prevents the verified code from being a drop-in replacement or ensuring that atomic orderings (`Acquire`/`Release`) are used correctly.

## Positive Observations
- **Clear Documentation:** The file header explicitly details the "Verification Model", "Verification Scope", and "Trust Boundaries", demonstrating awareness of the limitations (sequential model vs concurrent runtime).
- **Clean Split:** The separation between `exec`, `spec`, and `proof` files is excellent and follows the project's best practices.
- **Protocol Correctness:** The arithmetic properties of the fence protocol (counting, total, satisfaction logic) are well-specified and proven.

## Summary
The verification of `fence` is a high-quality "sequential model" of the synchronization primitive. It successfully proves that the counting logic is correct and that the state machine transitions (in a single-threaded context) preserve well-formedness. However, it abstracts away the two most critical aspects of the component: **concurrency** (via `AtomicUsize`) and **blocking** (via the spin loop). The verified `wait` function essentially assumes the work is already done. While the documentation acknowledges these gaps, the result is a specification that offers limited assurance about the actual runtime behavior in a concurrent OS kernel.
