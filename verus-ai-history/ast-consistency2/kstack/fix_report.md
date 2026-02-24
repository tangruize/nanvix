# Exec Consistency Fix: kstack

## Summary
- Mismatches fixed: 0 (all 4 are necessary Verus adaptations, documented)
- Missing functions added: 0 (both require Verus-incompatible types/traits, documented)
- Documented equivalences: 11
- Visibility fixes applied: 2 (`size()`, `base()` restored to private)

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `KernelStack` struct | DOCUMENTED | Original uses `kpages: Vec<KernelPage>` which requires `VirtMemoryManager`, `KernelPage`, and heap-allocated `Vec`. Verus cannot model these types. Replaced with `(base_addr: usize, num_pages: usize)` — captures the essential state (base address and page count) that all accessors depend on. The original struct's observable state is exactly `(kpages[0].base(), kpages.len())`, which maps directly to the verus fields. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | DOCUMENTED | Original takes `&mut VirtMemoryManager` and calls `mm.alloc_kpages(true, KSTACK_SIZE / PAGE_SIZE)`. Verus cannot model the allocator or `KernelPage` type. Changed to take `(base_addr, num_pages)` directly — the parameters encode what a correct allocator provides (page-aligned base, contiguous pages). The verus version adds explicit validation (alignment, bounds, overflow checks) that the original relies on the allocator to guarantee. Core semantics: both produce a stack descriptor from allocated memory. |
| `size` [size.diff](size.diff) [size_source.rs](size_source.rs) [size_verus.rs](size_verus.rs) | DOCUMENTED + VISIBILITY FIX | Original returns constant `config::kernel::KSTACK_SIZE`. Verus returns `self.num_pages * PAGE_SIZE`. These are equivalent when `num_pages = KSTACK_SIZE / PAGE_SIZE = 32768 / 4096 = 8`. The verus version generalizes to arbitrary page counts (documented in module header under "Fixed Stack Size Configuration"). The constant `DEFAULT_KSTACK_PAGES = 8` is provided for instantiation at the original fixed size. **Visibility restored from `pub` to private to match original.** |
| `base` [base.diff](base.diff) [base_source.rs](base_source.rs) [base_verus.rs](base_verus.rs) | DOCUMENTED + VISIBILITY FIX | Original returns `PageAligned::from_raw_value(self.kpages[0].base().into_raw_value()).unwrap()`. Verus returns `self.base_addr`. Both return the base address of the first allocated page. The original extracts it from the page object and wraps in `PageAligned<VirtualAddress>`; the verus version stores it directly as `usize` with alignment proven in postconditions (`spec_is_page_aligned(result as int)`). Return type difference (`PageAligned<VirtualAddress>` vs `usize`) documented in module header under "Return Type Abstraction". **Visibility restored from `pub` to private to match original.** |
| `top` [top.diff](top.diff) [top_source.rs](top_source.rs) [top_verus.rs](top_verus.rs) | DOCUMENTED | Original computes `base + KSTACK_SIZE` and wraps in `PageAligned<VirtualAddress>`. Verus computes `base_addr + num_pages * PAGE_SIZE`. Both compute base + size; the size computation differs only in using a constant vs a product (see `size` equivalence above). The `debug_assert!` alignment checks in the original are replaced by formal postconditions in the verus version. |
| `fmt` [fmt_source.rs](fmt_source.rs) | NOT ADDED | Original implements `fmt::Debug` with custom format `KernelStack {{ base: {:?}, top: {:?}, size={:?} }}`. Verus version uses `#[derive(Debug)]` instead. Verus cannot model `fmt::Formatter` or `fmt::Result` types. The derived `Debug` provides equivalent debug output functionality showing all struct fields. |
| `drop` [drop_source.rs](drop_source.rs) | NOT ADDED | Original `Drop` impl iterates `self.kpages.pop()` to deallocate each `KernelPage`. The verus struct has no `Vec<KernelPage>` — only `usize` fields with no resources to release. Verus does not support verifying `Drop` traits (documented in module header under "Drop / Resource Cleanup"). Resource lifecycle management should be verified separately with the allocator. |
| `num_pages` [num_pages_verus.rs](num_pages_verus.rs) | KEPT | Justified verification helper: accessor for the `num_pages` field, enabling specs and proofs to relate the page count to size and address bounds. No equivalent in original because the original uses a fixed `KSTACK_SIZE` constant. |
| `contains` [contains_verus.rs](contains_verus.rs) | KEPT | Justified verification helper: bounds-checking method enabling reasoning about stack overflow protection and page fault handling. Tests `addr ∈ [base, top)`. Could be added to the original implementation. |
| `page_index` [page_index_verus.rs](page_index_verus.rs) | KEPT | Justified verification helper: maps an address to its page index within the stack, enabling reasoning about page membership for fault handling. Proven to satisfy `addr_in_page` spec. |
| `initial_sp` [initial_sp_verus.rs](initial_sp_verus.rs) | KEPT | Justified verification helper: returns `top()` as the initial stack pointer value. Documents that stacks grow downward from this address. Thin wrapper with strengthened postconditions. |
| `has_room` [has_room_verus.rs](has_room_verus.rs) | KEPT | Justified verification helper: checks if the stack can grow by a given amount from the current stack pointer without underflowing past base. Uses subtraction instead of addition to avoid overflow — a verified safety property. The precondition accepts `current_sp == top()` because `top()` is the initial stack pointer before any pushes (stacks grow downward); comment added to document this. |

## Detailed Equivalence Analysis

### Struct Abstraction

The fundamental divergence is the struct representation:

- **Original:** `KernelStack { kpages: Vec<KernelPage> }` — owns heap-allocated page objects obtained from `VirtMemoryManager::alloc_kpages()`.
- **Verus:** `KernelStack { base_addr: usize, num_pages: usize }` — stores the essential address and size information directly.

This abstraction is necessary because Verus cannot model:
- `VirtMemoryManager` (complex allocator with mutable state)
- `KernelPage` (resource type with ownership semantics)
- `Vec<T>` (heap-allocated growable array)
- `PageAligned<VirtualAddress>` (type-level alignment wrapper)

The observable behavior of every accessor depends only on `(base_addr, num_pages)`:
- `base()` → `kpages[0].base()` ≡ `base_addr`
- `size()` → `KSTACK_SIZE` ≡ `num_pages * PAGE_SIZE` (when `num_pages = KSTACK_SIZE / PAGE_SIZE`)
- `top()` → `base + size` (identical formula)

### Return Type Abstraction

Original functions return `PageAligned<VirtualAddress>`, which encodes alignment at the type level. The verus version returns `usize` with alignment proven in postconditions:
- `base()` ensures `spec_is_page_aligned(result as int)`
- `top()` ensures `spec_is_page_aligned(result as int)`

Both approaches guarantee callers receive page-aligned addresses. The verus approach is proof-level rather than type-level but provides the same safety guarantee.

### `new()` — Allocator Abstraction

The original `new(&mut VirtMemoryManager)` delegates allocation to the memory manager and trusts it to return contiguous, page-aligned pages. The verus `new(base_addr, num_pages)` encodes these as explicit preconditions:
- `spec_is_page_aligned(base_addr)` — what the allocator guarantees
- `0 < num_pages <= MAX_STACK_PAGES` — bounded page count
- `base_addr + num_pages * PAGE_SIZE <= usize::MAX` — no overflow

This separation of concerns allows verifying the stack logic independently from the allocator. When both modules are verified, their guarantees compose.

### Logging Omission

The original `drop` impl uses `debug!("{:?}", &self)` for logging before deallocation. Logging is omitted in the verus version because:
- `debug!` macro is not available in the Verus verification context
- Logging is a side effect that does not affect correctness
- The verus struct has no resources to deallocate

## Review Items Addressed

| # | Issue | Resolution |
|---|-------|------------|
| Minor #1 | `base()` doc comment inaccuracy in original | Informational; verus version already has correct documentation ("lowest virtual address"). |
| Minor #2 | `size()` and `base()` visibility widened from private to public | **Fixed**: restored both to private (`fn` instead of `pub fn`) to match original. |
| Minor #3 | `has_room` precondition allowing `current_sp == top()` needs comment | **Fixed**: added comment explaining `top()` is the initial SP before any pushes. |
| Minor #4 | Original `base()` and `top()` use `unwrap()` | Informational; noted that the verus version replaces these with proof-level alignment guarantees, avoiding the coding standard violation. |

## Verification: PASS

```
verification results:: 18 verified, 0 errors
Duration: 10s
Command: ./verus-ai/scripts/verify.sh kstack
```

## Notes

All 4 MISMATCH functions and 2 MISSING functions involve Verus-incompatible constructs:
- **`VirtMemoryManager`**: Complex allocator type with mutable state — not modelable in Verus.
- **`KernelPage`**: Resource type with ownership and deallocation semantics — requires linear types.
- **`Vec<KernelPage>`**: Heap-allocated collection — Verus cannot reason about `Vec` with custom resource types.
- **`PageAligned<VirtualAddress>`**: Type-level alignment wrapper — replaced by proof-level alignment guarantees.
- **`fmt::Formatter`/`fmt::Result`**: Standard library formatting types — not available in Verus context.
- **`Drop` trait**: Verus does not support verifying `Drop` implementations.
- **`config::kernel::KSTACK_SIZE`**: External constant — parameterized as `num_pages` for generality.

The 5 EXTRA functions are justified verification helpers (documented in module header under "Verification Helper Methods") that enable reasoning about stack bounds, page membership, stack pointer validity, and growth safety. They do not alter the original API surface.
