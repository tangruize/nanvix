# Review: kcall_signal_cond Exec Consistency (claude-opus-4.6)

## Grade: A

## Summary

The exec consistency fix correctly addresses the single MISSING function (`signal_cond`) identified in the original consistency report, and the three EXTRA functions (`signal_cond_model`, `drop_cond_model`, `put_cond_model`) are properly documented as legitimate verification model components. The exec code faithfully represents the original source's control flow, and verification passes cleanly (22 verified, 0 errors) with no `assume`, `admit`, or unjustified `external_body` annotations.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**PASS.** No MISMATCH functions existed in the original report — only one MISSING and three EXTRA. All were addressed.

### 2. Were MISSING functions added with proper verification?

**PASS.** The `signal_cond` wrapper function (lines 569–589) was added. It:
- Matches the original function's four-parameter signature: `pid: u32, tid: u32, cond_addr: u32, broadcast: bool`.
- Correctly delegates to `signal_cond_model`, passing `pid` and `tid` as `Ghost` values.
- Includes a `requires` clause for safety preconditions and the ABI constraint.
- Includes an `ensures` clause propagating the `spec_put_cond_completed` postcondition on success.
- Follows the same wrapper pattern used in `lock_mutex` and `wait_cond`.

### 3. Are equivalence justifications sound?

**PASS.** The three EXTRA functions are justified:

- **`signal_cond_model`**: Correctly identified as the main verified exec model. The original `signal_cond` function's control flow — (1) get_cond, (2) notify (broadcast-dependent), (3) implicit drop at scope exit, (4) put_cond — is faithfully reproduced. Short-circuit error propagation via `?` is modeled by explicit `match` arms with early return.

- **`drop_cond_model`**: Models the implicit `Condvar::drop()` at the end of the inner block in the original code (line 71). The original code relies on Rust's RAII drop semantics to decrease the reference count. Making this explicit as a trust boundary (T3) is appropriate — it documents a resource-release event that would otherwise be invisible. The safety note about `CondvarInner::drop` panic unreachability is well-reasoned.

- **`put_cond_model`**: Models `ProcessManager::put_cond(cond_addr)` at line 72 of the original. Appropriately requires `spec_cond_ref_released` to enforce the original code's structural ordering (drop before put_cond).

### 4. Does the exec code faithfully represent the original source?

**PASS with one minor observation.**

Control flow fidelity analysis:

| Original (lines 50–75) | Exec Model | Faithful? |
|------------------------|------------|-----------|
| `ConditionAddress::from(cond_addr)` | Not modeled (type wrapper) | ✅ Justified |
| `ProcessManager::get_cond(cond_addr)?` | `get_cond_model(cond_addr)` + match | ✅ |
| `if broadcast { cond.notify_all()? } else { cond.notify_first()? }` | `notify_model(cond_addr, broadcast)` + match | ✅ |
| Implicit `Condvar::drop()` at `};` (line 71) | `drop_cond_model(cond_addr)` | ✅ |
| `ProcessManager::put_cond(cond_addr)?` | `put_cond_model(cond_addr)` + match | ✅ |
| `Ok(awakened)` | `SignalCondResultModel::Success { awakened }` | ✅ |
| `trace!(...)` | Ghost pid/tid parameters | ✅ Justified |

**Drop placement correctness**: In the original code, `cond` is dropped at the closing brace of the inner block (line 71), which occurs after the notify call regardless of its success/failure. The exec model calls `drop_cond_model` after the `notify_model` call but *before* matching the notify result (line 501), which correctly models this behavior — the drop happens unconditionally when get_cond succeeded. This is a precise and correct modeling of Rust's drop semantics.

**Minor observation** (non-blocking): The `signal_cond` wrapper's `ensures` clause only propagates `spec_put_cond_completed` on success. It does not propagate the condvar-ref-released guarantee from `signal_cond_model` (which ensures `spec_cond_ref_released` whenever get_cond succeeded). This is a conservative choice — callers lose visibility into partial-failure resource cleanup through the wrapper. However, callers can always use `signal_cond_model` directly for richer postconditions.

### 5. Does verification still pass?

**PASS.** Verification output: 22 verified, 0 errors. No `assume`, `admit`, or `axiom` found in any of the three split files. All 4 `external_body` annotations correspond to legitimate trust boundaries (T1–T4).

## Issues Found

### Critical
- None.

### Minor
- **Wrapper postcondition conservatism**: The `signal_cond` wrapper does not expose the `spec_cond_ref_released` postcondition available from `signal_cond_model`. This means callers using the wrapper cannot reason about resource cleanup on partial failure paths. If callers need this guarantee, they must call `signal_cond_model` directly. This is a design choice, not a bug.

## Detailed Verification Audit

| Aspect | Status |
|--------|--------|
| Verification passes | ✅ 22/22 |
| No assume/admit | ✅ |
| No unjustified external_body | ✅ (4 trust boundaries, all justified) |
| Spec-exec linkage | ✅ (ensures clause on `signal_cond_model` links to `spec_signal_cond_result`) |
| Ghost state correctness | ✅ (don't-care values on unreached paths) |
| Error propagation fidelity | ✅ (short-circuit matches `?` operator) |
| Drop ordering correct | ✅ (drop before notify-result match) |
| Broadcast semantics modeled | ✅ (propagated from T2 postcondition) |
| Original API traceability | ✅ (wrapper + API mapping table) |
