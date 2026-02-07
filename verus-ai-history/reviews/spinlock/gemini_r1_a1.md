# Review: spinlock (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **API Divergence (`&mut self` vs `&self`)**: 
  - **Location**: `Spinlock::lock` and `Spinlock::try_lock` (exec)
  - **Description**: The verified implementation uses `&mut self` for locking methods, whereas the original implementation uses `&self` (relying on `AtomicBool` interior mutability). This means the verification captures the sequential state machine logic but does not model concurrent access or thread safety. The borrow checker prevents the verified `Spinlock` from being shared across threads, which is the primary use case of the component.
  - **Suggested Fix**: This is likely a limitation of the current verification tools regarding atomic types. It should be clearly documented that this verification proves sequential correctness properties (protocol adherence) rather than concurrent safety.

- **Strong Precondition on `lock`**:
  - **Location**: `Spinlock::lock` (exec)
  - **Description**: The `lock` function requires `old(self).spec_is_unlocked()`. In a real spinlock, `lock()` is called when the lock state is unknown (and potentially locked), and it spins until acquired. The current spec effectively models `lock()` as `try_lock().expect("success")`, verifying the transition but not the waiting behavior or progress.
  - **Suggested Fix**: Acknowledge that liveness (progress) and the spinning logic are outside the scope of this verification.

### Low
- **RAII Divergence**:
  - **Location**: `SpinlockGuard` vs `Spinlock::unlock`
  - **Description**: The original code uses `SpinlockGuard` and `Drop` to automatically release locks. The verified code requires manual calls to `unlock` with a `LockToken`.
  - **Suggested Fix**: Standard limitation of Verus (no `Drop` support yet), but creates a usage gap between verified and original code.

- **Missing Public API**:
  - **Location**: `Spinlock::is_locked` (exec)
  - **Description**: The verified code adds `is_locked()`, which does not exist in the original `src/kernel/src/pm/sync/spinlock.rs`.
  - **Suggested Fix**: Either add `is_locked` to the original source or mark it as `#[doc(hidden)]` / helper-only in verified code.

## Positive Observations
- **Instance Identity**: The use of `Ghost<nat>` for `id` correctly models the binding between a lock instance and its tokens/guards. The `lemma_token_instance_isolation` effectively proves that a guard from one lock cannot mistakenly unlock another.
- **State Machine Correctness**: The `try_lock` logic correctly models the atomic Compare-And-Swap (CAS) state transition in a sequential context.
- **Clean Separation**: The split between executable code, specifications, and proofs is clean and well-organized.
- **Invariants**: The `wf()` invariant correctly enforces that tokens are only outstanding when the lock is held.

## Summary
The verification provides a high-confidence model of the `Spinlock` state machine protocol. It proves that the lock transitions correctly between locked and unlocked states and that the "guard" (token) mechanics preserve mutual exclusion properties (e.g., token conservation, instance binding). However, by modeling the state with `bool` and `&mut self` instead of `AtomicBool` and `&self`, the verification abstracts away the core complexity of a spinlock: concurrency and atomicity. It is a strong sequential verification of a concurrent primitive's logic.
