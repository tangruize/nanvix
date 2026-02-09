# Review: thread_manager (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Coverage gap for ThreadRef/ThreadRefMut remains** (thread_manager.rs exec): The updated module still omits `ThreadRef` / `ThreadRefMut` and their `thread_state()` / `thread_state_mut()` methods, so not all functions in the original source are verified. **Suggested Fix:** Add boundary models/specs for these enums or include a dedicated verified module and link it here.

### Medium
- **Overflow behavior still assumed** (thread_manager.rs exec `create_thread`): The precondition `next_id.value < i32::MAX` remains, but the original code has no overflow check. The fix did not address semantic equivalence or make the assumption explicit in the exec code. **Suggested Fix:** Add a runtime check/error path in the exec model or model wraparound behavior and weaken uniqueness properties accordingly.
- **Admission time still omitted** (thread_manager.rs exec boundary ReadyThread): The boundary ReadyThread still omits `admission_time`, which is set in the real constructor and may affect scheduling semantics. This continues to weaken equivalence for clients relying on admission-time properties. **Suggested Fix:** Import the ready module’s ReadyThread model or add admission_time to the boundary type and specs.

### Low
- **OOM behavior still elided** (thread_manager.rs exec docs): The model continues to assume allocation success without modeling the original panic-on-OOM behavior. **Suggested Fix:** Document the fatal OOM behavior as an explicit spec assumption or model an abort path if relevant.

## Positive Observations
- **Resource preservation fixed:** `ReadyThread::new` now ensures kernel/user stack and user_tda are preserved, and `create_thread` exposes these guarantees.
- Core ID allocation and uniqueness properties remain specified and proven, with a clean exec/spec/proof split and no `assume`/`external_body` in this module.

## Summary
The update fixes the resource-preservation weakness, but key gaps remain: coverage for ThreadRef/ThreadRefMut, overflow handling, and admission-time equivalence. Verification is improved but still incomplete against the stated criteria. Addressing the remaining issues would move the module toward a full and sound verification.
