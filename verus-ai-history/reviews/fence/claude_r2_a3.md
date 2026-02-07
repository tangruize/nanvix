# Review: fence — Round 2, Attempt 3 (claude_r2_a3)

## Grade: A

## Previous Issue Disposition (r2_a2 → r2_a3)

### Issue #9 (LOW): Doc inaccuracy `Fence::new(ncores)` vs `Fence::new(ncores - 1)` (fence.rs:64-69)

**Status: Fixed**

The previous wording "`Fence::new(ncores)` is created in `kmain.rs`" was inaccurate because `startup::init` is called with `ncores - 1` (line 347 of `kmain.rs`), not `ncores`.

The new wording (lines 64-66):
> "the startup fence is created via `startup::init(n)` which calls `Fence::new(n)`, and each application core calls `signal()` exactly once during boot"

Verified against `src/kernel/src/kmain.rs`:
- Line 115: `pub fn init(ncores: usize)` → line 117: `Fence::new(ncores)` — init does pass its argument directly to `Fence::new`. ✓
- Line 347: `startup::init(ncores - 1)` — the actual call site passes `ncores - 1`. The doc uses abstract `n` rather than specifying the concrete value, which is accurate and less fragile. ✓
- Line 490: Each application core calls `startup::signal()` exactly once. ✓

**Verdict: Genuinely fixed. The abstraction to `n` correctly describes the init→new relationship without committing to a specific value that could become stale.**

### Previously Accepted Issues (unchanged, no re-review needed)

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | `wait()` precondition inverts blocking semantics | High | Accepted (inherent limitation) |
| 2 | `signal(&mut self)` exclusive access | High | Accepted (inherent limitation) |
| 3 | `signal()` precondition strengthening | Medium | Fixed (r2_a2, audit added) |
| 4 | `new()` not `const fn` | Medium | Accepted (Verus limitation) |
| 5 | Trivial proof lemmas | Medium | Accepted (labeled as such) |
| 6 | Extra helper functions | Low | Accepted |
| 7 | `FenceView` pub fields | Low | Accepted |
| 8 | `Fence` struct pub fields | Low | Accepted |

All previously accepted issues remain unchanged. Spec and proof files have zero diff from the previous round.

## New Issues Introduced

_None._

The change was a single documentation edit (4 lines in the module doc comment). No code, spec, or proof changes were made. No new issues.

## Cumulative Issue Tracker

| # | Issue | Severity | Round Raised | Status |
|---|-------|----------|-------------|--------|
| 1 | `wait()` precondition inverts blocking semantics | High | r2_a1 | Accepted (inherent limitation) |
| 2 | `signal(&mut self)` exclusive access | High | r2_a1 | Accepted (inherent limitation) |
| 3 | `signal()` precondition strengthening needs audit | Medium | r2_a1 | Fixed (r2_a2) |
| 4 | `new()` not `const fn` | Medium | r2_a1 | Accepted (Verus limitation) |
| 5 | Trivial proof lemmas | Medium | r2_a1 | Accepted (labeled) |
| 6 | Extra helper functions | Low | r2_a1 | Accepted |
| 7 | `FenceView` pub fields | Low | r2_a1 | Accepted |
| 8 | `Fence` struct pub fields | Low | r2_a1 | Accepted |
| 9 | Doc inaccuracy: `Fence::new(ncores)` | Low | r2_a2 | Fixed (r2_a3) |

**All 9 issues are now resolved or accepted.** Zero open issues remain.

## Verification Status

- **Verification conditions:** 24 verified, 0 errors
- **Cheating patterns:** None (`assume`, `admit`, `external_body`, `trusted` — all absent)
- **Trust assumptions in code:** Zero

## Positive Observations

- **All actionable issues resolved.** Across the r2 review cycle (3 attempts), the prover addressed every actionable item: caller audit (r2_a2), doc precision fix (r2_a3).
- **Minimal, targeted changes.** Each round's diff was strictly scoped to the raised issue — no unnecessary refactoring or scope creep.
- **Verification remains stable.** 24 VCs across all rounds, no regressions.
- **Documentation is now accurate and thorough.** The Trust Boundaries, Verification Scope, API Divergence, and caller audit sections are all factually correct and well-written.
- **Sound sequential protocol model.** The `wf()` invariant is inductively maintained across all transitions. Satisfaction monotonicity, signal commutativity, and accumulation-to-satisfaction are all proven.

## Summary

The single remaining issue from r2_a2 — the doc inaccuracy in the caller audit — has been fixed with an appropriate abstraction (`startup::init(n)` / `Fence::new(n)`). No new issues were introduced. All 9 issues raised across the r2 review cycle are now resolved or accepted as inherent limitations of the sequential verification model.

The fence verification is a complete, sound, fully machine-checked sequential model with 24 verification conditions, no escape hatches, and excellent documentation. The grade reflects zero remaining open issues and a mature, well-documented verification artifact.
