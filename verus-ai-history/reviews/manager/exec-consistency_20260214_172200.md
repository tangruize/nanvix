# Review: manager Exec Consistency (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

1. **`alloc_many_user_frames` return type divergence is significant.**
   The original returns `Result<Vec<UserFrame>, Error>` (fallible), while the verified version returns `Ghost<Seq<int>>` (infallible ghost value). This means the verified API cannot model allocation failure for batch user frame allocation, whereas the original *can* fail. The Verus limitation is real, but the fix report should have flagged the loss of the `Result` error path as a semantic gap, not just a type difference. The precondition `has_upool_capacity_for(nframes)` partially compensates by requiring sufficient capacity upfront, but the original can also fail for other internal reasons (e.g., fragmentation in `alloc_many`). **Risk: low in practice**, since the underlying `Upool::alloc_many` is also verified, but the error path is unmodeled.

2. **`alloc_many_kernel_frames` return type loses frame identity.**
   The original returns `Result<Vec<KernelFrame>, Error>` (a collection of typed frame handles), while the verified version returns `Result<usize, Error>` (a starting index). Callers of the original API can iterate over individual `KernelFrame` objects and use their addresses; callers of the verified API get only a numeric starting index. This is documented as a Verus limitation, which is accurate, but the review notes that this means **downstream verification of code that uses individual frames from a batch allocation cannot be expressed**. Acceptable for current scope.

### Medium

1. **`alloc_kernel_frame` silently ignores `clear` parameter.**
   The verified version accepts `clear: bool` for API compatibility but binds it to an unused variable (`let _clear: bool = clear;`). The fix report correctly documents this as a security feature out of scope for memory safety verification. However, the `#[allow(unused_variables)]` annotation and the explicit binding are unnecessary boilerplate — a leading underscore in the parameter name (`_clear: bool`) would suffice. Minor style issue, no semantic impact.

2. **`alloc_kpages` and `alloc_upages` use `external_body` with `unimplemented!()`.**
   These are marked `#[verifier::external_body]` and their bodies are `unimplemented!()`. The specifications (pre/postconditions) are sound and well-documented. The justification for `external_body` — loop invariant coupling across pool and vmem state — is reasonable. However, two `external_body` functions in one module is worth noting. They represent **trusted assumptions** that callers rely on without machine-checked proof. The specifications appear correct by inspection (they scale the single-allocation postconditions), but this is a trust boundary.

3. **No `free_kernel_frame` in original or verified.**
   The original `PhysMemoryManager` has no `free_kernel_frame` method — only `free_user_frame`. The verified version correctly mirrors this omission. This is not a bug in the consistency fix, but worth noting: the original design assumes kernel frames are never freed (common in microkernel designs where kernel memory is statically partitioned). The fix report does not mention this, but it is not an error.

### Low

1. **Struct rename `PhysMemoryManager` → `VirtMemoryManager` is well-documented.**
   The fix report and module-level documentation (lines 48–51 of `manager.rs`) clearly explain the rationale: the verified version extends physical memory management with virtual memory operations (`alloc_upage`, `unmap_upage`, `ctrl_upage`, `new_vmem`). The field layout `{ kpool: Kpool, upool: Upool }` is identical. The rename is justified.

2. **Extra functions (`alloc_kpage`, `alloc_upage`, `unmap_upage`, `ctrl_upage`, `new_vmem`, `kpool_capacity`, `upool_capacity`) are properly categorized.**
   These are higher-level operations that compose the verified `PhysMemoryManager` delegate functions with vmem operations. They are not present in the original `PhysMemoryManager` but are part of the `VirtMemoryManager` abstraction. They are fully verified (not `external_body`), with sound pre/postconditions. No issues.

3. **`new()` constructor is exec-identical.**
   Original: `PhysMemoryManager { kpool, upool }`. Verified: `VirtMemoryManager { kpool, upool }`. Only the struct name differs. The documented equivalence is correct.

## Function-by-Function Consistency Matrix

| Original Function | Verified Function | Exec Body Match | Return Type Match | Notes |
|---|---|---|---|---|
| `new(kpool, upool) -> Self` | `new(kpool, upool) -> Self` | ✅ Identical | ✅ (struct rename only) | — |
| `alloc_user_frame(&mut self) -> Result<UserFrame, Error>` | `alloc_user_frame(&mut self) -> Result<UserFrame, Error>` | ✅ `self.upool.alloc()` | ✅ | — |
| `alloc_many_user_frames(&mut self, nframes) -> Result<Vec<UserFrame>, Error>` | `alloc_many_user_frames(&mut self, nframes) -> Ghost<Seq<int>>` | ✅ `self.upool.alloc_many(nframes)` | ⚠️ Ghost type, no Result | Verus limitation |
| `alloc_kernel_frame(&mut self, clear) -> Result<KernelFrame, Error>` | `alloc_kernel_frame(&mut self, clear) -> Result<KernelFrame, Error>` | ⚠️ `clear` unused | ✅ | Documented |
| `alloc_many_kernel_frames(&mut self, clear, count) -> Result<Vec<KernelFrame>, Error>` | `alloc_many_kernel_frames(&mut self, clear, count) -> Result<usize, Error>` | ⚠️ `clear` unused | ⚠️ usize vs Vec | Verus limitation |
| `free_user_frame(&mut self, frame) -> Result<(), Error>` | `free_user_frame(&mut self, frame) -> Result<(), Error>` | ✅ `self.upool.free(frame)` | ✅ | — |

## Verification Status

- **15 verified, 0 errors** — all specifications pass.
- 2 `external_body` functions (`alloc_kpages`, `alloc_upages`) represent trusted assumptions.
- Duration: ~10s.

## Summary

The exec consistency fix is **well-executed**. All 6 original `PhysMemoryManager` functions are present in the verified `VirtMemoryManager` with correct delegate bodies. The 3 documented equivalences (`new`, struct rename, field layout) are sound. The 5 added functions faithfully delegate to the underlying pool operations. The 2 return type divergences (`alloc_many_user_frames` → `Ghost<Seq<int>>`, `alloc_many_kernel_frames` → `Result<usize, Error>`) are genuine Verus limitations, well-documented, and acceptable for the verification scope. The `clear` parameter handling is correctly documented as out-of-scope for memory safety. The extra `VirtMemoryManager` functions (`alloc_upage`, `unmap_upage`, `ctrl_upage`, `new_vmem`, etc.) are fully verified and extend the original abstraction appropriately. The only notable concern is the 2 `external_body` functions, which are trusted but have sound specifications. Overall, this is a clean consistency fix with minor documentation gaps.
