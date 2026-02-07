# Verus Performance Issue: Removing Spec Function Alias Causes Slowdown

## Summary

Removing a trivial spec function alias (`used()` -> `num_allocated()`) causes significant SMT solver slowdown.

## Results

### Minimal Reproduction (This Directory)

| Version | SMT Time | rlimit | 
|---------|----------|--------|
| FAST (with `used()`) | ~204 ms | 309,813 |
| SLOW (without `used()`) | ~787 ms | 337,350 |
| **Difference** | **3.86x** | 1.09x |

### Full Nanvix Codebase

| Version | kheap rlimit | kheap time | Total rlimit |
|---------|-------------|------------|--------------|
| FAST (with `used()`) | 19.9M | 0.9s | 30.7M |
| SLOW (without `used()`) | 147M | 13s | 157.7M |

**Full codebase slowdown: 7.4x rlimit, 14x time**

## Key Observation

In the minimal example:
- **rlimit is nearly identical** (only 8.9% difference)
- **But actual SMT solving time differs by 3.86x!**

This suggests the issue is related to **Z3 internal term caching/sharing**, not just quantifier expansion count.

## Files

- `perf_issue.rs` - **FAST version** with `used()` alias (~204 ms)
- `perf_issue_slow.rs` - **SLOW version** without `used()` (~787 ms)  
- `issue_diff.patch` - The original diff from Nanvix codebase

## How to Reproduce

```bash
cd verus_perf_test

# Run FAST version (with used() alias)
verus --crate-type lib perf_issue.rs --time-expanded

# Run SLOW version (without used() alias)
verus --crate-type lib perf_issue_slow.rs --time-expanded
```

## The Change

The only change is removing this function from `SlabView`:

```rust
// FAST VERSION: Using alias function
pub open spec fn used(&self) -> int {
    self.num_allocated()  // Just an alias!
}

pub open spec fn free(&self) -> int {
    self.capacity() - self.used()  // Uses alias
}

// SLOW VERSION: Direct call (no alias)
// used() function REMOVED

pub open spec fn free(&self) -> int {
    self.capacity() - self.num_allocated()  // Direct call
}
```

Also in `KheapView::total_allocated()`:
```rust
// FAST
self.slab_8.used() + self.slab_16.used() + ...

// SLOW
self.slab_8.num_allocated() + self.slab_16.num_allocated() + ...
```

## Root Cause Hypothesis

The `used()` function acts as a **term-sharing hint** for the Z3 SMT solver. When multiple specifications reference the same underlying value through this alias, Z3 can cache and reuse the result. Without the alias, Z3 may repeatedly recompute equivalent terms.

## Code Structure

The minimal example contains:
- `BitmapView` / `Bitmap` - Simplified bitmap with quantifier-based specifications
- `SlabView` / `Slab` - Slab allocator with block allocation tracking  
- `KheapView` / `Kheap` - Kernel heap aggregating multiple slab sizes (8 slabs)

The bitmap quantifiers and multi-slab aggregation are necessary to trigger the performance issue.

## Environment

- Verus version: 0.2025.12.05.d3fefb4
- Z3: bundled with Verus
- Platform: Linux x86_64

## Related

- Original issue discovered in Nanvix at commit `e42d67be`
- The full codebase has more complex quantifiers which amplify the effect (7.4x vs 3.86x)
