# Review: mutex (gemini-3-pro-preview) - Round 3

## Grade: C+

## Status: Failed

## Summary
The documentation regarding the sequential model and liveness limitations is now excellent. However, the verification still fails to ensure its primary property: **mutual exclusion**.

The prover rejected the fix for "Token Forgery" (Trust Assumption T3) by arguing that making the `view` field private prevents specification access due to Verus opacity rules. While technically true for the *specific* suggestion of "private field + public getter", it ignores the standard Rust/Verus pattern for solving this exact problem: **preventing construction while allowing access**.

By leaving `MutexToken` fully constructible by the public, the verification allows any client to forge a token and unlock a mutex they do not own, violating safety. This is not a "Trust Assumption"; it is a security vulnerability in the verified interface.

## Remaining Issues

### High
- **Token Forgery Vulnerability (Fixable)**
  - **Location:** `mutex.spec.rs` (`MutexToken`)
  - **Current State:** `pub tracked struct MutexToken { pub ghost view: MutexView }`
  - **Problem:** Any external code can write `let t = MutexToken { view: ... }` and use it to unlock a mutex.
  - **Rejection Analysis:** The prover correctly noted that making `view` private makes it hard to inspect in specs. However, the goal is to prevent *construction*, not *inspection*.
  - **Required Fix (The "Private Marker" Pattern):**
    1. Keep `view` as `pub ghost` (so specs can read it).
    2. Add a private, zero-sized ghost field to `MutexToken` (e.g., `ghost _marker: PrivateMarker`).
    3. Define `pub struct PrivateMarker;` (or similar) but *do not make the struct definition visible/constructible* outside the module (e.g., keep the struct or its fields private to the module).
    4. Update `try_lock`/`lock` to construct the token with this marker.
    This prevents external construction (cannot name/construct the marker) while allowing full public read access to `view`.

### Low
- **Global ID Uniqueness (T1)**
  - **Location:** `mutex.rs`
  - **Status:** Unresolved but acceptable for this iteration given the focus on the token issue.

## Next Steps
Implement the "Private Marker" pattern to secure `MutexToken`. This requires:
1. Defining a private marker type in `mutex.rs` (or `spec.rs` if visibility allows).
2. Adding it to `MutexToken`.
3. Updating the constructor sites in `mutex.rs`.
