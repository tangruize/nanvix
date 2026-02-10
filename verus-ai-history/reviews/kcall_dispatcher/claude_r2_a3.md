# Review: kcall_dispatcher (claude-opus-4.6) — Round 3

## Grade: A-

## Verification Result

58 verified, 0 errors. All proofs pass cleanly.

## Changes Since Round 2

**No code, spec, or proof changes were made.** The files are byte-identical to round 2 (verified via checksums and git diff). The prover did not address the 3 low-priority remaining issues from round 2 (spec richness, magic numbers, trust surface). This is acceptable — all three were flagged as low priority and accepted in round 2.

## Previous Issues — Final Disposition

### From Round 1 (all resolved in Round 2, confirmed stable)

| Issue | Status | Verification |
|---|---|---|
| Critical: ETIMEDOUT 110→116 | **FIXED (R2)** | Spec returns 116, exec uses `116i32`, consistent with `src/libs/sysapi/src/errno.rs:209` |
| Medium: Tautological postcondition | **FIXED (R2)** | Replaced with meaningful i32-range error constraint |
| Medium: CondSignal bool arg | **DOCUMENTED (R2)** | Explicit doc on `pm_signal_cond` external body |
| Medium: `ensures false` external_body | **ACCEPTED (R1)** | Inherent to divergence modeling, well-documented as T4 |

### From Round 2 (3 low issues, unchanged)

All three low-priority issues from round 2 remain unchanged. This is expected — the prover chose not to address them, which is reasonable given their low impact.

## New Issue Found

### Low

1. **Stale "110" references in doc comments after ETIMEDOUT fix**
   - **Location:** `dispatcher.rs` lines 44, 463, 770
   - **Description:** When the ETIMEDOUT value was fixed from 110 to 116 in the spec (`SPEC_ERROR_TIMED_OUT() -> 116`) and exec code (`116i32`), three doc comments in the exec file were not updated:
     - Line 44: `//! TimedOut interruptions produce OperationTimedOut (error code 110).` — should be 116.
     - Line 463: `/// - InterruptedTimedOut → Error result with OperationTimedOut (110).` — should be 116.
     - Line 770: `/// - SleepError(TimedOut) → handle_sleep_error → error 110.` — should be 116.
   - **Impact:** Low. The actual spec and exec code are correct (116). These are documentation-only inconsistencies. However, they could mislead future reviewers into thinking the code still uses 110.
   - **Suggested Fix:** Replace "110" with "116" in these three doc comment lines.

## Carried-Forward Issues (Low, all accepted)

1. **`spec_dispatch_result_constrained` trivially true for 5/6 categories** — Accepted design choice; per-call postconditions provide the real guarantees.
2. **Magic numbers in exec code** — Mitigated by inline comments.
3. **16 external bodies as trust surface** — Inherent to the component's role.

## Positive Observations

- **Verification remains stable.** 58 verified items, 0 errors across all three rounds. No regressions.
- **All critical and medium issues from rounds 1-2 remain resolved.** The ETIMEDOUT fix (116), tautological postcondition replacement, and CondSignal documentation are all intact.
- **Comprehensive dispatch coverage.** Every branch of the original `do_kcall` match statement has a corresponding verified path in `do_kcall_dispatch`.
- **Meaningful postconditions at all levels.** `do_kcall_dispatch` → `do_kcall_context` → `do_kcall` propagate per-call correctness properties: GetPid/GetTid non-negative, ok()-calls return 0, JoinThread ≥ 0, terminal calls always error.
- **Clean separation.** Spec, proof, and exec remain properly partitioned across three files.
- **Well-documented trust boundaries.** T1–T5 clearly delineate what is verified vs. assumed.

## Summary

The verification is in good shape. All critical and medium issues from rounds 1-2 were fixed and remain stable. The only new finding is 3 stale doc comments still referencing "110" instead of "116" — a minor documentation inconsistency that doesn't affect the correctness of the spec or exec code.

The grade remains A-. The gap from A is primarily due to the cumulative low-priority observations (trivial `spec_dispatch_result_constrained`, magic numbers, large external body surface, stale docs) which individually are insignificant but collectively suggest polish opportunities. The core verification is sound and provides meaningful correctness guarantees for the kernel call dispatcher.
