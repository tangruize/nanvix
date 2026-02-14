# Review: kcall_create_thread Exec Consistency (claude-opus-4.6)

## Grade: A

## Files Reviewed
- Original: `src/kernel/src/pm/kcall/create_thread.rs`
- Exec model: `verus/split/kernel/pm/kcall/create_thread.rs`
- Spec: `verus/split/kernel/pm/kcall/create_thread.spec.rs`
- Proof: `verus/split/kernel/pm/kcall/create_thread.proof.rs`
- Consistency report: `verus-ai-history/ast-consistency/kcall_create_thread_20260214_193133_fix.md`

## Verification Result
- **24 verified, 0 errors** — PASS
- Command: `./verus-ai/scripts/verify.sh kcall_create_thread`

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?
**Yes.** The consistency report lists 0 mismatches to fix. Five existing functions (`create_thread_model`, `is_user_region`, `is_user_addr`, `copy_from_user`, `assert_user_stack_size`) were documented with clear equivalence justifications. The model pattern (`_model` + wrapper) is consistent with `terminate.rs` and other kcall modules.

### 2. Were MISSING functions added with proper verification?
**Yes.** One missing function was added: the `create_thread` wrapper (lines 880–927). It follows the same delegation pattern used by `terminate` (lines 561–577 of `terminate.rs`): calls `create_thread_model`, discards ghost witnesses, returns only the `KcallResultModel`. The wrapper has `requires`/`ensures` contracts and Verus verifies it as part of the 24 verified items.

### 3. Are equivalence justifications sound?

| Function | Justification | Sound? |
|----------|--------------|--------|
| `create_thread_model` | Core verified model; wrapper delegates to it | ✅ Sound — matches project pattern |
| `is_user_region` | `external_body` model of `Vmem::is_user_region` | ✅ Sound — boolean identity with ghost addr tracking |
| `is_user_addr` | `external_body` model of `Vmem::is_user_addr` | ✅ Sound — boolean identity with ghost addr tracking |
| `copy_from_user` | `external_body` model of `pm::copy_from_user` | ✅ Sound — returns `CopyOk{args}` with spec_view linkage |
| `assert_user_stack_size` | Build-time bridge for spec constant sync | ✅ Sound — prevents silent drift |
| `ThreadCreateArgsModel` | Model struct for `ThreadCreateArgs` | ✅ Sound — field mapping is correct |

### 4. Does the exec code now faithfully represent the original source?

**Yes, with appropriate abstractions.** Detailed structural comparison:

| Step | Original (lines) | Model (lines) | Match? |
|------|-------------------|---------------|--------|
| 1. Args addr check | 67–71 | 700–713 | ✅ `is_user_region` → `InvalidArgument` |
| 2. Copy from user | 74–84 | 715–746 | ✅ `copy_from_user` → propagate error code |
| 3. user_fn check | 87–91 | 752–765 | ✅ `is_user_addr` → `InvalidArgument` |
| 4. user_stack region | 94–102 | 767–784 | ✅ `is_user_region` → `InvalidArgument` |
| 4b. user_stack size | 105–112 | 786–798 | ✅ `< USER_STACK_SIZE` → `InvalidArgument` |
| 5. user_tda (optional) | 115–122 | 800–819 | ✅ `if let Some` → `if has_user_tda` |
| 6. PM create_thread | 125–131 | 821–846 | ✅ `Ok(tid)→Success`, `Err(e)→Error` |

- Validation order: identical in both.
- Error codes: all validation failures return `ErrorCode::InvalidArgument`; copy failure propagates its own error code. Both match.
- The `if let Some(user_tda)` → `if copied_args.has_user_tda` mapping correctly models the Option check.
- Success path: original's `KcallResult::Success(<i32>::from(tid).into())` maps to `KcallResultModel::Success { tid_value: tid }` where `tid: i32`. The `pm_create_thread` external body postcondition ensures `tid >= 0i32`.

### 5. Does verification still pass?
**Yes.** 24 verified, 0 errors.

## Issues Found

### Critical
- None.

### Minor
1. **Wrapper `create_thread` ensures are weaker than `create_thread_model`**: The wrapper only exposes 3 postconditions (exhaustiveness, args-addr error, copy error) versus `create_thread_model`'s full 10+ postconditions. This is intentional (callers needing richer postconditions use `create_thread_model` directly), but it means the wrapper cannot prove user_fn/user_stack/user_tda error paths at the call site. This matches the `terminate` wrapper pattern and is acceptable.

2. **`ThreadCreateArgsModel` field documentation**: The struct has `pub` fields (lines 300–321), which deviates from the Nanvix coding standard requiring private fields with getter/setter methods. However, this is in the Verus verification model (not the kernel source), and Verus struct patterns commonly use `pub` fields for proof accessibility. Acceptable in context.

### Observations
- The consistency report correctly identifies `ThreadCreateArgsModel` as a documented/kept item, not a function, which is slightly inconsistent with the report's "Function" column heading. Cosmetic issue only.
- The `copy_from_user` external body models an explicit `CopyOk { args }` return with `args.spec_view() == thread_args.spec_view()` postcondition, which creates the copy-to-validation linkage proven by `lemma_copy_output_determines_validation`. This is well-designed.
- Ghost parameter threading (`ghost_arg0` used in both `is_user_region` step 1 and `copy_from_user` step 2) correctly models the original's reuse of `args.arg0` for both validation and copy source address.

## Summary

The exec consistency fix is well-executed. The single added function (`create_thread` wrapper) follows the established project pattern exactly. All 5 existing functions have sound equivalence justifications. The model faithfully mirrors the original's 6-step validation pipeline with identical ordering, error codes, and short-circuit behavior. Verification passes cleanly with 24 verified items and 0 errors. The minor issues (weaker wrapper postconditions, pub fields in model struct) are intentional design choices consistent with the project's verification architecture.
